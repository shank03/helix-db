

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
    
pub struct User {
    pub phone: String,
    pub email: String,
    pub bio: String,
    pub age: i32,
    pub location: String,
    pub profilePic: String,
    pub color: String,
    pub elo: f64,
    pub sender: String,
}

pub struct Metadata {
    pub created_ts: DateTime<Utc>,
    pub last_updated_ts: DateTime<Utc>,
    pub archetype: bool,
    pub referredBy: String,
}

pub struct Name {
    pub first: String,
    pub last: String,
}

pub struct MetadataNotes {
    pub aiScore: i64,
    pub userScore: i64,
    pub text: String,
    pub flagged: bool,
}

pub struct WarmConnect {
    pub name: String,
    pub email: String,
}

pub struct LinkedinInfo {
    pub url: String,
}

pub struct LinkedinContent {
    pub name: String,
    pub email: String,
    pub linkedin_url: String,
    pub full_name: String,
    pub first_name: String,
    pub last_name: String,
    pub public_id: String,
    pub profile_picture: String,
    pub background_picture: String,
    pub current_position: String,
    pub summary: String,
    pub industry: String,
    pub region: String,
    pub country: String,
    pub country_code: String,
    pub connection_count: i64,
    pub follower_count: i64,
    pub languages: Vec<String>,
    pub skills: Vec<String>,
    pub certifications: Vec<String>,
    pub position_start_date: String,
    pub position_end_date: String,
    pub extracted_at: DateTime<Utc>,
    pub data_source: String,
}

pub struct LinkedinWebsite {
    pub url: String,
    pub category: String,
}

pub struct LinkedinExperience {
    pub company: String,
    pub title: String,
    pub field: String,
    pub date_start: String,
    pub date_end: String,
    pub location: String,
    pub description: String,
}

pub struct LinkedinCompany {
    pub name: String,
    pub domain: String,
    pub industry: String,
    pub staff_count: i64,
    pub founded: i64,
    pub website: String,
    pub headquarters: String,
    pub description: String,
    pub specialties: Vec<String>,
}

pub struct LinkedinEducation {
    pub school: String,
    pub field: String,
    pub title: String,
    pub date_start: String,
    pub date_end: String,
    pub location: String,
    pub description: String,
}

pub struct User_to_Name {
    pub from: User,
    pub to: Name,
    pub first: String,
}

pub struct User_to_Metadata {
    pub from: User,
    pub to: Metadata,
    pub created_ts: DateTime<Utc>,
    pub last_updated_ts: DateTime<Utc>,
}

pub struct User_to_EmbeddedBio {
    pub from: User,
    pub to: EmbeddedBio,
}

pub struct Metadata_to_MetadataNotes {
    pub from: Metadata,
    pub to: MetadataNotes,
}

pub struct Metadata_to_LinkedinInfo {
    pub from: Metadata,
    pub to: LinkedinInfo,
    pub url: String,
}

pub struct Metadata_to_WarmConnect {
    pub from: Metadata,
    pub to: WarmConnect,
}

pub struct LinkedinInfo_to_LinkedinContent {
    pub from: LinkedinInfo,
    pub to: LinkedinContent,
}

pub struct LinkedinContent_to_LinkedinWebsite {
    pub from: LinkedinContent,
    pub to: LinkedinWebsite,
}

pub struct LinkedinContent_to_LinkedinExperience {
    pub from: LinkedinContent,
    pub to: LinkedinExperience,
}

pub struct LinkedinContent_to_LinkedinCompany {
    pub from: LinkedinContent,
    pub to: LinkedinCompany,
}

pub struct LinkedinContent_to_LinkedinEducation {
    pub from: LinkedinContent,
    pub to: LinkedinEducation,
}

pub struct LinkedinContent_to_CurrentCompany {
    pub from: LinkedinContent,
    pub to: LinkedinCompany,
}

pub struct EmbeddedBio {
    pub bio: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct deleteLinkedinEducationInput {

pub education_id: ID
}
#[handler(with_write)]
pub fn deleteLinkedinEducation (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.education_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteEmbeddedBioInput {

pub user_id: ID
}
#[handler(with_write)]
pub fn deleteEmbeddedBio (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id)

.out_e("User_to_EmbeddedBio").collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createLinkedinWebsiteInput {

pub user_id: ID,
pub url: String,
pub category: String
}
#[handler(with_write)]
pub fn createLinkedinWebsite (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_website = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinWebsite", Some(props! { "category" => &data.category, "url" => &data.url }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content_linkedin_website = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("LinkedinContent_to_LinkedinWebsite", None, linkedin_content.id(), linkedin_website.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_website".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_website.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteLinkedinInfoInput {

pub linkedin_info_id: ID
}
#[handler(with_write)]
pub fn deleteLinkedinInfo (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_info_id)

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_info_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createLinkedinEducationInput {

pub user_id: ID,
pub school: String,
pub field: String,
pub title: String,
pub date_start: String,
pub date_end: String,
pub location: String,
pub description: String
}
#[handler(with_write)]
pub fn createLinkedinEducation (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_education = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinEducation", Some(props! { "date_end" => &data.date_end, "field" => &data.field, "date_start" => &data.date_start, "school" => &data.school, "location" => &data.location, "description" => &data.description, "title" => &data.title }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content_linkedin_education = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("LinkedinContent_to_LinkedinEducation", None, linkedin_content.id(), linkedin_education.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_education".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_education.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteLinkedinExperienceInput {

pub experience_id: ID
}
#[handler(with_write)]
pub fn deleteLinkedinExperience (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.experience_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct addLinkedinCompanyInput {

pub user_id: ID,
pub linkedin_company_id: ID
}
#[handler(with_write)]
pub fn addLinkedinCompany (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_company = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_company_id).collect_to_obj();
    let linkedin_content_linkedin_company = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("LinkedinContent_to_LinkedinCompany", None, linkedin_content.id(), linkedin_company.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_content_linkedin_company".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_content_linkedin_company.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createLinkedinCompanyInput {

pub user_id: ID,
pub name: String,
pub domain: String,
pub industry: String,
pub staff_count: i64,
pub founded: i64,
pub website: String,
pub headquarters: String,
pub description: String,
pub specialties: Vec<String>
}
#[handler(with_write)]
pub fn createLinkedinCompany (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_company = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinCompany", Some(props! { "description" => &data.description, "industry" => &data.industry, "headquarters" => &data.headquarters, "name" => &data.name, "staff_count" => &data.staff_count, "founded" => &data.founded, "domain" => &data.domain, "specialties" => &data.specialties, "website" => &data.website }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content_linkedin_company = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("LinkedinContent_to_LinkedinCompany", None, linkedin_content.id(), linkedin_company.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_company".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_company.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserLinkedinInfoInput {

pub user_id: ID
}
#[tool_call(linkedin_info, with_read)]
#[handler(with_read)]
pub fn getUserLinkedinInfo (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_info".to_string(), ReturnValue::from_traversal_value_array_with_mixin(linkedin_info.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getAllUsersInput {


}
#[tool_call(users, with_read)]
#[handler(with_read)]
pub fn getAllUsers (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let users = G::new(Arc::clone(&db), &txn)
.n_from_type("User").collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("users".to_string(), ReturnValue::from_traversal_value_array_with_mixin(users.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateLinkedinInfoInput {

pub linkedin_info_id: ID,
pub url: String
}
#[handler(with_write)]
pub fn updateLinkedinInfo (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_info = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_info_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "url" => &data.url }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_info".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_info.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteLinkedinCompanyInput {

pub company_id: ID
}
#[handler(with_write)]
pub fn deleteLinkedinCompany (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.company_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createMetadataInput {

pub user_id: ID,
pub created_ts: DateTime<Utc>,
pub last_updated_ts: DateTime<Utc>,
pub archetype: bool,
pub referredBy: String
}
#[handler(with_write)]
pub fn createMetadata (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let metadata = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Metadata", Some(props! { "archetype" => &data.archetype, "last_updated_ts" => &data.last_updated_ts, "created_ts" => &data.created_ts, "referredBy" => &data.referredBy }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let user_metadata = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("User_to_Metadata", Some(props! { "created_ts" => data.created_ts.clone(), "last_updated_ts" => data.last_updated_ts.clone() }), user.id(), metadata.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("metadata".to_string(), ReturnValue::from_traversal_value_with_mixin(metadata.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserLinkedinCurrentCompanyInput {

pub user_id: ID
}
#[tool_call(current_company, with_read)]
#[handler(with_read)]
pub fn getUserLinkedinCurrentCompany (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let current_company = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())

.out("LinkedinContent_to_CurrentCompany",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("current_company".to_string(), ReturnValue::from_traversal_value_array_with_mixin(current_company.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteUserInput {

pub user_id: ID
}
#[handler(with_write)]
pub fn deleteUser (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id)

.out("User_to_Name",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id)

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id)

.out_e("User_to_EmbeddedBio").collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateLinkedinCompanyInput {

pub company_id: ID,
pub name: String,
pub domain: String,
pub industry: String,
pub staff_count: i64,
pub founded: i64,
pub website: String,
pub headquarters: String,
pub description: String,
pub specialties: Vec<String>
}
#[handler(with_write)]
pub fn updateLinkedinCompany (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_company = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.company_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "name" => &data.name, "domain" => &data.domain, "industry" => &data.industry, "staff_count" => &data.staff_count, "founded" => &data.founded, "website" => &data.website, "headquarters" => &data.headquarters, "description" => &data.description, "specialties" => &data.specialties }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_company".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_company.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateMetadataNotesInput {

pub metadata_notes_id: ID,
pub aiScore: i64,
pub userScore: i64,
pub text: String,
pub flagged: bool
}
#[handler(with_write)]
pub fn updateMetadataNotes (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let metadata_notes = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.metadata_notes_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "aiScore" => &data.aiScore, "userScore" => &data.userScore, "text" => &data.text, "flagged" => &data.flagged }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("metadata_notes".to_string(), ReturnValue::from_traversal_value_with_mixin(metadata_notes.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createLinkedinExperienceInput {

pub user_id: ID,
pub company: String,
pub title: String,
pub field: String,
pub date_start: String,
pub date_end: String,
pub location: String,
pub description: String
}
#[handler(with_write)]
pub fn createLinkedinExperience (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_experience = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinExperience", Some(props! { "location" => &data.location, "title" => &data.title, "field" => &data.field, "description" => &data.description, "date_start" => &data.date_start, "date_end" => &data.date_end, "company" => &data.company }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content_linkedin_experience = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("LinkedinContent_to_LinkedinExperience", None, linkedin_content.id(), linkedin_experience.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_experience".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_experience.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserLinkedinExperiencesInput {

pub user_id: ID
}
#[tool_call(linkedin_experiences, with_read)]
#[handler(with_read)]
pub fn getUserLinkedinExperiences (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_experiences = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())

.out("LinkedinContent_to_LinkedinExperience",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_experiences".to_string(), ReturnValue::from_traversal_value_array_with_mixin(linkedin_experiences.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteNameInput {

pub name_id: ID
}
#[handler(with_write)]
pub fn deleteName (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.name_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createWarmConnectInput {

pub user_id: ID,
pub name: String,
pub email: String
}
#[handler(with_write)]
pub fn createWarmConnect (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let warm_connect = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("WarmConnect", Some(props! { "email" => &data.email, "name" => &data.name }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let metadata_to_warm_connect = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("Metadata_to_WarmConnect", None, metadata.id(), warm_connect.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("warm_connect".to_string(), ReturnValue::from_traversal_value_with_mixin(warm_connect.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct searchUsersByBioInput {

pub bio_vector: Vec<f64>,
pub k: i64
}
#[tool_call(users, with_read)]
#[handler(with_read)]
pub fn searchUsersByBio (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let similar_bios = G::new(Arc::clone(&db), &txn)
.search_v::<fn(&HVector, &RoTxn) -> bool>(&data.bio_vector, data.k as usize, None).collect_to::<Vec<_>>();
    let users = G::new_from(Arc::clone(&db), &txn, similar_bios.clone())

.in_("User_to_EmbeddedBio",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("users".to_string(), ReturnValue::from_traversal_value_array_with_mixin(users.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateLinkedinContentInput {

pub linkedin_content_id: ID,
pub name: String,
pub email: String,
pub linkedin_url: String,
pub full_name: String,
pub first_name: String,
pub last_name: String,
pub public_id: String,
pub profile_picture: String,
pub background_picture: String,
pub current_position: String,
pub summary: String,
pub industry: String,
pub region: String,
pub country: String,
pub country_code: String,
pub connection_count: i64,
pub follower_count: i64,
pub languages: Vec<String>,
pub skills: Vec<String>,
pub certifications: Vec<String>,
pub position_start_date: String,
pub position_end_date: String,
pub extracted_at: DateTime<Utc>,
pub data_source: String
}
#[handler(with_write)]
pub fn updateLinkedinContent (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_content = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_content_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "name" => &data.name, "email" => &data.email, "linkedin_url" => &data.linkedin_url, "full_name" => &data.full_name, "first_name" => &data.first_name, "last_name" => &data.last_name, "public_id" => &data.public_id, "profile_picture" => &data.profile_picture, "background_picture" => &data.background_picture, "current_position" => &data.current_position, "summary" => &data.summary, "industry" => &data.industry, "region" => &data.region, "country" => &data.country, "country_code" => &data.country_code, "connection_count" => &data.connection_count, "follower_count" => &data.follower_count, "languages" => &data.languages, "skills" => &data.skills, "certifications" => &data.certifications, "position_start_date" => &data.position_start_date, "position_end_date" => &data.position_end_date, "extracted_at" => &data.extracted_at, "data_source" => &data.data_source }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_content".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_content.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateLinkedinEducationInput {

pub education_id: ID,
pub school: String,
pub field: String,
pub title: String,
pub date_start: String,
pub date_end: String,
pub location: String,
pub description: String
}
#[handler(with_write)]
pub fn updateLinkedinEducation (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_education = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.education_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "school" => &data.school, "field" => &data.field, "title" => &data.title, "date_start" => &data.date_start, "date_end" => &data.date_end, "location" => &data.location, "description" => &data.description }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_education".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_education.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteMetadataInput {

pub metadata_id: ID
}
#[handler(with_write)]
pub fn deleteMetadata (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.metadata_id)

.out("Metadata_to_MetadataNotes",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.metadata_id)

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.metadata_id)

.out("Metadata_to_WarmConnect",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.metadata_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateLinkedinExperienceInput {

pub experience_id: ID,
pub company: String,
pub title: String,
pub field: String,
pub date_start: String,
pub date_end: String,
pub location: String,
pub description: String
}
#[handler(with_write)]
pub fn updateLinkedinExperience (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_experience = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.experience_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "company" => &data.company, "title" => &data.title, "field" => &data.field, "date_start" => &data.date_start, "date_end" => &data.date_end, "location" => &data.location, "description" => &data.description }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_experience".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_experience.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createLinkedinInfoInput {

pub user_id: ID,
pub url: String
}
#[handler(with_write)]
pub fn createLinkedinInfo (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_info = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinInfo", Some(props! { "url" => &data.url }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let metadata_linkedin_info = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("Metadata_to_LinkedinInfo", Some(props! { "url" => data.url.clone() }), metadata.id(), linkedin_info.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_info".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_info.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct findWarmConnectInput {

pub name: String,
pub email: String
}
#[tool_call(warm_connect, with_read)]
#[handler(with_read)]
pub fn findWarmConnect (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let warm_connect = G::new(Arc::clone(&db), &txn)
.n_from_type("WarmConnect")

.filter_ref(|val, txn|{
                if let Ok(val) = val { 
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())

.check_property("name")

.map_value_or(false, |v| *v == data.name.clone())? && G::new_from(Arc::clone(&db), &txn, val.clone())

.check_property("email")

.map_value_or(false, |v| *v == data.email.clone())?)
                } else {
                    Ok(false)
                }
            }).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("warm_connect".to_string(), ReturnValue::from_traversal_value_array_with_mixin(warm_connect.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createLinkedinContentInput {

pub user_id: ID,
pub name: String,
pub email: String,
pub linkedin_url: String,
pub full_name: String,
pub first_name: String,
pub last_name: String,
pub public_id: String,
pub profile_picture: String,
pub background_picture: String,
pub current_position: String,
pub summary: String,
pub industry: String,
pub region: String,
pub country: String,
pub country_code: String,
pub connection_count: i64,
pub follower_count: i64,
pub languages: Vec<String>,
pub skills: Vec<String>,
pub certifications: Vec<String>,
pub position_start_date: String,
pub position_end_date: String,
pub extracted_at: DateTime<Utc>,
pub data_source: String
}
#[handler(with_write)]
pub fn createLinkedinContent (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_content = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinContent", Some(props! { "profile_picture" => &data.profile_picture, "background_picture" => &data.background_picture, "linkedin_url" => &data.linkedin_url, "email" => &data.email, "region" => &data.region, "public_id" => &data.public_id, "connection_count" => &data.connection_count, "extracted_at" => &data.extracted_at, "country_code" => &data.country_code, "follower_count" => &data.follower_count, "last_name" => &data.last_name, "full_name" => &data.full_name, "skills" => &data.skills, "current_position" => &data.current_position, "name" => &data.name, "first_name" => &data.first_name, "languages" => &data.languages, "industry" => &data.industry, "summary" => &data.summary, "country" => &data.country, "position_start_date" => &data.position_start_date, "certifications" => &data.certifications, "data_source" => &data.data_source, "position_end_date" => &data.position_end_date }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content_linkedin_info = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("LinkedinInfo_to_LinkedinContent", None, linkedin_info.id(), linkedin_content.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_content".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_content.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateUserNameInput {

pub name_id: ID,
pub first: String,
pub last: String
}
#[handler(with_write)]
pub fn updateUserName (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let name = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.name_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "first" => &data.first, "last" => &data.last }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("name".to_string(), ReturnValue::from_traversal_value_with_mixin(name.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createNameInput {

pub user_id: ID,
pub first: String,
pub last: String
}
#[handler(with_write)]
pub fn createName (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let name = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Name", Some(props! { "last" => &data.last, "first" => &data.first }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let user_name = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("User_to_Name", Some(props! { "first" => data.first.clone() }), user.id(), name.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("name".to_string(), ReturnValue::from_traversal_value_with_mixin(name.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateLinkedinWebsiteInput {

pub website_id: ID,
pub url: String,
pub category: String
}
#[handler(with_write)]
pub fn updateLinkedinWebsite (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_website = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.website_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "url" => &data.url, "category" => &data.category }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_website".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_website.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserLinkedinWebsitesInput {

pub user_id: ID
}
#[tool_call(linkedin_websites, with_read)]
#[handler(with_read)]
pub fn getUserLinkedinWebsites (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_websites = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())

.out("LinkedinContent_to_LinkedinWebsite",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_websites".to_string(), ReturnValue::from_traversal_value_array_with_mixin(linkedin_websites.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteWarmConnectInput {

pub warm_connect_id: ID
}
#[handler(with_write)]
pub fn deleteWarmConnect (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.warm_connect_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUsersByReferrerInput {

pub referrer: String
}
#[tool_call(users, with_read)]
#[handler(with_read)]
pub fn getUsersByReferrer (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let metadata = G::new(Arc::clone(&db), &txn)
.n_from_type("Metadata")

.filter_ref(|val, txn|{
                if let Ok(val) = val { 
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())

.check_property("referredBy")

.map_value_or(false, |v| *v == data.referrer.clone())?)
                } else {
                    Ok(false)
                }
            }).collect_to::<Vec<_>>();
    let users = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.in_("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("users".to_string(), ReturnValue::from_traversal_value_array_with_mixin(users.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createMetadataNotesInput {

pub user_id: ID,
pub aiScore: i64,
pub userScore: i64,
pub text: String,
pub flagged: bool
}
#[handler(with_write)]
pub fn createMetadataNotes (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user_metadata_notes = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("MetadataNotes", Some(props! { "flagged" => &data.flagged, "aiScore" => &data.aiScore, "text" => &data.text, "userScore" => &data.userScore }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let metadata_metadata_notes = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("Metadata_to_MetadataNotes", None, metadata.id(), user_metadata_notes.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("user_metadata_notes".to_string(), ReturnValue::from_traversal_value_with_mixin(user_metadata_notes.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserMetadataNotesInput {

pub user_id: ID
}
#[tool_call(metadata_notes, with_read)]
#[handler(with_read)]
pub fn getUserMetadataNotes (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let metadata_notes = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_MetadataNotes",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("metadata_notes".to_string(), ReturnValue::from_traversal_value_array_with_mixin(metadata_notes.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserWarmConnectsInput {

pub user_id: ID
}
#[tool_call(warm_connects, with_read)]
#[handler(with_read)]
pub fn getUserWarmConnects (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let warm_connects = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_WarmConnect",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("warm_connects".to_string(), ReturnValue::from_traversal_value_array_with_mixin(warm_connects.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteMetadataNotesInput {

pub metadata_notes_id: ID
}
#[handler(with_write)]
pub fn deleteMetadataNotes (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.metadata_notes_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserLinkedinCompaniesInput {

pub user_id: ID
}
#[tool_call(linkedin_companies, with_read)]
#[handler(with_read)]
pub fn getUserLinkedinCompanies (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_companies = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())

.out("LinkedinContent_to_LinkedinCompany",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_companies".to_string(), ReturnValue::from_traversal_value_array_with_mixin(linkedin_companies.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserNameInput {

pub user_id: ID
}
#[tool_call(name, with_read)]
#[handler(with_read)]
pub fn getUserName (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let name = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Name",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("name".to_string(), ReturnValue::from_traversal_value_array_with_mixin(name.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getEmbedUserBioInput {

pub user_id: ID
}
#[tool_call(user_bio, with_read)]
#[handler(with_read)]
pub fn getEmbedUserBio (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let user_bio = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_EmbeddedBio",&EdgeType::Vec).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("user_bio".to_string(), ReturnValue::from_traversal_value_array_with_mixin(user_bio.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createUserInput {

pub phone: String,
pub email: String,
pub bio: String,
pub age: i32,
pub location: String,
pub profilePic: String,
pub color: String,
pub elo: f64,
pub sender: String
}
#[handler(with_write)]
pub fn createUser (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("User", Some(props! { "color" => &data.color, "profilePic" => &data.profilePic, "email" => &data.email, "phone" => &data.phone, "sender" => &data.sender, "age" => &data.age, "elo" => &data.elo, "bio" => &data.bio, "location" => &data.location }), None).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_with_mixin(user.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct findLinkedinCompanyInput {

pub name: String,
pub industry: String,
pub founded: i64
}
#[tool_call(linkedin_company, with_read)]
#[handler(with_read)]
pub fn findLinkedinCompany (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_company = G::new(Arc::clone(&db), &txn)
.n_from_type("LinkedinCompany")

.filter_ref(|val, txn|{
                if let Ok(val) = val { 
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())

.check_property("name")

.map_value_or(false, |v| *v == data.name.clone())? && G::new_from(Arc::clone(&db), &txn, val.clone())

.check_property("industry")

.map_value_or(false, |v| *v == data.industry.clone())? && G::new_from(Arc::clone(&db), &txn, val.clone())

.check_property("founded")

.map_value_or(false, |v| *v == data.founded.clone())?)
                } else {
                    Ok(false)
                }
            }).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_company".to_string(), ReturnValue::from_traversal_value_array_with_mixin(linkedin_company.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct addWarmConnectInput {

pub user_id: ID,
pub warm_connect_id: ID
}
#[handler(with_write)]
pub fn addWarmConnect (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let warm_connect = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.warm_connect_id).collect_to_obj();
    let metadata_to_warm_connect = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("Metadata_to_WarmConnect", None, metadata.id(), warm_connect.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("warm_connect".to_string(), ReturnValue::from_traversal_value_with_mixin(warm_connect.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserMetadataInput {

pub user_id: ID
}
#[tool_call(metadata, with_read)]
#[handler(with_read)]
pub fn getUserMetadata (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("metadata".to_string(), ReturnValue::from_traversal_value_array_with_mixin(metadata.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteLinkedinWebsiteInput {

pub website_id: ID
}
#[handler(with_write)]
pub fn deleteLinkedinWebsite (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.website_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateWarmConnectInput {

pub warm_connect_id: ID,
pub name: String,
pub email: String
}
#[handler(with_write)]
pub fn updateWarmConnect (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let warm_connect = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.warm_connect_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "name" => &data.name, "email" => &data.email }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("warm_connect".to_string(), ReturnValue::from_traversal_value_with_mixin(warm_connect.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct addCurrentCompanyInput {

pub user_id: ID,
pub linkedin_company_id: ID
}
#[handler(with_write)]
pub fn addCurrentCompany (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_company = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_company_id).collect_to_obj();
    let linkedin_content_current_company = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("LinkedinContent_to_CurrentCompany", None, linkedin_content.id(), linkedin_company.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_content_current_company".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_content_current_company.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateMetadataInput {

pub metadata_id: ID,
pub created_ts: DateTime<Utc>,
pub last_updated_ts: DateTime<Utc>,
pub archetype: bool,
pub referredBy: String
}
#[handler(with_write)]
pub fn updateMetadata (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let metadata = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.metadata_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "created_ts" => &data.created_ts, "last_updated_ts" => &data.last_updated_ts, "archetype" => &data.archetype, "referredBy" => &data.referredBy }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("metadata".to_string(), ReturnValue::from_traversal_value_with_mixin(metadata.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserLinkedinEducationsInput {

pub user_id: ID
}
#[tool_call(linkedin_educations, with_read)]
#[handler(with_read)]
pub fn getUserLinkedinEducations (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_educations = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())

.out("LinkedinContent_to_LinkedinEducation",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_educations".to_string(), ReturnValue::from_traversal_value_array_with_mixin(linkedin_educations.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserInput {

pub user_id: ID
}
#[tool_call(user, with_read)]
#[handler(with_read)]
pub fn getUser (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_with_mixin(user.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct createUserBioInput {

pub user_id: ID,
pub bio: Vec<f64>
}
#[handler(with_write)]
pub fn createUserBio (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user_bio = G::new_mut(Arc::clone(&db), &mut txn)
.insert_v::<fn(&HVector, &RoTxn) -> bool>(&data.bio, "EmbeddedBio", None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let user_user_bio = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("User_to_EmbeddedBio", None, user.id(), user_bio.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("user_bio".to_string(), ReturnValue::from_traversal_value_with_mixin(user_bio.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateUserInput {

pub user_id: ID,
pub phone: String,
pub email: String,
pub bio: String,
pub age: i32,
pub location: String,
pub profilePic: String,
pub color: String,
pub elo: f64,
pub sender: String
}
#[handler(with_write)]
pub fn updateUser (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "phone" => &data.phone, "email" => &data.email, "bio" => &data.bio, "age" => &data.age, "location" => &data.location, "profilePic" => &data.profilePic, "color" => &data.color, "elo" => &data.elo, "sender" => &data.sender }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_with_mixin(user.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct deleteLinkedinContentInput {

pub linkedin_content_id: ID
}
#[handler(with_write)]
pub fn deleteLinkedinContent (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_content_id)

.out("LinkedinContent_to_LinkedinWebsite",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_content_id)

.out("LinkedinContent_to_LinkedinExperience",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_content_id)

.out_e("LinkedinContent_to_LinkedinCompany").collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_content_id)

.out("LinkedinContent_to_LinkedinEducation",&EdgeType::Node).collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_content_id)

.out_e("LinkedinContent_to_CurrentCompany").collect_to::<Vec<_>>(),
                Arc::clone(&db),
                &mut txn,
            )?;;
    Drop::<Vec<_>>::drop_traversal(
                G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_content_id).collect_to_obj(),
                Arc::clone(&db),
                &mut txn,
            )?;;
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("success".to_string(), ReturnValue::from(Value::from("success")));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserLinkedinContentInput {

pub user_id: ID
}
#[tool_call(linkedin_content, with_read)]
#[handler(with_read)]
pub fn getUserLinkedinContent (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let user = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.user_id).collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())

.out("User_to_Metadata",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())

.out("Metadata_to_LinkedinInfo",&EdgeType::Node).collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())

.out("LinkedinInfo_to_LinkedinContent",&EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_content".to_string(), ReturnValue::from_traversal_value_array_with_mixin(linkedin_content.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct updateCurrentCompanyInput {

pub linkedin_company_id: ID,
pub name: String,
pub domain: String,
pub industry: String,
pub staff_count: i64,
pub founded: i64,
pub website: String,
pub headquarters: String,
pub description: String,
pub specialties: Vec<String>
}
#[handler(with_write)]
pub fn updateCurrentCompany (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let linkedin_company = {let update_tr = G::new(Arc::clone(&db), &txn)
.n_from_id(&data.linkedin_company_id)
    .collect_to::<Vec<_>>();G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "name" => &data.name, "domain" => &data.domain, "industry" => &data.industry, "staff_count" => &data.staff_count, "founded" => &data.founded, "website" => &data.website, "headquarters" => &data.headquarters, "description" => &data.description, "specialties" => &data.specialties }))
    .collect_to_obj()};
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("linkedin_company".to_string(), ReturnValue::from_traversal_value_with_mixin(linkedin_company.clone().clone(), remapping_vals.borrow_mut()));

}
    Ok(())
}
