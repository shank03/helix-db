use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helixdb::{
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
        router::router::HandlerInput,
    },
    helix_gateway::mcp::mcp::{
        MCPConnection, MCPHandler, MCPHandlerSubmission, MCPToolInput, McpBackend,
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
use proc_macros::{handler, mcp_handler, tool_call};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub struct Company {
    pub company_number: String,
    pub number_of_filings: i32,
}

pub struct DocumentEdge {
    pub from: Company,
    pub to: DocumentEmbedding,
    pub filing_id: String,
    pub category: String,
    pub subcategory: String,
    pub date: String,
    pub description: String,
}

pub struct DocumentEmbedding {
    pub text: String,
    pub chunk_id: String,
    pub page_number: u16,
    pub reference: String,
    pub source_link: String,
    pub source_date: String,
}

#[derive(Serialize, Deserialize)]
pub struct CompanyEmbeddingSearchInput {
    pub company_number: String,
    pub query: Vec<f64>,
    pub k: i32,
}
#[handler]
pub fn CompanyEmbeddingSearch(
    input: &HandlerInput,
    response: &mut Response,
) -> Result<(), GraphError> {
    let data: CompanyEmbeddingSearchInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let c = G::new(Arc::clone(&db), &txn)
        .n_from_index("company_number", &data.company_number)
        .out_e("DocumentEdge")
        .to_v()
        .collect_to::<Vec<_>>();
    let embedding_search = G::new_from(Arc::clone(&db), &txn, c.clone())
        .brute_force_search_v(&data.query, data.k as usize)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "embedding_search".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            embedding_search.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct GetAllCompanyEmbeddingsInput {
    pub company_number: String,
}
#[tool_call]
pub fn GetAllCompanyEmbeddings(
    input: &HandlerInput,
    response: &mut Response,
) -> Result<(), GraphError> {
    let data: GetAllCompanyEmbeddingsInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let c = G::new(Arc::clone(&db), &txn)
        .n_from_index("company_number", &data.company_number)
        .collect_to_obj();
    let embeddings = G::new_from(Arc::clone(&db), &txn, c.clone())
        .out("DocumentEdge", &EdgeType::Vec)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "embeddings".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            embeddings.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct HasCompanyInput {
    pub company_number: String,
}
#[handler]
pub fn HasCompany(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: HasCompanyInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let company = G::new(Arc::clone(&db), &txn)
        .n_from_index("company_number", &data.company_number)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "company".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct HasDocumentEmbeddingsInput {
    pub company_number: String,
}
#[handler]
pub fn HasDocumentEmbeddings(
    input: &HandlerInput,
    response: &mut Response,
) -> Result<(), GraphError> {
    let data: HasDocumentEmbeddingsInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let c = G::new(Arc::clone(&db), &txn)
        .n_from_index("company_number", &data.company_number)
        .collect_to_obj();
    let embeddings = G::new_from(Arc::clone(&db), &txn, c.clone())
        .out("DocumentEdge", &EdgeType::Vec)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "embeddings".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            embeddings.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct SearchVectorInput {
    pub query: Vec<f64>,
    pub k: i32,
}
#[handler]
pub fn SearchVector(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: SearchVectorInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let embedding_search = G::new(Arc::clone(&db), &txn)
        .search_v::<fn(&HVector, &RoTxn) -> bool>(&data.query, data.k as usize, None)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "embedding_search".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            embedding_search.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct AddVectorInput {
    pub vector: Vec<f64>,
    pub text: String,
    pub chunk_id: String,
    pub page_number: i32,
    pub reference: String,
}
#[handler]
pub fn AddVector(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: AddVectorInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    let embedding = G::new_mut(Arc::clone(&db), &mut txn)
.insert_v::<fn(&HVector, &RoTxn) -> bool>(&data.vector, "DocumentEmbedding", Some(props! { "text" => data.text.clone(), "reference" => data.reference.clone(), "chunk_id" => data.chunk_id.clone(), "page_number" => data.page_number.clone() })).collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "embedding".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            embedding.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct DeleteCompanyInput {
    pub company_number: String,
}
#[handler]
pub fn DeleteCompany(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: DeleteCompanyInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_index("company_number", &data.company_number)
            .out("DocumentEdge", &EdgeType::Vec)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_index("company_number", &data.company_number)
            .collect_to_obj(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "success".to_string(),
        ReturnValue::from(Value::from("success")),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct embeddings_dataData {
    pub reference: String,
    pub date1: String,
    pub date2: String,
    pub chunk_id: String,
    pub description: String,
    pub category: String,
    pub vector: Vec<f64>,
    pub page_number: i32,
    pub source: String,
    pub subcategory: String,
    pub filing_id: String,
    pub text: String,
}
#[derive(Serialize, Deserialize)]
pub struct AddEmbeddingsToCompanyInput {
    pub company_number: String,
    pub embeddings_data: Vec<embeddings_dataData>,
}
#[handler]
pub fn AddEmbeddingsToCompany(
    input: &HandlerInput,
    response: &mut Response,
) -> Result<(), GraphError> {
    let data: AddEmbeddingsToCompanyInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    let c = G::new(Arc::clone(&db), &txn)
        .n_from_index("company_number", &data.company_number)
        .collect_to_obj();
    for embeddings_dataData {
        vector,
        text,
        chunk_id,
        page_number,
        reference,
        filing_id,
        category,
        subcategory,
        date1,
        date2,
        source,
        description,
    } in data.embeddings_data
    {
        let embedding = G::new_mut(Arc::clone(&db), &mut txn)
.insert_v::<fn(&HVector, &RoTxn) -> bool>(&vector, "DocumentEmbedding", Some(props! { "text" => text.clone(), "page_number" => page_number.clone(), "chunk_id" => chunk_id.clone(), "source_link" => source.clone(), "source_date" => date1.clone(), "reference" => reference.clone() })).collect_to_obj();
        let edges = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("DocumentEdge", Some(props! { "category" => category.clone(), "date" => date2.clone(), "description" => description.clone(), "filing_id" => filing_id.clone(), "subcategory" => subcategory.clone() }), c.id(), embedding.id(), true, EdgeType::Node).collect_to_obj();
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "success".to_string(),
        ReturnValue::from(Value::from("success")),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct AddCompanyInput {
    pub company_number: String,
    pub number_of_filings: i32,
}
#[handler]
pub fn AddCompany(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: AddCompanyInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    let company = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Company", Some(props! { "number_of_filings" => &data.number_of_filings, "company_number" => &data.company_number }), Some(&["company_number"])).collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "company".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[handler]
pub fn DeleteAll(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Company")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "success".to_string(),
        ReturnValue::from(Value::from("success")),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[handler]
pub fn GetCompanies(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let companies = G::new(Arc::clone(&db), &txn)
        .n_from_type("Company")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "companies".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            companies.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
