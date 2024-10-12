use std::collections::BTreeMap;
use syn::__private::ToTokens;
use syn::parse::Parse;
use syn::visit::Visit;
use syn::{
    Attribute, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lit, Meta, PatIdent,
    PatType, PathArguments, ReturnType, TraitItem, TraitItemFn, Type, TypePath,
};

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub structs: BTreeMap<String, StructInfo>,
    pub enums: BTreeMap<String, EnumInfo>,
    pub traits: Vec<TraitInfo>,
    pub submodules: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub name: String,
    pub functions: Vec<FunctionInfo>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub derives: Vec<String>,
    pub impl_traits: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub members: Vec<MemberInfo>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub variants: Vec<String>,
    pub derives: Vec<String>,
    pub impl_traits: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub public: bool,
    pub return_type: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub name: String,
    pub member_type: String,
}

pub struct ModulesVisitor {
    pub module_map: BTreeMap<String, ModuleInfo>,
    pub current_module: String,
}

impl<'ast> Visit<'ast> for ModulesVisitor {
    fn visit_item_struct(&mut self, item_struct: &'ast ItemStruct) {
        let struct_name = item_struct.ident.to_string();
        let description = extract_doc_comment(&item_struct.attrs);
        let members = extract_struct_members(&item_struct.fields, &self.current_module);

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: BTreeMap::new(),
                enums: BTreeMap::new(),
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
                    members,
                    description,
                },
            );

        syn::visit::visit_item_struct(self, item_struct);
    }

    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        if let syn::Type::Path(TypePath { path, .. }) = &*item_impl.self_ty {
            if let Some(struct_name) = path.get_ident() {
                let struct_name = struct_name.to_string();
                let module_info = self
                    .module_map
                    .entry(self.current_module.clone())
                    .or_insert_with(|| ModuleInfo {
                        structs: BTreeMap::new(),
                        enums: BTreeMap::new(),
                        traits: Vec::new(),
                        submodules: Vec::new(),
                        functions: Vec::new(),
                        description: String::new(),
                    });

                // Retrieve or create the StructInfo
                let struct_info = module_info
                    .structs
                    .entry(struct_name.clone())
                    .or_insert_with(|| StructInfo {
                        derives: Vec::new(),
                        impl_traits: Vec::new(),
                        functions: Vec::new(),
                        members: Vec::new(),
                        description: String::new(),
                    });

                // Check if this impl block implements a trait
                if let Some((_, trait_path, _)) = &item_impl.trait_ {
                    // Trait-based impl: Add the trait name
                    let trait_name = format_path(trait_path);
                    struct_info.impl_traits.push(trait_name);

                    // Functions in trait impl are public by default
                    for item in &item_impl.items {
                        if let syn::ImplItem::Fn(method) = item {
                            let fn_name = method.sig.ident.to_string();
                            let parameters =
                                extract_function_params(&method.sig.inputs, &self.current_module);
                            let return_type = match &method.sig.output {
                                ReturnType::Default => "()".to_string(),
                                ReturnType::Type(_, ty) => get_type_name(ty, &self.current_module),
                            };

                            let function_info = FunctionInfo {
                                name: fn_name,
                                parameters,
                                public: true, // Trait impl functions are public
                                return_type,
                                description: extract_doc_comment(&method.attrs),
                            };

                            // Add function to the struct's function list
                            struct_info.functions.push(function_info);
                        }
                    }
                } else {
                    // Regular impl block (no trait): Functions can be private or public
                    for item in &item_impl.items {
                        if let syn::ImplItem::Fn(method) = item {
                            let fn_name = method.sig.ident.to_string();
                            let parameters =
                                extract_function_params(&method.sig.inputs, &self.current_module);
                            let return_type = match &method.sig.output {
                                ReturnType::Default => "()".to_string(),
                                ReturnType::Type(_, ty) => get_type_name(ty, &self.current_module),
                            };

                            // Check if the function is public or private
                            let public = matches!(method.vis, syn::Visibility::Public(_));

                            let function_info = FunctionInfo {
                                name: fn_name,
                                parameters,
                                public,
                                return_type,
                                description: extract_doc_comment(&method.attrs),
                            };

                            // Add function to the struct's function list
                            struct_info.functions.push(function_info);
                        }
                    }
                }
            }
        }

        syn::visit::visit_item_impl(self, item_impl);
    }

    fn visit_item_enum(&mut self, item_enum: &'ast ItemEnum) {
        let enum_name = item_enum.ident.to_string();
        let description = extract_doc_comment(&item_enum.attrs);
        let variants = extract_enum_variants(&item_enum);

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: BTreeMap::new(),
                enums: BTreeMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
                description: String::new(),
            })
            .enums
            .insert(
                enum_name.clone(),
                EnumInfo {
                    variants,
                    derives: extract_derives(&item_enum.attrs),
                    impl_traits: Vec::new(),
                    description,
                },
            );

        syn::visit::visit_item_enum(self, item_enum);
    }

    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        let fn_name = item_fn.sig.ident.to_string();
        let description = extract_doc_comment(&item_fn.attrs);
        let parameters = extract_function_params(&item_fn.sig.inputs, &self.current_module);
        let return_type = match &item_fn.sig.output {
            ReturnType::Default => "()".to_string(),
            ReturnType::Type(_, ty) => get_type_name(ty, &self.current_module),
        };

        let public = matches!(item_fn.vis, syn::Visibility::Public(_));

        let function_info = FunctionInfo {
            name: fn_name.clone(),
            parameters,
            public,
            return_type,
            description,
        };

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: BTreeMap::new(),
                enums: BTreeMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
                description: String::new(),
            })
            .functions
            .push(function_info.clone());

        syn::visit::visit_item_fn(self, item_fn);
    }

    fn visit_item_mod(&mut self, item_mod: &'ast ItemMod) {
        let description = extract_doc_comment(&item_mod.attrs);

        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: BTreeMap::new(),
                enums: BTreeMap::new(),
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

    fn visit_item_trait(&mut self, item_trait: &'ast ItemTrait) {
        let trait_name = item_trait.ident.to_string();
        let description = extract_doc_comment(&item_trait.attrs);

        let mut functions = Vec::new();

        // Extract functions from trait items
        for item in &item_trait.items {
            if let TraitItem::Fn(TraitItemFn { sig, attrs, .. }) = item {
                let fn_name = sig.ident.to_string();
                let parameters = extract_function_params(&sig.inputs, &self.current_module);
                let return_type = match &sig.output {
                    ReturnType::Default => "()".to_string(),
                    ReturnType::Type(_, ty) => get_type_name(ty, &self.current_module),
                };

                let public = true; // Trait methods are generally public unless specified otherwise

                let description = extract_doc_comment(attrs);

                functions.push(FunctionInfo {
                    name: fn_name,
                    parameters,
                    public,
                    return_type,
                    description,
                });
            }
        }

        // Store trait-related information (like functions)
        let trait_info = TraitInfo {
            name: trait_name,
            functions,
            description,
        };

        // Add the trait and its functions to the module map
        self.module_map
            .entry(self.current_module.clone())
            .or_insert_with(|| ModuleInfo {
                structs: BTreeMap::new(),
                enums: BTreeMap::new(),
                traits: Vec::new(),
                submodules: Vec::new(),
                functions: Vec::new(),
                description: String::new(),
            })
            .traits
            .push(trait_info.clone());

        syn::visit::visit_item_trait(self, item_trait);
    }
}

fn extract_struct_members(fields: &syn::Fields, current_module: &str) -> Vec<MemberInfo> {
    fields
        .iter()
        .filter_map(|field| {
            let name = field
                .ident
                .as_ref()
                .map_or("<unnamed>".to_string(), |ident| ident.to_string());
            let member_type = get_type_name(&field.ty, current_module); // Fully qualified name
            Some(MemberInfo { name, member_type })
        })
        .collect()
}

fn extract_enum_variants(item_enum: &syn::ItemEnum) -> Vec<String> {
    item_enum
        .variants
        .iter()
        .map(|variant| variant.ident.to_string())
        .collect()
}

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

fn get_type_name(ty: &Type, current_module: &str) -> String {
    match ty {
        Type::Path(type_path) => type_path.to_token_stream().to_string(),
        Type::Reference(type_ref) => format!("&{}", get_type_name(&*type_ref.elem, current_module)),
        Type::Tuple(tuple_type) => {
            let types: Vec<String> = tuple_type
                .elems
                .iter()
                .map(|elem| get_type_name(elem, current_module))
                .collect();
            format!("({})", types.join(", "))
        }
        Type::Slice(slice_type) => {
            format!("[{}]", get_type_name(&*slice_type.elem, current_module))
        }
        _ => ty.into_token_stream().to_string(),
    }
}

fn extract_doc_comment(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            match attr.style {
                syn::AttrStyle::Outer => {
                    if let Meta::NameValue(meta) = &attr.meta {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) = &meta.value
                        {
                            return Some(lit_str.value());
                        }
                    }
                }
                _ => {}
            }
            None
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn extract_function_params(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    current_module: &str,
) -> Vec<ParameterInfo> {
    inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(PatType { pat, ty, .. }) = arg {
                if let syn::Pat::Ident(PatIdent { ident, .. }) = &**pat {
                    return Some(ParameterInfo {
                        name: ident.to_string(),
                        param_type: get_type_name(&**ty, current_module),
                    });
                }
            }
            None
        })
        .collect()
}

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
