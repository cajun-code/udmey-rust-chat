use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::Utc;
use common::*;
use rocket::{State, tokio::sync::Mutex};
use rocket::futures::{SinkExt, StreamExt, stream::SplitSink};
use rocket_ws::{WebSocket, Channel, stream::DuplexStream, Message};
use serde_json::json;

static USER_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Default)]
struct ChatRoom {
    connections: Mutex<HashMap<usize, SplitSink<DuplexStream, Message>>>
}

impl ChatRoom {
    pub async fn add(&self, id: usize, sink: SplitSink<DuplexStream, Message>) {
        let mut conns = self.connections.lock().await;
        conns.insert(id, sink);
    }

    pub async fn broadcast_message(&self, message: Message, author_id: usize) {
        let chat_message = ChatMessage {
            message: message.to_string(),
            author: format!("User #{}", author_id),
            created_at: Utc::now().naive_utc(),
        };
        let websocket_message = WebSocketMessage {
            message_type: WebSocketMessageType::NewMessage,
            message: Some(chat_message),
            username: None,
            users: None,
        };
        let mut conns = self.connections.lock().await;
        for (_id, sink) in conns.iter_mut() {
            let _ = sink.send(
                Message::Text(json!(websocket_message).to_string())
            ).await;
        }
    }

    pub async fn broadcast_user_list(&self) {
        let mut conns = self.connections.lock().await;
        let mut users = vec![];

        for (id, _) in conns.iter() {
            users.push(format!("User #{}", id));
        }

        let websocket_message = WebSocketMessage {
            message_type: WebSocketMessageType::UsersList,
            message: None,
            username: None,
            users: Some(users),
        };

        for (_id, sink) in conns.iter_mut() {
            let _ = sink.send(
                Message::Text(json!(websocket_message).to_string())
            ).await;
        }
    }

    pub async fn remove(&self, id: usize) {
        let mut conns = self.connections.lock().await;
        conns.remove(&id);
    }
}

#[rocket::get("/")]
fn chat<'r>(ws: WebSocket, state: &'r State<ChatRoom>) -> Channel<'r> {
    ws.channel(move |stream| Box::pin(async move {
        let user_id = USER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let (ws_sink, mut ws_stream) = stream.split();
        state.add(user_id, ws_sink).await;
        state.broadcast_user_list().await;

        while let Some(message) = ws_stream.next().await {
            state.broadcast_message(message?, user_id).await;
        }

        state.remove(user_id).await;
        state.broadcast_user_list().await;

        Ok(())
    }))
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount("/", rocket::routes![
            chat
        ])
        .manage(ChatRoom::default())
        .launch()
        .await;
}
