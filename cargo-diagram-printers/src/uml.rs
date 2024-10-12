//! Plantuml Uml printer
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cargo_diagram_visitors::module_visitor::{FunctionInfo, ModulesVisitor};

use crate::{PrintOptions, Printer};

struct UMLPrinter;

impl Printer for UMLPrinter {
    fn print(visitor: &ModulesVisitor, opt: PrintOptions) -> String {
        let mut uml_content = String::new();

        // Start the UML diagram
        uml_content.push_str("@startuml\n");
        uml_content.push_str("left to right direction\nskinparam lineType ortho\n"); // Left-to-right layout, orthogonal lines

        // Process each module recursively to display package structure
        for (module, info) in &visitor.module_map {
            if module.ends_with("__lib") || module.ends_with("__main") {
                let package_name = module.replace(".", "_");
                // Call recursive function to handle module and submodules
                add_module_to_uml(
                    &mut uml_content,
                    package_name,
                    info,
                    visitor,
                    0,
                    opt.clone(),
                );
            }
        }

        if opt.relations {
            // Add relationships (arrows) between structs, enums, and their members
            add_relations(&mut uml_content, visitor);
        }
        // End the UML diagram
        uml_content.push_str("@enduml\n");

        uml_content
    }
}

/// Print uml (Plantuml)
pub fn print_uml_to_file(visitor: &ModulesVisitor, output_path: &PathBuf, opt: PrintOptions) {
    let uml_content = UMLPrinter::print(visitor, opt);

    // Write the UML content to a file
    let mut file = File::create(output_path).expect("Unable to create file");
    file.write_all(uml_content.as_bytes())
        .expect("Unable to write data");
}

/// Recursive function to process modules and their submodules as nested packages
fn add_module_to_uml(
    uml_content: &mut String,
    package_name: String,
    info: &cargo_diagram_visitors::module_visitor::ModuleInfo,
    visitor: &ModulesVisitor,
    level: usize,
    opt: PrintOptions,
) {
    let indent = "  ".repeat(level);

    // Print the module as a package
    uml_content.push_str(&format!(
        "{}package {} {} {{\n",
        indent, package_name, opt.module_color,
    ));

    // Add module description if available
    if !info.description.is_empty() {
        uml_content.push_str(&format!("{}  ' {}\n", indent, &info.description));
    }

    // Add structs (as classes)
    for (struct_name, struct_info) in &info.structs {
        let fully_qualified_struct_name = format!("{package_name}::{}", struct_name); // Create fully qualified name
        uml_content.push_str(&format!(
            "{}  class {} as \"{}\" <<struct>> {{\n",
            indent, fully_qualified_struct_name, struct_name
        ));

        // Add struct description if available
        if !struct_info.description.is_empty() {
            uml_content.push_str(&format!("{}    ' {}\n", indent, &struct_info.description));
        }

        // Add derives
        if !struct_info.derives.is_empty() {
            uml_content.push_str(&format!("{}    .. Derives ..\n", indent));
            for derive in &struct_info.derives {
                uml_content.push_str(&format!("{}    {}\n", indent, derive));
            }
        }

        // Add implemented traits
        if !struct_info.impl_traits.is_empty() {
            uml_content.push_str(&format!("{}    .. Implements ..\n", indent));
            for impl_trait in &struct_info.impl_traits {
                uml_content.push_str(&format!("{}    {}\n", indent, impl_trait));
            }
        }

        // Add members (fields) of the struct
        if !struct_info.members.is_empty() {
            uml_content.push_str(&format!("{}    .. Members ..\n", indent));
            for member in &struct_info.members {
                uml_content.push_str(&format!(
                    "{}    {}: {}\n",
                    indent, member.name, member.member_type
                ));
            }
        }

        // Add functions associated with the struct
        if !struct_info.functions.is_empty() {
            uml_content.push_str(&format!("{}    .. Functions ..\n", indent));
            print_functions(
                uml_content,
                indent.clone(),
                &struct_info.functions,
                opt.clone(),
            );
        }

        // Close the struct (class) definition
        uml_content.push_str(&format!("{}  }}\n", indent));
    }

    // Add traits (as interfaces)
    for trait_info in &info.traits {
        let fully_qualified_trait_name = format!(
            "{package_name}::{} as \"{}\"",
            trait_info.name, trait_info.name
        );
        uml_content.push_str(&format!(
            "{}  interface {} {} {{\n",
            indent, fully_qualified_trait_name, opt.trait_color
        ));

        if !trait_info.functions.is_empty() {
            uml_content.push_str(&format!("{}    .. Functions ..\n", indent));

            print_functions(
                uml_content,
                indent.clone(),
                &trait_info.functions,
                opt.clone(),
            );
        }

        uml_content.push_str(&format!("{}  }}\n", indent));
    }

    // Add public functions of the module (inside mod.rs-like class)
    if !info.functions.is_empty() {
        let result: Option<&FunctionInfo> = info.functions.iter().find(|f| f.public);

        if result.is_some() || opt.functions_private {
            uml_content.push_str(&format!(
                "{}  class {package_name}_mod <<mod>> {} {{\n",
                indent, opt.module_color
            )); // Create a class for mod.rs functions

            uml_content.push_str(&format!("{}    .. Module functions ..\n", indent));
            print_functions(uml_content, indent.clone(), &info.functions, opt.clone());
            uml_content.push_str(&format!("{}  }}\n", indent));
        }
    }

    // Recursively handle submodules
    for submodule in &info.submodules {
        if let Some(sub_info) = visitor.module_map.get(submodule) {
            let sub_package_name = submodule.replace(".", "_");
            add_module_to_uml(
                uml_content,
                sub_package_name,
                sub_info,
                visitor,
                level + 1,
                opt.clone(),
            );
        }
    }

    // Close the package definition
    uml_content.push_str(&format!("{}}}\n", indent));
}

fn print_functions(
    uml_content: &mut String,
    indent: String,
    functions: &Vec<FunctionInfo>,
    opt: PrintOptions,
) {
    for function in functions {
        if !opt.functions_private && !function.public {
            continue;
        }
        let fn_signature = format_function_signature(function);
        if !function.description.is_empty() {
            uml_content.push_str(&format!("{}    // {}\n", indent, function.description));
        }

        let prefix = if function.public { "+" } else { "-" };
        uml_content.push_str(&format!("{}    {prefix} {}\n", indent, fn_signature));
    }
}

/// Add relationships between structs and enums based on their members
fn add_relations(uml_content: &mut String, visitor: &ModulesVisitor) {
    // Iterate through all modules and structs to identify relations
    for (module, info) in &visitor.module_map {
        let package_name = module.replace(".", "_");

        // Process structs
        for (struct_name, struct_info) in &info.structs {
            let struct_class_name_root = format!("{package_name}::{}", struct_name); // Fully qualified struct name

            // Check each member of the struct to find if it's another struct or enum
            for member in &struct_info.members {
                let member_type_path = member.member_type.replace("::", "_");

                for (struct_name, _struct_info) in &info.structs {
                    let struct_class_name = format!("{package_name}::{}", struct_name);
                    // Fully qualified struct name

                    if member_type_path.contains(struct_name) {
                        uml_content.push_str(&format!(
                            "{} --> {}\n",
                            struct_class_name_root, struct_class_name
                        ));
                    }
                }
            }
        }
    }
}

/// Helper function to format function signature with parameters
fn format_function_signature(function: &FunctionInfo) -> String {
    let input_params = function
        .parameters
        .iter()
        .map(|param| format!("{}: {}", param.name, param.param_type)) // Assuming you have a name and type for parameters
        .collect::<Vec<_>>()
        .join(", ");

    let output_param = format!(" -> {}", function.return_type);

    format!("{}({}){}", function.name, input_params, output_param)
}
