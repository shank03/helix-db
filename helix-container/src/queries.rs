use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helix_db::{
    embed, exclude_field, field_remapping,
    helix_engine::{
        graph_core::ops::{
            bm25::search_bm25::SearchBM25Adapter,
            g::G,
            in_::{in_::InAdapter, in_e::InEdgesAdapter, to_n::ToNAdapter, to_v::ToVAdapter},
            out::{
                from_n::FromNAdapter, from_v::FromVAdapter, out::OutAdapter, out_e::OutEdgesAdapter,
            },
            source::{
                add_e::{AddEAdapter, EdgeType},
                add_n::AddNAdapter,
                e_from_id::EFromIdAdapter,
                e_from_type::EFromTypeAdapter,
                n_from_id::NFromIdAdapter,
                n_from_index::NFromIndexAdapter,
                n_from_type::NFromTypeAdapter,
            },
            tr_val::{Traversable, TraversalVal},
            util::{
                dedup::DedupAdapter, drop::Drop, exist::Exist, filter_mut::FilterMut,
                filter_ref::FilterRefAdapter, map::MapAdapter, paths::ShortestPathAdapter,
                props::PropsAdapter, range::RangeAdapter, update::UpdateAdapter,
            },
            vectors::{
                brute_force_search::BruteForceSearchVAdapter, insert::InsertVAdapter,
                search::SearchVAdapter,
            },
        },
        types::GraphError,
        vector_core::vector::HVector,
    },
    helix_gateway::{
        embedding_providers::embedding_providers::{EmbeddingModel, get_embedding_model},
        mcp::mcp::{MCPHandler, MCPHandlerSubmission, MCPToolInput},
        router::router::HandlerInput,
    },
    identifier_remapping, node_matches, props,
    protocol::{
        remapping::{Remapping, RemappingMap, ResponseRemapping},
        response::Response,
        return_values::ReturnValue,
        value::Value,
    },
    traversal_remapping,
    utils::{
        count::Count,
        filterable::Filterable,
        id::ID,
        items::{Edge, Node},
    },
    value_remapping,
};
use helix_macros::{handler, mcp_handler, tool_call};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub struct File {
    pub name: String,
    pub extension: String,
    pub text: String,
    pub extracted_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct getFileMultInput {
    pub file_id: ID,
}
#[handler(with_read)]
pub fn getFileMult(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        let file = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.file_id)
            .collect_to_obj();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("file".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, file.clone())

.map_traversal(|item, txn| { traversal_remapping!(remapping_vals, item.clone(), "text" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("text").collect_to::<Vec<_>>())?;
traversal_remapping!(remapping_vals, item.clone(), "name" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("name").collect_to::<Vec<_>>())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createFileInput {
    pub name: String,
    pub extension: String,
    pub text: String,
}
#[handler(with_write)]
pub fn createFile(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        let file = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("File", Some(props! { "extension" => &data.extension, "extracted_at" => chrono::Utc::now().to_rfc3339(), "name" => &data.name, "text" => &data.text }), None).collect_to_obj();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "file".to_string(),
            ReturnValue::from_traversal_value_with_mixin(
                file.clone().clone(),
                remapping_vals.borrow_mut(),
            ),
        );
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getAllFilesInput {}
#[handler(with_read)]
pub fn getAllFiles(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        let files = G::new(Arc::clone(&db), &txn)
            .n_from_type("File")
            .collect_to::<Vec<_>>();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "files".to_string(),
            ReturnValue::from_traversal_value_array_with_mixin(
                files.clone().clone(),
                remapping_vals.borrow_mut(),
            ),
        );
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getAllFiles2Input {}
#[handler(with_read)]
pub fn getAllFiles2(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        let files = G::new(Arc::clone(&db), &txn)
            .n_from_type("File")
            .collect_to::<Vec<_>>();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("files".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, files.clone())

.map_traversal(|item, txn| { traversal_remapping!(remapping_vals, item.clone(), "file_id" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("id").collect_to_obj())?;
traversal_remapping!(remapping_vals, item.clone(), "name" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("name").collect_to_obj())?;
traversal_remapping!(remapping_vals, item.clone(), "extension" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("extension").collect_to_obj())?;
traversal_remapping!(remapping_vals, item.clone(), "extracted_at" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("extracted_at").collect_to_obj())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getAllFileIdsInput {}
#[handler(with_read)]
pub fn getAllFileIds(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        let files = G::new(Arc::clone(&db), &txn)
            .n_from_type("File")
            .collect_to::<Vec<_>>();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "files".to_string(),
            ReturnValue::from_traversal_value_array_with_mixin(
                G::new_from(Arc::clone(&db), &txn, files.clone())
                    .check_property("id")
                    .collect_to::<Vec<_>>()
                    .clone(),
                remapping_vals.borrow_mut(),
            ),
        );
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getFileTextInput {
    pub file_id: ID,
}
#[handler(with_read)]
pub fn getFileText(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        let file = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.file_id)
            .collect_to_obj();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "file".to_string(),
            ReturnValue::from_traversal_value_with_mixin(
                G::new_from(Arc::clone(&db), &txn, file.clone())
                    .check_property("text")
                    .collect_to_obj()
                    .clone(),
                remapping_vals.borrow_mut(),
            ),
        );
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getAllFiles1Input {}
#[handler(with_read)]
pub fn getAllFiles1(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        let files = G::new(Arc::clone(&db), &txn)
            .n_from_type("File")
            .collect_to::<Vec<_>>();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "files".to_string(),
            ReturnValue::from_traversal_value_array_with_mixin(
                G::new_from(Arc::clone(&db), &txn, files.clone())
                    .map_traversal(|item, txn| {
                        exclude_field!(remapping_vals, item.clone(), "text")?;
                        Ok(item)
                    })
                    .collect_to::<Vec<_>>()
                    .clone(),
                remapping_vals.borrow_mut(),
            ),
        );
    }
    Ok(())
}
