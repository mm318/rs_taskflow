extern crate proc_macro2;
extern crate quote;
extern crate syn;

use quote::quote;
use syn::parse::Parse;

pub(crate) struct TaskInterfaceOptions {
    input_types: Vec<syn::Type>,
    output_types: Vec<syn::Type>,
}

impl Parse for TaskInterfaceOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let types = input.parse_terminated(syn::Type::parse, syn::Token![,])?;
        let types_vec: Vec<syn::Type> = types.clone().into_iter().collect();
        if types_vec.len() != 2 {
            return Err(syn::Error::new_spanned(types, "expected two tuples"));
        }

        let input_types = match types_vec.get(0).unwrap() {
            syn::Type::Tuple(tuple) => Ok(tuple.elems.clone()),
            t @ _ => Err(syn::Error::new_spanned(
                t,
                "expected a tuple of input types",
            )),
        }
        .unwrap();

        let output_types = match types_vec.get(1).unwrap() {
            syn::Type::Tuple(tuple) => Ok(tuple.elems.clone()),
            t @ _ => Err(syn::Error::new_spanned(
                t,
                "expected a tuple of output types",
            )),
        }
        .unwrap();

        Ok(TaskInterfaceOptions {
            input_types: input_types.into_iter().collect(),
            output_types: output_types.into_iter().collect(),
        })
    }
}

impl TaskInterfaceOptions {
    pub(crate) fn get_num_inputs(&self) -> usize {
        self.input_types.len()
    }

    pub(crate) fn get_num_outputs(&self) -> usize {
        self.output_types.len()
    }

    fn get_struct_fields(&self) -> proc_macro2::TokenStream {
        let mut struct_fields = quote! {};

        for (i, input_type) in self.input_types.iter().enumerate() {
            let field_name = quote::format_ident!("input{}_handle", i);
            struct_fields.extend(quote! {#field_name: Option<TaskInputHandle<#input_type>>,});
        }
        for (i, output_type) in self.output_types.iter().enumerate() {
            let field_name = quote::format_ident!("output{}", i);
            struct_fields.extend(quote! {#field_name: Option<#output_type>,});
        }
        struct_fields.extend(quote! {func: FuncType});

        struct_fields
    }

    fn get_struct_field_inits(&self) -> proc_macro2::TokenStream {
        let mut struct_field_init = quote! {};

        for i in 0..self.get_num_inputs() {
            let field_name = quote::format_ident!("input{}_handle", i);
            struct_field_init.extend(quote! {#field_name: None,})
        }
        for i in 0..self.get_num_outputs() {
            let field_name = quote::format_ident!("output{}", i);
            struct_field_init.extend(quote! {#field_name: None,})
        }
        struct_field_init.extend(quote! {func: task_func});

        struct_field_init
    }

    fn get_func_signature(&self) -> proc_macro2::TokenStream {
        let mut input_params = quote! {};
        for (i, input_type) in self.input_types.iter().enumerate() {
            if i > 0 {
                input_params.extend(quote! {,});
            }
            input_params.extend(quote! {&#input_type})
        }

        let mut output_params = quote! {};
        for (i, output_type) in self.output_types.iter().enumerate() {
            if i > 0 {
                output_params.extend(quote! {,});
            }
            output_params.extend(quote! {#output_type})
        }

        quote! {Fn(#input_params) -> (#output_params)}
    }
}

pub(crate) struct TaskStructOptions {
    struct_name: syn::Ident,
    struct_visibility: syn::Visibility,
}

impl Parse for TaskStructOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let def = input.parse::<syn::Item>()?;
        let struct_def = match def {
            syn::Item::Struct(s) => Ok(s),
            t @ _ => Err(syn::Error::new_spanned(t, "expected a struct item")),
        }
        .unwrap();

        Ok(TaskStructOptions {
            struct_name: struct_def.ident,
            struct_visibility: struct_def.vis,
        })
    }
}

pub(crate) fn generate_task_struct_impls(
    struct_options: &TaskStructOptions,
    iface_options: &TaskInterfaceOptions,
) -> proc_macro2::TokenStream {
    let TaskStructOptions {
        struct_name,
        struct_visibility: struct_visbility,
    } = struct_options;

    let struct_fields = iface_options.get_struct_fields();
    let mut result = quote! {
        #[derive(Clone)]
        #struct_visbility struct #struct_name<FuncType> {
            #struct_fields
        }
    };

    let struct_field_inits = iface_options.get_struct_field_inits();
    result.extend(quote! {
        impl<FuncType> #struct_name<FuncType> {
            pub fn new(task_func: FuncType) -> Self {
                Self {
                    #struct_field_inits
                }
            }
        }
    });

    let func_signature = iface_options.get_func_signature();

    let mut input_handles = quote! {};
    let mut input_matches = quote! {};
    let mut get_input_vals = quote! {};
    let mut input_params = quote! {};
    for i in 0..iface_options.get_num_inputs() {
        if i > 0 {
            input_handles.extend(quote! {,});
            input_matches.extend(quote! {,});
            input_params.extend(quote! {,});
        }

        let field_name = quote::format_ident!("input{}_handle", i);
        input_handles.extend(quote! {&self.#field_name});

        let input_name = quote::format_ident!("input{}", i);
        input_matches.extend(quote! {Some(#input_name)});

        let value_name = quote::format_ident!("input{}_value", i);
        get_input_vals.extend(quote! {let #value_name = #input_name.get_value(flow);});

        input_params.extend(quote! {#value_name.unwrap()})
    }

    let mut output_params = quote! {};
    let mut set_output_vals = quote! {};
    for i in 0..iface_options.get_num_outputs() {
        if i > 0 {
            output_params.extend(quote! {,});
        }

        let value_name = quote::format_ident!("output{}", i);
        output_params.extend(quote! {#value_name});
        set_output_vals.extend(quote! {self.#value_name = Some(#value_name);})
    }

    result.extend(quote! {
        impl<FuncType: 'static + Clone + Send + Sync + #func_signature> ExecutableTask for #struct_name<FuncType> {
            fn exec(&mut self, flow: &Flow) {
                match (#input_handles) {
                    (#input_matches) => {
                        #get_input_vals
                        let (#output_params) = (self.func)(#input_params);
                        #set_output_vals
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }
        }
    });

    result
}

pub(crate) fn generate_task_input_impl(
    struct_options: &TaskStructOptions,
    iface_options: &TaskInterfaceOptions,
    index: usize,
) -> proc_macro2::TokenStream {
    let TaskStructOptions {
        struct_name,
        struct_visibility: _,
    } = struct_options;
    let TaskInterfaceOptions {
        input_types,
        output_types: _,
    } = iface_options;

    let func_signature = iface_options.get_func_signature();
    let trait_name = quote::format_ident!("TaskInput{}", index);
    let method_name = quote::format_ident!("set_input_{}", index);
    let input_type = input_types.get(index).unwrap();
    let field_name = quote::format_ident!("input{}_handle", index);

    let mut trait_params = quote! {};
    for i in 0..=index {
        if i > 0 {
            trait_params.extend(quote! {,})
        }
        let param_type = input_types.get(i).unwrap();
        trait_params.extend(quote! {#param_type})
    }

    quote! {
        impl<FuncType: 'static + Clone + Send + Sync + #func_signature> #trait_name<#trait_params> for #struct_name<FuncType> {
            fn #method_name(&mut self, task_input: TaskInputHandle<#input_type>) {
                self.#field_name = Some(task_input);
            }
        }
    }
}

pub(crate) fn generate_task_output_impl(
    struct_options: &TaskStructOptions,
    iface_options: &TaskInterfaceOptions,
    index: usize,
) -> proc_macro2::TokenStream {
    let TaskStructOptions {
        struct_name,
        struct_visibility: _,
    } = struct_options;
    let TaskInterfaceOptions {
        input_types: _,
        output_types,
    } = iface_options;

    let func_signature = iface_options.get_func_signature();
    let trait_name = quote::format_ident!("TaskOutput{}", index);
    let method_name = quote::format_ident!("get_output_{}", index);
    let output_type = output_types.get(index).unwrap();
    let field_name = quote::format_ident!("output{}", index);

    let mut trait_params = quote! {};
    for i in 0..=index {
        if i > 0 {
            trait_params.extend(quote! {,})
        }
        let param_type = output_types.get(i).unwrap();
        trait_params.extend(quote! {#param_type})
    }

    quote! {
        impl<FuncType: 'static + Clone + Send + Sync + #func_signature> #trait_name<#trait_params> for #struct_name<FuncType> {
            fn #method_name(task: &dyn ExecutableTask) -> Option<&#output_type> {
                task.as_any()
                    .downcast_ref::<Self>()
                    .unwrap()
                    .#field_name
                    .as_ref()
            }
        }
    }
}
