use std::{ops::Deref, fmt::Debug, convert::Infallible};

use hyper::StatusCode;
use log::{debug, warn};
use serde::Serialize;
use tokio::sync::broadcast;
use warp::{hyper::Uri, Filter, reject::{Reject}};

use super::{shared::{SharedServerStatus, Message}, webhook::NetboxWebHook, config::WebhookConfig};

pub async fn server(config: &WebhookConfig, status: &SharedServerStatus, message_tx: &broadcast::Sender<Message>) {

    let index_route = warp::get()
        .and(warp::path::end())
        .map(|| { warp::redirect::found(Uri::from_static("/status")) });

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
    let secret_clone = config.secret().map(String::to_owned).clone();
    let webhook_route = warp::post()
        .and(warp::path("webhook")).and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(netbox_webhook_body(secret_clone))
        .and(status_filter).and(message_filter)
        .and_then(|body: NetboxWebHook, status: SharedServerStatus, message_tx: broadcast::Sender<Message>| async move {
            {
                let mut status = status.lock().await;
                status.needs_sync = true;
            }
            debug!("Received Webhook: {:?}", body);
            match message_tx.send(Message::TriggerSync) {
                Ok(_) => Ok(r#"{"info": "Sync triggerd"}"#),
                Err(_) => Err(warp::reject()),
            }
        });

    let route = warp::any().and(
        index_route
        .or(status_route)
        .or(webhook_route)
    )
    .recover(handle_rejection)
    .map(|reply| {
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

use sha2::Sha512;
use hmac::{Hmac, Mac};
type HmacSha512 = Hmac<Sha512>;

#[derive(Debug, PartialEq)]
pub enum WebErrors {
    MissingSignature,
    BadSecret,
    BadSignature,
    BadFormat,
}
impl Reject for WebErrors {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

fn netbox_webhook_body(secret: Option<String>) -> impl Filter<Extract = (NetboxWebHook, ), Error = warp::Rejection> + Clone {

    let secret = secret.clone();
    let f = warp::any().map( move || secret.clone());

    warp::any()
        .and(f)
        .and(warp::header::optional("X-Hook-Signature"))
        .and(warp::body::bytes())
        .and_then(|secret: Option<String>, signature: Option<String>, body: bytes::Bytes| async move {
            match secret {
                Some(secret) => match signature {
                    Some(signature) => {
                        let mut hmac = HmacSha512::new_from_slice(secret.as_bytes())
                            .map_err(|_| warp::reject::custom(WebErrors::BadSecret))?;
                        hmac.update(&body);
                        let hmac = hmac.finalize();
                        let bytes = hmac.into_bytes();
                        if format!("{:x}", bytes) == signature {
                            Ok(body)
                        } else {
                            Err(warp::reject::custom(WebErrors::BadSignature))
                        }
                    }
                    None => Err(warp::reject::custom(WebErrors::MissingSignature)),
                },
                None => Ok(body)
            }
        })
        .and_then(|body: bytes::Bytes| async move {
            serde_json::from_slice::<NetboxWebHook>(&body).map_err(|e| {
                warp::reject::custom(WebErrors::BadFormat)
            }
        )
        })
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let (code, message) = match err.find::<WebErrors>() {
        Some(WebErrors::BadFormat) => (StatusCode::BAD_REQUEST, "Bad Content"),
        Some(WebErrors::BadSecret) => (StatusCode::INTERNAL_SERVER_ERROR, "Bad Secret"),
        Some(WebErrors::BadSignature) => (StatusCode::FORBIDDEN, "Bad Signature"),
        Some(WebErrors::MissingSignature) => (StatusCode::BAD_REQUEST, "Missing Signature"),
        None => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_REJECTION"),
    };

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    warn!("Request error {:?} - {}", code, message);

    Ok(warp::reply::with_status(json, code))
}

#[cfg(test)]
mod tests {
    use chrono::{Utc, TimeZone};

    use crate::server::webhook::NetboxWebHookEvent;

    use super::*;

    const PAYLOAD: &str = r#"{
        "event": "created",
        "timestamp": "2021-03-09 17:55:33+00:00",
        "model": "prefix",
        "username": "jstretch",
        "request_id": "fdbca812-3142-4783-b364-2e2bd5c16c6a",
        "data": {},
        "snapshots": {
            "prechange": null,
            "postchange": {}
        }
    }"#;

    #[tokio::test]
    async fn netbox_webhook_body_wo_secret() {
        let filter = netbox_webhook_body(None);

        // Execute `sum` and get the `Extract` back.
        let res = warp::test::request()
            .body(PAYLOAD)
            .filter(&filter)
            .await;

        assert!(res.is_ok());
        assert_eq!(res.unwrap().username, "jstretch");
    }

    #[tokio::test]
    async fn netbox_webhook_body_rejects_wo_signature() {
        let filter = netbox_webhook_body(Some(String::from("SECRET")));

        // Execute `sum` and get the `Extract` back.
        let res = warp::test::request()
            .body(PAYLOAD)
            .filter(&filter)
            .await;

        assert!(res.is_err());
        assert_eq!(res.unwrap_err().find::<WebErrors>(), Some(&WebErrors::MissingSignature));
    }

    #[tokio::test]
    async fn netbox_webhook_body_rejects_w_invalid_signature() {
        let filter = netbox_webhook_body(Some(String::from("SECRET")));

        // Execute `sum` and get the `Extract` back.
        let res = warp::test::request()
            .header("X-Hook-Signature", "SEGNATURE")
            .body(PAYLOAD)
            .filter(&filter)
            .await;

        assert!(res.is_err());
        assert_eq!(res.unwrap_err().find::<WebErrors>(), Some(&WebErrors::BadSignature));
    }

    #[tokio::test]
    async fn netbox_webhook_body_rejects_w_valid_signature() {
        let filter = netbox_webhook_body(Some(String::from("SECRET")));

        // Execute `sum` and get the `Extract` back.
        let res = warp::test::request()
            .method("POST")
            .path("/webhook")
            .header("X-Hook-Signature", "5ccf9ac371fafa61922c0c5bfbfe6882542eac8ae1b8c26ccf13a3a46108859026cc804c2b211c63c8f9918f9f79f85bbd4fc1f300fc623789264e7650e9d6f2")
            .body(PAYLOAD)
            .filter(&filter)
            .await
            .unwrap();

        assert_eq!(res, NetboxWebHook{
            event: NetboxWebHookEvent::Created,
            timestamp: Utc.with_ymd_and_hms(2021, 3, 9, 17, 55, 33).unwrap(),
            model: String::from("prefix"),
            username: String::from("jstretch"),
            request_id: String::from("fdbca812-3142-4783-b364-2e2bd5c16c6a"),
            data: serde_json::Map::new() });
    }
}