use leptos::prelude::*;

#[component]
pub fn Markdown(#[prop(into)] content: String) -> impl IntoView {
    // TODO: render via pulldown-cmark in step 8
    view! { <div class="markdown-body">{content}</div> }
}
