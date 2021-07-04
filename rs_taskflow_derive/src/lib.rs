extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;

struct TaskInterfaceOptions {
    trait_name: syn::Ident,
    function_name: syn::Ident,
    num: u32,
}

impl Parse for TaskInterfaceOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let ident1 = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let ident2 = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let n_lit = input.parse::<syn::LitInt>()?;
        let n = n_lit.base10_parse::<u32>()?;
        Ok(TaskInterfaceOptions {
            trait_name: ident1,
            function_name: ident2,
            num: n,
        })
    }
}

#[proc_macro]
pub fn generate_task_input_iface_traits(input: TokenStream) -> TokenStream {
    // println!("{:?}", input);

    let TaskInterfaceOptions {
        trait_name,
        function_name,
        num,
    } = syn::parse_macro_input!(input as TaskInterfaceOptions);

    let mut result = quote! {};
    for i in 0..num {
        let mut new_trait_param = quote::format_ident!("I{}", i);
        let new_trait = {
            let new_trait_name = quote::format_ident!("{}{}", trait_name, i);

            let mut new_trait_params = quote! {};
            for j in 0..=i {
                if j > 0 {
                    new_trait_params.extend(quote! {,});
                }
                new_trait_param = quote::format_ident!("I{}", j);
                new_trait_params.extend(quote! {#new_trait_param});
            }

            quote! {#new_trait_name<#new_trait_params>}
        };

        let base_trait = if i == 0 {
            let base_trait_name = quote::format_ident!("{}", "ExecutableTask");
            quote! {#base_trait_name}
        } else {
            let base_trait_name = quote::format_ident!("{}{}", trait_name, i - 1);

            let mut base_trait_params = quote! {};
            for j in 0..i {
                if j > 0 {
                    base_trait_params.extend(quote! {,});
                }
                let param_ident = quote::format_ident!("I{}", j);
                base_trait_params.extend(quote! {#param_ident});
            }

            quote! {#base_trait_name<#base_trait_params>}
        };

        let function_name = quote::format_ident!("{}_{}", function_name, i);

        result.extend(quote! {
            pub trait #new_trait: #base_trait {
                fn #function_name(&mut self, task_input: TaskInputHandle<#new_trait_param>);
            }
        });
    }

    result.into()
}

#[proc_macro]
pub fn generate_task_output_iface_traits(input: TokenStream) -> TokenStream {
    let TaskInterfaceOptions {
        trait_name,
        function_name,
        num,
    } = syn::parse_macro_input!(input as TaskInterfaceOptions);

    let mut result = quote! {};
    for i in 0..num {
        let mut new_trait_param = quote::format_ident!("I{}", i);
        let new_trait = {
            let new_trait_name = quote::format_ident!("{}{}", trait_name, i);

            let mut new_trait_params = quote! {};
            for j in 0..=i {
                if j > 0 {
                    new_trait_params.extend(quote! {,});
                }
                new_trait_param = quote::format_ident!("I{}", j);
                new_trait_params.extend(quote! {#new_trait_param});
            }

            quote! {#new_trait_name<#new_trait_params>}
        };

        let base_trait = if i == 0 {
            let base_trait_name = quote::format_ident!("{}", "ExecutableTask");
            quote! {#base_trait_name}
        } else {
            let base_trait_name = quote::format_ident!("{}{}", trait_name, i - 1);

            let mut base_trait_params = quote! {};
            for j in 0..i {
                if j > 0 {
                    base_trait_params.extend(quote! {,});
                }
                let param_ident = quote::format_ident!("I{}", j);
                base_trait_params.extend(quote! {#param_ident});
            }

            quote! {#base_trait_name<#base_trait_params>}
        };

        let function_name = quote::format_ident!("{}_{}", function_name, i);

        result.extend(quote! {
            pub trait #new_trait: #base_trait {
                fn #function_name(task: &dyn ExecutableTask) -> #new_trait_param;
            }
        });
    }

    result.into()
}
