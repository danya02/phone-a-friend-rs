extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree, Ident, Span};

/// This proc macro allows you to call a friend to help you specify the type of a struct's field.
/// 
/// When the magic type `PhoneAFriend(...)` is used,
/// it will be replaced with u32.
#[proc_macro_attribute]
pub fn phone_a_friend(attr: TokenStream, body: TokenStream) -> TokenStream {
    println!("{:?}", attr);
    println!("{:?}", body);

    let mut tokens: Vec<TokenStream> = vec![];

    for item in body {
        tokens.push(item.into());
    }

    let mut out = TokenStream::new();
    out.extend(tokens.into_iter());
    out
}