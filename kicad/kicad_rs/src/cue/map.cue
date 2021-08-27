// This file "maps" all attributes and labels for a component to a "generated" sub-field
// where both the class-defined requirements and defaults are merged with the set values of
// the component
// This file depends on common.cue

#Component: {
    labels: #Labels
    attributes: [string]: #Attribute
    classes: [...string]

    for comp_class in classes {
        for pol_class, pol_class_spec in #Policy {
            if pol_class == comp_class {
                generated: "\(comp_class)": {
                    // Merge attributes from both the class spec and the component
                    for attr, attr_spec in pol_class_spec.attributes {
                        attributes: "\(attr)": attr_spec
                    }
                    for attr, attr_spec in attributes {
                        attributes: "\(attr)": attr_spec
                    }
                    // Merge labels from both the class spec and the component
                    for key, val in pol_class_spec.labels {
                        labels: "\(key)": val
                    }
                    for key, val in labels {
                        labels: "\(key)": val
                    }
                }
            }
        }
    }
}
