use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Debug, Serialize, Deserialize)]
pub struct Results {
  pub game: String,
  pub date: String,
  pub results: Vec<ScoreEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreEntry {
  pub name: String,
  pub score: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
  pub pages_path: String,
  pub acme_path: String,
  pub data_path: String,
  pub cert_path: String,
  pub host: String,
  pub http_port: String,
  pub https_port: String,
}

#[derive(Debug, Deserialize)]
pub struct Input {
  pub game: String,
  pub date: String,
  pub player_1_name: String,
  pub player_1_score: String,
  pub player_2_name: String,
  pub player_2_score: String,
  pub player_3_name: String,
  pub player_3_score: String,
  pub player_4_name: String,
  pub player_4_score: String,
  pub player_5_name: String,
  pub player_5_score: String,
  pub player_6_name: String,
  pub player_6_score: String,
}
