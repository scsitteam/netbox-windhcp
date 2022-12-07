use std::{ops::Deref};

use log::debug;
use tokio::sync::broadcast;
use warp::{hyper::Uri, Filter};

use super::{shared::{SharedServerStatus, Message}, webhook::NetboxWebHook, config::WebhookConfig};

pub async fn server(config: &WebhookConfig, status: &SharedServerStatus, message_tx: &broadcast::Sender<Message>) {

    let index_route = warp::get()
        .and(warp::path::end())
        .map(|| {  warp::redirect::found(Uri::from_static("/status")) });

    let status_clone = status.clone();
    let status_filter = warp::any().map(move || status_clone.clone());
    let status_route = warp::get()
        .and(warp::path("status")).and(warp::path::end())
        .and(status_filter)
        .and_then(|status: SharedServerStatus| async move {
                let status = status.lock().await;
                match serde_json::to_string_pretty(&status.deref()) {
                    Ok(s) => Ok(s),
                    Err(_e) => Err(warp::reject()),
                }
        }).map(|reply| {
            warp::reply::with_header(reply, warp::http::header::REFRESH, "5")
        });

    let status_clone = status.clone();
    let status_filter = warp::any().map(move || status_clone.clone());
    let message_clone = message_tx.clone();
    let message_filter = warp::any().map(move || message_clone.clone());
    let webhook_route = warp::post()
        .and(warp::path("webhook")).and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and(status_filter).and(message_filter)
        .and_then(|body: NetboxWebHook, status: SharedServerStatus, message_tx: broadcast::Sender<Message>| async move {
            {
                let mut status = status.lock().await;
                status.needs_sync = true;
            }
            debug!("Hook: {:?}", body);
            match message_tx.send(Message::TriggerSync) {
                Ok(_) => Ok(r#"{"info": "Sync triggerd"}"#),
                Err(_) => Err(warp::reject()),
            }
        });

    let route = warp::any().and(
        index_route
        .or(status_route)
        .or(webhook_route)
    ).map(|reply| {
        warp::reply::with_header(reply, "server", format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
    }).with(warp::log(module_path!()));

    let message_clone = message_tx.clone();

    if config.enable_tls() {
        let (_addr, server) = warp::serve(route)
            .tls()
            .cert_path("cert.pem")
            .key_path("key.rsa")
            .bind_with_graceful_shutdown(config.listen, async move {
                let mut message_rx = message_clone.subscribe();
                while let Ok(msg) = message_rx.recv().await {
                    if msg == Message::Shutdown {
                        break;
                    }
                }
            });
        server.await
    } else {
        let (_addr, server) = warp::serve(route)
            .bind_with_graceful_shutdown(config.listen, async move {
                let mut message_rx = message_clone.subscribe();
                while let Ok(msg) = message_rx.recv().await {
                    if msg == Message::Shutdown {
                        break;
                    }
                }
            });
        server.await
    }
}