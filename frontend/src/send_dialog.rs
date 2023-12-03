use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub send_message_callback: Callback<String>,
    pub username: String,
}

#[function_component(SendDialog)]
pub fn send_dialog(props: &Props) -> Html {
    let new_message_handle = use_state(String::default);
    let new_message = (*new_message_handle).clone();

    let cloned_new_message_handle = new_message_handle.clone();
    let on_message_change = Callback::from(move |e: Event| {
        let target = e.target_dyn_into::<HtmlTextAreaElement>();
        if let Some(textarea) = target {
            cloned_new_message_handle.set(textarea.value());
        }
    });

    let cloned_new_message = new_message.clone();
    let callback = props.send_message_callback.clone();
    let on_button_click = Callback::from(move |_: MouseEvent| {
        callback.emit(cloned_new_message.clone());
        new_message_handle.set("".to_string());
    });

    html! {
        <div class="input-group">
            <button class="btn btn-secondary">{props.username.clone()}</button>
            <span class="input-group-text">{"Your message:"}</span>
            <textarea class="form-control" onchange={on_message_change} value={new_message}></textarea>
            <button type="submit" class="btn-primary" onclick={on_button_click}>{"Send"}</button>
        </div>
    }
}