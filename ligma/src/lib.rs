use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, Pat, Type};

#[proc_macro_attribute]
pub fn http_handler(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let original_handler = rename_original_handler(&input);
    let wrapper_handler_ident = input.sig.ident;
    let original_handler_ident = original_handler.clone().sig.ident;

    let mut args = input.sig.inputs.iter();
    let route = args.next().unwrap();
    let route_pat = extract_arg_pat(route.clone());

    let query = args.next().unwrap();
    let query_pat = extract_arg_pat(query.clone());

    let body = args.next().unwrap();
    let body_type = extract_arg_type(body.clone());

    let return_type = input.sig.output;

    quote! {
        fn #wrapper_handler_ident(#route, #query, body_string: &str) #return_type {
            let body_obj: #body_type = serde_json::from_str(body_string).unwrap();
            return #original_handler_ident(#route_pat, #query_pat, body_obj);
        }

        #original_handler
    }
    .into()
}

fn rename_original_handler(input: &ItemFn) -> ItemFn {
    let mut input_clone = input.clone();
    let fn_ident = &input.sig.ident;
    let new_fn_ident_string = format!("_{}", fn_ident.to_string());
    let new_fn_ident = Ident::new(&new_fn_ident_string, fn_ident.span());
    input_clone.sig.ident = new_fn_ident.clone();

    return input_clone;
}

fn extract_arg_pat(a: FnArg) -> Box<Pat> {
    match a {
        FnArg::Typed(p) => p.pat,
        _ => panic!("Not supported on types with `self`!"),
    }
}

fn extract_arg_type(a: FnArg) -> Box<Type> {
    match a {
        FnArg::Typed(p) => p.ty,
        _ => panic!("Not supported on types with `self`!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
