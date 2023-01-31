use std::collections::HashMap;

use proc_macro::{Literal, TokenStream};

/// This function takes in a TokenStream
/// and returns a HashMap of the attributes
/// that were specified using the `something = "something"` syntax.
pub fn parse_attrs(attrs: TokenStream) -> HashMap<String, Literal> {
    let mut map = HashMap::new();
    let mut current_ident = None;
    #[derive(PartialEq)]
    enum Expecting {
        Ident,
        Equals,
        Value,
    }
    let mut currently_expecting = Expecting::Ident;
    for token in attrs {
        match token {
            proc_macro::TokenTree::Ident(ident) => {
                if currently_expecting == Expecting::Ident {
                    current_ident = Some(ident.to_string());
                    currently_expecting = Expecting::Equals;
                } else {
                    panic!("Unexpected ident");
                }
            }
            proc_macro::TokenTree::Punct(punct) => {
                if punct.as_char() == '=' {
                    if currently_expecting == Expecting::Equals {
                        currently_expecting = Expecting::Value;
                    } else {
                        panic!("Unexpected equals");
                    }
                } else {
                    // Otherwise, it's a comma, which we ignore
                }
            }
            proc_macro::TokenTree::Literal(literal) => {
                if currently_expecting == Expecting::Value {
                    let ident = current_ident.take().unwrap();
                    map.insert(ident.to_string(), literal);
                    currently_expecting = Expecting::Ident;
                } else {
                    panic!("Unexpected literal");
                }
            }
            proc_macro::TokenTree::Group(_group) => {
                panic!("Unexpected group");
            }
        }
    }
    map
}


/// This function takes a TokenStream that starts with a group in [square brackets],
/// and returns two TokenStreams: one is the contents of that group, and the other is the rest of the input.
/// It returns None if there was no such group at the front.
pub fn extract_attrs(input: TokenStream) -> Option<(TokenStream, TokenStream)> {
    // Try to get the first item of the TokenStream. If there isn't one, bail.
    let first = input.clone().into_iter().next()?;
    // If the first item is a group, check the delimiter.
    if let proc_macro::TokenTree::Group(group) = first {
        if group.delimiter() == proc_macro::Delimiter::Bracket {
            // If it's a square bracket group, return the contents and the rest of the TokenStream.
            let mut rest = input.clone().into_iter();
            rest.next();
            return Some((group.stream(), rest.collect()));
        }
    }
    // Otherwise, return None.
    None
}