// This file describes the schema of the CUE file given to the classifier
// This file depends on common.cue

// The user-defined #Policy object specifies what attributes and labels are mandated
// for each class, along with any needed defaults
#Policy: [string]: {
    attributes: [string]: #Attribute
    labels: #Labels
}

// The user-defined #Classifiers object specifies how various attributes are labels
// can be used to classify a component to belonging to a class.
#Classifiers: [...#Classifier]
#Classifier: {
    class: string
    labels?: #RequirementMap
    attributes?: #RequirementMap
}

// The requirement string can be arbitrarily defined, it's used to enable
// the merging of classes extending each other
#RequirementMap: [string]: {
    key: string
    op: string
    values: [...string] | [float]
}
