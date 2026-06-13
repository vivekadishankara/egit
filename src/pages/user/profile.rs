use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoListItem {
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub joined: String,
    pub repos: Vec<RepoListItem>,
}

#[server(GetUserProfile, "/api")]
pub async fn get_user_profile(username: String) -> Result<UserProfile, ServerFnError> {
    use crate::auth;
    use axum::http::HeaderMap;
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();

    let user = sqlx::query!(
        r#"
        SELECT username, bio, avatar_url, created_at
        FROM users
        WHERE username = $1
        "#,
        username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("User not found"))?;

    let headers: HeaderMap = leptos_axum::extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = auth::session_id_from_headers(&headers);
    let session = auth::get_session(&pool, session_id.as_deref()).await;
    let is_owner = session
        .as_ref()
        .is_some_and(|s| s.username == username);

    let repo_base: String = expect_context::<String>();

    let rows = sqlx::query!(
        r#"
        SELECT r.name, r.description, r.is_private
        FROM repositories r
        JOIN users u ON u.id = r.owner_id
        WHERE u.username = $1
            AND ($2 OR r.is_private = false)
        ORDER BY r.created_at DESC
        "#,
        username,
        is_owner,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    let month_name = match user.created_at.month() {
        time::Month::January => "January",
        time::Month::February => "February",
        time::Month::March => "March",
        time::Month::April => "April",
        time::Month::May => "May",
        time::Month::June => "June",
        time::Month::July => "July",
        time::Month::August => "August",
        time::Month::September => "September",
        time::Month::October => "October",
        time::Month::November => "November",
        time::Month::December => "December",
    };
    let joined = format!("Joined {month_name} {}", user.created_at.year());

    let repo_username = user.username.clone();

    Ok(UserProfile {
        username: user.username,
        bio: user.bio,
        avatar_url: user.avatar_url,
        joined,
        repos: rows
            .into_iter()
            .map(|r| {
                let default_branch = crate::git::get_default_branch(
                    &repo_base,
                    &repo_username,
                    &r.name,
                )
                .unwrap_or_else(|| "HEAD".to_string());
                RepoListItem {
                    name: r.name,
                    description: r.description,
                    is_private: r.is_private,
                    default_branch,
                }
            })
            .collect(),
    })
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();

    let username = move || {
        params
            .get()
            .get("username")
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    let profile = Resource::new(
        move || username(),
        |u| async move { get_user_profile(u).await },
    );

    view! {
        <div class="container">
            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    profile.get().map(|result| match result {
                        Ok(profile) => {
                            let username = profile.username.clone();
                            let bio = profile.bio.clone();
                            let avatar_url = profile.avatar_url.clone();
                            let joined = profile.joined.clone();
                            let repos = profile.repos.clone();

                            view! {
                                <div class="max-w-3xl mx-auto">
                                    <div class="flex items-start gap-6 mb-8 p-6 bg-surface-secondary rounded-lg border border-theme">
                                        {if let Some(url) = avatar_url {
                                            view! {
                                                <img
                                                    src=url
                                                    alt=format!("{username}'s avatar")
                                                    class="w-16 h-16 rounded-full border-2 border-theme"
                                                />
                                            }.into_any()
                                        } else {
                                            let initial = username.chars().next()
                                                .map(|c| c.to_uppercase().to_string())
                                                .unwrap_or_default();
                                            view! {
                                                <div class="w-16 h-16 rounded-full border-2 border-theme bg-surface-tertiary flex items-center justify-center text-xl font-bold text-accent shrink-0">
                                                    {initial}
                                                </div>
                                            }.into_any()
                                        }}

                                        <div class="min-w-0">
                                            <h1 class="text-3xl font-bold mb-1">{username.clone()}</h1>
                                            {bio.as_ref().map(|b| {
                                                view! { <p class="text-muted mb-2">{b.clone()}</p> }
                                            })}
                                            <p class="text-sm text-muted">{joined}</p>
                                        </div>
                                    </div>

                                    <h2 class="text-xl font-semibold mb-4">"Repositories"</h2>

                                    {if repos.is_empty() {
                                        view! {
                                            <div class="card text-center py-12">
                                                <p class="text-muted">"No repositories yet."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="flex flex-col gap-2">
                                                {repos.into_iter().map(|r| {
                                                    let name = r.name.clone();
                                                    let desc = r.description.clone();
                                                    let is_private = r.is_private;
                                                    let repo_href = format!("/{}/{}", username, r.name);

                                                    view! {
                                                        <div class="card">
                                                            <div class="flex items-center gap-2 mb-1">
                                                                <a
                                                                    href=repo_href
                                                                    class="text-lg font-semibold text-accent no-underline hover:underline"
                                                                >
                                                                    {name}
                                                                </a>
                                                                {if is_private {
                                                                    view! {
                                                                        <span class="px-2 py-0.5 text-xs rounded-full border border-theme text-muted">
                                                                            "Private"
                                                                        </span>
                                                                    }
                                                                } else {
                                                                    view! {
                                                                        <span class="px-2 py-0.5 text-xs rounded-full border border-theme text-muted">
                                                                            "Public"
                                                                        </span>
                                                                    }
                                                                }}
                                                            </div>
                                                            {desc.as_ref().map(|d| {
                                                                view! { <p class="text-sm text-muted">{d.clone()}</p> }
                                                            })}
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }}
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
