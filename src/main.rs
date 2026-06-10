use axum::{extract::Request, middleware, response::IntoResponse, Router};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use egit::{app::App, auth, db};

/// Injected into Axum request extensions by `theme_middleware`.
/// The Leptos shell reads it to set `data-theme` during SSR.
#[derive(Clone)]
pub struct ResolvedTheme(pub String);

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

    let site_root = leptos_options.site_root.to_string();
    let pkg_path = format!("{}/pkg", site_root);

    // Clones for the various closures.
    let pool_ctx = pool.clone();
    let repo_ctx = repo_base_path.clone();
    let pool_shell = pool.clone();
    let pool_mw = pool.clone();

    let app = Router::new()
        .nest_service("/pkg", ServeDir::new(pkg_path))
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                provide_context(pool_ctx.clone());
                provide_context(repo_ctx.clone());
            },
            {
                let opts = leptos_options.clone();
                let p = pool_shell.clone();
                move || shell(opts.clone(), p.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler({
            let p = pool.clone();
            move |opts: LeptosOptions| {
                let p = p.clone();
                shell(opts, p)
            }
        }))
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(middleware::from_fn(move |req, next| {
                    let pool = pool_mw.clone();
                    theme_middleware(pool, req, next)
                })),
        )
        .with_state(leptos_options);

    tracing::info!("eGit listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

/// Axum middleware: resolve the user's preferred theme from their session
/// cookie and attach it as a request extension so the shell fn can read it
/// synchronously during SSR rendering.
async fn theme_middleware(
    pool: sqlx::PgPool,
    mut req: Request,
    next: middleware::Next,
) -> impl IntoResponse {
    let sid = auth::session_id_from_headers(req.headers());
    let theme = match auth::get_session(&pool, sid.as_deref()).await {
        Some(s) => s.theme,
        None => "dark".to_string(),
    };
    req.extensions_mut().insert(ResolvedTheme(theme));
    next.run(req).await
}

/// Build the full HTML document shell with the correct `data-theme` attribute.
/// Called once per SSR request by `leptos_routes_with_context`.
fn shell(options: LeptosOptions, _pool: sqlx::PgPool) -> impl IntoView {
    // leptos_axum::extract() works inside the Leptos SSR rendering pipeline:
    // Leptos sets up a task-local context that bridges to Axum's request
    // extensions, so extract::<Extension<ResolvedTheme>>() returns what we
    // injected in `theme_middleware`.
    let theme = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            leptos_axum::extract::<axum::Extension<ResolvedTheme>>()
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
