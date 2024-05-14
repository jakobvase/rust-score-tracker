mod data;
mod routes;

use crate::{data::Config, routes::get_router};
use axum::handler::HandlerWithoutStateExt;
use axum::http::{StatusCode, Uri};
use axum::response::Redirect;
use axum::BoxError;
use axum_server::tls_rustls::RustlsConfig;
use serde_json::from_str;
use std::fs;
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use std::{env, vec::Vec};
use tokio::signal;

fn get_config() -> Config {
  let args: Vec<String> = env::args().collect();
  if args.len() < 3 {
    panic!("Too few arguments");
  }

  if &args[1] != "--config" {
    panic!("Need --config argument");
  }

  let config_file = &args[2];
  let config_string = fs::read_to_string(config_file)
    .expect(&format!("Couldn't open config file at {}", config_file));
  from_str(&config_string).expect("Failed to deserialize config.")
}

#[tokio::main]
async fn main() {
  println!("Setting up");
  let config = get_config();
  let http_address = config.host.clone() + ":" + &config.http_port;
  let https_address = config.host.clone() + ":" + &config.https_port;

  // build the application
  let app = get_router(config.clone());

  let certificate_path: &str = &(config.cert_path.clone() + "/fullchain.pem");
  let private_key_path: &str = &(config.cert_path.clone() + "/privkey.pem");

  let ssl_config_res = RustlsConfig::from_pem_file(certificate_path, private_key_path).await;

  match ssl_config_res {
    Ok(ssl_config) => {
      //Create a handle for our TLS server so the shutdown signal can all shutdown
      let handle = axum_server::Handle::new();
      //save the future for easy shutting down of redirect server
      let shutdown_future = shutdown_signal(handle.clone());
      let socket_address: SocketAddr = https_address.parse().unwrap();
      println!("Listening securely for https on {}", https_address);
      tokio::spawn(redirect_http_to_https(config, shutdown_future));
      axum_server::bind_rustls(socket_address, ssl_config)
        .serve(app.into_make_service())
        .await
        .unwrap()
    }
    Err(err) => {
      println!(
        "Error getting certificates {}. Paths {}, {}.",
        err.to_string(),
        certificate_path,
        private_key_path
      );
      let socket_address: SocketAddr = http_address.parse().unwrap();
      println!("Listening for http on {}", http_address);
      axum_server::bind(socket_address)
        .serve(app.into_make_service())
        .await
        .unwrap()
    }
  }
  println!("Stopping.");
}

async fn redirect_http_to_https<F>(config: Config, signal: F)
where
  F: Future<Output = ()> + Send + 'static,
{
  println!("Setting up https redirect listener");

  fn make_https(config: Config, uri: Uri) -> Result<Uri, BoxError> {
    let mut parts = uri.into_parts();

    parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

    if parts.path_and_query.is_none() {
      parts.path_and_query = Some("/".parse().unwrap());
    }

    parts.authority = Some((config.host + ":" + &config.https_port).parse()?);

    Ok(Uri::from_parts(parts)?)
  }

  let config2 = config.clone();
  let redirect = move |uri: Uri| async move {
    match make_https(config2, uri) {
      Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
      _ => Err(StatusCode::BAD_REQUEST),
    }
  };

  let addr: SocketAddr = (config.host.clone() + ":" + &config.http_port)
    .parse()
    .unwrap();
  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

  println!("Starting https redirect listener");
  axum::serve(listener, redirect.into_make_service())
    .with_graceful_shutdown(signal)
    .await
    .unwrap();
}

async fn shutdown_signal(handle: axum_server::Handle) {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
  };

  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }

  // 10 secs is how long docker will wait
  // to force shutdown
  handle.graceful_shutdown(Some(Duration::from_secs(10)));
}
