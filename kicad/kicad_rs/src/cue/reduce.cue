// For each component: use the "generated" field as the input, and from that make sure
// that each class agree on all the attributes (i.e. the definitions of labels and attributes
// will be merged for each class, and there must not be any conflicts).
#Component: {
    generated: [string]: {
        attributes: [string]: #Attribute
        labels: #Labels
    }
    for class, gen in generated {
        attributes: gen.attributes
        labels: gen.labels
    }
}
