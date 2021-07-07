extern crate proc_macro2;
extern crate quote;
extern crate syn;

use quote::quote;
use syn::parse::Parse;

pub(super) struct ConnectTasksOptions {
    flow_name: syn::Ident,
    task1_handle_name: syn::Ident,
    task1_oport_num: usize,
    task2_handle_name: syn::Ident,
    task2_iport_num: usize,
}

impl Parse for ConnectTasksOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let flow_ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        let task1_ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let task1_port = input.parse::<syn::LitInt>()?;
        let task1_port_num = task1_port.base10_parse::<usize>()?;
        input.parse::<syn::Token![,]>()?;

        let task2_ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let task2_port = input.parse::<syn::LitInt>()?;
        let task2_port_num = task2_port.base10_parse::<usize>()?;
        input.parse::<syn::Token![,]>()?;

        Ok(ConnectTasksOptions {
            flow_name: flow_ident,
            task1_handle_name: task1_ident,
            task1_oport_num: task1_port_num,
            task2_handle_name: task2_ident,
            task2_iport_num: task2_port_num,
        })
    }
}

pub(super) fn generate_connect_task_ports_snippet(options: &ConnectTasksOptions) -> proc_macro2::TokenStream {
    let ConnectTasksOptions {
        flow_name,
        task1_handle_name,
        task1_oport_num,
        task2_handle_name,
        task2_iport_num,
    } = options;

    let get_output_fname = quote::format_ident!("get_output_{}", task1_oport_num);
    let set_input_fname = quote::format_ident!("set_input_{}", task2_iport_num);

    quote! {{
        #flow_name.connect(#task1_handle_name);
    }}
}
