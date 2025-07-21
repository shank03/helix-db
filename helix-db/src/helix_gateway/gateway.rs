use std::sync::atomic::{self, AtomicUsize};
use std::{collections::HashMap, sync::Arc};

use axum::body::{Body, Bytes};
use axum::extract::Path;
use axum::routing::post;
use core_affinity::{CoreId, set_for_current};
use tokio::sync::oneshot;
use tracing::{trace, warn};

use super::connection::connection::ConnectionHandler;
use super::router::router::{HandlerFn, HelixRouter};
use crate::helix_gateway::thread_pool::ThreadPool;
use crate::{
    helix_engine::graph_core::graph_core::HelixGraphEngine, helix_gateway::mcp::mcp::MCPHandlerFn,
};

pub struct GatewayOpts {}

impl GatewayOpts {
    pub const DEFAULT_POOL_SIZE: usize = 8;
}

pub struct HelixGateway {
    pub connection_handler: ConnectionHandler,
}

const IO_CORE_NUM: usize = 2;

impl HelixGateway {
    pub fn new(
        address: &str,
        graph: Arc<HelixGraphEngine>,
        size: usize,
        routes: Option<HashMap<String, HandlerFn>>,
        mcp_routes: Option<HashMap<String, MCPHandlerFn>>,
    ) -> HelixGateway {
        let router = HelixRouter::new(routes, mcp_routes);
        let connection_handler = ConnectionHandler::new(address, graph, size, router).unwrap();
        println!("Gateway created");
        HelixGateway { connection_handler }
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let (io_setter, worker_setter) = match core_affinity::get_core_ids() {
            Some(all_cores) => {
                let io_cores = CoreSetter::new(&all_cores[0..IO_CORE_NUM]);
                let worker_cores = CoreSetter::new(&all_cores[IO_CORE_NUM..]);
                (Some(io_cores), Some(worker_cores))
            }
            None => {
                warn!("Failed to get core ids");
                (None, None)
            }
        };

        // let worker_pool = ThreadPool

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(IO_CORE_NUM)
            .on_thread_start({
                let local_setter = io_setter.clone();
                move || {
                    if let Some(s) = &local_setter {
                        s.set_current();
                    }
                }
            })
            .build()?;

        let axum_app = axum::Router::new().route("/*path", post(post_handler));
        rt.spawn(async move {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:7000").await.unwrap();
            axum::serve(listener, axum_app).await.unwrap()
        });

        Ok(())
    }
}

async fn post_handler(req: crate::protocol::request::Request) -> axum::http::Response<Body> {
    let ret_chan = oneshot::channel::<crate::protocol::response::Response>();

    axum::http::Response::new(Body::empty())
}

#[derive(Clone)]
pub struct CoreSetter(Arc<CoreSetterInner>);

pub struct CoreSetterInner {
    cores: Vec<CoreId>,
    index: AtomicUsize,
}

impl CoreSetter {
    pub fn new(cores: &[CoreId]) -> Self {
        Self(Arc::new(CoreSetterInner {
            cores: cores.to_vec(),
            index: AtomicUsize::new(0),
        }))
    }

    pub fn set_current(&self) {
        let inner = &self.0;
        let idx = inner.index.fetch_add(1, atomic::Ordering::SeqCst);
        match inner.cores.get(idx) {
            Some(c) => {
                set_for_current(*c);
                trace!("Set core affinity to: {c:?}");
            }
            None => warn!("Tried to set core affinity, but all cores already used"),
        };
    }
}
