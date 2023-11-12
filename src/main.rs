use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::Result;

mod solve;
mod routes;
mod data;


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
        .route("/:profile/", get(routes::index))
        .route("/:profile/solve", post(routes::solve))
        .route("/:profile/commit", post(routes::commit));
    let router = Router::new()
        .nest("/profile", profile_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(tower_livereload::LiveReloadLayer::new());
    let port = 8000_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    info!("router initialized, now listening on port {}", port);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .context("error while starting server")?;

    Ok(())
}


