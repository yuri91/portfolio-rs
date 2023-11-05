use axum::response::IntoResponse;
use crate::data::*;

pub async fn index() -> impl IntoResponse {
    let mut p = load_portfolio();
    if update_quotes(&mut p.securities).await {
        save_portfolio(&p);
    }
    let rows = populate_rows(&p).await;
    Index { rows }
}

pub async fn solve() -> impl IntoResponse {
    let p = load_portfolio();
    
    ""
}

#[derive(askama::Template)]
#[template(path = "index.html")]
struct Index {
    rows: Vec<Row>,
}

#[derive(askama::Template)]
#[template(path = "solve.html")]
struct Solve {
    rows: Vec<Row>,
}
