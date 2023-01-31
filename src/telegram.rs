use std::time::Duration;

use tokio::time::Instant;

use crate::error::AskAFriendError;

/// Implementation of the "ask friend" feature using the Telegram API as a backend.
///
/// When this function is called, it will attempt to connect to the Telegram API with the given parameters,
/// then send a message to the given user and wait for a response.
/// This response is then returned in the `Ok` variant.

pub(crate) fn ask_friend_via_tg(
    params: &mut TelegramParams,
    query: &str,
) -> Result<String, AskAFriendError> {
    // Start a Tokio runtime and run it until the future completes.
    // This is necessary because the Telegram API is asynchronous.
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(ask_friend_via_tg_inner(params, query))
}

async fn ask_friend_via_tg_inner(
    params: &mut TelegramParams,
    query: &str,
) -> Result<String, AskAFriendError> {
    get_user_valid(params).await?;
    let answer = send_message_and_wait(params, query).await?;
    Ok(answer)
}

pub(crate) struct TelegramParams {
    pub token: String,
    pub chat_id: i64,
    pub is_token_valid: bool,
    pub response_timeout: Duration,
}

/// Check whether the bot's corresponding user exists.
/// This is used to make sure that the given TelegramParams are valid;
/// the response is stored in the TelegramParams.
async fn get_user_valid(params: &mut TelegramParams) -> Result<(), AskAFriendError> {
    if params.is_token_valid {
        return Ok(());
    }

    let ok = reqwest::get(format!(
        "https://api.telegram.org/bot{}/getMe",
        params.token
    ))
    .await
    .map_err(|e| AskAFriendError::NetworkError(e))?
    .status()
    .is_success();
    if !ok {
        return Err(AskAFriendError::TokenInvalid);
    } else {
        params.is_token_valid = true;
        return Ok(());
    }
}

/// Send a message to the given chat, without waiting for a response.
/// This is used to send informational messages.
#[allow(dead_code)]
async fn send_message(params: &TelegramParams, message: &str) -> Result<(), AskAFriendError> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "https://api.telegram.org/bot{}/sendMessage",
            params.token
        ))
        .json(&serde_json::json!({
            "chat_id": params.chat_id,
            "text": message,
        }))
        .send()
        .await
        .map_err(|e| AskAFriendError::NetworkError(e))?;
    if res.status() == reqwest::StatusCode::BAD_REQUEST {
        return Err(AskAFriendError::UnknownChatId);
    } else if res.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(AskAFriendError::ChatClosed);
    } else if !res.status().is_success() {
        return Err(AskAFriendError::SendMessageError);
    } else {
        return Ok(());
    }
}

/// Send a message to the given chat, using the `force_reply` layout, and wait for a response.
/// This is used to send questions and wait for answers.
async fn send_message_and_wait(
    params: &TelegramParams,
    message: &str,
) -> Result<String, AskAFriendError> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "https://api.telegram.org/bot{}/sendMessage",
            params.token
        ))
        .json(&serde_json::json!({
            "chat_id": params.chat_id,
            "text": message,
            "reply_markup": {
                "force_reply": true,
            }
        }))
        .send()
        .await
        .map_err(|e| AskAFriendError::NetworkError(e))?;
        println!("{res:?}");
    if res.status() == reqwest::StatusCode::BAD_REQUEST {
        return Err(AskAFriendError::UnknownChatId);
    } else if res.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(AskAFriendError::ChatClosed);
    } else if !(res.status().is_success()) {
        return Err(AskAFriendError::SendMessageError);
    }

    let json: serde_json::Value = res.json().await.map_err(|e| AskAFriendError::APIError(e))?;
    let message_id = json.get("result").and_then(|r| r.get("message_id")).and_then(|m| m.as_i64());
    if message_id.is_none() {
        return Err(AskAFriendError::UnknownError("message_id not found in successful response".to_string()));
    }

    let mut last_update_id = 0;
    let waiting_period_start = Instant::now();
    loop {
        let res = client
            .get(format!(
                "https://api.telegram.org/bot{}/getUpdates",
                params.token
            ))
            .query(&[("offset", last_update_id + 1), ("timeout", 5)])
            .send()
            .await
            .map_err(|e| AskAFriendError::NetworkError(e))?;
        if !res.status().is_success() {
            return Err(AskAFriendError::UnknownError(
                "getUpdates returned non-200 status code".to_string(),
            ));
        }

        let json: serde_json::Value = res.json().await.map_err(|e| AskAFriendError::APIError(e))?;
        let updates = json.get("result").and_then(|r| r.as_array());
        if updates.is_none() {
            return Err(AskAFriendError::UnknownError(
                "getUpdates returned non-array result".to_string(),
            ));
        }
        let updates = updates.unwrap();

        // Find the update that is a reply to the message we sent
        for update in updates {
            println!("Got update {update}");
            if let Some(update_id) = update.get("update_id").and_then(|u| u.as_i64()) {
                last_update_id = last_update_id.max(update_id);
            }
            if let Some(message) = update.get("message").and_then(|m| m.as_object()) {
                if let Some(reply_to_message) = message.get("reply_to_message").and_then(|r| r.as_object()) {
                    if let Some(reply_to_message_id) = reply_to_message.get("message_id").and_then(|m| m.as_i64()) {
                        if reply_to_message_id == message_id.unwrap() {
                            if let Some(text) = message.get("text").and_then(|t| t.as_str()) {
                                return Ok(text.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Check if we're out of time
        if waiting_period_start.elapsed() > params.response_timeout {
            return Err(AskAFriendError::Timeout);
        }
    }
}
