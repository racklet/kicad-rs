use crate::codec;
use crate::error::{errorf, DynamicResult};
use crate::labels::LabelsMatch;
use crate::requirements::Requirement;
use crate::types::{Component, Schematic};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::iter::FromIterator;
use std::path::Path;
use std::process;
use std::process::{Command, Stdio};
use std::str;
use tempfile::tempdir;

const CUE_COMMON_BYTES: &'static str = include_str!("cue/common.cue");
const CUE_MAP_BYTES: &'static str = include_str!("cue/map.cue");
const CUE_MAP_FILE: &'static str = "map.cue";
const CUE_REDUCE_BYTES: &'static str = include_str!("cue/reduce.cue");
const CUE_REDUCE_FILE: &'static str = "reduce.cue";
const CUE_POLICY_SCHEMA_BYTES: &'static str = include_str!("cue/policy_schema.cue");
const CUE_POLICY_SCHEMA_FILE: &'static str = "policy_schema.cue";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SchematicHolder {
    pub schematic: Schematic,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ComponentClassifier {
    // The class that shall be applied to a component matching these requirements
    pub class: String,
    // Label matching
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub labels: HashMap<String, Requirement>,
    // Attribute matching
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub attributes: HashMap<String, Requirement>,
}

pub fn apply(cue_policy_file: &Path, cue_bin: &Path, sch: Schematic) -> DynamicResult<Schematic> {
    // Write the in-binary policy schema file to a temporary directory
    let tmp_dir = tempdir()?;
    let mut m = HashMap::new();
    m.insert(
        CUE_POLICY_SCHEMA_FILE.into(),
        Vec::from([CUE_COMMON_BYTES, CUE_POLICY_SCHEMA_BYTES]),
    );
    let m = write_temp_files(&tmp_dir, m)?;

    // Execute the given policy file using CUE and decode the resulting YAML ComponentClassifier list
    // This will use the first, classification part of the given CUE file
    let c = Command::new(cue_bin)
        .arg("export")
        .arg(&m.get(CUE_POLICY_SCHEMA_FILE).unwrap())
        .arg(cue_policy_file)
        .arg("--expression=#Classifiers")
        .arg("--out=yaml")
        .output()?;

    // Decode the classifiers from the YAML output
    let classifiers: Vec<ComponentClassifier> = codec::unmarshal_yaml(c.stdout.as_slice())?;

    // Make the now-owned schematic mutable for passing into the classifier function
    let mut sch = sch;
    classify_components(&mut sch, &classifiers);

    // Marshal the now-classified Schematic back to YAML, inside the SchematicHolder struct
    // (to support arbitrary Schematic nesting) for piping to CUE defaulting and validation step
    let mut schematic_yaml: Vec<u8> = Vec::new();
    let sch_holder = SchematicHolder { schematic: sch };
    codec::marshal_yaml(&sch_holder, &mut schematic_yaml)?;

    // Write the in-binary "map and reduction" CUE files to a temporary directory
    let tmp_dir = tempdir()?;
    let mut m = HashMap::new();
    m.insert(
        CUE_MAP_FILE.into(),
        Vec::from([CUE_COMMON_BYTES, CUE_MAP_BYTES, CUE_POLICY_SCHEMA_BYTES]),
    );
    m.insert(
        CUE_REDUCE_FILE.into(),
        Vec::from([CUE_COMMON_BYTES, CUE_REDUCE_BYTES]),
    );
    let m = write_temp_files(&tmp_dir, m)?;

    // Assemble the CUE command that will apply the policy of the given cue_policy_file
    let cmd = format!(
        "{} export --out=yaml {} {} yaml: - | {} export --out=yaml {} yaml: -",
        cue_bin.display(),
        cue_policy_file.display(),
        m.get(CUE_MAP_FILE).unwrap(),
        cue_bin.display(),
        m.get(CUE_REDUCE_FILE).unwrap(),
    );

    // Execute the command with schematic_yaml passed to stdin, and capture stdout/stderr.
    let output = exec_shell_pipe(&cmd, schematic_yaml)?;
    // If there's data in stderr, we got an error we shall pass through
    if !output.stderr.is_empty() {
        writeln!(std::io::stderr(), "{}", str::from_utf8(&output.stderr)?)?;
        return Err(errorf("policy error occurred"));
    }

    // If we were successful in passing it through, unmarshal back into the Schematic
    let sch_holder: SchematicHolder = codec::unmarshal_yaml(output.stdout.as_slice())?;

    Ok(sch_holder.schematic)
}

// classify_components recursively walks through a Schematic, and assigns the Component.classes field
fn classify_components(sch: &mut Schematic, classifiers: &Vec<ComponentClassifier>) {
    for (_, comp) in sch.components.iter_mut() {
        comp.classes = classify_component(comp, classifiers);
    }
    for (_, sch) in sch.sub_schematics.iter_mut() {
        classify_components(sch, classifiers);
    }
}

// classify_component returns a list of classes for a given component, given the set of classifiers
fn classify_component(comp: &Component, classifiers: &Vec<ComponentClassifier>) -> Vec<String> {
    // Map all classifiers to their name if the component matches the classifier
    let matched_classes: Vec<String> = classifiers
        .iter()
        .filter_map(|classifier| {
            // Require that both all labels and attribute requirements match
            if !classifier.labels.matches(&comp.labels.to_map()) {
                return None;
            }
            if !classifier.attributes.matches(&comp.attributes) {
                return None;
            }

            // If we get all the way here, we have "matched" with this class.
            return Some(classifier.class.clone());
        })
        .collect();

    // As there might be many classifiers of the same name that have matched with a component,
    // filter all duplicates
    filter_duplicates(&matched_classes)
}

// filter_duplicates inserts all items into a HashSet, and builds a new vector without any duplicates
fn filter_duplicates<T: Eq + Clone + Hash>(list: &Vec<T>) -> Vec<T> {
    let f: HashSet<&T> = HashSet::from_iter(list.iter());
    f.iter().map(|s| s.clone().to_owned()).collect()
}

fn write_temp_files(
    tmp_dir: &tempfile::TempDir,
    m: HashMap<String, Vec<&str>>,
) -> DynamicResult<HashMap<String, String>> {
    m.into_iter()
        .map(|a| {
            let p = tmp_dir.path().join(&a.0);
            let p = p.to_str().ok_or("couldn't build path")?;
            let mut f = File::create(p)?;
            for bytes in a.1 {
                writeln!(f, "{}", bytes)?;
            }
            f.flush()?;
            Ok((a.0.clone(), p.into()))
        })
        .collect()
}

fn exec_shell_pipe(cmd: &str, stdin_data: Vec<u8>) -> DynamicResult<process::Output> {
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(&stdin_data)
            .expect("Failed to write to stdin");
    });

    Ok(child.wait_with_output()?)
}
