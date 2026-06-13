#![cfg_attr(not(feature = "ssr"), allow(unused_imports))]
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::components::repo_header::RepoHeader;
use crate::components::repo_tab_bar::{url_encode_branch, BranchSelector, RepoTabBar, get_repo_tab_meta};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobData {
    pub content: String,
    pub highlighted: String,
    pub extension: String,
    pub size: usize,
    pub is_binary: bool,
    pub line_count: usize,
}

#[cfg(feature = "ssr")]
fn syntax_set() -> &'static syntect::parsing::SyntaxSet {
    static SS: OnceLock<syntect::parsing::SyntaxSet> = OnceLock::new();
    SS.get_or_init(|| syntect::parsing::SyntaxSet::load_defaults_newlines())
}

#[cfg(feature = "ssr")]
fn theme_set() -> &'static syntect::highlighting::ThemeSet {
    static TS: OnceLock<syntect::highlighting::ThemeSet> = OnceLock::new();
    TS.get_or_init(|| syntect::highlighting::ThemeSet::load_defaults())
}

#[cfg(feature = "ssr")]
fn highlight(code: &str, extension: &str) -> String {
    let ss = syntax_set();
    let ts = theme_set();

    let syntax = ss
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme = ts
        .themes
        .get("base16-ocean.dark")
        .unwrap_or_else(|| ts.themes.values().next().unwrap());

    syntect::html::highlighted_html_for_string(code, ss, syntax, theme)
        .unwrap_or_else(|e| format!("<pre>Highlight error: {e}</pre>"))
}

#[server(GetBlobContent, "/api")]
pub async fn get_blob_content(
    username: String,
    reponame: String,
    revision: String,
    path: String,
) -> Result<BlobData, ServerFnError> {
    let repo_base: String = expect_context::<String>();

    let (data, ext) = crate::git::read_file(&repo_base, &username, &reponame, &revision, &path)
        .map_err(|e| ServerFnError::new(format!("Failed to read file: {e}")))?;

    let is_binary = data.contains(&0);

    let (content, highlighted, line_count) = if is_binary {
        let info = format!("Binary file ({} bytes)", data.len());
        (info.clone(), format!("<pre class=\"text-muted text-sm\">{info}</pre>"), 0)
    } else {
        let s = String::from_utf8_lossy(&data).to_string();
        let lines = s.lines().count();
        let html = highlight(&s, &ext);
        (s, html, lines)
    };

    Ok(BlobData {
        content,
        highlighted,
        extension: ext,
        size: data.len(),
        is_binary,
        line_count,
    })
}

#[component]
pub fn BlobPage() -> impl IntoView {
    let params = use_params_map();

    let username = move || {
        params
            .get()
            .get("username")
            .unwrap_or_default()
            .to_string()
    };
    let reponame = move || {
        params
            .get()
            .get("reponame")
            .unwrap_or_default()
            .to_string()
    };
    let branch = move || {
        params
            .get()
            .get("branch")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "HEAD".to_string())
    };
    let path = move || {
        params
            .get()
            .get("path")
            .map(|s| s.trim_start_matches('/').to_string())
            .unwrap_or_default()
    };

    let blob = Resource::new(
        move || (username(), reponame(), branch(), path()),
        |(u, r, b, p)| async move { get_blob_content(u, r, b, p).await },
    );

    let repo_meta = Resource::new(
        move || (username(), reponame()),
        |(u, r)| async move { get_repo_tab_meta(u, r).await },
    );

    let tree_url = move || {
        let u = username();
        let r = reponame();
        let b = url_encode_branch(&branch());
        let p = path();
        match p.rsplit_once('/') {
            Some((parent, _)) => {
                if parent.is_empty() {
                    format!("/{u}/{r}/tree/{b}")
                } else {
                    format!("/{u}/{r}/tree/{b}/{parent}")
                }
            }
            None => format!("/{u}/{r}/tree/{b}"),
        }
    };

    view! {
        <div class="container">
            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    repo_meta.get().map(|result| match result {
                        Ok(meta) => {
                            view! {
                                <>
                                    <RepoHeader
                                        owner={username()}
                                        name={reponame()}
                                        is_private={false}
                                        desc={meta.description}
                                        link_to={Some(format!("/{}/{}", username(), reponame()))}
                                    />
                                    <BranchSelector
                                        owner={username()}
                                        name={reponame()}
                                        current_branch={branch()}
                                        redirect_to="/tree/"
                                    />
                                    <RepoTabBar
                                        active="code"
                                        owner={username()}
                                        name={reponame()}
                                        default_branch={meta.default_branch}
                                        has_commits={meta.has_commits}
                                        current_branch={branch()}

                                    />
                                </>
                            }.into_any()
                        }
                        Err(e) => {
                            view! { <div class="alert-error">{e.to_string()}</div> }.into_any()
                        }
                    })
                }}
            </Suspense>

            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    blob.get().map(|result| match result {
                        Ok(data) => {
                            view! {
                                <div class="card mb-4">
                                    <div class="flex items-center justify-between text-sm text-muted px-1 pb-2 border-b border-theme mb-2">
                                        <span>
                                            {if data.extension.is_empty() {
                                                "Unknown file".to_string()
                                            } else {
                                                format!(".{} file", data.extension)
                                            }}
                                            " — "
                                            {data.size} " bytes"
                                        </span>
                                        {if !data.is_binary {
                                            view! {
                                                <span>{data.line_count} " lines"</span>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                    <div
                                        class="overflow-x-auto text-sm leading-relaxed [&_pre]:!bg-transparent"
                                        inner_html=data.highlighted
                                    ></div>
                                </div>
                            }.into_any()
                        }
                        Err(e) => {
                            view! { <div class="alert-error">{e.to_string()}</div> }.into_any()
                        }
                    })
                }}
            </Suspense>

            <a href=tree_url() class="btn-secondary text-sm no-underline">
                "← Back to tree"
            </a>
        </div>
    }
}
