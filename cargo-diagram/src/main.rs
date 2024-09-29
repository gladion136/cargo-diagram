//! Creates diagrams about your crate
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use analyzer::analyze_repository;
use cargo_diagram_printers::console::print_to_console;
use cargo_diagram_printers::uml::print_uml;
use cargo_diagram_visitors::module_visitor::ModulesVisitor;

mod analyzer;

fn main() {
    let project_root = Path::new(".");

    let mut visitor = ModulesVisitor {
        module_map: HashMap::new(),
        current_module: String::new(),
    };

    analyze_repository(project_root, &mut visitor);

    print_to_console(&visitor);
    print_uml(&visitor, &PathBuf::from("overview.puml"));
}
