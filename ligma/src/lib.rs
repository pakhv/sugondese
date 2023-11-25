use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, Ident, Pat};

#[proc_macro_attribute]
pub fn get(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let mut input_clone = input.clone();
    let fn_ident = input.sig.ident;
    let new_fn_ident_string = format!("{}_1", fn_ident.to_string());
    let new_fn_ident = Ident::new(&new_fn_ident_string, fn_ident.span());
    input_clone.sig.ident = new_fn_ident.clone();

    let mut args = input.sig.inputs.iter();
    let route = args.next().unwrap();
    let route_pat = extract_arg_pat(route.clone());

    let query = args.next().unwrap();
    let query_pat = extract_arg_pat(query.clone());
    //let body = args.next();

    //let block = input.block;

    quote! {
        #input_clone

        pub fn #fn_ident(#route, #query) -> () {
            #new_fn_ident(#route_pat, #query_pat);
            println!("hello from macro");
        }
    }
    .into()
}

fn extract_arg_pat(a: FnArg) -> Box<Pat> {
    match a {
        FnArg::Typed(p) => p.pat,
        _ => panic!("Not supported on types with `self`!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
