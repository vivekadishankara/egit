use axum::Extension;
use leptos::config::LeptosOptions;
use leptos::prelude::*;

use super::middleware::ResolvedTheme;
use crate::app::App;

pub fn html_shell(options: LeptosOptions, _pool: sqlx::PgPool) -> impl IntoView {
    let theme = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            leptos_axum::extract::<Extension<ResolvedTheme>>()
                .await
                .map(|ext| ext.0 .0.clone())
                .unwrap_or_else(|_| "dark".to_string())
        })
    });

    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme=theme>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options islands=false/>
                <leptos_meta::MetaTags/>
            </head>
            <body class="bg-surface min-h-screen">
                <App/>
            </body>
        </html>
    }
}
