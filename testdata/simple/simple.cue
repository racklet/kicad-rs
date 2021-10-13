// This is a sample policy file that can be passed to the classifier

// Define all classifiers
#Classifiers: [{
	class: "capacitor"
	// Note: "c_symbol" is a user-defined key and only used for object merging when extending
	labels: c_symbol: {
		key: "symbolName"
		op:  "In"
		values: ["C_Small", "C_Large"]
	}
}, {
	class: "capacitor"
	// Note: "footprint_equals_smd" is a user-defined key and only used for object merging
	labels: footprint_equals_smd: {
		key: "footprintLibrary"
		op:  "Equals"
		values: ["Capacitor_SMD"]
	}
}, _tmpl_resistor & {
	class: "resistor"
}, _tmpl_resistor & { // shunt_resistor extends all the properties of resistor
	class: "shunt_resistor"
	attributes: {
		lt_1: {
			key: "tolerance"
			op:  "Lt"
			values: [1.1]
		}
	}
}]

// #Policy defines requirements and defaults for all components belonging to a class
#Policy: {
	shunt_resistor: {
		labels: {
			extra: shunt: "true"
			datasheet: "foo"
		}
		attributes: tolerance: {
			value: <1.1 | string
			unit:  string | *"Ohm"
		}
	}
	capacitor: attributes: Value: {
		comment: string | *"This is a capacitor :D!"
	}
	resistor: attributes: Value: {
		comment: string | *"This is a resistor!!! :DD"
	}
}

// Common "templates"
_tmpl_resistor: {
	labels: r_in_symbol: {
		key: "symbolName"
		op:  "In"
		values: ["R", "R_Small"]
	}
}
