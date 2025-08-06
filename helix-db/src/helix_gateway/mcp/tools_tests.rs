#[cfg(test)]
mod tests {
    use crate::{
        helix_engine::{
            graph_core::{
                config::Config,
                graph_core::{HelixGraphEngine, HelixGraphEngineOpts},
                ops::{
                    bm25::search_bm25::SearchBM25Adapter, g::G, in_::{in_::InAdapter, in_e::InEdgesAdapter, to_n::ToNAdapter}, out::{from_n::FromNAdapter, out::OutAdapter, out_e::OutEdgesAdapter}, source::{
                        add_e::{AddEAdapter, EdgeType}, add_n::AddNAdapter, e_from_id::EFromIdAdapter, e_from_type::EFromTypeAdapter, n_from_id::NFromIdAdapter, n_from_index::NFromIndexAdapter, n_from_type::NFromTypeAdapter
                    }, tr_val::{Traversable, TraversalVal}, util::{
                        dedup::DedupAdapter, drop::Drop, filter_mut::FilterMut, filter_ref::FilterRefAdapter, map::MapAdapter, paths::ShortestPathAdapter, props::PropsAdapter, range::RangeAdapter, update::UpdateAdapter
                    }, vectors::{insert::InsertVAdapter, search::SearchVAdapter}
                },
            },
            types::GraphError,
        }, helix_gateway::mcp::{
            mcp::{init, InitRequest, MCPBackend, MCPConnection, MCPToolInput},
            tools::{ToolArgs, search_keyword},
        }, props, protocol::{value::Value, Request}
    };
    use tempfile::TempDir;
    use tracing::Level;
    use std::{
        sync::Arc,
        collections::HashMap,
    };

    pub fn config() -> Option<Config> {
        None
    }

    fn setup_test_environment() -> (Arc<HelixGraphEngine>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().join("test_db");

        std::fs::create_dir_all(&path).expect("Failed to create test db directory");

        let path_str = path.to_str().expect("Could not convert path to string");

        let config = config().unwrap_or_default();

        let opts = HelixGraphEngineOpts {
            path: path_str.to_string(),
            config,
        };

        let graph = Arc::new(
            HelixGraphEngine::new(opts)
            .expect("Failed to create HelixGraphEngine for testing")
        );

        (graph, temp_dir)
    }

    fn populate_test_data_for_search(graph: &Arc<HelixGraphEngine>) -> Result<(), GraphError> {
        let db = Arc::clone(&graph.storage);
        let mut txn = db.graph_env.write_txn().unwrap();

        // Example nodes with text content for BM25 search
        let test_documents = vec![
            ("doc1", "artificial intelligence machine learning"),
            ("doc2", "natural language processing algorithms"),
            ("doc3", "graph database traversal algorithms"),
            ("doc4", "search engine optimization techniques"),
            ("doc5", "machine learning neural networks"),
        ];

        for (doc_id, content) in test_documents {
            let _ = G::new_mut(Arc::clone(&db), &mut txn)
                .add_n(
                    "document",
                    Some(props! {
                        "doc_id" => doc_id.clone(),
                        "content" => content.clone()
                    }),
                    None
                ).collect_to::<Vec<_>>();
        }
        txn.commit().unwrap();

        Ok(())
    }

    #[test]
    fn test_mcp_tool_out_step() {
    }

    #[test] fn test_mcp_tool_out_e_step() {
    }

    #[test]
    fn test_mcp_tool_in_step() {
    }

    #[test]
    fn test_mcp_tool_in_e_step() {
    }

    #[test]
    fn test_mcp_tool_n_from_type() {
    }

    #[test]
    fn test_mcp_tool_e_from_type() {
    }

    #[test]
    fn test_mcp_tool_filter_items() {
    }

    // TODO
    #[test]
    fn test_mcp_tool_search_keyword() {
        let (graph, _temp_dir) = setup_test_environment();

        populate_test_data_for_search(&graph).expect("Failed to populate test data");
        let mcp_backend = graph.mcp_backend.as_ref().expect("MCP backend should be available");
        let txn = graph.storage.graph_env.read_txn().expect("Failed to create read transaction");

        let mcp_backend = graph.mcp_backend
            .as_ref()
            .expect("MCP backend should be available");

        let mut mcp_input = MCPToolInput {
            request: ,
            mcp_backend: graph.mcp_backend.as_ref().expect("...").clone(),
            mcp_connections: graph.mcp_connections.as_ref().expect("...").clone(),
            schema: None,
        };

        let init_response = init(&mut mcp_input)
            .expect("Failed to initialize MCP connection");

        // Extract connection_id from response
        let connection_id = match init_response.body {
            Some(body) => {
                // Parse the JSON response to get connection_id
                // Adjust this based on your Response/ReturnValue structure
                sonic_rs::from_str::<String>(&body)
                    .expect("Failed to parse connection_id from response")
            },
            None => panic!("Init response should contain connection_id"),
        };

        println!("Created MCP connection with ID: {}", connection_id);

        let connection = MCPConnection {
            connection_id: 123,
            iter: vec![],
        };

        let limit = 5;
        let label = "machine learning";

        let res = search_keyword(
            txn,
            connection,
            query,
            limit,
            label,
        );

        // basic search
        let result = mcp_backend.search_keyword(
            &txn,
            &connection,
            "machine learning".to_string(),
            5,
        );
        assert!(result.is_ok(), "Search should succeed");
        let search_results = result.unwrap();
        assert!(!search_results.is_empty(), "Should return search results");
        assert!(search_results.len() <= 5, "Should respect limit parameter");

        // results contain relevant documents
        let found_ml_doc = search_results.iter().any(|result| {
            // Adjust this based on how you access node content from TraversalVal
            match result.get_property("content") {
                Ok(content) => {
                    if let Value::String(content_str) = content {
                        content_str.contains("machine learning")
                    } else {
                        false
                    }
                },
                Err(_) => false,
            }
        });
        assert!(found_ml_doc, "Should find documents containing 'machine learning'");

        // different query
        let algorithm_result = mcp_backend.search_keyword(
            &txn,
            &connection,
            "algorithms".to_string(),
            3,
        );
        assert!(algorithm_result.is_ok(), "Algorithm search should succeed");
        let algorithm_results = algorithm_result.unwrap();
        assert!(!algorithm_results.is_empty(), "Should find documents with 'algorithms'");
        assert!(algorithm_results.len() <= 3, "Should respect limit of 3");

        // non-existent term
        let no_result = mcp_backend.search_keyword(
            &txn,
            &connection,
            "nonexistent_term_xyz".to_string(),
            10,
        );
        assert!(no_result.is_ok(), "Search should succeed even with no results");
        let no_results = no_result.unwrap();
        assert!(no_results.is_empty() || no_results.len() == 0, "Should return empty results for non-existent term");

        // limit 0
        let zero_limit_result = mcp_backend.search_keyword(
            &txn,
            &connection,
            "machine learning".to_string(),
            0,
        );
        assert!(zero_limit_result.is_ok(), "Search with 0 limit should succeed");
        let zero_results = zero_limit_result.unwrap();
        assert!(zero_results.is_empty(), "Should return empty results with 0 limit");

        // very large limit
        let large_limit_result = mcp_backend.search_keyword(
            &txn,
            &connection,
            "learning".to_string(),
            1000,
        );
        assert!(large_limit_result.is_ok(), "Search with large limit should succeed");
        let large_results = large_limit_result.unwrap();
        // Should return all available matching documents, not more than what exists
        assert!(large_results.len() <= 5, "Should not return more results than available documents");

        println!("✅ All search_keyword tests passed!");
        println!("   - Basic search: {} results", search_results.len());
        println!("   - Algorithm search: {} results", algorithm_results.len());
        println!("   - Non-existent term: {} results", no_results.len());
        println!("   - Zero limit: {} results", zero_results.len());
        println!("   - Large limit: {} results", large_results.len());
    }

    #[test]
    fn test_mcp_tool_search_vector_text() {
    }
}

