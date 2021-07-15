// This file defines common definitions for all the policy-related CUE files

// This "schematic" field is coming from SchematicHolder in policy.rs, and is used to enforce a
// schema for the top-level object, which is of type schematic. Then, once this rule is in place,
// we can enforce subSchematics to also abide by the same schema.
schematic: #Schematic
#Schematic: {
    // The most important field of stdin that should be validated is the components map.
    // The components map uses the component reference as a key.
    components: [string]: #Component
    // Support nested schematics
    subSchematics: [string]: #Schematic
    // TODO: Maybe enforce other rules on globals etc,?
    ...
}

#Component: {
    labels: #Labels
    attributes: [string]: #Attribute
    classes: [...string]
}

#Attribute: {
    value: string | float
    expression: string
    type: "Float" | "String"
    unit?: string
    comment?: string
}

#Labels: {
    footprintLibrary: string
    footprintName: string
    reference: string
    symbolLibrary: string
    symbolName: string
    model?: string
    datasheet?: string
    extra: [string]: string
}
