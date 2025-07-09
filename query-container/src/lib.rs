use std::sync::Arc;

use helixdb::{
    helix_engine::types::GraphError,
    helix_gateway::router::{QueryHandler, router::HandlerInput},
    protocol::response::Response,
};

mod query;

type DynQueryFn = fn(&HandlerInput, &mut Response) -> Result<(), GraphError>;

// basic type for function pointer
pub type BasicHandlerFn = fn(&HandlerInput, &mut Response) -> Result<(), GraphError>;

// thread safe type for multi threaded use
pub type HandlerFn = Arc<dyn QueryHandler + Send + Sync>;

#[derive(Clone, Debug)]
pub struct HandlerSubmission(pub Handler);

#[derive(Clone, Debug)]
pub struct Handler {
    pub name: &'static str,
    pub func: BasicHandlerFn,
}

impl Handler {
    pub const fn new(name: &'static str, func: BasicHandlerFn) -> Self {
        Self { name, func }
    }
}

inventory::collect!(HandlerSubmission);

#[unsafe(no_mangle)]
pub extern "Rust" fn get_queries() -> Vec<(String, DynQueryFn)> {
    let submissions: Vec<_> = inventory::iter::<HandlerSubmission>.into_iter().collect();

    todo!()
}
