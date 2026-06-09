use axum::Router;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use egit::{app::App, db};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "egit=debug,info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let conf = get_configuration(None).expect("Failed to read Leptos config");
    let leptos_options = conf.leptos_options.clone();
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let repo_base_path = std::env::var("REPO_BASE_PATH")
        .unwrap_or_else(|_| "./data/repos".into());

    let pool_for_context = pool.clone();
    let repo_path_for_context = repo_base_path.clone();

    let site_root = leptos_options.site_root.to_string();
    let pkg_path = format!("{}/pkg", site_root);

    let app = Router::new()
        .nest_service("/pkg", ServeDir::new(pkg_path))
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                provide_context(pool_for_context.clone());
                provide_context(repo_path_for_context.clone());
            },
            {
                let opts = leptos_options.clone();
                move || shell(opts.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(CompressionLayer::new())
        .with_state(leptos_options);

    tracing::info!("eGit listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options islands=false/>
                <leptos_meta::MetaTags/>
            </head>
            <body class="bg-bg text-text-primary min-h-screen">
                <App/>
            </body>
        </html>
    }
}
