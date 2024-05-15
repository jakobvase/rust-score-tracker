use crate::data::{Config, Input, Results, ScoreEntry};
use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::Html,
  routing::get,
  Form, Router,
};
use minijinja::render;
use serde_json::{from_str, to_string};
use std::fs;
use std::vec::Vec;
use tower_http::validate_request::ValidateRequestHeaderLayer;

async fn get_data(config: Config) -> Vec<Results> {
  from_str(&fs::read_to_string(config.data_path + "/data.json").unwrap_or("[]".to_string()))
    .unwrap_or(Vec::new())
}

async fn set_data(config: Config, data: Vec<Results>) {
  fs::write(config.data_path + "/data.json", to_string(&data).unwrap()).unwrap()
}

async fn home(State(config): State<Config>) -> Html<String> {
  let template_read = fs::read_to_string(config.pages_path.clone() + "/index.html");
  match template_read {
    Ok(template) => {
      let data = get_data(config).await;
      let rendered = render!(&template, games => data);
      Html(rendered)
    }
    Err(reason) => Html(format!(
      "<!DOCTYPE html><html><body>{}</body></html>",
      reason.to_string()
    )),
  }
}

async fn acme_challenge(State(config): State<Config>, Path(path): Path<String>) -> String {
  fs::read_to_string(config.acme_path + "/" + &path).unwrap()
}

async fn add_entry(state: State<Config>, Form(form): Form<Input>) -> Html<String> {
  let mut scores = vec![ScoreEntry {
    name: form.player_1_name,
    score: form.player_1_score,
  }];

  if form.player_2_name != "" && form.player_2_score != "" {
    scores.push(ScoreEntry {
      name: form.player_2_name,
      score: form.player_2_score,
    })
  }

  if form.player_3_name != "" && form.player_3_score != "" {
    scores.push(ScoreEntry {
      name: form.player_3_name,
      score: form.player_3_score,
    })
  }

  if form.player_4_name != "" && form.player_4_score != "" {
    scores.push(ScoreEntry {
      name: form.player_4_name,
      score: form.player_4_score,
    })
  }

  if form.player_5_name != "" && form.player_5_score != "" {
    scores.push(ScoreEntry {
      name: form.player_5_name,
      score: form.player_5_score,
    })
  }

  if form.player_6_name != "" && form.player_6_score != "" {
    scores.push(ScoreEntry {
      name: form.player_6_name,
      score: form.player_6_score,
    })
  }

  let new_entry = Results {
    game: form.game,
    date: form.date,
    results: scores,
  };

  let mut data = get_data(state.0.clone()).await;
  data.insert(0, new_entry);
  set_data(state.0.clone(), data).await;
  home(state).await
}

async fn fallback() -> (StatusCode, String) {
  (StatusCode::NOT_FOUND, "Not found".to_string())
}

pub fn get_router(config: Config) -> Router {
  Router::new()
    .route("/", get(home).post(add_entry))
    .route("/.well-known/acme-challenge/*path", get(acme_challenge))
    .route_layer(ValidateRequestHeaderLayer::basic("", "password"))
    .fallback(fallback)
    // .layer(TraceLayer::new_for_http()) // TODO doesn't work yet, need to add tracing-subscriber
    .with_state(config)
}
