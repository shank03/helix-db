// router

// takes in raw [u8] data
// parses to request type

// then locks graph and passes parsed data and graph to handler to execute query

// returns response

use crate::{
    helix_engine::{graph_core::graph_core::HelixGraphEngine, types::GraphError},
    helix_gateway::mcp::mcp::{MCPHandlerFn, MCPToolInput},
    protocol::HelixError,
};
use core::fmt;
use std::{collections::HashMap, sync::Arc};

use crate::protocol::{Request, Response};

pub struct HandlerInput {
    pub request: Request,
    pub graph: Arc<HelixGraphEngine>,
}

// basic type for function pointer
pub type BasicHandlerFn = fn(&HandlerInput) -> Result<Response, GraphError>;

// thread safe type for multi threaded use
pub type HandlerFn = Arc<dyn Fn(&HandlerInput) -> Result<Response, GraphError> + Send + Sync>;

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

/// Router for handling requests and MCP requests
///
/// Standard Routes and MCP Routes are stored in a HashMap with the method and path as the key
pub struct HelixRouter {
    /// Name => Function
    pub routes: HashMap<String, HandlerFn>,
    pub mcp_routes: HashMap<String, MCPHandlerFn>,
}

impl HelixRouter {
    /// Create a new router with a set of routes
    pub fn new(
        routes: Option<HashMap<String, HandlerFn>>,
        mcp_routes: Option<HashMap<String, MCPHandlerFn>>,
    ) -> Self {
        let rts = match routes {
            Some(routes) => routes,
            None => HashMap::new(),
        };
        let mcp_rts = match mcp_routes {
            Some(routes) => routes,
            None => HashMap::new(),
        };
        Self {
            routes: rts,
            mcp_routes: mcp_rts,
        }
    }

    /// Add a route to the router
    pub fn add_route(&mut self, name: &str, handler: BasicHandlerFn) {
        self.routes.insert(name.to_string(), Arc::new(handler));
    }

    /// Handle a request by finding the appropriate handler and executing it
    ///
    /// ## Arguments
    ///
    /// * `graph_access` - A reference to the graph engine
    /// * `request` - The request to handle
    ///
    /// ## Returns
    ///
    /// * `Ok(Response)` if the request was handled successfully
    /// * `Err(RouterError)` if there was an error handling the request
    pub fn handle(
        &self,
        graph_access: Arc<HelixGraphEngine>,
        request: Request,
    ) -> Result<Response, HelixError> {
        if let Some(handler) = self.routes.get(&request.name) {
            let input = HandlerInput {
                request,
                graph: Arc::clone(&graph_access),
            };
            return handler(&input).map_err(Into::into);
        }

        if let Some(mcp_handler) = self.mcp_routes.get(&request.name) {
            let mut mcp_input = MCPToolInput {
                request,
                mcp_backend: Arc::clone(&graph_access.mcp_backend.as_ref().unwrap()),
                mcp_connections: Arc::clone(&graph_access.mcp_connections.as_ref().unwrap()),
                schema: Some(graph_access.storage.storage_config.schema.clone()),
            };
            return mcp_handler(&mut mcp_input).map_err(Into::into);
        };

        return Err(HelixError::NotFound {
            ty: request.req_type,
            name: request.name,
        });
    }
}

#[derive(Debug)]
pub enum RouterError {
    Io(std::io::Error),
    New(String),
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::Io(e) => write!(f, "IO error: {}", e),
            RouterError::New(msg) => write!(f, "Graph error: {}", msg),
        }
    }
}

impl From<String> for RouterError {
    fn from(error: String) -> Self {
        RouterError::New(error)
    }
}

impl From<std::io::Error> for RouterError {
    fn from(error: std::io::Error) -> Self {
        RouterError::Io(error)
    }
}

impl From<GraphError> for RouterError {
    fn from(error: GraphError) -> Self {
        RouterError::New(error.to_string())
    }
}

impl From<RouterError> for GraphError {
    fn from(error: RouterError) -> Self {
        GraphError::New(error.to_string())
    }
}
