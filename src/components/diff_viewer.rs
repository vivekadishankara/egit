use leptos::prelude::*;

#[component]
pub fn DiffViewer(#[prop(into)] diff: String) -> impl IntoView {
    view! { <pre class="diff-viewer">{diff}</pre> }
}
