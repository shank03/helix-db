use libloading::{self, Library, Symbol};
use std::{
    collections::HashMap,
    error::Error,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    helix_engine::types::GraphError,
    helix_gateway::router::{
        router::{HandlerFn, HandlerInput, HelixRouter},
        QueryHandler,
    },
    protocol::response::Response,
};

use crate::protocol::request::Method;

#[derive(Clone)]
pub struct DynHandler {
    // holding this guarentees that the Symbol is still valid
    _source: Arc<Library>,
    func: extern "Rust" fn(&HandlerInput, &mut Response) -> Result<(), GraphError>,
}

impl QueryHandler for DynHandler {
    fn handle(&self, input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
        (self.func)(input, response)
    }
}

type DynQueryFn = extern "Rust" fn(&HandlerInput, &mut Response) -> Result<(), GraphError>;
type GetQueryFn = extern "Rust" fn() -> Vec<(String, DynQueryFn)>;

pub struct Plugin {
    lib: Arc<Library>,
}

impl Plugin {
    /// SAFETY: This must be called with a path to Helix query dynamic library, compiled with the same version of Rust as the main database
    pub unsafe fn open(lib_path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let lib = Library::new(lib_path.as_ref())?;
        Ok(Plugin { lib: Arc::new(lib) })
    }

    pub fn get_queries(&self) -> Result<HashMap<(Method, String), HandlerFn>, Box<dyn Error>> {
        // SAFETY: If a valid file was opened it will have a get_queries function of this type
        let get_fn: Symbol<GetQueryFn> = unsafe { self.lib.get(b"get_queries")? };

        println!("before get_fn call");
        let queries = get_fn();

        let mut acc: HashMap<(Method, String), HandlerFn> = HashMap::new();

        for (n, func) in queries.into_iter() {
            let handler = DynHandler {
                _source: self.lib.clone(),
                func,
            };

            acc.insert((Method::POST, format!("/{n}")), Arc::new(handler));
        }
        Ok(acc)
    }

    pub async fn add_queries(&self, router: &HelixRouter) -> Result<(), Box<dyn Error>> {
        let queries = self.get_queries()?;
        let mut guard = router.routes.write().await;

        guard.extend(queries);

        Ok(())
    }
}
