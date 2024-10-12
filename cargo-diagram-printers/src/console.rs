//! Print results in console
use cargo_diagram_visitors::module_visitor::ModulesVisitor;

use crate::{PrintOptions, Printer};

struct ConsolePrinter;

impl Printer for ConsolePrinter {
    fn print(visitor: &ModulesVisitor, _opt: PrintOptions) -> String {
        let mut result = String::new();

        for (module, info) in &visitor.module_map {
            result = format!("{}\nModul: {}", result, module);
            result.extend(format!("Modul: {}", module).chars());
            for (struct_name, struct_info) in &info.structs {
                result = format!("{}\n  Struktur: {}", result, struct_name);

                result = format!("{}\n    Derives:", result);
                for derive in &struct_info.derives {
                    result = format!("{}\n      - {}", result, derive);
                }
                result = format!("{}\n    Impl Traits:", result);
                for impl_trait in &struct_info.impl_traits {
                    result = format!("{}\n      - {}", result, impl_trait);
                }
            }
            result = format!("{}\n  Submodule:", result);
            for submodule in &info.submodules {
                result = format!("{}\n    - {}", result, submodule);
            }
            println!();
        }

        result
    }
}

/// Prints Modules, structs, traits and implementations
pub fn print_to_console(visitor: &ModulesVisitor, opt: PrintOptions) {
    let result = ConsolePrinter::print(visitor, opt);
    print!("{}", result);
}
