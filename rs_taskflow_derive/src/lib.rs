mod connect_helper;
mod gen_traits_helper;

use quote::quote;

#[proc_macro]
pub fn generate_task_input_iface_traits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // println!("{:?}", input);
    let options = syn::parse_macro_input!(input as gen_traits_helper::TaskInterfaceOptions);

    let mut result = quote! {};
    for i in 0..options.get_num() {
        let (new_trait, new_trait_param, base_trait, function_ident) =
            gen_traits_helper::generate_iface_trait_components(&options, i);

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
    let options = syn::parse_macro_input!(input as gen_traits_helper::TaskInterfaceOptions);

    let mut result = quote! {};
    for i in 0..options.get_num() {
        let (new_trait, new_trait_param, base_trait, function_ident) =
            gen_traits_helper::generate_iface_trait_components(&options, i);
        result.extend(quote! {
            pub trait #new_trait: #base_trait {
                fn #function_ident(task: &dyn ExecutableTask) -> #new_trait_param;
            }
        });
    }

    result.into()
}

#[proc_macro]
pub fn flow_connect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let options = syn::parse_macro_input!(input as gen_traits_helper::TaskInterfaceOptions);

    let result = quote! {};

    result.into()
}
