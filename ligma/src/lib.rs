use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, FnArg, Ident, ItemFn, LitStr, Pat,
    Type,
};

#[derive(Clone)]
struct FnArgInfo {
    name: String,
    arg: FnArg,
}

const ROUTE_NAME: &str = "Route";
const QUERY_NAME: &str = "Query";

#[proc_macro_attribute]
pub fn http_handler(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as LitStr);
    let route = args.value();

    let input = parse_macro_input!(item as syn::ItemFn);

    let original_handler = rename_original_handler(&input);
    let handler_ident = input.sig.ident.clone();
    let wrapper_handler_ident = format_ident!("{}_wrapper", input.sig.ident.clone());
    let original_handler_ident = original_handler.clone().sig.ident;

    let args = input.sig.inputs;
    let args = get_args_types_names(&args);

    let route_arg = quote! { route: Route }.into();
    let route_arg = parse_macro_input!(route_arg as syn::FnArg);

    let query_arg = quote! { query: Query }.into();
    let query_arg = parse_macro_input!(query_arg as syn::FnArg);

    let args_quote = build_args_quote(&args, route_arg, query_arg);
    let body_quote = get_body_quote(&args);

    let response_mapping = map_response();

    if body_quote.is_none() {
        return quote! {
            fn #handler_ident() -> sugondese::http_handler_info::HttpHandlerInfo {
                return sugondese::http_handler_info::HttpHandlerInfo {
                    handler: Box::new(#wrapper_handler_ident),
                    route: #route.to_string(),
                };
            }

            fn #wrapper_handler_ident(route: Route, query: Query, _body_string: Option<String>) -> sugondese::http_response::HttpResponse {
                let result = #original_handler_ident(#args_quote);

                #response_mapping
            }

            #original_handler
        }
        .into();
    }

    quote! {
        fn #handler_ident() -> sugondese::http_handler_info::HttpHandlerInfo {
            return sugondese::http_handler_info::HttpHandlerInfo {
                handler: Box::new(#wrapper_handler_ident),
                route: #route.to_string(),
            };
        }

        fn #wrapper_handler_ident(route: Route, query: Query, body_string: Option<String>) -> sugondese::http_response::HttpResponse {
            if body_string.is_none() {
                return sugondese::http_response::HttpResponse {
                    status: sugondese::http_response::HttpStatus::BadRequest,
                    body: Some("body missing".to_string())
                };
            }

            #body_quote
            let result = #original_handler_ident(#args_quote);

            #response_mapping
        }

        #original_handler
    }
    .into()
}

fn map_response() -> proc_macro2::TokenStream {
    quote! {
        let body_string = if result.data.is_some() {
            Some(serde_json::to_string(&result.data.unwrap()).unwrap())
        }
        else {
            None
        };

        sugondese::http_response::HttpResponse {
            status: result.status,
            body: body_string
        }
    }
}

fn get_body_quote(args_types_names: &Vec<FnArgInfo>) -> Option<proc_macro2::TokenStream> {
    let body_arg_type_name = args_types_names
        .iter()
        .find(|&a| a.name != ROUTE_NAME && a.name != QUERY_NAME);

    if body_arg_type_name.is_none() {
        return None;
    }

    let body_type = extract_arg_type(body_arg_type_name.unwrap().arg.clone());

    Some(quote! {
        let body_obj: serde_json::Result<#body_type> = serde_json::from_str(&body_string.unwrap());

        if body_obj.is_err() {
            return sugondese::http_response::HttpResponse {
                status: sugondese::http_response::HttpStatus::BadRequest,
                body: Some("Unable to deserialize body".to_string())
            };
        }

        let body_obj = body_obj.unwrap();
    })
}

fn build_args_quote(
    args_types_names: &Vec<FnArgInfo>,
    route_arg: FnArg,
    query_arg: FnArg,
) -> proc_macro2::TokenStream {
    let mut result = quote! {};

    if args_types_names.len() == 0 {
        return result;
    }

    let body_args_count = args_types_names
        .iter()
        .filter(|a| a.name != ROUTE_NAME && a.name != QUERY_NAME)
        .count();

    if body_args_count > 1 {
        panic!("Too many body parameters");
    }

    for (idx, arg_info) in args_types_names.iter().enumerate() {
        let arg = get_arg_quote(arg_info.clone(), route_arg.clone(), query_arg.clone());

        result = if idx == 0 {
            quote! { #arg }
        } else {
            quote! { #result, #arg }
        }
    }

    result
}

fn get_arg_quote(
    arg_info: FnArgInfo,
    route_arg: FnArg,
    query_arg: FnArg,
) -> proc_macro2::TokenStream {
    match &arg_info.name {
        name if name == &ROUTE_NAME => {
            let pat = extract_arg_pat(route_arg);
            return quote! { #pat };
        }
        name if name == &QUERY_NAME => {
            let pat = extract_arg_pat(query_arg);
            return quote! { #pat };
        }
        _ => quote! { body_obj },
    }
}

//fn get_body_fn_arg()

fn get_args_types_names(args: &Punctuated<FnArg, Comma>) -> Vec<FnArgInfo> {
    let mut args_types_names: Vec<FnArgInfo> = vec![];

    for arg in args {
        args_types_names.push(FnArgInfo {
            name: get_fn_arg_type(arg),
            arg: arg.clone(),
        })
    }

    args_types_names
}

fn rename_original_handler(input: &ItemFn) -> ItemFn {
    let mut input_clone = input.clone();
    let fn_ident = &input.sig.ident;
    let new_fn_ident_string = format!("_{}", fn_ident.to_string());
    let new_fn_ident = Ident::new(&new_fn_ident_string, fn_ident.span());
    input_clone.sig.ident = new_fn_ident.clone();

    return input_clone;
}

fn get_fn_arg_type(arg: &FnArg) -> String {
    let arg_type = extract_arg_type(arg.clone());

    let path = match *arg_type {
        Type::Path(val) => val.path,
        _ => panic!("Arg type not implemented"),
    };
    let segment = path.segments.iter().next().unwrap();

    segment.ident.to_string()
}

fn extract_arg_pat(a: FnArg) -> Box<Pat> {
    match a {
        FnArg::Typed(p) => p.pat,
        _ => panic!("Not supported argument type"),
    }
}

fn extract_arg_type(a: FnArg) -> Box<Type> {
    match a {
        FnArg::Typed(p) => p.ty,
        _ => panic!("Not supported argument type"),
    }
}
