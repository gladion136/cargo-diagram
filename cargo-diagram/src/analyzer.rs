//! Analyze a repository with cargo-diagram-visitors
use std::fs;
use std::path::Path;
use tracing::debug;

use cargo_diagram_visitors::analyze_file;
use cargo_diagram_visitors::module_visitor::ModulesVisitor;

/// Analyze a repository
pub fn analyze_repository(current_dir: &Path, visitor: &mut ModulesVisitor) {
    if current_dir.is_dir() {
        // Search for Cargo.toml
        if current_dir.join("Cargo.toml").exists() {
            debug!("Analysiere Crate: {:?}", current_dir);
            analyze_crate_files(current_dir, visitor);
        }

        // Analyze Subcrates
        for entry in fs::read_dir(current_dir).expect("Fehler beim Lesen des Verzeichnisses") {
            let entry = entry.expect("Fehler beim Lesen des Verzeichnis-Eintrags");
            let path = entry.path();
            if path.file_name().unwrap() == "target" {
                continue;
            }
            analyze_repository(&path, visitor); // Rekursive Analyse
        }
    }
}

/// Analyze crate files (main.rs / lib.rs)
fn analyze_crate_files(crate_dir: &Path, visitor: &mut ModulesVisitor) {
    let src_dir = crate_dir.join("src");

    let file_name = crate_dir
        .file_name()
        .and_then(|name| name.to_str().map(|s| s.to_string()))
        .unwrap_or("l".to_string())
        .replace('-', "_");

    // main.rs
    let main_file = src_dir.join("main.rs");
    if main_file.exists() {
        debug!("Analysiere Datei: {:?}", main_file);
        analyze_file(
            &main_file,
            &src_dir,
            format!("{}__main", file_name).as_str(),
            visitor,
        );
    }

    // lib.rs
    let lib_file = src_dir.join("lib.rs");
    if lib_file.exists() {
        debug!("Analysiere Datei: {:?}", lib_file);
        analyze_file(
            &lib_file,
            &src_dir,
            format!("{}__lib", file_name).as_str(),
            visitor,
        );
    }
}
