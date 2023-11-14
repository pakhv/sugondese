use proc_macro::TokenStream;

#[proc_macro]
pub fn get(input: TokenStream) -> TokenStream {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
