use leptos::prelude::*;

#[component]
pub fn DiffViewer(#[prop(into)] diff: String) -> impl IntoView {
    let lines: Vec<_> = diff
        .lines()
        .map(|line| {
            let line_type = if line.starts_with("diff --git")
                || line.starts_with("---")
                || line.starts_with("+++")
            {
                "diff-info"
            } else if line.starts_with("@@") {
                "diff-hunk"
            } else if line.starts_with('+') {
                "diff-add"
            } else if line.starts_with('-') {
                "diff-del"
            } else {
                ""
            };
            view! {
                <div class=line_type>
                    <span class="diff-content">{line.to_string()}</span>
                </div>
            }
        })
        .collect();

    view! { <div class="diff-viewer">{lines}</div> }
}
