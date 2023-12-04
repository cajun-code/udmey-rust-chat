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
    connections: Mutex<HashMap<usize, ChatRoomConnection>>
}

struct ChatRoomConnection {
    username: String,
    sink: SplitSink<DuplexStream, Message>,
}

impl ChatRoom {
    pub async fn add(&self, id: usize, sink: SplitSink<DuplexStream, Message>) {
        let mut conns = self.connections.lock().await;
        let connection = ChatRoomConnection {
            username: format!("User #{}", id),
            sink
        };
        conns.insert(id, connection);
    }

    pub async fn send_username(&self, id: usize) {
        let mut conns = self.connections.lock().await;
        if let Some(connection) = conns.get_mut(&id) {
            let websocket_message = WebSocketMessage {
                message_type: WebSocketMessageType::UsernameChange,
                message: None,
                username: Some(connection.username.clone()),
                users: None,
            };

            let _ = connection.sink.send(
                Message::Text(json!(websocket_message).to_string())
            ).await;
        }
    }

    pub async fn change_username(&self, new_username: String, id: usize) {
        let result = {
            let mut conns = self.connections.lock().await;
            if let Some(connection) = conns.get_mut(&id) {
                let old_username = connection.username.clone();
                connection.username = new_username.clone();
                Some(old_username)
            } else {
                None
            }
        };

        if let Some(old_username) = result {
            let message = ChatMessage {
                message: format!("User {} changed username to {}", old_username, new_username),
                author: "System".to_string(), 
                created_at: Utc::now().naive_utc(),
            };
            Self::broadcast_message(&self, message).await;
        }
    }

    pub async fn broadcast_message(&self, message: ChatMessage) {
        let mut conns = self.connections.lock().await;
        let websocket_message = WebSocketMessage {
            message_type: WebSocketMessageType::NewMessage,
            message: Some(message),
            username: None,
            users: None,
        };

        for (_id, connection) in conns.iter_mut() {
            let _ = connection.sink.send(
                Message::Text(json!(websocket_message).to_string())
            ).await;
        }
    }

    pub async fn broadcast_user_list(&self) {
        let mut conns = self.connections.lock().await;
        let mut users = vec![];

        for (_id, connection) in conns.iter() {
            users.push(connection.username.clone());
        }

        let websocket_message = WebSocketMessage {
            message_type: WebSocketMessageType::UsersList,
            message: None,
            username: None,
            users: Some(users),
        };

        for (_id, connection) in conns.iter_mut() {
            let _ = connection.sink.send(
                Message::Text(json!(websocket_message).to_string())
            ).await;
        }
    }

    pub async fn remove(&self, id: usize) {
        let mut conns = self.connections.lock().await;
        conns.remove(&id);
    }
}

async fn handle_incoming_message(message_contents: Message, state: &State<ChatRoom>, connection_id: usize) {
    match message_contents {
        Message::Text(json) => {
            if let Ok(websocket_message) = serde_json::from_str::<WebSocketMessage>(&json) {
                match websocket_message.message_type {
                    WebSocketMessageType::NewMessage => {
                        if let Some(ws_msg) = websocket_message.message {
                            state.broadcast_message(ws_msg).await
                        }
                    },
                    WebSocketMessageType::UsernameChange => {
                        if let Some(ws_username) = websocket_message.username {
                            state.change_username(ws_username, connection_id).await;
                            state.send_username(connection_id).await;
                            state.broadcast_user_list().await;
                        }
                    },
                    _ => {}
                }
            }
        },
        _ => {
            // Unsupported.
        }
    }
}

#[rocket::get("/")]
fn chat<'r>(ws: WebSocket, state: &'r State<ChatRoom>) -> Channel<'r> {
    ws.channel(move |stream| Box::pin(async move {
        let user_id = USER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let (ws_sink, mut ws_stream) = stream.split();
        state.add(user_id, ws_sink).await;
        state.broadcast_user_list().await;
        state.send_username(user_id).await;

        while let Some(message) = ws_stream.next().await {
            if let Ok(message_contents) = message {
                handle_incoming_message(message_contents, state, user_id).await;
            }
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
