//! Print results in console
use cargo_diagram_visitors::module_visitor::ModulesVisitor;

/// Prints Modules, structs, traits and implementations
pub fn print_to_console(visitor: &ModulesVisitor) {
    for (module, info) in &visitor.module_map {
        println!("Modul: {}", module);
        for (struct_name, struct_info) in &info.structs {
            println!("  Struktur: {}", struct_name);
            println!("    Derives:");
            for derive in &struct_info.derives {
                println!("      - {}", derive);
            }
            println!("    Impl Traits:");
            for impl_trait in &struct_info.impl_traits {
                println!("      - {}", impl_trait);
            }
        }
        println!("  Submodule:");
        for submodule in &info.submodules {
            println!("    - {}", submodule);
        }
        println!();
    }
}
