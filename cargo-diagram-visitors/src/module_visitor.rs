//! Modules Overview Visitor
use std::collections::HashMap;
use syn::parse::Parse;
use syn::{
    visit::Visit, Attribute, ItemFn, ItemImpl, ItemMod, ItemStruct, PathArguments, TypePath,
};

/// Info about one module
#[derive(Debug)]
pub struct ModuleInfo {
    pub structs: HashMap<String, StructInfo>,
    pub traits: Vec<String>,
    pub submodules: Vec<String>,
    pub functions: Vec<String>,
}

/// Info about one struct
#[derive(Debug)]
pub struct StructInfo {
    pub derives: Vec<String>,
    pub impl_traits: Vec<String>,
    pub functions: Vec<String>,
}

/// Collects information about multiple modules
pub struct ModulesVisitor {
    pub module_map: HashMap<String, ModuleInfo>,
    pub current_module: String,
}

impl<'ast> Visit<'ast> for ModulesVisitor {
    fn visit_item_struct(&mut self, item_struct: &'ast ItemStruct) {
        let struct_name = item_struct.ident.to_string();

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: HashMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
            })
            .structs
            .insert(
                struct_name.clone(),
                StructInfo {
                    derives: extract_derives(&item_struct.attrs),
                    impl_traits: Vec::new(),
                    functions: Vec::new(),
                },
            );

        syn::visit::visit_item_struct(self, item_struct);
    }

    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        let fn_name = item_fn.sig.ident.to_string();

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: HashMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
            })
            .functions
            .push(fn_name.clone());

        if let Some(receiver) = item_fn.sig.receiver() {
            if let syn::Type::Path(type_path) = &*receiver.ty {
                if let Some(segment) = type_path.path.segments.last() {
                    let struct_name = segment.ident.to_string();
                    if let Some(module_info) = self.module_map.get_mut(&self.current_module) {
                        if let Some(struct_info) = module_info.structs.get_mut(&struct_name) {
                            struct_info.functions.push(fn_name);
                        }
                    }
                }
            }
        }

        syn::visit::visit_item_fn(self, item_fn);
    }

    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        if let syn::Type::Path(TypePath { path, .. }) = &*item_impl.self_ty {
            if let Some(struct_name) = path.get_ident() {
                let struct_name = struct_name.to_string();

                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    let trait_name = format_path(trait_path);

                    if let Some(module) = self.module_map.get_mut(&self.current_module) {
                        if let Some(struct_info) = module.structs.get_mut(&struct_name) {
                            struct_info.impl_traits.push(trait_name);
                        }
                    }
                }
            }
        }

        syn::visit::visit_item_impl(self, item_impl);
    }

    fn visit_item_mod(&mut self, item_mod: &'ast ItemMod) {
        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: HashMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
            })
            .submodules
            .push(item_mod.ident.to_string());

        syn::visit::visit_item_mod(self, item_mod);
    }
}

/// Extract the traits that are implemented via derive
fn extract_derives(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("derive") {
                attr.parse_args_with(|nested: syn::parse::ParseStream| {
                    nested.parse_terminated(syn::Path::parse, syn::token::Comma)
                })
                .ok()
                .map(|nested| nested.iter().map(format_path).collect::<Vec<String>>())
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

/// Format path
fn format_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| {
            if let PathArguments::None = segment.arguments {
                segment.ident.to_string()
            } else {
                format!("{}<...>", segment.ident)
            }
        })
        .collect::<Vec<_>>()
        .join("::")
}
