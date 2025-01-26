use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, PatType};

#[proc_macro_attribute]
pub fn json_rpc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let handler_name = format_ident!("{}Handler", fn_name);

    // Get the method name - either from attribute or function name
    let method_name = if attr.is_empty() {
        fn_name.to_string()
    } else {
        attr.to_string().replace('"', "")
    };

    // Extract the parameter type from the function signature
    let param_type = match &input.sig.inputs.first() {
        Some(FnArg::Typed(PatType { ty, .. })) => ty,
        _ => panic!("Expected exactly one parameter"),
    };

    let expanded = quote! {
        #input

        pub struct #handler_name;

        #[async_trait]
        impl RpcHandler for #handler_name {
            async fn handle(&self, params: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
                let params: #param_type = serde_json::from_value(params)?;
                let result = #fn_name(params).await?;
                Ok(serde_json::to_value(result)?)
            }
        }

        #[allow(unused_variables)]
        const _: () = {
            get_registry()
                .lock()
                .unwrap()
                .register(#method_name, #handler_name);
        };
    };

    TokenStream::from(expanded)
}
