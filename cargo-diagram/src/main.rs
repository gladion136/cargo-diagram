//! Creates diagrams about your crate
use std::collections::BTreeMap;
use std::path::PathBuf;

use analyzer::analyze_repository;
use cargo_diagram_printers::uml::print_uml_to_file;
use cargo_diagram_printers::PrintOptions;
use cargo_diagram_visitors::module_visitor::ModulesVisitor;
use clap::{command, Parser};

mod analyzer;

/// Creates diagrams about your crate
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Show relations inside of the diagram (alpha)
    #[arg(short, long, default_value_t = false)]
    relations: bool,

    /// Select a different path to search
    #[arg(short, long, default_value = "./")]
    path: PathBuf,

    /// Select a different path to search
    #[arg(short, long, default_value = "./overview.puml")]
    output: PathBuf,

    // The color of a module (plantuml colors)
    #[arg(short, long, default_value = "#lightskyblue")]
    module_color: String,

    // The color of a trait (plantuml colors)
    #[arg(short, long, default_value = "#violet")]
    trait_color: String,

    /// Draw private functions
    #[arg(short, long, default_value_t = false)]
    functions_private: bool,
}

fn main() {
    let args = Args::parse();

    let project_root = &args.path;

    let mut visitor = ModulesVisitor {
        module_map: BTreeMap::new(),
        current_module: String::new(),
    };

    let options = PrintOptions {
        relations: args.relations,
        module_color: args.module_color,
        trait_color: args.trait_color,
        functions_private: args.functions_private,
    };

    analyze_repository(project_root, &mut visitor);

    print_uml_to_file(&visitor, &PathBuf::from(args.output), options);
}
