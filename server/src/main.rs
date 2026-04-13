mod cert;
mod chat;
mod db;
mod error;
mod routes;

use anyhow::{Context, Result};
use salvo::prelude::{QuinnListener, Router, Server, TcpListener};
use salvo::serve_static::StaticDir;
use salvo::{Listener, affix_state};
use sqlx::SqlitePool;
use std::{
    net::{Ipv6Addr, SocketAddr, SocketAddrV6},
    path::PathBuf,
    sync::Arc,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub chat_hub: Arc<chat::ChatHub>,
    pub cert_fingerprint_hex: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("install rustls crypto provider");

    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("server crate must be inside workspace")
        .join(".data");
    std::fs::create_dir_all(&data_dir).context("create .data directory")?;

    let pool = db::connect(&data_dir).await.context("sqlite connect")?;
    db::init_schema(&pool).await.context("init schema")?;

    let cert_material = cert::CertificateMaterial::load_or_generate(&data_dir)?;

    let state = Arc::new(AppState {
        pool,
        chat_hub: Arc::new(chat::ChatHub::default()),
        cert_fingerprint_hex: cert_material.fingerprint_hex.clone(),
    });

    let router = app(state.clone());
    let bind_addr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 3000, 0, 0));

    tracing::info!(
        cert_sha256 = %state.cert_fingerprint_hex,
        "listening on https://localhost:3000/"
    );

    let rustls_config = cert_material.salvo_rustls_config();

    let acceptor = QuinnListener::new(rustls_config.clone(), bind_addr)
        .join(TcpListener::new(bind_addr).rustls(rustls_config))
        .bind()
        .await;

    Server::new(acceptor).serve(router).await;

    Ok(())
}

fn app(state: Arc<AppState>) -> Router {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    Router::new()
        .hoop(affix_state::inject(state))
        .push(Router::with_path("/").get(routes::home))
        .push(
            Router::with_path("todo/")
                .get(routes::todo_page)
                .post(routes::create_todo),
        )
        .push(Router::with_path("todo/{id}/toggle").post(routes::toggle_todo))
        .push(Router::with_path("todo/{id}/delete").post(routes::delete_todo))
        .push(Router::with_path("chat/").get(routes::chat_page))
        .push(Router::with_path("webtransport/chat").goal(chat::connect))
        .push(Router::with_path("assets/{**}").get(StaticDir::new([assets_dir])))
}

#[cfg(test)]
mod tests {
    use super::*;
    use salvo::http::StatusCode;
    use salvo::prelude::Service;
    use salvo::test::{ResponseExt, TestClient};
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_service() -> Service {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        db::init_schema(&pool).await.unwrap();

        Service::new(app(Arc::new(AppState {
            pool,
            chat_hub: Arc::new(chat::ChatHub::default()),
            cert_fingerprint_hex: "00".repeat(32),
        })))
    }

    #[tokio::test]
    async fn home_page_renders() {
        let service = test_service().await;
        let mut response = TestClient::get("http://127.0.0.1:3000/")
            .send(&service)
            .await;

        assert_eq!(response.status_code.unwrap(), StatusCode::OK);
        let html = response.take_string().await.unwrap();
        assert!(html.contains("Counter"));
        assert!(html.contains("page-frame"));
    }

    #[tokio::test]
    async fn chat_page_renders() {
        let service = test_service().await;
        let mut response = TestClient::get("http://127.0.0.1:3000/chat/")
            .send(&service)
            .await;

        assert_eq!(response.status_code.unwrap(), StatusCode::OK);
        let html = response.take_string().await.unwrap();
        assert!(html.contains("WebTransport chat"));
        assert!(html.contains("data-cert-hash"));
    }

    #[tokio::test]
    async fn create_todo_redirects_and_renders_item() {
        let service = test_service().await;

        let response = TestClient::post("http://127.0.0.1:3000/todo/")
            .add_header("content-type", "application/x-www-form-urlencoded", true)
            .body("title=Write%20tests")
            .send(&service)
            .await;

        assert_eq!(response.status_code.unwrap(), StatusCode::SEE_OTHER);

        let mut response = TestClient::get("http://127.0.0.1:3000/todo/")
            .send(&service)
            .await;

        assert_eq!(response.status_code.unwrap(), StatusCode::OK);
        let html = response.take_string().await.unwrap();
        assert!(html.contains("Write tests"));
        assert!(html.contains("Mark done"));
    }

    #[tokio::test]
    async fn toggled_todo_renders_completed_variant() {
        let service = test_service().await;

        TestClient::post("http://127.0.0.1:3000/todo/")
            .add_header("content-type", "application/x-www-form-urlencoded", true)
            .body("title=Ship%20feature")
            .send(&service)
            .await;

        TestClient::post("http://127.0.0.1:3000/todo/1/toggle")
            .send(&service)
            .await;

        let mut response = TestClient::get("http://127.0.0.1:3000/todo/")
            .send(&service)
            .await;

        let html = response.take_string().await.unwrap();
        assert!(html.contains("Completed"));
        assert!(html.contains("Mark active"));
        assert!(html.contains("is-complete"));
    }
}
