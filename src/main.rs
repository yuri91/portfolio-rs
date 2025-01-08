use anyhow::Context;
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod data;
mod routes;
mod solve;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "portfolio=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("initializing router...");

    let profile_routes = Router::new()
        .route("/:profile", get(routes::redirect_add_slash))
        .route("/:profile/", get(routes::profile))
        .route("/:profile/solve", post(routes::solve))
        .route("/:profile/commit", post(routes::commit));
    let router = Router::new()
        .route("/", get(routes::index))
        .nest("/profile", profile_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );
    #[cfg(debug_assertions)]
    let router = router.layer(tower_livereload::LiveReloadLayer::new());

    let port = 7309_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    info!("router initialized, now listening on port {}", port);

    let listener = TcpListener::bind(&addr)
        .await
        .context("cannot start listener")?;
    axum::serve(listener, router)
        .await
        .context("cannot start server")?;

    Ok(())
}
