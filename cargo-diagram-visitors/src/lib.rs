//! Code visitors for cargo-diagram
use std::fs;
use std::path::{Path as StdPath, PathBuf};
use syn::visit::Visit;
use tracing::trace;

use module_visitor::ModulesVisitor;

pub mod module_visitor;

/// Parse a rust file
fn parse_rust_file(file_path: &StdPath) -> syn::File {
    let code = fs::read_to_string(file_path).expect("Konnte die Datei nicht lesen");

    syn::parse_file(&code).expect("Fehler beim Parsen des Quellcodes")
}

/// Analyze a file and add content to visitor
pub fn analyze_file(
    file_path: &StdPath,
    base_dir: &StdPath,
    module_name: &str,
    visitor: &mut ModulesVisitor,
) {
    let syntax_tree = parse_rust_file(file_path);

    visitor.current_module = module_name.to_string();

    visitor.visit_file(&syntax_tree);

    let submodules: Vec<String> = visitor
        .module_map
        .get(module_name)
        .map(|info| info.submodules.clone())
        .unwrap_or_default();

    for submodule in submodules {
        let module_path = find_module_path(&submodule, base_dir);
        if let Some(mod_path) = module_path {
            trace!("Analysiere Modul: {:?}", mod_path);
            analyze_file(&mod_path, mod_path.parent().unwrap(), &submodule, visitor);
        } else {
            trace!("Modul {:?} nicht gefunden!", submodule);
        }
    }
}

/// Get module path
fn find_module_path(module: &str, base_dir: &StdPath) -> Option<PathBuf> {
    let mod_file = base_dir.join(format!("{}.rs", module));
    let mod_dir = base_dir.join(module).join("mod.rs");

    if mod_file.exists() {
        Some(mod_file)
    } else if mod_dir.exists() {
        Some(mod_dir)
    } else {
        None
    }
}
