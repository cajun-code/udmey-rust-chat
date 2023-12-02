use yew::prelude::*;

#[function_component]
fn App() -> Html {
    html! {
        <div id="chat"></div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
