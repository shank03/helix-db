use crate::helix_engine::graph_core::graph_core::HelixGraphEngine;
use crate::helix_gateway::gateway::CoreSetter;
use crate::protocol::{self, HelixError};
use flume::{Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use tokio::sync::oneshot;
use tracing::trace;

use crate::helix_gateway::router::router::HelixRouter;
use crate::protocol::request::ReqMsg;
use crate::protocol::response::Response;

/// A Thread Pool of workers to execute Database operations
pub struct WorkerPool {
    tx: Sender<ReqMsg>,
    _workers: Vec<Worker>,
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

        WorkerPool {
            tx,
            _workers: workers,
        }
    }

    /// Process a request on the Worker Pool
    pub async fn process(&self, req: protocol::request::Request) -> Result<Response, HelixError> {
        let (ret_tx, ret_rx) = oneshot::channel();

        // this read by Worker in start()
        self.tx
            .send_async((req, ret_tx))
            .await
            .expect("todo: request on closing channel");

        let res = ret_rx.await.expect("todo");
        res
    }
}

struct Worker {
    _handle: JoinHandle<()>,
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
                let res = router.handle(graph_access.clone(), req);

                ret_chan
                    .send(res)
                    .expect("Should always be able to send, as only one worker processes a request")
            }
            trace!("thread shutting down");
        });
        Worker { _handle: handle }
    }
}
