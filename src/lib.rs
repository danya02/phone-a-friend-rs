extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree, Ident, Span, Group, Delimiter};
use quote::quote_spanned;
use litrs::StringLit;

/// This proc macro allows you to call a friend to help you specify the type of a struct's field.
/// 
/// When the magic type `PhoneAFriend(...)` is used,
/// it will be replaced with u32.
#[proc_macro_attribute]
pub fn phone_a_friend(attr: TokenStream, body: TokenStream) -> TokenStream {
    println!("{attr:?}");
    println!("{body:?}");

    let resp = match replace_magic_type(body) {
        Ok(result) => result,
        Err(error) => error
    };
    println!("Final output: {resp}");
    resp
}

fn replace_magic_type(body: TokenStream) -> Result<TokenStream, TokenStream> {
    let mut tokens: Vec<TokenStream> = vec![];
    enum ParsingState {
        /// Waiting for the magic identifier.
        /// This is the default state.
        WaitingForIdent,
        /// Waiting for a group containing the literal we'd like to extract.
        WaitingForGroup,
    }

    let mut state = ParsingState::WaitingForIdent;

    for item in body {
        println!("Recv: {item:?}");
        match item {

            // If this is a Group, then emit a new Group that has been passed through this function.
            TokenTree::Group(grp) => {
                // If we are not waiting for the phone-a-friend string, then just pass this group through the function again.
                if matches!(state, ParsingState::WaitingForIdent) {
                    tokens.push(TokenTree::Group(
                        Group::new(grp.delimiter(), replace_magic_type(grp.stream())?
                    )).into());
                } else {
                    // Otherwise, it is a group that is expected to contain the phone-a-friend string.
                    // Check that the inner token stream has only one element.

                    let inner = grp.stream();
                    if inner.is_empty() {
                        return Err(quote_spanned! {
                            grp.span().into() => compile_error!("expected the phone-a-friend string, found nothing");
                        }.into())
                    }
                    let mut saw_one_element = false;
                    let mut single_item = None;
                    for grp_item in inner {
                        if !saw_one_element {
                            single_item = Some(grp_item);
                            saw_one_element = true;
                        } else {
                            return Err(quote_spanned! {
                                grp_item.span().into() => compile_error!("expected a single string literal here, found more than one item");
                            }.into())
                        }
                    }

                    let single_item = single_item.unwrap(); // There is definitely something here, because earlier we checked for the empty tokenstream.

                    // Check that the single item we saw is a literal
                    let literal;
                    if let TokenTree::Literal(lit) = single_item {
                        literal = lit;
                    } else {
                        return Err(quote_spanned! {
                            single_item.span().into() => compile_error!("expected a single string literal here");
                        }.into())
                    }

                    match StringLit::try_from(&literal) {
                        Ok(string) => {},
                        Err(_) => {
                            return Err(quote_spanned! {
                                literal.span().into() => compile_error!("expected a string literal here");
                            }.into())
                        }
                    };


    

                    let inner = grp.stream().to_string();
                    println!("Would have asked: {inner}");
                    tokens.push(TokenTree::Ident(Ident::new("u32", grp.span())).into());
                    state = ParsingState::WaitingForIdent;
                }
            },
            // If this is an Ident, and the name is "PhoneAFriend", then we are beginning to parse the magic type.
            TokenTree::Ident(ident) => {
                if !(matches!(state, ParsingState::WaitingForIdent)) {
                    return Err(quote_spanned! {
                        ident.span().into() => compile_error!("expected the phone-a-friend string at this position, but found identifier");
                    }.into())
                }

                if ident.to_string() == "PhoneAFriend" {
                    println!("FOUND MAGIC");
                    state = ParsingState::WaitingForGroup;
                } else {
                    println!("Adding ident: {ident}");
                    tokens.push(TokenTree::Ident(ident).into());
                }
            }
            _ => {
                // If we aren't currently waiting for a group, then just emit this as is.
                if matches!(state, ParsingState::WaitingForIdent) {
                    tokens.push(item.into());
                } else {
                    return Err(quote_spanned! {
                        item.span().into() => compile_error!("expected the phone-a-friend string at this position, but found punctuation or literal");
                    }.into())
                }
            },
        }
    }

    let mut out = TokenStream::new();
    out.extend(tokens.into_iter());
    println!("Emitting: {out}");
    Ok(out)
}