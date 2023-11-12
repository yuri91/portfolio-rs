use axum::extract::Path;
use axum::http::Uri;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::Form;

use crate::data::*;
use crate::solve;

pub async fn redirect_add_slash(uri: Uri) -> impl IntoResponse {
    Redirect::permanent(&format!(".{}/", uri.path()))
}

pub async fn index() -> impl IntoResponse {
    let profiles = std::fs::read_dir(std::env::current_dir().expect("no current dir")).expect("cannot list dir").filter_map(|e| {
        let entry = e.expect("cannot access entry");
        if entry.path().extension().and_then(std::ffi::OsStr::to_str) != Some("toml") {
           return None; 
        }
        if let Some(p) = entry.path().file_stem() {
            return Some(p.to_string_lossy().to_string());
        }
        None
    }).collect();
    Index { profiles }
}

pub async fn profile(Path(profile): Path<String>) -> impl IntoResponse {
    let mut p = Portfolio::load(&profile).expect("cannot load portfolio");
    if p.update_quotes().await {
        p.save(&profile).expect("cannot save portfolio");
    }
    let rows = populate_rows(&p);
    Profile { rows }
}

pub async fn solve(Path(profile): Path<String>, Form(form): Form<SolveForm>) -> impl IntoResponse {
    let mut p = Portfolio::load(&profile).expect("cannot load portfolio");
    let new_budget = form.amount;
    let to_buy = solve::solve(&p, new_budget);
    p.update_amounts(&to_buy);
    let rows = populate_rows(&p);
    let to_buy = to_buy
        .into_iter()
        .zip(p.securities)
        .filter(|&(a, _)| a > 0)
        .collect();
    Solve {
        rows,
        to_buy,
        new_budget,
    }
}

pub async fn commit(Path(profile): Path<String>, Form(form): Form<SolveForm>) -> impl IntoResponse {
    let mut p = Portfolio::load(&profile).expect("cannot load portfolio");
    let new_budget = form.amount;
    let to_buy = solve::solve(&p, new_budget);
    p.update_amounts(&to_buy);
    p.save(&profile).expect("cannot save portfolio");
    let rows = populate_rows(&p);
    Profile { rows }
}

#[derive(serde::Deserialize)]
pub struct SolveForm {
    amount: f64,
}

#[derive(askama::Template)]
#[template(path = "index.html")]
struct Index {
    profiles: Vec<String>,
}

#[derive(askama::Template)]
#[template(path = "profile.html")]
struct Profile {
    rows: Vec<Row>,
}

#[derive(askama::Template)]
#[template(path = "solve.html")]
struct Solve {
    rows: Vec<Row>,
    to_buy: Vec<(u32, Security)>,
    new_budget: f64,
}
