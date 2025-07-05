use helixdb::{
    helix_engine::{
        storage_core::storage_core::HelixGraphStorage,
        types::GraphError,
    },
    protocol::{
        id::v6_uuid, request::Request, response::Response,
        return_values::ReturnValue,
    },
    helix_gateway::router::router::{HandlerInput, Handler},
};
use get_routes::{get_handler, handler};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    vec::IntoIter,
};

#[get_handler]
pub fn graphvis(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let db = Arc::clone(&input.graph.storage);
    //let json_ne: String = db.get_ne_json().unwrap();
    // TODO: (func) need a preprocessing step here to add colors, arrows, shape etc.

    let nodes = json!([{"color": "#97c2fc", "id": "41234901881305623165119301202951275782", "label": "Marie Curie", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881312936930209927040952698118", "label": "physicist", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881325033567103984291497116934", "label": "chemist", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881332271516076467183241397510", "label": "radioactivity", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881343122997745549039051736326", "label": "Nobel Prize", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881392692535019506509751256326", "label": "Pierre Curie", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881401157745875213297964549382", "label": "Nobel Prize", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881409656714272574974657299718", "label": "Curie family", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881416848361917432855427024134", "label": "University of Paris", "shape": "dot"}, {"color": "#97c2fc", "id": "41234901881421742467587910211542975750", "label": "Robin Williams", "shape": "dot"}]);
    let edges = json!([{"arrows": "to", "from": "41234901881305623165119301202951275782", "title": "conducted research on", "to": "41234901881332271516076467183241397510"}, {"arrows": "to", "from": "41234901881305623165119301202951275782", "title": "was", "to": "41234901881312936930209927040952698118"}, {"arrows": "to", "from": "41234901881305623165119301202951275782", "title": "was", "to": "41234901881325033567103984291497116934"}, {"arrows": "to", "from": "41234901881305623165119301202951275782", "title": "won", "to": "41234901881343122997745549039051736326"}, {"arrows": "to", "from": "41234901881392692535019506509751256326", "title": "co-winner", "to": "41234901881343122997745549039051736326"}, {"arrows": "to", "from": "41234901881409656714272574974657299718", "title": "legacy", "to": "41234901881343122997745549039051736326"}]);

    let html_template = include_str!("graphvis.html");
    let html_content = html_template
        .replace("{NODES_JSON_DATA}", &serde_json::to_string(&nodes).unwrap())
        .replace("{EDGES_JSON_DATA}", &serde_json::to_string(&edges).unwrap());

    response.headers.insert("Content-Type".to_string(), "text/html".to_string());
    response.body = html_content.as_bytes().to_vec();
    Ok(())
}

