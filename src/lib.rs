extern crate proc_macro;
use litrs::StringLit;
use proc_macro::{Group, Ident, TokenStream, TokenTree};
use quote::{quote, quote_spanned};

mod error;
mod parse_attrs;
mod telegram;
use crate::telegram::TelegramParams;
use std::time::Duration;
use crate::telegram::ask_friend_via_tg;

/// This proc macro allows you to call a friend to help you specify the type of a struct's field.
///
/// When the magic type `PhoneAFriend(...)` is used,
/// it will be replaced with u32.
#[proc_macro]
pub fn phone_a_friend_telegram(body: TokenStream) -> TokenStream {
    let maybe_attr = parse_attrs::extract_attrs(body);
    if maybe_attr.is_none() {
        return quote! {
            compile_error!("first item must be an attribute group (put square brackets at the beginning of the macro invocation)");
        }
        .into();
    }

    let (attr, body) = maybe_attr.unwrap();

    // Assert that there must be some attributes.
    if attr.is_empty() {
        return quote! {
            compile_error!("expected attributes: `token` and `chat_id`");
        }
        .into();
    }


    let attrs = parse_attrs::parse_attrs(attr);

    println!("{attrs:?}");

    // Assert that there must be attributes:
    // - token: a string,
    // - chat_id: an integer
    let token: String;
    let chat_id: i64;
    match attrs.get("token") {
        None => {
            return quote! {
                compile_error!("expected attribute `token`");
            }
            .into()
        }
        Some(lit) => {
            token = match StringLit::try_from(lit) {
                Ok(string) => string.into_value().to_string(),
                Err(_) => return quote_spanned! {
                    lit.span().into() => compile_error!("expected a string literal for the token");
                }
                .into(),
            };
            println!("Token: {token}");
        }
    }

    match attrs.get("chat_id") {
        None => {
            return quote! {
                compile_error!("expected attribute `chat_id`");
            }
            .into()
        },
        Some(lit) => {
            chat_id = match lit.to_string().parse::<i64>() {
                Ok(id) => id,
                Err(_) => return quote_spanned! {
                    lit.span().into() => compile_error!("expected an integer literal for the chat_id");
                }
                .into(),
            };
            println!("Chat ID: {chat_id}");
        }
    }

    let mut telegram_params = TelegramParams {
        token, chat_id, is_token_valid: false, response_timeout: Duration::from_secs(60),
    };

    let resp = match replace_magic_type(body, &mut telegram_params) {
        Ok(result) => result,
        Err(error) => error,
    };
    println!("Final output: {resp}");
    resp
}

fn replace_magic_type(
    body: TokenStream,
    params: &mut TelegramParams,
) -> Result<TokenStream, TokenStream> {
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
        match item {
            // If this is a Group, then emit a new Group that has been passed through this function.
            TokenTree::Group(grp) => {
                // If we are not waiting for the phone-a-friend string, then just pass this group through the function again.
                if matches!(state, ParsingState::WaitingForIdent) {
                    tokens.push(
                        TokenTree::Group(Group::new(
                            grp.delimiter(),
                            replace_magic_type(grp.stream(), params)?,
                        ))
                        .into(),
                    );
                } else {
                    // Otherwise, it is a group that is expected to contain the phone-a-friend string.
                    // Check that the inner token stream has only one element.

                    let inner = grp.stream();
                    if inner.is_empty() {
                        return Err(quote_spanned! {
                            grp.span().into() => compile_error!("expected the phone-a-friend string, found nothing");
                        }.into());
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
                            }.into());
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
                        }.into());
                    }

                    let question;
                    match StringLit::try_from(&literal) {
                        Ok(string) => {
                            question = string.into_value();
                        },
                        Err(_) => {
                            return Err(quote_spanned! {
                                literal.span().into() => compile_error!("expected a string literal here");
                            }.into())
                        }
                    };
                    
                    let type_ident = ask_friend_via_tg(params, &question);
                    println!("Got answer: {type_ident:?}");
                    match type_ident {
                        Ok(value) => {
                            tokens.push(TokenTree::Ident(Ident::new(&value, grp.span())).into());
                            state = ParsingState::WaitingForIdent;
                        }, Err(error) => {
                            println!("Error while phoning friend: {:?}", error);
                            use crate::error::AskAFriendError::*;
                            let how_error = match error {
                                NetworkError(_) => Err(quote_spanned! {grp.span().into() => compile_error!("failed to phone a friend: error communicating with the network");}.into()),
                                SendMessageError => Err(quote_spanned! {grp.span().into() => compile_error!("failed to phone a friend: error sending messages");}.into()),
                                TokenInvalid => Err(quote_spanned! {grp.span().into() => compile_error!("failed to phone a friend: the provided token could not be used");}.into()),
                                UnknownChatId => Err(quote_spanned! {grp.span().into() => compile_error!("failed to phone a friend: the chat_id is not known to the bot");}.into()),
                                ChatClosed => Err(quote_spanned! {grp.span().into() => compile_error!("failed to phone a friend: the user has blocked the chat with the bot");}.into()),
                                APIError(_) => Err(quote_spanned! {grp.span().into() => compile_error!("failed to phone a friend: some error with parsing the API response");}.into()),
                                Timeout => Err(quote_spanned! {grp.span().into() => compile_error!("failed to phone a friend: user did not provide an answer in time");}.into()),
                                UnknownError(_) => Err(quote_spanned! {grp.span().into() => compile_error!("failed to phone a friend: unknown error (check compiler output for details)");}.into()),
                            };
                            return how_error;
                        }
                    }
                }
            }
            // If this is an Ident, and the name is "PhoneAFriend", then we are beginning to parse the magic type.
            TokenTree::Ident(ident) => {
                if !(matches!(state, ParsingState::WaitingForIdent)) {
                    return Err(quote_spanned! {
                        ident.span().into() => compile_error!("expected the phone-a-friend string at this position, but found identifier");
                    }.into());
                }

                if ident.to_string() == "PhoneAFriend" {
                    state = ParsingState::WaitingForGroup;
                } else {
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
                    }.into());
                }
            }
        }
    }

    let mut out = TokenStream::new();
    out.extend(tokens.into_iter());
    println!("Emitting: {out}");
    Ok(out)
}
