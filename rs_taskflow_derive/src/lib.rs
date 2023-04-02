mod derive_task_helper;
mod gen_connect_tasks_helper;
mod gen_task_output_helper;
mod gen_task_traits_helper;

use quote::quote;

#[proc_macro]
pub fn generate_task_input_iface_traits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // println!("{:?}", input);
    let options = syn::parse_macro_input!(input as gen_task_traits_helper::TaskTraitOptions);

    let mut result = quote! {};
    for i in 0..options.get_num() {
        let (new_trait, new_trait_param, base_trait, function_ident) =
            gen_task_traits_helper::generate_iface_trait_components(&options, i);

        result.extend(quote! {
            pub trait #new_trait: #base_trait {
                fn #function_ident(&mut self, task_input: TaskInputHandle<#new_trait_param>);
            }
        });
    }

    result.into()
}

#[proc_macro]
pub fn generate_task_output_iface_traits(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let options = syn::parse_macro_input!(input as gen_task_traits_helper::TaskTraitOptions);

    let mut result = quote! {};
    for i in 0..options.get_num() {
        let (new_trait, new_trait_param, base_trait, function_ident) =
            gen_task_traits_helper::generate_iface_trait_components(&options, i);
        result.extend(quote! {
            pub trait #new_trait: #base_trait {
                fn #function_ident(task: &dyn ExecutableTask) -> Option<&#new_trait_param>;
            }
        });
    }

    result.into()
}

#[proc_macro]
pub fn generate_connect_tasks_funcs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let options = syn::parse_macro_input!(input as gen_connect_tasks_helper::TaskFlowOptions);

    let mut result = quote! {};
    for i in 0..options.get_num_ports() {
        for j in 0..options.get_num_ports() {
            let func = gen_connect_tasks_helper::generate_connect_tasks_func(i, j);
            result.extend(func);
        }
    }

    result.into()
}

#[proc_macro]
pub fn generate_get_task_output_funcs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let options = syn::parse_macro_input!(input as gen_connect_tasks_helper::TaskFlowOptions);

    let mut result = quote! {};
    for i in 0..options.get_num_ports() {
        let func = gen_task_output_helper::generate_get_task_output_func(i);
        result.extend(func);
    }

    result.into()
}

#[proc_macro_attribute]
pub fn derive_task(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let struct_options = syn::parse_macro_input!(input as derive_task_helper::TaskStructOptions);
    let iface_options = syn::parse_macro_input!(args as derive_task_helper::TaskInterfaceOptions);

    let mut result = quote! {};
    result.extend(derive_task_helper::generate_task_struct_impls(
        &struct_options,
        &iface_options,
    ));
    for i in 0..iface_options.get_num_inputs() {
        result.extend(derive_task_helper::generate_task_input_impl(
            &struct_options,
            &iface_options,
            i,
        ));
    }
    for i in 0..iface_options.get_num_outputs() {
        result.extend(derive_task_helper::generate_task_output_impl(
            &struct_options,
            &iface_options,
            i,
        ));
    }

    // println!("output: {}", result);
    result.into()
}
