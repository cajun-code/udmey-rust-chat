use rocket::futures::StreamExt;
use rocket_ws::{WebSocket, Channel};

#[rocket::get("/")]
fn chat(ws: WebSocket) -> Channel<'static> {
    ws.channel(move |stream| Box::pin(async move {
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
        .launch()
        .await;
}
