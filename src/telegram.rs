use serde::{Deserialize, Serialize};

/// Incoming update from Telegram Bot API.
/// https://core.telegram.org/bots/api#update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Update {
    pub update_id: i64,
    #[serde(default)]
    pub message: Option<Message>,
    #[serde(default)]
    pub inline_query: Option<InlineQuery>,
}

/// Telegram Message object.
/// https://core.telegram.org/bots/api#message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub message_id: i64,
    pub chat: Chat,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub reply_to_message: Option<Box<Message>>,
}

/// Telegram Chat object.
/// https://core.telegram.org/bots/api#chat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chat {
    pub id: i64,
    pub r#type: String,
}

/// Telegram User object.
/// https://core.telegram.org/bots/api#user
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
}

/// Telegram InlineQuery object.
/// https://core.telegram.org/bots/api#inlinequery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InlineQuery {
    pub id: String,
    pub query: String,
}

/// Content of a text message to be sent as the result of an inline query.
/// https://core.telegram.org/bots/api#inputtextmessagecontent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTextMessageContent {
    pub message_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_web_page_preview: Option<bool>,
}

impl InputTextMessageContent {
    pub fn markdown(text: impl Into<String>) -> Self {
        Self {
            message_text: text.into(),
            parse_mode: Some("Markdown".to_string()),
            disable_web_page_preview: Some(true),
        }
    }
}

/// Represents a link to an article or web page.
/// https://core.telegram.org/bots/api#inlinequeryresultarticle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InlineQueryResultArticle {
    pub r#type: String,
    pub id: String,
    pub title: String,
    pub input_message_content: InputTextMessageContent,
}

impl InlineQueryResultArticle {
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: InputTextMessageContent) -> Self {
        Self {
            r#type: "article".to_string(),
            id: id.into(),
            title: title.into(),
            input_message_content: content,
        }
    }
}

/// A Telegram Bot API method call.
/// When using webhooks, returning this object serialized to JSON
/// automatically executes the method as the webhook response.
/// https://core.telegram.org/bots/api#making-requests-when-getting-updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method")]
pub enum TelegramMethod {
    #[serde(rename = "sendMessage")]
    SendMessage {
        chat_id: i64,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to_message_id: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parse_mode: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_web_page_preview: Option<bool>,
    },
    #[serde(rename = "answerInlineQuery")]
    AnswerInlineQuery {
        inline_query_id: String,
        results: Vec<InlineQueryResultArticle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_time: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_personal: Option<bool>,
    },
}

impl TelegramMethod {
    /// Helper to construct a Markdown reply to a specific message.
    pub fn reply_markdown(chat_id: i64, reply_to_message_id: i64, text: impl Into<String>) -> Self {
        TelegramMethod::SendMessage {
            chat_id,
            text: text.into(),
            reply_to_message_id: Some(reply_to_message_id),
            parse_mode: Some("Markdown".to_string()),
            disable_web_page_preview: Some(true),
        }
    }

    /// Helper to answer an inline query with articles.
    pub fn answer_inline_query(
        inline_query_id: impl Into<String>,
        results: Vec<InlineQueryResultArticle>,
    ) -> Self {
        TelegramMethod::AnswerInlineQuery {
            inline_query_id: inline_query_id.into(),
            results,
            cache_time: Some(0),
            is_personal: Some(false),
        }
    }
}
