use std::{sync::atomic::{AtomicUsize, Ordering}, collections::HashMap};
use rocket::State;
use rocket::futures::{StreamExt, stream::SplitSink};
use rocket_ws::{WebSocket, Channel, stream::DuplexStream, Message};

static USER_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Default)]
struct ChatRoom {
    connections: HashMap<usize, SplitSink<DuplexStream, Message>>
}

#[rocket::get("/")]
fn chat(ws: WebSocket, state: &State<ChatRoom>) -> Channel<'static> {
    ws.channel(move |stream| Box::pin(async move {
        let user_id = USER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let (mut ws_sink, mut ws_stream) = stream.split();
        while let Some(message) = ws_stream.next().await {
            // let _ = ws_sink.send(message?).await;
        }

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
