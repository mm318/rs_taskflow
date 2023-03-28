extern crate proc_macro2;
extern crate quote;
extern crate syn;

use quote::quote;
use syn::parse::Parse;

pub(super) struct ConnectTasksFuncsOptions {
    num_ports: usize,
}

impl ConnectTasksFuncsOptions {
    pub(super) fn get_num_ports(&self) -> usize {
        self.num_ports
    }
}

impl Parse for ConnectTasksFuncsOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let num_lit = input.parse::<syn::LitInt>()?;
        let num = num_lit.base10_parse::<usize>()?;
        Ok(ConnectTasksFuncsOptions { num_ports: num })
    }
}

pub(super) fn generate_connect_tasks_func(
    task1_oport_num: usize,
    task2_iport_num: usize,
) -> proc_macro2::TokenStream {
    let func_name = quote::format_ident!(
        "connect_output{}_to_input{}",
        task1_oport_num,
        task2_iport_num
    );
    let output_trait_name = quote::format_ident!("TaskOutput{}", task1_oport_num);
    let input_trait_name = quote::format_ident!("TaskInput{}", task2_iport_num);
    let output_func_name = quote::format_ident!("get_output_{}", task1_oport_num);
    let input_func_name = quote::format_ident!("set_input_{}", task2_iport_num);

    let mut output_trait_params = quote! {};
    for j in 0..task1_oport_num {
        let output_trait_param = quote::format_ident!("O{}", j);
        output_trait_params.extend(quote! {#output_trait_param});
        output_trait_params.extend(quote! {,});
    }

    let mut input_trait_params = quote! {};
    for j in 0..task2_iport_num {
        let input_trait_param = quote::format_ident!("I{}", j);
        input_trait_params.extend(quote! {#input_trait_param});
        input_trait_params.extend(quote! {,});
    }

    quote! {
        pub fn #func_name<
                #output_trait_params
                #input_trait_params
                T: 'static,
                A: #output_trait_name<#output_trait_params T>,
                B: #input_trait_name<#input_trait_params T>,
        >(
            &mut self,
            task1_handle: &TaskHandle<A>,
            task2_handle: &TaskHandle<B>,
        ) {
            self.connect(task1_handle, A::#output_func_name, task2_handle, B::#input_func_name);
        }
    }
}
