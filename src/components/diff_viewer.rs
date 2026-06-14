use leptos::prelude::*;

use crate::diff::DiffFile;

#[component]
pub fn DiffViewer(#[prop(into)] files: Vec<DiffFile>) -> impl IntoView {
    let items: Vec<_> = files
        .into_iter()
        .map(|file| {
            view! { <FileDiff file=file/> }
        })
        .collect();

    view! { <div class="diff-viewer">{items}</div> }
}

#[component]
fn FileDiff(file: DiffFile) -> impl IntoView {
    let collapsed = RwSignal::new(false);

    let toggle = move |_| {
        collapsed.update(|c| *c = !*c);
    };

    let body_display = move || {
        if collapsed.get() { "none" } else { "block" }
    };

    let collapse_icon = move || {
        if collapsed.get() { "▶" } else { "▼" }
    };

    let status_icon = match file.status.as_str() {
        "added" => "⊕",
        "deleted" => "⊖",
        "binary" => "☰",
        _ => "≡",
    };

    let path_display = if file.new_path == "/dev/null" || file.new_path.starts_with("dev/null") {
        file.old_path.clone()
    } else {
        file.new_path.clone()
    };

    let total = file.stats.additions + file.stats.deletions;
    let add_pct = if total > 0 {
        (file.stats.additions as f64 / total as f64 * 100.0) as u32
    } else {
        50
    };
    let del_pct = if total > 0 {
        (file.stats.deletions as f64 / total as f64 * 100.0) as u32
    } else {
        50
    };

    let body = if file.status == "binary" {
        view! {
            <div class="diff-body" style:display=body_display>
                <div class="diff-line diff-line-context">
                    <span class="diff-line-num-old"></span>
                    <span class="diff-line-num-new"></span>
                    <div class="diff-line-content text-muted">"(Binary file)"</div>
                </div>
            </div>
        }.into_any()
    } else {
        let hunks: Vec<_> = file
            .hunks
            .into_iter()
            .map(|hunk| {
                let lines: Vec<_> = hunk
                    .lines
                    .into_iter()
                    .map(|line| {
                        let line_type = line.line_type.clone();
                        let old_lineno = line.old_lineno.map(|n| n.to_string()).unwrap_or_default();
                        let new_lineno = line.new_lineno.map(|n| n.to_string()).unwrap_or_default();
                        let content_html = line.inline_diff.clone().unwrap_or(line.highlighted);

                        view! {
                            <div class=format!("diff-line diff-line-{}", line_type)>
                                <span class="diff-line-num-old">{old_lineno}</span>
                                <span class="diff-line-num-new">{new_lineno}</span>
                                <div class="diff-line-content" inner_html=content_html></div>
                            </div>
                        }
                    })
                    .collect();

                let hunk_header = hunk.header.clone();

                view! {
                    <div class="diff-hunk-header">
                        <span>{hunk_header}</span>
                    </div>
                    {lines}
                }
            })
            .collect();

        view! {
            <div class="diff-body" style:display=body_display>
                {hunks}
            </div>
        }.into_any()
    };

    view! {
        <div class="diff-file">
            <div class="diff-file-header" on:click=toggle>
                <div class="diff-file-header-left">
                    <span class="diff-file-icon">{status_icon}</span>
                    <span class="diff-file-path">{path_display}</span>
                    {if file.status == "binary" {
                        view! { <span class="text-muted text-xs ml-2">"(binary)"</span> }.into_any()
                    } else {
                        view! {
                            <>
                                <span class="diff-stat-add">"+"{file.stats.additions}</span>
                                <span class="diff-stat-del">"-"{file.stats.deletions}</span>
                            </>
                        }.into_any()
                    }}
                </div>
                <div class="diff-file-header-right">
                    {if file.status != "binary" {
                        view! {
                            <div class="diff-bar">
                                <div
                                    class="diff-bar-add"
                                    style=format!("width: {}%", add_pct)
                                ></div>
                                <div
                                    class="diff-bar-del"
                                    style=format!("width: {}%", del_pct)
                                ></div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                    <span class="diff-collapse-icon">{collapse_icon}</span>
                </div>
            </div>
            {body}
        </div>
    }
}
