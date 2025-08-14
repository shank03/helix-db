// DEFAULT CODE
// use helix_db::helix_engine::graph_core::config::Config;

// pub fn config() -> Option<Config> {
//     None
// }

use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helix_db::{
    embed, embed_async, exclude_field, field_addition_from_old_field, field_addition_from_value,
    field_remapping, field_type_cast,
    helix_engine::{
        graph_core::{
            config::{Config, GraphConfig, VectorConfig},
            ops::{
                bm25::search_bm25::SearchBM25Adapter,
                g::G,
                in_::{in_::InAdapter, in_e::InEdgesAdapter, to_n::ToNAdapter, to_v::ToVAdapter},
                out::{
                    from_n::FromNAdapter, from_v::FromVAdapter, out::OutAdapter,
                    out_e::OutEdgesAdapter,
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
                    filter_ref::FilterRefAdapter, map::MapAdapter, order::OrderByAdapter,
                    paths::ShortestPathAdapter, props::PropsAdapter, range::RangeAdapter,
                    update::UpdateAdapter,
                },
                vectors::{
                    brute_force_search::BruteForceSearchVAdapter, insert::InsertVAdapter,
                    search::SearchVAdapter,
                },
            },
        },
        types::GraphError,
        vector_core::vector::HVector,
    },
    helix_gateway::{
        embedding_providers::embedding_providers::{EmbeddingModel, get_embedding_model},
        mcp::mcp::{MCPHandler, MCPHandlerSubmission, MCPToolInput},
        router::router::{HandlerInput, IoContFn},
    },
    identifier_remapping, node_matches, props,
    protocol::{
        format::Format,
        remapping::{Remapping, RemappingMap, ResponseRemapping},
        response::Response,
        return_values::ReturnValue,
        value::{
            Value,
            casting::{CastType, cast},
        },
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
use helix_macros::{handler, mcp_handler, migration, tool_call};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub fn config() -> Option<Config> {
    return Some(Config {
        vector_config: Some(VectorConfig {
            m: Some(16),
            ef_construction: Some(128),
            ef_search: Some(768),
        }),
        graph_config: Some(GraphConfig {
            secondary_indices: Some(vec![]),
        }),
        db_max_size_gb: Some(20),
        mcp: Some(true),
        bm25: Some(true),
        schema: Some(
            r#"{
  "schema": {
    "nodes": [
      {
        "name": "Metadata",
        "properties": {
          "referredBy": "String",
          "id": "ID",
          "last_updated_ts": "Date",
          "archetype": "Boolean",
          "created_ts": "Date"
        }
      },
      {
        "name": "MetadataNotes",
        "properties": {
          "aiScore": "I64",
          "id": "ID",
          "flagged": "Boolean",
          "text": "String",
          "userScore": "I64"
        }
      },
      {
        "name": "LinkedinEducation",
        "properties": {
          "description": "String",
          "location": "String",
          "id": "ID",
          "school": "String",
          "title": "String",
          "date_end": "String",
          "field": "String",
          "date_start": "String"
        }
      },
      {
        "name": "LinkedinContent",
        "properties": {
          "position_end_date": "String",
          "summary": "String",
          "data_source": "String",
          "full_name": "String",
          "background_picture": "String",
          "region": "String",
          "country": "String",
          "industry": "String",
          "first_name": "String",
          "follower_count": "I64",
          "name": "String",
          "linkedin_url": "String",
          "extracted_at": "Date",
          "skills": "Array(String)",
          "current_position": "String",
          "email": "String",
          "last_name": "String",
          "connection_count": "I64",
          "certifications": "Array(String)",
          "country_code": "String",
          "public_id": "String",
          "languages": "Array(String)",
          "position_start_date": "String",
          "id": "ID",
          "profile_picture": "String"
        }
      },
      {
        "name": "User",
        "properties": {
          "age": "I32",
          "email": "String",
          "elo": "F64",
          "color": "String",
          "bio": "String",
          "phone": "String",
          "profilePic": "String",
          "location": "String",
          "sender": "String",
          "id": "ID"
        }
      },
      {
        "name": "WarmConnect",
        "properties": {
          "id": "ID",
          "email": "String",
          "name": "String"
        }
      },
      {
        "name": "Name",
        "properties": {
          "last": "String",
          "first": "String",
          "id": "ID"
        }
      },
      {
        "name": "LinkedinInfo",
        "properties": {
          "url": "String",
          "id": "ID"
        }
      },
      {
        "name": "LinkedinExperience",
        "properties": {
          "company": "String",
          "location": "String",
          "description": "String",
          "date_end": "String",
          "title": "String",
          "date_start": "String",
          "id": "ID",
          "field": "String"
        }
      },
      {
        "name": "LinkedinWebsite",
        "properties": {
          "category": "String",
          "id": "ID",
          "url": "String"
        }
      },
      {
        "name": "LinkedinCompany",
        "properties": {
          "description": "String",
          "founded": "I64",
          "website": "String",
          "id": "ID",
          "domain": "String",
          "industry": "String",
          "headquarters": "String",
          "name": "String",
          "staff_count": "I64",
          "specialties": "Array(String)"
        }
      }
    ],
    "vectors": [
      {
        "name": "EmbeddedBio",
        "properties": {
          "bio": "Array(F64)",
          "id": "ID"
        }
      }
    ],
    "edges": [
      {
        "name": "LinkedinContent_to_LinkedinWebsite",
        "from": "LinkedinContent",
        "to": "LinkedinWebsite",
        "properties": {}
      },
      {
        "name": "Metadata_to_MetadataNotes",
        "from": "Metadata",
        "to": "MetadataNotes",
        "properties": {}
      },
      {
        "name": "LinkedinContent_to_CurrentCompany",
        "from": "LinkedinContent",
        "to": "LinkedinCompany",
        "properties": {}
      },
      {
        "name": "LinkedinContent_to_LinkedinExperience",
        "from": "LinkedinContent",
        "to": "LinkedinExperience",
        "properties": {}
      },
      {
        "name": "LinkedinInfo_to_LinkedinContent",
        "from": "LinkedinInfo",
        "to": "LinkedinContent",
        "properties": {}
      },
      {
        "name": "User_to_Metadata",
        "from": "User",
        "to": "Metadata",
        "properties": {
          "created_ts": "Date",
          "last_updated_ts": "Date"
        }
      },
      {
        "name": "Metadata_to_LinkedinInfo",
        "from": "Metadata",
        "to": "LinkedinInfo",
        "properties": {
          "url": "String"
        }
      },
      {
        "name": "User_to_Name",
        "from": "User",
        "to": "Name",
        "properties": {
          "first": "String"
        }
      },
      {
        "name": "User_to_EmbeddedBio",
        "from": "User",
        "to": "EmbeddedBio",
        "properties": {}
      },
      {
        "name": "Metadata_to_WarmConnect",
        "from": "Metadata",
        "to": "WarmConnect",
        "properties": {}
      },
      {
        "name": "LinkedinContent_to_LinkedinCompany",
        "from": "LinkedinContent",
        "to": "LinkedinCompany",
        "properties": {}
      },
      {
        "name": "LinkedinContent_to_LinkedinEducation",
        "from": "LinkedinContent",
        "to": "LinkedinEducation",
        "properties": {}
      }
    ]
  },
  "queries": [
    {
      "name": "getUserName",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "name"
      ]
    },
    {
      "name": "updateUserName",
      "parameters": {
        "first": "String",
        "name_id": "ID",
        "last": "String"
      },
      "returns": [
        "name"
      ]
    },
    {
      "name": "getUserEmbeddedBio",
      "parameters": {
        "text": "String"
      },
      "returns": [
        "vs"
      ]
    },
    {
      "name": "getUsersByReferrer",
      "parameters": {
        "referrer": "String"
      },
      "returns": [
        "users"
      ]
    },
    {
      "name": "updateCurrentCompany",
      "parameters": {
        "industry": "String",
        "name": "String",
        "founded": "I64",
        "specialties": "Array(String)",
        "website": "String",
        "linkedin_company_id": "ID",
        "domain": "String",
        "staff_count": "I64",
        "headquarters": "String",
        "description": "String"
      },
      "returns": [
        "linkedin_company"
      ]
    },
    {
      "name": "getUserLinkedinExperiences",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "linkedin_experiences"
      ]
    },
    {
      "name": "deleteLinkedinInfo",
      "parameters": {
        "linkedin_info_id": "ID"
      },
      "returns": []
    },
    {
      "name": "deleteWarmConnect",
      "parameters": {
        "warm_connect_id": "ID"
      },
      "returns": []
    },
    {
      "name": "updateLinkedinContent",
      "parameters": {
        "linkedin_url": "String",
        "follower_count": "I64",
        "linkedin_content_id": "ID",
        "full_name": "String",
        "profile_picture": "String",
        "position_end_date": "String",
        "data_source": "String",
        "skills": "Array(String)",
        "background_picture": "String",
        "first_name": "String",
        "country_code": "String",
        "last_name": "String",
        "connection_count": "I64",
        "position_start_date": "String",
        "region": "String",
        "summary": "String",
        "languages": "Array(String)",
        "certifications": "Array(String)",
        "country": "String",
        "current_position": "String",
        "email": "String",
        "industry": "String",
        "extracted_at": "Date",
        "name": "String",
        "public_id": "String"
      },
      "returns": [
        "linkedin_content"
      ]
    },
    {
      "name": "findLinkedinCompany",
      "parameters": {
        "founded": "I64",
        "industry": "String",
        "name": "String"
      },
      "returns": [
        "linkedin_company"
      ]
    },
    {
      "name": "getUser",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "user"
      ]
    },
    {
      "name": "createMetadata",
      "parameters": {
        "archetype": "Boolean",
        "created_ts": "Date",
        "last_updated_ts": "Date",
        "referredBy": "String",
        "user_id": "ID"
      },
      "returns": [
        "metadata"
      ]
    },
    {
      "name": "getUserLinkedinCurrentCompany",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "current_company"
      ]
    },
    {
      "name": "createUserBio",
      "parameters": {
        "user_id": "ID",
        "bio": "Array(F64)"
      },
      "returns": [
        "user_bio"
      ]
    },
    {
      "name": "updateMetadataNotes",
      "parameters": {
        "flagged": "Boolean",
        "metadata_notes_id": "ID",
        "userScore": "I64",
        "text": "String",
        "aiScore": "I64"
      },
      "returns": [
        "metadata_notes"
      ]
    },
    {
      "name": "addWarmConnect",
      "parameters": {
        "user_id": "ID",
        "warm_connect_id": "ID"
      },
      "returns": [
        "warm_connect"
      ]
    },
    {
      "name": "searchUsersByBio",
      "parameters": {
        "bio_vector": "Array(F64)",
        "k": "I64"
      },
      "returns": [
        "users"
      ]
    },
    {
      "name": "getUserLinkedinInfo",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "linkedin_info"
      ]
    },
    {
      "name": "addLinkedinCompany",
      "parameters": {
        "linkedin_company_id": "ID",
        "user_id": "ID"
      },
      "returns": [
        "linkedin_content_linkedin_company"
      ]
    },
    {
      "name": "createUser",
      "parameters": {
        "sender": "String",
        "location": "String",
        "phone": "String",
        "age": "I32",
        "elo": "F64",
        "profilePic": "String",
        "email": "String",
        "color": "String",
        "bio": "String"
      },
      "returns": [
        "user"
      ]
    },
    {
      "name": "getUserLinkedinEducations",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "linkedin_educations"
      ]
    },
    {
      "name": "getUserEmbeddedBioMCP",
      "parameters": {
        "text": "String"
      },
      "returns": [
        "vs"
      ]
    },
    {
      "name": "deleteUser",
      "parameters": {
        "user_id": "ID"
      },
      "returns": []
    },
    {
      "name": "createLinkedinEducation",
      "parameters": {
        "date_start": "String",
        "title": "String",
        "location": "String",
        "description": "String",
        "date_end": "String",
        "school": "String",
        "field": "String",
        "user_id": "ID"
      },
      "returns": [
        "linkedin_education"
      ]
    },
    {
      "name": "createLinkedinInfo",
      "parameters": {
        "url": "String",
        "user_id": "ID"
      },
      "returns": [
        "linkedin_info"
      ]
    },
    {
      "name": "deleteLinkedinWebsite",
      "parameters": {
        "website_id": "ID"
      },
      "returns": []
    },
    {
      "name": "getAllUsers",
      "parameters": {},
      "returns": [
        "users"
      ]
    },
    {
      "name": "deleteLinkedinContent",
      "parameters": {
        "linkedin_content_id": "ID"
      },
      "returns": []
    },
    {
      "name": "createLinkedinExperience",
      "parameters": {
        "date_end": "String",
        "description": "String",
        "company": "String",
        "field": "String",
        "location": "String",
        "user_id": "ID",
        "date_start": "String",
        "title": "String"
      },
      "returns": [
        "linkedin_experience"
      ]
    },
    {
      "name": "createLinkedinContent",
      "parameters": {
        "email": "String",
        "name": "String",
        "first_name": "String",
        "current_position": "String",
        "skills": "Array(String)",
        "position_end_date": "String",
        "last_name": "String",
        "summary": "String",
        "industry": "String",
        "full_name": "String",
        "country": "String",
        "background_picture": "String",
        "languages": "Array(String)",
        "extracted_at": "Date",
        "country_code": "String",
        "connection_count": "I64",
        "user_id": "ID",
        "region": "String",
        "follower_count": "I64",
        "public_id": "String",
        "position_start_date": "String",
        "profile_picture": "String",
        "linkedin_url": "String",
        "certifications": "Array(String)",
        "data_source": "String"
      },
      "returns": [
        "linkedin_content"
      ]
    },
    {
      "name": "deleteMetadata",
      "parameters": {
        "metadata_id": "ID"
      },
      "returns": []
    },
    {
      "name": "updateLinkedinInfo",
      "parameters": {
        "url": "String",
        "linkedin_info_id": "ID"
      },
      "returns": [
        "linkedin_info"
      ]
    },
    {
      "name": "createLinkedinWebsite",
      "parameters": {
        "category": "String",
        "url": "String",
        "user_id": "ID"
      },
      "returns": [
        "linkedin_website"
      ]
    },
    {
      "name": "getUserWarmConnects",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "warm_connects"
      ]
    },
    {
      "name": "deleteLinkedinExperience",
      "parameters": {
        "experience_id": "ID"
      },
      "returns": []
    },
    {
      "name": "updateLinkedinExperience",
      "parameters": {
        "location": "String",
        "field": "String",
        "date_start": "String",
        "company": "String",
        "experience_id": "ID",
        "title": "String",
        "date_end": "String",
        "description": "String"
      },
      "returns": [
        "linkedin_experience"
      ]
    },
    {
      "name": "addCurrentCompany",
      "parameters": {
        "linkedin_company_id": "ID",
        "user_id": "ID"
      },
      "returns": [
        "linkedin_content_current_company"
      ]
    },
    {
      "name": "createName",
      "parameters": {
        "last": "String",
        "first": "String",
        "user_id": "ID"
      },
      "returns": [
        "name"
      ]
    },
    {
      "name": "deleteEmbeddedBio",
      "parameters": {
        "user_id": "ID"
      },
      "returns": []
    },
    {
      "name": "deleteLinkedinCompany",
      "parameters": {
        "company_id": "ID"
      },
      "returns": []
    },
    {
      "name": "updateUser",
      "parameters": {
        "location": "String",
        "profilePic": "String",
        "sender": "String",
        "phone": "String",
        "elo": "F64",
        "email": "String",
        "age": "I32",
        "color": "String",
        "user_id": "ID",
        "bio": "String"
      },
      "returns": [
        "user"
      ]
    },
    {
      "name": "getUserLinkedinContent",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "linkedin_content"
      ]
    },
    {
      "name": "getUserLinkedinWebsites",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "linkedin_websites"
      ]
    },
    {
      "name": "createWarmConnect",
      "parameters": {
        "name": "String",
        "email": "String",
        "user_id": "ID"
      },
      "returns": [
        "warm_connect"
      ]
    },
    {
      "name": "getUserMetadata",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "metadata"
      ]
    },
    {
      "name": "updateLinkedinCompany",
      "parameters": {
        "company_id": "ID",
        "description": "String",
        "industry": "String",
        "domain": "String",
        "specialties": "Array(String)",
        "name": "String",
        "website": "String",
        "founded": "I64",
        "headquarters": "String",
        "staff_count": "I64"
      },
      "returns": [
        "linkedin_company"
      ]
    },
    {
      "name": "getUserLinkedinCompanies",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "linkedin_companies"
      ]
    },
    {
      "name": "deleteName",
      "parameters": {
        "name_id": "ID"
      },
      "returns": []
    },
    {
      "name": "findWarmConnect",
      "parameters": {
        "name": "String",
        "email": "String"
      },
      "returns": [
        "warm_connect"
      ]
    },
    {
      "name": "createMetadataNotes",
      "parameters": {
        "user_id": "ID",
        "text": "String",
        "aiScore": "I64",
        "userScore": "I64",
        "flagged": "Boolean"
      },
      "returns": [
        "user_metadata_notes"
      ]
    },
    {
      "name": "getEmbedUserBio",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "user_bio"
      ]
    },
    {
      "name": "createLinkedinCompany",
      "parameters": {
        "headquarters": "String",
        "name": "String",
        "domain": "String",
        "industry": "String",
        "staff_count": "I64",
        "description": "String",
        "website": "String",
        "user_id": "ID",
        "founded": "I64",
        "specialties": "Array(String)"
      },
      "returns": [
        "linkedin_company"
      ]
    },
    {
      "name": "deleteMetadataNotes",
      "parameters": {
        "metadata_notes_id": "ID"
      },
      "returns": []
    },
    {
      "name": "getUserMetadataNotes",
      "parameters": {
        "user_id": "ID"
      },
      "returns": [
        "metadata_notes"
      ]
    },
    {
      "name": "updateLinkedinWebsite",
      "parameters": {
        "website_id": "ID",
        "category": "String",
        "url": "String"
      },
      "returns": [
        "linkedin_website"
      ]
    },
    {
      "name": "updateWarmConnect",
      "parameters": {
        "warm_connect_id": "ID",
        "email": "String",
        "name": "String"
      },
      "returns": [
        "warm_connect"
      ]
    },
    {
      "name": "updateLinkedinEducation",
      "parameters": {
        "education_id": "ID",
        "location": "String",
        "title": "String",
        "description": "String",
        "field": "String",
        "date_end": "String",
        "school": "String",
        "date_start": "String"
      },
      "returns": [
        "linkedin_education"
      ]
    },
    {
      "name": "deleteLinkedinEducation",
      "parameters": {
        "education_id": "ID"
      },
      "returns": []
    },
    {
      "name": "updateMetadata",
      "parameters": {
        "created_ts": "Date",
        "archetype": "Boolean",
        "last_updated_ts": "Date",
        "metadata_id": "ID",
        "referredBy": "String"
      },
      "returns": [
        "metadata"
      ]
    }
  ]
}"#
            .to_string(),
        ),
        embedding_model: None,
        graphvis_node_label: None,
    });
}

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

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserNameInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserName(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserNameInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let name = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Name", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "name".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            name.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserNameMcpInput {
    connection_id: String,
    data: getUserNameInput,
}
#[mcp_handler]
pub fn getUserNameMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserNameMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let name = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Name", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        name.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct updateUserNameInput {
    pub name_id: ID,
    pub first: String,
    pub last: String,
}
#[handler]
pub fn updateUserName(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateUserNameInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let name = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.name_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(
                props! { "first" => &data.first, "last" => &data.last },
            ))
            .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "name".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            name.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserEmbeddedBioInput {
    pub text: String,
}
#[handler]
pub fn getUserEmbeddedBio(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserEmbeddedBioInput>(&input.request.body)?
        .into_owned();
    Err(IoContFn::create_err(
        move |__internal_cont_tx, __internal_ret_chan| {
            Box::pin(async move {
                let __internal_embed_data_0 = embed_async!(db, &data.text);
                __internal_cont_tx
                    .send_async((
                        __internal_ret_chan,
                        Box::new(move || {
                            let __internal_embed_data_0: Vec<f64> = __internal_embed_data_0?;
                            let mut remapping_vals = RemappingMap::new();
                            let txn = db.graph_env.read_txn().unwrap();
                            let vs = G::new(Arc::clone(&db), &txn)
                                .search_v::<fn(&HVector, &RoTxn) -> bool, _>(
                                    &__internal_embed_data_0,
                                    10,
                                    "EmbeddedBio",
                                    None,
                                )
                                .collect_to::<Vec<_>>();
                            let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
                            return_vals.insert(
                                "vs".to_string(),
                                ReturnValue::from_traversal_value_array_with_mixin(
                                    vs.clone().clone(),
                                    remapping_vals.borrow_mut(),
                                ),
                            );

                            txn.commit().unwrap();
                            Ok(input.request.out_fmt.create_response(&return_vals))
                        }),
                    ))
                    .await
                    .expect("Cont Channel should be alive")
            })
        },
    ))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUsersByReferrerInput {
    pub referrer: String,
}
#[handler]
pub fn getUsersByReferrer(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUsersByReferrerInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let metadata = G::new(Arc::clone(&db), &txn)
        .n_from_type("Metadata")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("referredBy")
                    .map_value_or(false, |v| *v == data.referrer.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let users = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .in_("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "users".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            users.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUsersByReferrerMcpInput {
    connection_id: String,
    data: getUsersByReferrerInput,
}
#[mcp_handler]
pub fn getUsersByReferrerMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUsersByReferrerMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let metadata = G::new(Arc::clone(&db), &txn)
            .n_from_type("Metadata")
            .filter_ref(|val, txn| {
                if let Ok(val) = val {
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                        .check_property("referredBy")
                        .map_value_or(false, |v| *v == data.referrer.clone())?)
                } else {
                    Ok(false)
                }
            })
            .collect_to::<Vec<_>>();
        let users = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .in_("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        users.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
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
    pub specialties: Vec<String>,
}
#[handler]
pub fn updateCurrentCompany(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateCurrentCompanyInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_company = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_company_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "name" => &data.name, "domain" => &data.domain, "industry" => &data.industry, "staff_count" => &data.staff_count, "founded" => &data.founded, "website" => &data.website, "headquarters" => &data.headquarters, "description" => &data.description, "specialties" => &data.specialties }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_company".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserLinkedinExperiencesInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserLinkedinExperiences(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinExperiencesInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_experiences = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
        .out("LinkedinContent_to_LinkedinExperience", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_experiences".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            linkedin_experiences.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserLinkedinExperiencesMcpInput {
    connection_id: String,
    data: getUserLinkedinExperiencesInput,
}
#[mcp_handler]
pub fn getUserLinkedinExperiencesMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinExperiencesMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
            .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_experiences = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
            .out("LinkedinContent_to_LinkedinExperience", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        linkedin_experiences.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteLinkedinInfoInput {
    pub linkedin_info_id: ID,
}
#[handler]
pub fn deleteLinkedinInfo(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteLinkedinInfoInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_info_id)
            .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_info_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteWarmConnectInput {
    pub warm_connect_id: ID,
}
#[handler]
pub fn deleteWarmConnect(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteWarmConnectInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.warm_connect_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
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
    pub data_source: String,
}
#[handler]
pub fn updateLinkedinContent(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateLinkedinContentInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_content = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_content_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "name" => &data.name, "email" => &data.email, "linkedin_url" => &data.linkedin_url, "full_name" => &data.full_name, "first_name" => &data.first_name, "last_name" => &data.last_name, "public_id" => &data.public_id, "profile_picture" => &data.profile_picture, "background_picture" => &data.background_picture, "current_position" => &data.current_position, "summary" => &data.summary, "industry" => &data.industry, "region" => &data.region, "country" => &data.country, "country_code" => &data.country_code, "connection_count" => &data.connection_count, "follower_count" => &data.follower_count, "languages" => &data.languages, "skills" => &data.skills, "certifications" => &data.certifications, "position_start_date" => &data.position_start_date, "position_end_date" => &data.position_end_date, "extracted_at" => &data.extracted_at, "data_source" => &data.data_source }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_content".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_content.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct findLinkedinCompanyInput {
    pub name: String,
    pub industry: String,
    pub founded: i64,
}
#[handler]
pub fn findLinkedinCompany(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<findLinkedinCompanyInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let linkedin_company = G::new(Arc::clone(&db), &txn)
        .n_from_type("LinkedinCompany")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("name")
                    .map_value_or(false, |v| *v == data.name.clone())?
                    && G::new_from(Arc::clone(&db), &txn, val.clone())
                        .check_property("industry")
                        .map_value_or(false, |v| *v == data.industry.clone())?
                    && G::new_from(Arc::clone(&db), &txn, val.clone())
                        .check_property("founded")
                        .map_value_or(false, |v| *v == data.founded.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_company".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            linkedin_company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct findLinkedinCompanyMcpInput {
    connection_id: String,
    data: findLinkedinCompanyInput,
}
#[mcp_handler]
pub fn findLinkedinCompanyMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<findLinkedinCompanyMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let linkedin_company = G::new(Arc::clone(&db), &txn)
            .n_from_type("LinkedinCompany")
            .filter_ref(|val, txn| {
                if let Ok(val) = val {
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                        .check_property("name")
                        .map_value_or(false, |v| *v == data.name.clone())?
                        && G::new_from(Arc::clone(&db), &txn, val.clone())
                            .check_property("industry")
                            .map_value_or(false, |v| *v == data.industry.clone())?
                        && G::new_from(Arc::clone(&db), &txn, val.clone())
                            .check_property("founded")
                            .map_value_or(false, |v| *v == data.founded.clone())?)
                } else {
                    Ok(false)
                }
            })
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        linkedin_company.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserInput {
    pub user_id: ID,
}
#[handler]
pub fn getUser(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserMcpInput {
    connection_id: String,
    data: getUserInput,
}
#[mcp_handler]
pub fn getUserMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        txn.commit().unwrap();
        user.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createMetadataInput {
    pub user_id: ID,
    pub created_ts: DateTime<Utc>,
    pub last_updated_ts: DateTime<Utc>,
    pub archetype: bool,
    pub referredBy: String,
}
#[handler]
pub fn createMetadata(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createMetadataInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let metadata = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Metadata", Some(props! { "last_updated_ts" => &data.last_updated_ts, "created_ts" => &data.created_ts, "referredBy" => &data.referredBy, "archetype" => &data.archetype }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let user_metadata = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("User_to_Metadata", Some(props! { "last_updated_ts" => data.last_updated_ts.clone(), "created_ts" => data.created_ts.clone() }), user.id(), metadata.id(), true, EdgeType::Node).collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "metadata".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            metadata.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserLinkedinCurrentCompanyInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserLinkedinCurrentCompany(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinCurrentCompanyInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let current_company = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
        .out("LinkedinContent_to_CurrentCompany", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "current_company".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            current_company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserLinkedinCurrentCompanyMcpInput {
    connection_id: String,
    data: getUserLinkedinCurrentCompanyInput,
}
#[mcp_handler]
pub fn getUserLinkedinCurrentCompanyMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinCurrentCompanyMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
            .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let current_company = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
            .out("LinkedinContent_to_CurrentCompany", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        current_company.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createUserBioInput {
    pub user_id: ID,
    pub bio: Vec<f64>,
}
#[handler]
pub fn createUserBio(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createUserBioInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user_bio = G::new_mut(Arc::clone(&db), &mut txn)
        .insert_v::<fn(&HVector, &RoTxn) -> bool>(&data.bio, "EmbeddedBio", None)
        .collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let user_user_bio = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "User_to_EmbeddedBio",
            None,
            user.id(),
            user_bio.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user_bio".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user_bio.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct updateMetadataNotesInput {
    pub metadata_notes_id: ID,
    pub aiScore: i64,
    pub userScore: i64,
    pub text: String,
    pub flagged: bool,
}
#[handler]
pub fn updateMetadataNotes(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateMetadataNotesInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let metadata_notes = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.metadata_notes_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "aiScore" => &data.aiScore, "userScore" => &data.userScore, "text" => &data.text, "flagged" => &data.flagged }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "metadata_notes".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            metadata_notes.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct addWarmConnectInput {
    pub user_id: ID,
    pub warm_connect_id: ID,
}
#[handler]
pub fn addWarmConnect(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<addWarmConnectInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let warm_connect = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.warm_connect_id)
        .collect_to_obj();
    let metadata_to_warm_connect = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Metadata_to_WarmConnect",
            None,
            metadata.id(),
            warm_connect.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "warm_connect".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            warm_connect.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct searchUsersByBioInput {
    pub bio_vector: Vec<f64>,
    pub k: i64,
}
#[handler]
pub fn searchUsersByBio(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<searchUsersByBioInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let similar_bios = G::new(Arc::clone(&db), &txn)
        .search_v::<fn(&HVector, &RoTxn) -> bool, _>(
            &data.bio_vector,
            data.k.clone(),
            "EmbeddedBio",
            None,
        )
        .collect_to::<Vec<_>>();
    let users = G::new_from(Arc::clone(&db), &txn, similar_bios.clone())
        .in_("User_to_EmbeddedBio", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "users".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            users.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct searchUsersByBioMcpInput {
    connection_id: String,
    data: searchUsersByBioInput,
}
#[mcp_handler]
pub fn searchUsersByBioMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<searchUsersByBioMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let similar_bios = G::new(Arc::clone(&db), &txn)
            .search_v::<fn(&HVector, &RoTxn) -> bool, _>(
                &data.bio_vector,
                data.k.clone(),
                "EmbeddedBio",
                None,
            )
            .collect_to::<Vec<_>>();
        let users = G::new_from(Arc::clone(&db), &txn, similar_bios.clone())
            .in_("User_to_EmbeddedBio", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        users.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserLinkedinInfoInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserLinkedinInfo(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinInfoInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_info".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            linkedin_info.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserLinkedinInfoMcpInput {
    connection_id: String,
    data: getUserLinkedinInfoInput,
}
#[mcp_handler]
pub fn getUserLinkedinInfoMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinInfoMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        linkedin_info.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct addLinkedinCompanyInput {
    pub user_id: ID,
    pub linkedin_company_id: ID,
}
#[handler]
pub fn addLinkedinCompany(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<addLinkedinCompanyInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_company = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.linkedin_company_id)
        .collect_to_obj();
    let linkedin_content_linkedin_company = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "LinkedinContent_to_LinkedinCompany",
            None,
            linkedin_content.id(),
            linkedin_company.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_content_linkedin_company".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_content_linkedin_company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createUserInput {
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
#[handler]
pub fn createUser(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createUserInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("User", Some(props! { "bio" => &data.bio, "phone" => &data.phone, "location" => &data.location, "sender" => &data.sender, "profilePic" => &data.profilePic, "age" => &data.age, "email" => &data.email, "elo" => &data.elo, "color" => &data.color }), None).collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserLinkedinEducationsInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserLinkedinEducations(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinEducationsInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_educations = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
        .out("LinkedinContent_to_LinkedinEducation", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_educations".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            linkedin_educations.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserLinkedinEducationsMcpInput {
    connection_id: String,
    data: getUserLinkedinEducationsInput,
}
#[mcp_handler]
pub fn getUserLinkedinEducationsMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinEducationsMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
            .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_educations = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
            .out("LinkedinContent_to_LinkedinEducation", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        linkedin_educations.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserEmbeddedBioMCPInput {
    pub text: String,
}
#[handler]
pub fn getUserEmbeddedBioMCP(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserEmbeddedBioMCPInput>(&input.request.body)?
        .into_owned();
    Err(IoContFn::create_err(
        move |__internal_cont_tx, __internal_ret_chan| {
            Box::pin(async move {
                let __internal_embed_data_0 = embed_async!(db, &data.text);
                __internal_cont_tx
                    .send_async((
                        __internal_ret_chan,
                        Box::new(move || {
                            let __internal_embed_data_0: Vec<f64> = __internal_embed_data_0?;
                            let mut remapping_vals = RemappingMap::new();
                            let txn = db.graph_env.read_txn().unwrap();
                            let vs = G::new(Arc::clone(&db), &txn)
                                .search_v::<fn(&HVector, &RoTxn) -> bool, _>(
                                    &__internal_embed_data_0,
                                    10,
                                    "EmbeddedBio",
                                    None,
                                )
                                .collect_to::<Vec<_>>();
                            let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
                            return_vals.insert(
                                "vs".to_string(),
                                ReturnValue::from_traversal_value_array_with_mixin(
                                    vs.clone().clone(),
                                    remapping_vals.borrow_mut(),
                                ),
                            );

                            txn.commit().unwrap();
                            Ok(input.request.out_fmt.create_response(&return_vals))
                        }),
                    ))
                    .await
                    .expect("Cont Channel should be alive")
            })
        },
    ))
}
#[derive(Deserialize, Clone)]
pub struct getUserEmbeddedBioMCPMcpInput {
    connection_id: String,
    data: getUserEmbeddedBioMCPInput,
}
#[mcp_handler]
pub fn getUserEmbeddedBioMCPMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserEmbeddedBioMCPMcpInput>(&input.request.body)?
        .into_owned();
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = data.data;
    let connections = Arc::clone(&input.mcp_connections);
    Err(IoContFn::create_err(
        move |__internal_cont_tx, __internal_ret_chan| {
            Box::pin(async move {
                let __internal_embed_data_0 = embed_async!(db, &data.text);
                __internal_cont_tx
                    .send_async((
                        __internal_ret_chan,
                        Box::new(move || {
                            let __internal_embed_data_0: Vec<f64> = __internal_embed_data_0?;
                            let mut result = {
                                let mut remapping_vals = RemappingMap::new();
                                let txn = db.graph_env.read_txn().unwrap();
                                let vs = G::new(Arc::clone(&db), &txn)
                                    .search_v::<fn(&HVector, &RoTxn) -> bool, _>(
                                        &__internal_embed_data_0,
                                        10,
                                        "EmbeddedBio",
                                        None,
                                    )
                                    .collect_to::<Vec<_>>();
                                txn.commit().unwrap();
                                vs.into_iter()
                            };
                            let first = result.next().unwrap_or(TraversalVal::Empty);
                            connection.iter = result.into_iter();
                            let mut connections = connections.lock().unwrap();
                            connections.add_connection(connection);
                            drop(connections);
                            Ok(helix_db::protocol::format::Format::Json
                                .create_response(&ReturnValue::from(first)))
                        }),
                    ))
                    .await
                    .expect("Cont Channel should be alive")
            })
        },
    ))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteUserInput {
    pub user_id: ID,
}
#[handler]
pub fn deleteUser(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteUserInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .out("User_to_Name", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .out_e("User_to_EmbeddedBio")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createLinkedinEducationInput {
    pub user_id: ID,
    pub school: String,
    pub field: String,
    pub title: String,
    pub date_start: String,
    pub date_end: String,
    pub location: String,
    pub description: String,
}
#[handler]
pub fn createLinkedinEducation(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createLinkedinEducationInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_education = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinEducation", Some(props! { "date_start" => &data.date_start, "field" => &data.field, "school" => &data.school, "location" => &data.location, "description" => &data.description, "date_end" => &data.date_end, "title" => &data.title }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content_linkedin_education = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "LinkedinContent_to_LinkedinEducation",
            None,
            linkedin_content.id(),
            linkedin_education.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_education".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_education.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createLinkedinInfoInput {
    pub user_id: ID,
    pub url: String,
}
#[handler]
pub fn createLinkedinInfo(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createLinkedinInfoInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_info = G::new_mut(Arc::clone(&db), &mut txn)
        .add_n("LinkedinInfo", Some(props! { "url" => &data.url }), None)
        .collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let metadata_linkedin_info = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Metadata_to_LinkedinInfo",
            Some(props! { "url" => data.url.clone() }),
            metadata.id(),
            linkedin_info.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_info".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_info.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteLinkedinWebsiteInput {
    pub website_id: ID,
}
#[handler]
pub fn deleteLinkedinWebsite(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteLinkedinWebsiteInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.website_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getAllUsersInput {}
#[handler]
pub fn getAllUsers(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getAllUsersInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let users = G::new(Arc::clone(&db), &txn)
        .n_from_type("User")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "users".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            users.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getAllUsersMcpInput {
    connection_id: String,
    data: getAllUsersInput,
}
#[mcp_handler]
pub fn getAllUsersMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getAllUsersMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let users = G::new(Arc::clone(&db), &txn)
            .n_from_type("User")
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        users.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteLinkedinContentInput {
    pub linkedin_content_id: ID,
}
#[handler]
pub fn deleteLinkedinContent(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteLinkedinContentInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_content_id)
            .out("LinkedinContent_to_LinkedinWebsite", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_content_id)
            .out("LinkedinContent_to_LinkedinExperience", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_content_id)
            .out_e("LinkedinContent_to_LinkedinCompany")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_content_id)
            .out("LinkedinContent_to_LinkedinEducation", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_content_id)
            .out_e("LinkedinContent_to_CurrentCompany")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_content_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createLinkedinExperienceInput {
    pub user_id: ID,
    pub company: String,
    pub title: String,
    pub field: String,
    pub date_start: String,
    pub date_end: String,
    pub location: String,
    pub description: String,
}
#[handler]
pub fn createLinkedinExperience(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createLinkedinExperienceInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_experience = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinExperience", Some(props! { "date_end" => &data.date_end, "field" => &data.field, "description" => &data.description, "company" => &data.company, "date_start" => &data.date_start, "title" => &data.title, "location" => &data.location }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content_linkedin_experience = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "LinkedinContent_to_LinkedinExperience",
            None,
            linkedin_content.id(),
            linkedin_experience.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_experience".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_experience.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
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
    pub data_source: String,
}
#[handler]
pub fn createLinkedinContent(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createLinkedinContentInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_content = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinContent", Some(props! { "languages" => &data.languages, "industry" => &data.industry, "skills" => &data.skills, "position_end_date" => &data.position_end_date, "name" => &data.name, "summary" => &data.summary, "connection_count" => &data.connection_count, "position_start_date" => &data.position_start_date, "profile_picture" => &data.profile_picture, "follower_count" => &data.follower_count, "certifications" => &data.certifications, "data_source" => &data.data_source, "country" => &data.country, "full_name" => &data.full_name, "region" => &data.region, "email" => &data.email, "linkedin_url" => &data.linkedin_url, "current_position" => &data.current_position, "first_name" => &data.first_name, "last_name" => &data.last_name, "country_code" => &data.country_code, "background_picture" => &data.background_picture, "extracted_at" => &data.extracted_at, "public_id" => &data.public_id }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content_linkedin_info = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "LinkedinInfo_to_LinkedinContent",
            None,
            linkedin_info.id(),
            linkedin_content.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_content".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_content.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteMetadataInput {
    pub metadata_id: ID,
}
#[handler]
pub fn deleteMetadata(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteMetadataInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.metadata_id)
            .out("Metadata_to_MetadataNotes", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.metadata_id)
            .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.metadata_id)
            .out("Metadata_to_WarmConnect", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.metadata_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct updateLinkedinInfoInput {
    pub linkedin_info_id: ID,
    pub url: String,
}
#[handler]
pub fn updateLinkedinInfo(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateLinkedinInfoInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_info = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.linkedin_info_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(props! { "url" => &data.url }))
            .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_info".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_info.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createLinkedinWebsiteInput {
    pub user_id: ID,
    pub url: String,
    pub category: String,
}
#[handler]
pub fn createLinkedinWebsite(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createLinkedinWebsiteInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_website = G::new_mut(Arc::clone(&db), &mut txn)
        .add_n(
            "LinkedinWebsite",
            Some(props! { "url" => &data.url, "category" => &data.category }),
            None,
        )
        .collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content_linkedin_website = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "LinkedinContent_to_LinkedinWebsite",
            None,
            linkedin_content.id(),
            linkedin_website.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_website".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_website.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserWarmConnectsInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserWarmConnects(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserWarmConnectsInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let warm_connects = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_WarmConnect", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "warm_connects".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            warm_connects.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserWarmConnectsMcpInput {
    connection_id: String,
    data: getUserWarmConnectsInput,
}
#[mcp_handler]
pub fn getUserWarmConnectsMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserWarmConnectsMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let warm_connects = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_WarmConnect", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        warm_connects.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteLinkedinExperienceInput {
    pub experience_id: ID,
}
#[handler]
pub fn deleteLinkedinExperience(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteLinkedinExperienceInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.experience_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct updateLinkedinExperienceInput {
    pub experience_id: ID,
    pub company: String,
    pub title: String,
    pub field: String,
    pub date_start: String,
    pub date_end: String,
    pub location: String,
    pub description: String,
}
#[handler]
pub fn updateLinkedinExperience(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateLinkedinExperienceInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_experience = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.experience_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "company" => &data.company, "title" => &data.title, "field" => &data.field, "date_start" => &data.date_start, "date_end" => &data.date_end, "location" => &data.location, "description" => &data.description }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_experience".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_experience.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct addCurrentCompanyInput {
    pub user_id: ID,
    pub linkedin_company_id: ID,
}
#[handler]
pub fn addCurrentCompany(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<addCurrentCompanyInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_company = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.linkedin_company_id)
        .collect_to_obj();
    let linkedin_content_current_company = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "LinkedinContent_to_CurrentCompany",
            None,
            linkedin_content.id(),
            linkedin_company.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_content_current_company".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_content_current_company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createNameInput {
    pub user_id: ID,
    pub first: String,
    pub last: String,
}
#[handler]
pub fn createName(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createNameInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let name = G::new_mut(Arc::clone(&db), &mut txn)
        .add_n(
            "Name",
            Some(props! { "first" => &data.first, "last" => &data.last }),
            None,
        )
        .collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let user_name = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "User_to_Name",
            Some(props! { "first" => data.first.clone() }),
            user.id(),
            name.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "name".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            name.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteEmbeddedBioInput {
    pub user_id: ID,
}
#[handler]
pub fn deleteEmbeddedBio(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteEmbeddedBioInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .out_e("User_to_EmbeddedBio")
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteLinkedinCompanyInput {
    pub company_id: ID,
}
#[handler]
pub fn deleteLinkedinCompany(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteLinkedinCompanyInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.company_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
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
    pub sender: String,
}
#[handler]
pub fn updateUser(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateUserInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "phone" => &data.phone, "email" => &data.email, "bio" => &data.bio, "age" => &data.age, "location" => &data.location, "profilePic" => &data.profilePic, "color" => &data.color, "elo" => &data.elo, "sender" => &data.sender }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserLinkedinContentInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserLinkedinContent(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinContentInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_content".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            linkedin_content.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserLinkedinContentMcpInput {
    connection_id: String,
    data: getUserLinkedinContentInput,
}
#[mcp_handler]
pub fn getUserLinkedinContentMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinContentMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
            .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        linkedin_content.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserLinkedinWebsitesInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserLinkedinWebsites(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinWebsitesInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_websites = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
        .out("LinkedinContent_to_LinkedinWebsite", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_websites".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            linkedin_websites.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserLinkedinWebsitesMcpInput {
    connection_id: String,
    data: getUserLinkedinWebsitesInput,
}
#[mcp_handler]
pub fn getUserLinkedinWebsitesMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinWebsitesMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
            .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_websites = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
            .out("LinkedinContent_to_LinkedinWebsite", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        linkedin_websites.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createWarmConnectInput {
    pub user_id: ID,
    pub name: String,
    pub email: String,
}
#[handler]
pub fn createWarmConnect(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createWarmConnectInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let warm_connect = G::new_mut(Arc::clone(&db), &mut txn)
        .add_n(
            "WarmConnect",
            Some(props! { "email" => &data.email, "name" => &data.name }),
            None,
        )
        .collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let metadata_to_warm_connect = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Metadata_to_WarmConnect",
            None,
            metadata.id(),
            warm_connect.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "warm_connect".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            warm_connect.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserMetadataInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserMetadata(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserMetadataInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "metadata".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            metadata.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserMetadataMcpInput {
    connection_id: String,
    data: getUserMetadataInput,
}
#[mcp_handler]
pub fn getUserMetadataMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserMetadataMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        metadata.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
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
    pub specialties: Vec<String>,
}
#[handler]
pub fn updateLinkedinCompany(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateLinkedinCompanyInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_company = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.company_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "name" => &data.name, "domain" => &data.domain, "industry" => &data.industry, "staff_count" => &data.staff_count, "founded" => &data.founded, "website" => &data.website, "headquarters" => &data.headquarters, "description" => &data.description, "specialties" => &data.specialties }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_company".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserLinkedinCompaniesInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserLinkedinCompanies(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinCompaniesInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_companies = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
        .out("LinkedinContent_to_LinkedinCompany", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_companies".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            linkedin_companies.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserLinkedinCompaniesMcpInput {
    connection_id: String,
    data: getUserLinkedinCompaniesInput,
}
#[mcp_handler]
pub fn getUserLinkedinCompaniesMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserLinkedinCompaniesMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
            .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let linkedin_companies = G::new_from(Arc::clone(&db), &txn, linkedin_content.clone())
            .out("LinkedinContent_to_LinkedinCompany", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        linkedin_companies.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteNameInput {
    pub name_id: ID,
}
#[handler]
pub fn deleteName(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteNameInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.name_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct findWarmConnectInput {
    pub name: String,
    pub email: String,
}
#[handler]
pub fn findWarmConnect(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<findWarmConnectInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let warm_connect = G::new(Arc::clone(&db), &txn)
        .n_from_type("WarmConnect")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("name")
                    .map_value_or(false, |v| *v == data.name.clone())?
                    && G::new_from(Arc::clone(&db), &txn, val.clone())
                        .check_property("email")
                        .map_value_or(false, |v| *v == data.email.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "warm_connect".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            warm_connect.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct findWarmConnectMcpInput {
    connection_id: String,
    data: findWarmConnectInput,
}
#[mcp_handler]
pub fn findWarmConnectMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<findWarmConnectMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let warm_connect = G::new(Arc::clone(&db), &txn)
            .n_from_type("WarmConnect")
            .filter_ref(|val, txn| {
                if let Ok(val) = val {
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                        .check_property("name")
                        .map_value_or(false, |v| *v == data.name.clone())?
                        && G::new_from(Arc::clone(&db), &txn, val.clone())
                            .check_property("email")
                            .map_value_or(false, |v| *v == data.email.clone())?)
                } else {
                    Ok(false)
                }
            })
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        warm_connect.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct createMetadataNotesInput {
    pub user_id: ID,
    pub aiScore: i64,
    pub userScore: i64,
    pub text: String,
    pub flagged: bool,
}
#[handler]
pub fn createMetadataNotes(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createMetadataNotesInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user_metadata_notes = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("MetadataNotes", Some(props! { "text" => &data.text, "aiScore" => &data.aiScore, "flagged" => &data.flagged, "userScore" => &data.userScore }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let metadata_metadata_notes = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Metadata_to_MetadataNotes",
            None,
            metadata.id(),
            user_metadata_notes.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user_metadata_notes".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user_metadata_notes.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getEmbedUserBioInput {
    pub user_id: ID,
}
#[handler]
pub fn getEmbedUserBio(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getEmbedUserBioInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let user_bio = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_EmbeddedBio", &EdgeType::Vec)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user_bio".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            user_bio.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getEmbedUserBioMcpInput {
    connection_id: String,
    data: getEmbedUserBioInput,
}
#[mcp_handler]
pub fn getEmbedUserBioMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getEmbedUserBioMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let user_bio = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_EmbeddedBio", &EdgeType::Vec)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        user_bio.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
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
    pub specialties: Vec<String>,
}
#[handler]
pub fn createLinkedinCompany(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<createLinkedinCompanyInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_company = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("LinkedinCompany", Some(props! { "industry" => &data.industry, "name" => &data.name, "founded" => &data.founded, "domain" => &data.domain, "headquarters" => &data.headquarters, "website" => &data.website, "staff_count" => &data.staff_count, "specialties" => &data.specialties, "description" => &data.description }), None).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_info = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_LinkedinInfo", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content = G::new_from(Arc::clone(&db), &txn, linkedin_info.clone())
        .out("LinkedinInfo_to_LinkedinContent", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let linkedin_content_linkedin_company = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "LinkedinContent_to_LinkedinCompany",
            None,
            linkedin_content.id(),
            linkedin_company.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_company".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_company.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteMetadataNotesInput {
    pub metadata_notes_id: ID,
}
#[handler]
pub fn deleteMetadataNotes(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteMetadataNotesInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.metadata_notes_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct getUserMetadataNotesInput {
    pub user_id: ID,
}
#[handler]
pub fn getUserMetadataNotes(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserMetadataNotesInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
        .out("User_to_Metadata", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let metadata_notes = G::new_from(Arc::clone(&db), &txn, metadata.clone())
        .out("Metadata_to_MetadataNotes", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "metadata_notes".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            metadata_notes.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
#[derive(Deserialize, Clone)]
pub struct getUserMetadataNotesMcpInput {
    connection_id: String,
    data: getUserMetadataNotesInput,
}
#[mcp_handler]
pub fn getUserMetadataNotesMcp(input: &mut MCPToolInput) -> Result<Response, GraphError> {
    let data = input
        .request
        .in_fmt
        .deserialize::<getUserMetadataNotesMcpInput>(&input.request.body)?;
    let mut connections = input.mcp_connections.lock().unwrap();
    let mut connection = match connections.remove_connection(&data.connection_id) {
        Some(conn) => conn,
        None => return Err(GraphError::Default),
    };
    drop(connections);
    let db = Arc::clone(&input.mcp_backend.db);
    let data = &data.data;
    let connections = Arc::clone(&input.mcp_connections);
    let mut result = {
        let mut remapping_vals = RemappingMap::new();
        let txn = db.graph_env.read_txn().unwrap();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.user_id)
            .collect_to_obj();
        let metadata = G::new_from(Arc::clone(&db), &txn, user.clone())
            .out("User_to_Metadata", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        let metadata_notes = G::new_from(Arc::clone(&db), &txn, metadata.clone())
            .out("Metadata_to_MetadataNotes", &EdgeType::Node)
            .collect_to::<Vec<_>>();
        txn.commit().unwrap();
        metadata_notes.into_iter()
    };
    let first = result.next().unwrap_or(TraversalVal::Empty);
    connection.iter = result.into_iter();
    let mut connections = connections.lock().unwrap();
    connections.add_connection(connection);
    drop(connections);
    Ok(helix_db::protocol::format::Format::Json.create_response(&ReturnValue::from(first)))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct updateLinkedinWebsiteInput {
    pub website_id: ID,
    pub url: String,
    pub category: String,
}
#[handler]
pub fn updateLinkedinWebsite(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateLinkedinWebsiteInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_website = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.website_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(
                props! { "url" => &data.url, "category" => &data.category },
            ))
            .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_website".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_website.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct updateWarmConnectInput {
    pub warm_connect_id: ID,
    pub name: String,
    pub email: String,
}
#[handler]
pub fn updateWarmConnect(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateWarmConnectInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let warm_connect = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.warm_connect_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(
                props! { "name" => &data.name, "email" => &data.email },
            ))
            .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "warm_connect".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            warm_connect.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct updateLinkedinEducationInput {
    pub education_id: ID,
    pub school: String,
    pub field: String,
    pub title: String,
    pub date_start: String,
    pub date_end: String,
    pub location: String,
    pub description: String,
}
#[handler]
pub fn updateLinkedinEducation(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateLinkedinEducationInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let linkedin_education = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.education_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "school" => &data.school, "field" => &data.field, "title" => &data.title, "date_start" => &data.date_start, "date_end" => &data.date_end, "location" => &data.location, "description" => &data.description }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "linkedin_education".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            linkedin_education.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct deleteLinkedinEducationInput {
    pub education_id: ID,
}
#[handler]
pub fn deleteLinkedinEducation(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<deleteLinkedinEducationInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.education_id)
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
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct updateMetadataInput {
    pub metadata_id: ID,
    pub created_ts: DateTime<Utc>,
    pub last_updated_ts: DateTime<Utc>,
    pub archetype: bool,
    pub referredBy: String,
}
#[handler]
pub fn updateMetadata(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<updateMetadataInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let metadata = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.metadata_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "created_ts" => &data.created_ts, "last_updated_ts" => &data.last_updated_ts, "archetype" => &data.archetype, "referredBy" => &data.referredBy }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "metadata".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            metadata.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
