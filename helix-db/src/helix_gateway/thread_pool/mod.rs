use crate::helix_engine::graph_core::graph_core::HelixGraphEngine;
use crate::helix_gateway::gateway::CoreSetter;
use crate::protocol;
use flume::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tracing::{error, info, trace};

use crate::helix_gateway::router::router::{HelixRouter, RouterError};
use crate::protocol::request::{ReqMsg, Request};
use crate::protocol::response::Response;

pub struct WorkerPool {
    tx: Sender<ReqMsg>,
    workers: Vec<Worker>,
}

impl WorkerPool {
    pub fn new(
        size: usize,
        core_setter: Option<CoreSetter>,
        graph_access: Arc<HelixGraphEngine>,
        router: Arc<HelixRouter>,
    ) -> WorkerPool {
        assert!(
            size > 0,
            "Expected number of threads in thread pool to be more than 0, got {}",
            size
        );

        let (tx, rx) = flume::bounded::<ReqMsg>(1000); // TODO: make this configurable
        let workers = (0..size)
            .map(|_| {
                Worker::start(
                    rx.clone(),
                    core_setter.clone(),
                    graph_access.clone(),
                    router.clone(),
                )
            })
            .collect::<Vec<_>>();

        WorkerPool { tx, workers }
    }

    pub async fn process(&self, req: protocol::request::Request) -> protocol::response::Response {
        let (ret_tx, ret_rx) = oneshot::channel::<protocol::response::Response>();

        self.tx.send_async((req, ret_tx)).await.expect("todo");

        let res = ret_rx.await.expect("todo");
        res
    }
}

struct Worker {
    handle: JoinHandle<()>,
}

impl Worker {
    pub fn start(
        rx: Receiver<ReqMsg>,
        core_setter: Option<CoreSetter>,
        graph_access: Arc<HelixGraphEngine>,
        router: Arc<HelixRouter>,
    ) -> Worker {
        let handle = std::thread::spawn(move || {
            if let Some(cs) = core_setter {
                cs.set_current();
            }

            trace!("thread started");

            while let Ok((req, ret_chan)) = rx.recv() {
                let mut response = Response::new();
                if let Err(e) = router.handle(graph_access.clone(), req, &mut response) {
                    error!("Error handling request: {e:?}");
                    response.status = 500;
                    response.body = format!("\n{:?}", e).into_bytes();
                }
                ret_chan
                    .send(response)
                    .expect("Should always be able to send, as only one worker processes a request")
            }
            trace!("thread shutting down");
        });
        Worker { handle }
    }
}
