# kicad-rs

`kicad-rs` is a set of UNIX-style command line tools that automate some otherwise mundane tasks when working with KiCad schematic files (those ending with `.sch`). We like to think about this project as a way of doing "declarative electronics".

For example, sometimes, if you change one component's value (of some attribute) in your schematic, suddenly a lot of other values need changing as well, creating a "snowball effect". Instead, `kicad-rs` allows you to parameterize all your component's values in the schematics, and dynamically re-calculate them based on your rules by just executing a command.

Further, it allows for "unit testing" your schematics, by parsing the schematic data into a YAML file with a schema a bit easier (higher-level) to look at than the schematic text itself. When updating the schematic "source of truth" you can also see the change of the component values in the auto-generated YAML file.

Finally, it is possible to apply policy to your schematics, i.e. that all resistors in a given projects should have a tolerance of 5% or less. Eventually, we'd like to be able to generate BoM (Bill of Materials) documents in Markdown, aggregating information on what to buy in one page, and possibly even have the automation order the right components from a electronic parts vendor.

## Sample usage

Clone this repository with the `--recurse-submodules` flag in order to also pull in the KiCad symbol library used for the exmaple schematic. Alternatively, if you have already cloned the repository you can run `git submodule update --init --recursive` to clone the submodules.

### Evaluator

The evaluator reads a KiCad schematic file, parses all equations in the schematic, resolves dependencies between component values, and finally calculates and updates the presented component values in the schematic. The schematic file is updated in place.

> **Note**: Make sure you have saved the schematic file before running this command, and maybe even, as a safety measure, closed KiCad operating on the given file. This to avoid race conditions and lost data if two parties would be reading and writing the same data.

In order to make a certain component value automatically calculated, double-click the component in KiCad. A dialog called "Symbol properties" should pop up, allowing you to edit name-value fields. If the component is e.g. a resistor called `R1`, and you would like to make its primary value (i.e. resistance) twice as large as another resistor `R2`'s resistance, add a new field called `Value_expr` and write `R2*2`.

After executing the evaluator like below, the `Value` field of `R1` should be twice as that of `R2`.

- Reads from Stdin: No
- Writes to Stdout: No

Arguments:

1. Schematic file to evaluate, will update in-place

```bash
# This command will update the file in place
cargo run --bin=evaluator testdata/test.sch
```

### Parser

The parser parses a KiCad schematic file into a YAML representation that focuses on key metadata about the schematic and its components (see `testdata/test.yaml` for an example). The YAML file can e.g. be used for "unit testing" that the schematic is as expected (take a look at `.github/workflows/main.yml` for an example of this). The YAML data can also be further processed, e.g. as input to the classifier binary below.

- Reads from Stdin: No
- Writes to Stdout: Yes

Arguments:

1. Schematic file to parse

```bash
# Or any other .sch file. This command writes to stdout, you can save it in a
# file like this or pipe it to the classifier.
cargo run --bin=parser testdata/test.sch > parsed.yaml
```

### Classifier

The classifier is used for classifying components into groups, e.g. all components with a `symbolName: C_Small` or `footprintLibrary: Capacitor_SMD` shall belong to the class `capacitor`. And for example, `capacitor`s with a `Value` (i.e. capacitance) less than `100nF` shall be also belong to the class `small_capacitor`. These rules are written using [CUE] in the `#Classifiers` sub-object (see `testdata/test.cue` for an example).

Further, after classification, one can apply policy, that is, a set of rules, on components belonging to a given class. For example, you might want to enforce the tolerance of all your resistors to be less than 5%, or to enforce a temperature rating attribute for all your capacitors. These rules are written using [CUE] in the `#Policy` sub-object (see `testdata/test.cue` for an example).

> **Important**: Before you use the classifier, make sure to [install CUE].
> If you have Go installed, run:
>
> ```bash
> GO111MODULE=on go get cuelang.org/go/cmd/cue
> ```

[CUE]: https://cuelang.org
[install CUE]: https://cuelang.org/docs/install/

- Reads from Stdin: Yes
- Writes to Stdout: Yes

Arguments:

1. Policy file written in CUE to use. A sample file is given in `testdata/test.cue`
2. (Optional) Path to the `cue` binary, defaults to resolving from your `PATH`.

```bash
# Read from the parsed file like this and save to a file, or...
cat parsed.yaml | cargo run --bin=classifier testdata/test.cue > classified.yaml

# ... pipe the output from the parser like this (writes to stdout)
cargo run --bin=parser testdata/test.sch | cargo run --bin=classifier testdata/test.cue
```

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) and our [Code Of Conduct](CODE_OF_CONDUCT.md).

Other interesting resources include:

- [The issue tracker](https://github.com/racklet/racklet/issues)
- [The discussions forum](https://github.com/racklet/racklet/discussions)
- [The list of milestones](https://github.com/racklet/racklet/milestones)
- [The roadmap](https://github.com/orgs/racklet/projects/1)
- [The changelog](https://github.com/racklet/racklet/blob/main/CHANGELOG.md)

## Getting Help

If you have any questions about, feedback for or problems with Racklet:

- Invite yourself to the [Open Source Firmware Slack](https://slack.osfw.dev/).
- Ask a question on the [#racklet](https://osfw.slack.com/messages/racklet/) slack channel.
- Ask a question on the [discussions forum](https://github.com/racklet/racklet/discussions).
- [File an issue](https://github.com/racklet/racklet/issues/new).
- Join our [community meetings](https://hackmd.io/@racklet/Sk8jHHc7_) (see also the [meeting-notes](https://github.com/racklet/meeting-notes) repo).

Your feedback is always welcome!

## Maintainers

In alphabetical order:

- Dennis Marttinen, [@twelho](https://github.com/twelho)
- Jaakko Sirén, [@Jaakkonen](https://github.com/Jaakkonen)
- Lucas Käldström, [@luxas](https://github.com/luxas)
- Verneri Hirvonen, [@chiplet](https://github.com/chiplet)

## License

[Apache 2.0](LICENSE)
