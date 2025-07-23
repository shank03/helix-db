use helix_db::helixc::prelude::bin::*;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish()
        .init();
    let config = queries::config().unwrap_or(Config::default());

    let path = match std::env::var("HELIX_DATA_DIR") {
        Ok(val) => std::path::PathBuf::from(val).join("user"),
        Err(_) => {
            println!("HELIX_DATA_DIR not set, using default");
            let home = dirs::home_dir().expect("Could not retrieve home directory");
            home.join(".helix/user")
        }
    };

    let port = match std::env::var("HELIX_PORT") {
        Ok(val) => val.parse::<u16>().unwrap(),
        Err(_) => 6969,
    };

    println!("Running with the following setup:");
    println!("\tconfig: {:?}", config);
    println!("\tpath: {}", path.display());
    println!("\tport: {}", port);

    let path_str = path.to_str().expect("Could not convert path to string");
    let opts = HelixGraphEngineOpts {
        path: path_str.to_string(),
        config,
    };

    let graph = Arc::new(HelixGraphEngine::new(opts).unwrap());

    // generates routes from handler proc macro
    let submissions: Vec<_> = inventory::iter::<HandlerSubmission>.into_iter().collect();
    println!("Found {} route submissions", submissions.len());

    let query_routes: HashMap<String, HandlerFn> = inventory::iter::<HandlerSubmission>
        .into_iter()
        .map(|submission| {
            println!(
                "Processing POST submission for handler: {}",
                submission.0.name
            );
            let handler = &submission.0;
            let func: HandlerFn = Arc::new(handler.func);
            (handler.name.to_string(), func)
        })
        .collect();

    let mcp_routes = inventory::iter::<MCPHandlerSubmission>
        .into_iter()
        .into_iter()
        .map(|submission| {
            println!("Processing submission for handler: {}", submission.0.name);
            let handler = &submission.0;
            let func: MCPHandlerFn = Arc::new(handler.func);
            (handler.name.to_string(), func)
        })
        .collect::<HashMap<String, MCPHandlerFn>>();

    println!("Routes: {:?}", query_routes.keys());
    let gateway = HelixGateway::new(
        &format!("0.0.0.0:{}", port),
        graph,
        GatewayOpts::DEFAULT_POOL_SIZE,
        2,
        Some(query_routes),
        Some(mcp_routes),
    );

    gateway.run().unwrap()
}
