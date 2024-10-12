//! Printers for cargo-diagram-visitors

use cargo_diagram_visitors::module_visitor::ModulesVisitor;
pub mod console;
pub mod uml;

trait Printer {
    fn print(visitor: &ModulesVisitor, options: PrintOptions) -> String;
}

#[derive(Clone)]
pub struct PrintOptions {
    pub relations: bool,
    pub module_color: String,
    pub trait_color: String,
    pub functions_private: bool,
}
