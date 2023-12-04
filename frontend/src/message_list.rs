use common::ChatMessage;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub messages: Vec<ChatMessage>
}

#[function_component(MessageList)]
pub fn message_list(props: &Props) -> Html {

    html! {
        <div class="list-group">
        {
            props.messages.iter().map(|m| {
                let mut classes = classes!("list-group-item", "list-group-item-action");
                if m.author == "System" {
                    classes.push("list-group-item-info");
                }
                html! {
                    <div class={classes}>
                        <div class="d-flex w-100 justify-content-between">
                            <h5>{m.author.clone()}</h5>
                            <small>{m.created_at.format("%Y-%m-%d %H:%M:%S").to_string()}</small>
                        </div>
                        <p>{m.message.clone()}</p>
                    </div>
                }
            }).collect::<Html>()
        }
        </div>
    }
}