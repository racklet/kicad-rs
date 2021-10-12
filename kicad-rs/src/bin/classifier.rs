use clap::{App, Arg};
use kicad_rs::codec;
use kicad_rs::error::DynamicResult;
use kicad_rs::policy;
use kicad_rs::types::Schematic;
use std::io;
use std::path::Path;
use std::process::Command;

// Get crate version information from Cargo
const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

fn main() -> DynamicResult<()> {
    // Read the Schematic YAML from stdin
    let sch: Schematic = codec::unmarshal_yaml(io::stdin())?;

    let matches = App::new("KiCad classifier")
        .about("Classifies components in schematics based on policy expressed in CUE")
        .author("Lucas Käldström (@luxas), The Racklet Project")
        .version(VERSION.unwrap_or("unknown"))
        .version_short("v")
        .arg(
            Arg::with_name("CUE_POLICY")
                .help("Path to the CUE policy to process")
                .required(true),
        ) // TODO: Allow passing the YAML file as well, instead of stdin.
        .arg(
            Arg::with_name("CUE_BIN")
                .default_value("cue")
                .env("CUE_BIN")
                .help("Path to the cue binary. Download from cuelang.org."),
        )
        .get_matches();

    // Calling .unwrap() is safe here because "CUE_POLICY" is required (if "CUE_POLICY"
    // wasn't required we could have used an 'if let' to conditionally get the value)
    let policy_path = Path::new(matches.value_of("CUE_POLICY").unwrap());
    let cue_path = Path::new(matches.value_of("CUE_BIN").unwrap());

    // Check if the cue binary can be executed from the given path
    Command::new(cue_path)
        .output()
        .expect(format!("Could not execute cue with the invocation: '{}'. \
            Install cue before attempting to run this program.", cue_path.display()).as_str());

    // Apply the policy in the given file
    let processed_sch = policy::apply(&policy_path, &cue_path, sch)?;

    // Marshal the resulting schematic as YAML
    codec::marshal_yaml(&processed_sch, io::stdout())?;
    Ok(())
}
