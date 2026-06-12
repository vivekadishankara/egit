use leptos::prelude::*;

#[allow(unused_variables)]
#[component]
pub fn Markdown(#[prop(into)] content: String) -> impl IntoView {
    #[cfg(feature = "ssr")]
    let html = {
        let parser = pulldown_cmark::Parser::new(&content);
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);
        html
    };

    #[cfg(not(feature = "ssr"))]
    let html = String::new();

    view! { <div class="markdown-body" inner_html=html></div> }
}
