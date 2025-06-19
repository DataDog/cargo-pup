// Test proc macro crate for cargo-pup proc macro detection
use proc_macro::TokenStream;

// Function-like proc macro - should trigger lint
#[proc_macro]
pub fn forbidden_proc_macro(input: TokenStream) -> TokenStream {
    input
}

// Attribute proc macro - should trigger lint
#[proc_macro_attribute]
pub fn forbidden_attr_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// Derive proc macro - should trigger lint
#[proc_macro_derive(ForbiddenDerive)]
pub fn forbidden_derive_macro(input: TokenStream) -> TokenStream {
    input
}