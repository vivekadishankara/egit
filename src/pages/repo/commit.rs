use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::diff_viewer::DiffViewer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetail {
    pub id: String,
    pub short_id: String,
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub message_body: String,
    pub timestamp: i64,
    pub diff: Vec<crate::diff::DiffFile>,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[server(GetCommitDetail, "/api")]
pub async fn get_commit_detail(
    username: String,
    reponame: String,
    commit_id: String,
) -> Result<CommitDetail, ServerFnError> {
    let repo_base: String = expect_context::<String>();
    let raw = crate::git::get_commit_detail(&repo_base, &username, &reponame, &commit_id)
        .map_err(|e| ServerFnError::new(format!("Failed to read commit: {e}")))?;
    Ok(CommitDetail {
        id: raw.id,
        short_id: raw.short_id,
        author_name: raw.author_name,
        author_email: raw.author_email,
        message: raw.message,
        message_body: raw.message_body,
        timestamp: raw.timestamp,
        diff: raw.diff,
        files_changed: raw.files_changed,
        insertions: raw.insertions,
        deletions: raw.deletions,
    })
}

fn format_timestamp_full(ts: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(ts)
        .map(|dt| {
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                dt.year(),
                dt.month() as u8,
                dt.day(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )
        })
        .unwrap_or_default()
}

fn format_timestamp_relative(ts: i64) -> String {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let diff = now - ts;
    if diff < 60 {
        format!("{diff}s ago")
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 2592000 {
        format!("{}d ago", diff / 86400)
    } else {
        format_timestamp_full(ts)
    }
}

#[component]
pub fn CommitPage() -> impl IntoView {
    let params = use_params_map();

    let username = move || {
        params
            .get()
            .get("username")
            .map(|s| s.to_string())
            .unwrap_or_default()
    };
    let reponame = move || {
        params
            .get()
            .get("reponame")
            .map(|s| s.to_string())
            .unwrap_or_default()
    };
    let commit_id = move || {
        params
            .get()
            .get("id")
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    let detail = Resource::new(
        move || (username(), reponame(), commit_id()),
        |(u, r, c)| async move { get_commit_detail(u, r, c).await },
    );

    view! {
        <div class="container">
            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    detail.get().map(|result| match result {
                        Ok(d) => {
                            view! {
                                <div>
                                    <div class="mb-4">
                                        <a
                                            href=format!("/{}/{}/commits", username(), reponame())
                                            class="text-accent hover:underline no-underline text-sm"
                                        >
                                            "← Back to commits"
                                        </a>
                                    </div>

                                    <div class="card mb-4">
                                        <div class="text-lg font-semibold mb-3 text-text">
                                            {d.message.clone()}
                                        </div>

                                        {if !d.message_body.is_empty() {
                                            view! {
                                                <pre class="text-sm text-muted mb-3 whitespace-pre-wrap font-sans">
                                                    {d.message_body.clone()}
                                                </pre>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}

                                        <div class="flex items-center gap-4 text-sm text-muted border-t border-theme pt-3">
                                            <span>
                                                <span class="font-mono text-accent">{d.short_id.clone()}</span>
                                            </span>
                                            <span>
                                                <span class="font-medium text-text">{d.author_name.clone()}</span>
                                                " authored on "
                                                {format_timestamp_relative(d.timestamp)}
                                            </span>
                                        </div>
                                    </div>

                                    <div class="flex items-center gap-4 text-sm text-muted mb-4">
                                        <span>
                                            <span class="font-medium text-text">{d.files_changed}</span>
                                            " file(s) changed"
                                        </span>
                                        <span class="text-success">
                                            <span class="font-medium">"+"{d.insertions}</span>
                                            " additions"
                                        </span>
                                        <span class="text-danger">
                                            <span class="font-medium">"-"{d.deletions}</span>
                                            " deletions"
                                        </span>
                                    </div>

                                    <DiffViewer files=d.diff.clone()/>
                                </div>
                            }.into_any()
                        }
                        Err(e) => {
                            view! { <div class="alert-error">{e.to_string()}</div> }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
