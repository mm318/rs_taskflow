extern crate proc_macro2;
extern crate quote;
extern crate syn;

use quote::quote;

pub(super) fn generate_get_task_output_func(oport_num: usize) -> proc_macro2::TokenStream {
    let func_name = quote::format_ident!("get_task_output{}", oport_num);
    let output_trait_name = quote::format_ident!("TaskOutput{}", oport_num);
    let output_func_name = quote::format_ident!("get_output_{}", oport_num);

    let mut output_trait_params = quote! {};
    for j in 0..oport_num {
        let output_trait_param = quote::format_ident!("O{}", j);
        output_trait_params.extend(quote! {#output_trait_param});
        output_trait_params.extend(quote! {,});
    }

    quote! {
        pub fn #func_name<
                #output_trait_params
                O: 'static,
                T: #output_trait_name<#output_trait_params O>,
        >(
            &self,
            task_handle: &TaskHandle<T>,
        ) -> Option<&O> {
            let read_handle = self.flow.get_task(task_handle);
            let val_ref = T::#output_func_name(read_handle.borrow());
            let val_ptr: *const O = val_ref.unwrap();
            unsafe { Some(&*val_ptr) }
        }
    }
}
