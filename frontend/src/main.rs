use common::{WebSocketMessage, WebSocketMessageType};
use yew::prelude::*;
use yew_hooks::use_websocket;
use crate::message_list::MessageList;
use crate::send_dialog::SendDialog;
use crate::users_list::UsersList;

mod message_list;
mod send_dialog;
mod users_list;

#[function_component]
fn App() -> Html {
    let messages_handle = use_state(Vec::default);
    let messages = (*messages_handle).clone();
    let users_handle = use_state(Vec::default);
    let users = (*users_handle).clone();
    let username_handle = use_state(String::default);
    let username = (*username_handle).clone();

    let ws = use_websocket("ws://127.0.0.1:8000".to_string());

    let mut cloned_messages = messages.clone();
    use_effect_with(ws.message.clone(), move |ws_message| {
        if let Some(ws_msg) = &**ws_message {
            let websocket_message: WebSocketMessage = serde_json::from_str(&ws_msg).unwrap();
            match websocket_message.message_type {
                WebSocketMessageType::NewMessage => {
                    let msg = websocket_message.message.expect("Missing message payload");
                    cloned_messages.push(msg);
                    messages_handle.set(cloned_messages);
                },
                WebSocketMessageType::UsersList => {
                    let users = websocket_message.users.expect("Missing users payload");
                    users_handle.set(users);
                },
                WebSocketMessageType::UsernameChange => {
                    let username = websocket_message.username.expect("Missing username payload");
                    username_handle.set(username);
                }
            }
        }
    });

    let cloned_ws = ws.clone();
    let send_message_callback = Callback::from(move |msg: String| {
        cloned_ws.send(msg.clone());
    });

    html! {
        <div class="container-fluid">
            <div class="row">
                <div class="col-sm-3">
                    <UsersList users={users} />
                </div>
                <div class="col-sm-9">
                    <MessageList messages={messages} />
                </div>
            </div>
            <div class="row">
                <SendDialog send_message_callback={send_message_callback} username={username} />
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
