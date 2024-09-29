//! Modules Overview Visitor
use std::collections::HashMap;
use syn::__private::ToTokens;
use syn::parse::Parse;
use syn::{
    visit::Visit, Attribute, ItemFn, ItemImpl, ItemMod, ItemStruct, PatIdent, PatType,
    PathArguments, ReturnType, TypePath,
};
use syn::{Lit, Meta, Type};

/// Info about one module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub structs: HashMap<String, StructInfo>,
    pub traits: Vec<String>,
    pub submodules: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub description: String, // To store module description
}

/// Info about one struct
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub derives: Vec<String>,
    pub impl_traits: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub description: String, // To store struct description
}

/// Info about one function
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: String,
    pub description: String, // To store function description
}

/// Info about function parameters
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
}

/// Collects information about multiple modules
pub struct ModulesVisitor {
    pub module_map: HashMap<String, ModuleInfo>,
    pub current_module: String,
}

impl<'ast> Visit<'ast> for ModulesVisitor {
    // Collect struct info and description
    fn visit_item_struct(&mut self, item_struct: &'ast ItemStruct) {
        let struct_name = item_struct.ident.to_string();
        let description = extract_doc_comment(&item_struct.attrs);

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: HashMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
                description: String::new(),
            })
            .structs
            .insert(
                struct_name.clone(),
                StructInfo {
                    derives: extract_derives(&item_struct.attrs),
                    impl_traits: Vec::new(),
                    functions: Vec::new(),
                    description,
                },
            );

        syn::visit::visit_item_struct(self, item_struct);
    }

    // Collect function info, description, parameters, and return type
    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        let fn_name = item_fn.sig.ident.to_string();
        let description = extract_doc_comment(&item_fn.attrs);
        let parameters = extract_function_params(&item_fn.sig.inputs);
        let return_type = match &item_fn.sig.output {
            ReturnType::Default => "()".to_string(), // Default return type is `()`
            ReturnType::Type(_, ty) => get_type_name(ty), // Extract the specified return type
        };

        let function_info = FunctionInfo {
            name: fn_name.clone(),
            parameters,
            return_type,
            description,
        };

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: HashMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
                description: String::new(),
            })
            .functions
            .push(function_info.clone());

        if let Some(receiver) = item_fn.sig.receiver() {
            if let syn::Type::Path(type_path) = &*receiver.ty {
                if let Some(segment) = type_path.path.segments.last() {
                    let struct_name = segment.ident.to_string();
                    if let Some(module_info) = self.module_map.get_mut(&self.current_module) {
                        if let Some(struct_info) = module_info.structs.get_mut(&struct_name) {
                            struct_info.functions.push(function_info);
                        }
                    }
                }
            }
        }

        syn::visit::visit_item_fn(self, item_fn);
    }

    // Collect module descriptions
    fn visit_item_mod(&mut self, item_mod: &'ast ItemMod) {
        let description = extract_doc_comment(&item_mod.attrs);

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: HashMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
                description: String::new(),
            })
            .submodules
            .push(item_mod.ident.to_string());

        self.module_map
            .get_mut(&self.current_module)
            .unwrap()
            .description = description;

        syn::visit::visit_item_mod(self, item_mod);
    }

    // Collect trait implementations
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

// A function that takes a Type and returns its string representation
fn get_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            // Convert the path type to string (e.g., `String`, `Option<T>`, etc.)
            type_path.to_token_stream().to_string()
        }
        Type::Reference(type_ref) => {
            // Handle references (e.g., `&String` -> `String`)
            get_type_name(&*type_ref.elem)
        }
        Type::Tuple(tuple_type) => {
            // Handle tuples (e.g., `(i32, String)`)
            let types: Vec<String> = tuple_type.elems.iter().map(get_type_name).collect();
            format!("({})", types.join(", "))
        }
        Type::Slice(slice_type) => {
            // Handle slices (e.g., `[T]`)
            format!("[{}]", get_type_name(&*slice_type.elem))
        }
        // Handle other type variants as necessary
        _ => ty.to_token_stream().to_string(),
    }
    .split("::")
    .last()
    .unwrap()
    .to_string()
}

fn extract_doc_comment(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            match attr.style {
                syn::AttrStyle::Outer => {
                    // Outer attributes are doc comments
                    if let Meta::NameValue(meta) = &attr.meta {
                        // MetaNameValue now has `value` field instead of `lit`
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) = &meta.value
                        {
                            // Return the string content inside the doc comment
                            return Some(lit_str.value());
                        }
                    }
                }
                _ => {}
            }

            // Check if this is a documentation comment (/// or /** */)

            // Parse the meta to extract the literal (the actual comment content)
            if let Meta::NameValue(meta) = &attr.meta {
                // MetaNameValue now has `value` field instead of `lit`
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &meta.value
                {
                    // Return the string content inside the doc comment
                    return Some(lit_str.value());
                }
            }

            None
        })
        .collect::<Vec<String>>()
        .join(" ")
}
fn extract_function_params(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Vec<ParameterInfo> {
    inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(PatType { pat, ty, .. }) = arg {
                if let syn::Pat::Ident(PatIdent { ident, .. }) = &**pat {
                    return Some(ParameterInfo {
                        name: ident.to_string(),
                        param_type: get_type_name(&**ty),
                    });
                }
            }
            None
        })
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
