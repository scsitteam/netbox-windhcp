pub mod service;

use std::sync::mpsc as std_mpsc;
use log::debug;
use tokio::sync::mpsc;
use warp::Filter;

#[derive(Debug)]
pub enum Message {
    Shutdown,
}

pub fn run(shutdown_rx: Option<std_mpsc::Receiver<Message>>) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            
            let (shutdown_tx, mut shutdown_rx2) = mpsc::channel(32);

            if let Some(shutdown_rx) = shutdown_rx {
                debug!("Start Proxy");
                tokio::spawn(async move {
                    loop {
                        let msg = match shutdown_rx.recv() {
                            Ok(msg) => msg,
                            Err(_) => { break; },
                        };
                        debug!("Channel proxy got: {:?}", &msg);
                        shutdown_tx.send(msg).await.unwrap();
                    };
                });
                debug!("Start Proxy Done");
            }

            let hello = warp::path!("hello" / String)
                .map(|name| format!("Hello, {}!", name));

            let (_addr, server) = warp::serve(hello)
                .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
                    shutdown_rx2.recv().await;
               });
            server.await;
        
        println!("Hello world");
    })
}