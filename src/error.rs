#[derive(Debug)]
pub(crate) enum AskAFriendError {
    NetworkError(reqwest::Error),
    TokenInvalid,
    SendMessageError,
    UnknownChatId,
    ChatClosed,
    APIError(reqwest::Error),
    Timeout,
    UnknownError(String),
}
