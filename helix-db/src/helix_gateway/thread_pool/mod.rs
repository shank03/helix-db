use crate::helix_engine::graph_core::graph_core::HelixGraphEngine;
use crate::helix_gateway::gateway::CoreSetter;
use flume::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tokio::net::TcpStream;

use crate::helix_gateway::router::router::{HelixRouter, RouterError};
use crate::protocol::request::Request;
use crate::protocol::response::Response;

pub struct WorkerPool {
    tx: Sender<TcpStream>,
    workers: Vec<Worker>,
}

impl WorkerPool {
    pub fn new(
        size: usize,
        core_setter: Option<CoreSetter>,
        graph: Arc<HelixGraphEngine>,
        router: Arc<HelixRouter>,
    ) -> WorkerPool {
        assert!(
            size > 0,
            "Expected number of threads in thread pool to be more than 0, got {}",
            size
        );

        let (tx, rx) = flume::bounded::<TcpStream>(1000); // TODO: make this configurable
        let workers = (0..size)
            .map(|_| {
                Worker::start(
                    rx.clone(),
                    core_setter.clone(),
                    graph.clone(),
                    router.clone(),
                )
            })
            .collect::<Vec<_>>();

        WorkerPool { tx, workers }
    }
}

pub struct Worker {
    handle: JoinHandle<()>,
}

impl Worker {
    pub fn start(
        rx: Receiver<TcpStream>,
        core_setter: Option<CoreSetter>,
        graph: Arc<HelixGraphEngine>,
        router: Arc<HelixRouter>,
    ) -> Worker {
        let handle = std::thread::spawn(move || {
            if let Some(cs) = core_setter {
                cs.set_current();
            }
            loop {}
        });
        Worker { handle }
    }
}
