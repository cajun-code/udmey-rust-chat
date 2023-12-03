use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize)]
pub struct ChatMessage {
    pub message: String,
    pub author: String,
    pub created_at: NaiveDateTime
}
