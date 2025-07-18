

use heed3::RoTxn;
use helix_macros::{handler, tool_call, mcp_handler};
use helix_db::{
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
        mcp::mcp::{MCPHandlerSubmission, MCPToolInput, MCPHandler}
    },
    node_matches, props, embed,
    field_remapping, identifier_remapping, 
    traversal_remapping, exclude_field, value_remapping, 
    protocol::{
        remapping::{Remapping, RemappingMap, ResponseRemapping},
        response::Response,
        return_values::ReturnValue,
        value::Value,
        format::Format,
    },
    utils::{
        count::Count,
        filterable::Filterable,
        id::ID,
        items::{Edge, Node},
    },
};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use chrono::{DateTime, Utc};
    
pub struct Continent {
    pub name: String,
}

pub struct Country {
    pub name: String,
    pub currency: String,
    pub population: i64,
    pub gdp: f64,
}

pub struct City {
    pub name: String,
    pub description: String,
    pub zip_codes: Vec<String>,
}

pub struct Continent_to_Country {
    pub from: Continent,
    pub to: Country,
}

pub struct Country_to_City {
    pub from: Country,
    pub to: City,
}

pub struct Country_to_Capital {
    pub from: Country,
    pub to: City,
}


#[derive(Serialize, Deserialize)]
pub struct getContinentCitiesInput {

pub continent_name: String,
pub k: i64
}
#[handler(with_read)]
pub fn getContinentCities (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let continent = G::new(Arc::clone(&db), &txn)
.n_from_type("Continent")

.filter_ref(|val, txn|{
                if let Ok(val) = val { 
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())

.map_value_or(false, |v| *v == data.continent_name.clone())?)
                } else {
                    Ok(false)
                }
            }).collect_to::<Vec<_>>();
    let countries = G::new_from(Arc::clone(&db), &txn, continent.clone())

.out("Continent_to_Country",&EdgeType::Node).collect_to::<Vec<_>>();
    let cities = G::new_from(Arc::clone(&db), &txn, countries.clone())

.out("Country_to_City",&EdgeType::Node)

.range(0, data.k.clone()).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("cities".to_string(), ReturnValue::from_traversal_value_array_with_mixin(cities.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createContinentInput {

pub name: String
}
#[handler(with_write)]
pub fn createContinent (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let continent = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Continent", Some(props! { "name" => &data.name }), None).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("continent".to_string(), ReturnValue::from_traversal_value_with_mixin(continent.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createCountryInput {

pub continent_id: ID,
pub name: String,
pub currency: String,
pub population: i64,
pub gdp: f64
}
#[handler(with_write)]
pub fn createCountry (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let country = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Country", Some(props! { "gdp" => &data.gdp, "name" => &data.name, "population" => &data.population, "currency" => &data.currency }), None).collect_to_obj();
    let continent = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.continent_id).collect_to_obj();
    let continent_country = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("Continent_to_Country", None, continent.id(), country.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("country".to_string(), ReturnValue::from_traversal_value_with_mixin(country.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}
