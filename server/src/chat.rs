use anyhow::Result;
use salvo::prelude::*;
use salvo::proto::{quic, webtransport};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{RwLock, mpsc},
};

use crate::{AppState, db, error::AppError};

#[derive(Default)]
pub struct ChatHub {
    next_session_id: AtomicU64,
    sessions: RwLock<HashMap<u64, mpsc::UnboundedSender<String>>>,
}

impl ChatHub {
    pub async fn register(&self) -> (u64, mpsc::UnboundedReceiver<String>) {
        let session_id = self.next_session_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::unbounded_channel();
        self.sessions.write().await.insert(session_id, sender);
        (session_id, receiver)
    }

    async fn unregister(&self, session_id: u64) {
        self.sessions.write().await.remove(&session_id);
    }

    pub async fn broadcast(&self, payload: String) {
        for sender in self.sessions.read().await.values() {
            let _ = sender.send(payload.clone());
        }
    }
}

#[handler]
pub async fn connect(req: &mut Request, depot: &mut Depot) -> Result<(), salvo::Error> {
    let state = AppError::obtain::<Arc<AppState>>(depot)
        .expect("AppState must be injected")
        .clone();

    let session = req.web_transport_mut().await?;
    let session_id = session.session_id();

    let (local_session_id, mut outbound_receiver) = state.chat_hub.register().await;

    loop {
        tokio::select! {
            outbound = outbound_receiver.recv() => {
                let Some(payload) = outbound else { break };
                let mut stream = session.open_uni(session_id).await?;
                stream.write_all(payload.as_bytes()).await?;
                stream.shutdown().await?;
            }
            incoming = session.accept_bi() => {
                let Some(webtransport::server::AcceptedBi::BidiStream(_, stream)) = incoming? else { break };
                let (_, mut recv) = quic::BidiStream::split(stream);
                let state = state.clone();

                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    if let Err(e) = recv.read_to_end(&mut buffer).await {
                        tracing::error!("read stream: {e}");
                        return;
                    }
                    if let Err(e) = process_incoming_chat_message(state, &buffer).await {
                        tracing::error!("process message: {e}");
                    }
                });
            }
        }
    }

    state.chat_hub.unregister(local_session_id).await;
    Ok(())
}

async fn process_incoming_chat_message(state: Arc<AppState>, buffer: &[u8]) -> Result<()> {
    let incoming: ClientChatMessage = serde_json::from_slice(buffer)?;
    let body = incoming.body.trim();

    if body.is_empty() {
        return Ok(());
    }

    let message = db::insert_chat_message(&state.pool, body).await?;
    let payload = serde_json::to_string(&message)?;
    state.chat_hub.broadcast(payload).await;

    Ok(())
}

#[derive(Deserialize)]
struct ClientChatMessage {
    body: String,
}
