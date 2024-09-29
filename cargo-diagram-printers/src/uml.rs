//! Plantuml Uml printer
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cargo_diagram_visitors::module_visitor::ModulesVisitor;

/// Print uml (Plantuml)
pub fn print_uml(visitor: &ModulesVisitor, output_path: &PathBuf) {
    let mut uml_content = String::new();

    // Start the UML diagram
    uml_content.push_str("@startuml\n");

    // Iterate through modules
    for (module, info) in &visitor.module_map {
        // Print the module as a class
        uml_content.push_str(&format!("class {} {{\n", module.replace(".", "_"))); // Replace '.' with '_' for valid class names

        // Add structs
        for (struct_name, struct_info) in &info.structs {
            uml_content.push_str(&format!("  + {} : struct\n", struct_name));

            // Add derives
            for derive in &struct_info.derives {
                uml_content.push_str(&format!("    Derives: {}\n", derive));
            }

            // Add implemented traits
            for impl_trait in &struct_info.impl_traits {
                uml_content.push_str(&format!("    Implements: {}\n", impl_trait));
            }

            // Add functions associated with the struct
            for function in &struct_info.functions {
                uml_content.push_str(&format!("    + {}()\n", function)); // Add public function signature
            }
        }

        // Add public functions of the module
        for function in &info.functions {
            uml_content.push_str(&format!("  + {}()\n", function)); // Add public function signature
        }

        // Close the class definition
        uml_content.push_str("}\n");

        // Add submodules
        for submodule in &info.submodules {
            uml_content.push_str(&format!(
                "{} --> {}\n",
                module.replace(".", "_"),
                submodule.replace(".", "_")
            )); // Replace '.' with '_' for valid relationships
        }
    }

    // End the UML diagram
    uml_content.push_str("@enduml\n");

    // Write the UML content to a file
    let mut file = File::create(output_path).expect("Unable to create file");
    file.write_all(uml_content.as_bytes())
        .expect("Unable to write data");
}
