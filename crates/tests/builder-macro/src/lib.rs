use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(HelloBuilder)]
pub fn builder_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    crate::impl_hello_builder(&ast)
}

fn impl_hello_builder(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = &ast.data;
    let gen = quote! {
        impl HelloBuilder for #name {
            fn hello_builder() {
                println!("Hello, Builder! My name is {}!", stringify!(#name));
            }
        }
    };
    gen.into()
}

macro_rules! make_public{
    (
     $vis:vis struct $struct_name:ident {
        $(
        // vis for field visibility, ident for field name and ty for field data type
        $field_vis:vis $field_name:ident : $field_type:ty
        ),*
    }
    ) => {
        {
            pub struct $struct_name{
                $(
                pub $field_name : $field_type,
                )*
            }
        }
    }
}