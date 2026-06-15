#![recursion_limit = "512"]
#[cfg(feature = "ssr")]
mod ssr_main {
    use axum::{Extension, Router};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower::ServiceBuilder;
    use tower_http::compression::CompressionLayer;
    use tower_http::services::ServeDir;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    use egit::{
        app::App,
        server::{db, git_routes::GitSmartHttpState, middleware, shell},
    };

    #[tokio::main]
    pub async fn main() {
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

        if let Err(e) = std::fs::create_dir_all(&repo_base_path) {
            tracing::error!("Failed to create repo base directory {}: {}", repo_base_path, e);
        }

        let site_root = leptos_options.site_root.to_string();
        let pkg_path = format!("{}/pkg", site_root);

        let pool_ctx = pool.clone();
        let repo_ctx = repo_base_path.clone();
        let pool_shell = pool.clone();
        let pool_mw = pool.clone();

        let git_state = GitSmartHttpState {
            pool: pool.clone(),
            repo_base_path: repo_base_path.clone(),
        };

        let git_router = Router::new()
            .route(
                "/{username}/{reponame}/info/refs",
                axum::routing::get(egit::server::git_routes::handle_info_refs),
            )
            .route(
                "/{username}/{reponame}/git-upload-pack",
                axum::routing::post(egit::server::git_routes::handle_upload_pack),
            )
            .route(
                "/{username}/{reponame}/git-receive-pack",
                axum::routing::post(egit::server::git_routes::handle_receive_pack),
            )
            .layer(Extension(git_state));

        let app_router = Router::new()
            .nest_service("/pkg", ServeDir::new(pkg_path))
            .nest_service("/assets", ServeDir::new("public"))
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
                    move || shell::html_shell(opts.clone(), p.clone())
                },
            )
            .fallback(leptos_axum::file_and_error_handler({
                let p = pool.clone();
                move |opts: LeptosOptions| {
                    let p = p.clone();
                    shell::html_shell(opts, p)
                }
            }))
            .layer(
                ServiceBuilder::new()
                    .layer(CompressionLayer::new())
                    .layer(axum::middleware::from_fn(move |req, next| {
                        let pool = pool_mw.clone();
                        middleware::theme_middleware(pool, req, next)
                    })),
            );

        let app = Router::new()
            .merge(git_router)
            .merge(app_router)
            .with_state(leptos_options);

        tracing::info!("eGit listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service()).await.unwrap();
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    eprintln!(
        "The egit binary requires the 'ssr' feature. \
         Use `cargo leptos build` or `cargo build --features ssr`."
    );
    std::process::exit(1);
}

#[cfg(feature = "ssr")]
fn main() {
    ssr_main::main();
}
