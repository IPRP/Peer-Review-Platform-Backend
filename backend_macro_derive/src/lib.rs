extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

// See: https://doc.rust-lang.org/book/ch19-06-macros.html
#[proc_macro_derive(SimpleValidation)]
pub fn simple_validation_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_simple_validation_macro(&ast)
}

fn impl_simple_validation_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl SimpleValidation for #name {}

        impl FromDataSimple for #name {
            type Error = ValidationErrors;

            fn from_data(request: &Request, data: Data) -> Outcome<Self, Self::Error> {
                SimpleValidation::from_data(request, data)
            }
        }
    };
    gen.into()
}
