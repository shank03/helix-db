use crate::{
    helix_engine::{
        types::{GraphError, VectorError},
        vector_core::vector_distance::DistanceCalc,
    },
    protocol::{
        return_values::ReturnValue,
        value::Value,
    },
    utils::filterable::{Filterable, FilterableType},
};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap, fmt::Debug};

// TODO: make this generic over the type of encoding (f32, f64, etc)
// TODO: use const param to set dimension
// TODO: set level as u8

#[repr(C, align(16))] // TODO: see performance impact of repr(C) and align(16)
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct HVector {
    /// The id of the HVector
    pub id: u128,
    /// Whether the HVector is deleted (will be used for soft deletes)
    pub is_deleted: bool,
    /// The level of the HVector
    pub level: usize,
    /// The distance of the HVector
    pub distance: Option<f64>,
    /// The actual vector
    pub data: Vec<f64>,
    /// The properties of the HVector
    pub properties: Option<HashMap<String, Value>>,
}

impl Eq for HVector {}
impl PartialOrd for HVector {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.distance.partial_cmp(&self.distance)
    }
}
impl Ord for HVector {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Debug for HVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ \nid: {},\nis_deleted: {},\nlevel: {},\ndistance: {:?},\ndata: {:?},\nproperties: {:#?} }}", uuid::Uuid::from_u128(self.id).to_string(), self.is_deleted, self.level, self.distance, self.data, self.properties)
    }
}

impl HVector {
    #[inline(always)]
    pub fn new(data: Vec<f64>) -> Self {
        let id = uuid::Uuid::new_v4().as_u128();
        HVector {
            id,
            is_deleted: false,
            level: 0,
            data,
            distance: None,
            properties: None,
        }
    }

    #[inline(always)]
    pub fn from_slice(level: usize, data: Vec<f64>) -> Self {
        let id = uuid::Uuid::new_v4().as_u128();
        HVector {
            id,
            is_deleted: false,
            level,
            data,
            distance: None,
            properties: None,
        }
    }

    /// Returns the data of the HVector
    #[inline(always)]
    pub fn get_data(&self) -> &[f64] {
        &self.data
    }

    /// Returns the id of the HVector
    #[inline(always)]
    pub fn get_id(&self) -> u128 {
        self.id
    }

    /// Returns the level of the HVector
    #[inline(always)]
    pub fn get_level(&self) -> usize {
        self.level
    }

    /// Converts the HVector to an vec of bytes by accessing the data field directly
    /// and converting each f64 to a byte slice
    pub fn to_bytes(&self) -> Vec<u8> {
        let size = self.data.len() * std::mem::size_of::<f64>();
        let mut bytes = Vec::with_capacity(size);
        for &value in &self.data {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        bytes
    }

    // will make to use const param for type of encoding (f32, f64, etc)
    /// Converts a byte array into a HVector by chunking the bytes into f64 values
    pub fn from_bytes(id: u128, level: usize, bytes: &[u8]) -> Result<Self, VectorError> {
        if bytes.len() % std::mem::size_of::<f64>() != 0 {
            return Err(VectorError::InvalidVectorData);
        }

        let mut data = Vec::with_capacity(bytes.len() / std::mem::size_of::<f64>());
        let chunks = bytes.chunks_exact(std::mem::size_of::<f64>());

        for chunk in chunks {
            let value = f64::from_be_bytes(chunk.try_into().unwrap());
            data.push(value);
        }

        Ok(HVector {
            id,
            is_deleted: false,
            level,
            data,
            distance: None,
            properties: None,
        })
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline(always)]
    pub fn distance_to(&self, other: &HVector) -> Result<f64, VectorError> {
        HVector::distance(self, other)
    }

    #[inline(always)]
    pub fn set_distance(&mut self, distance: f64) {
        self.distance = Some(distance);
    }

    #[inline(always)]
    pub fn get_distance(&self) -> f64 {
        match self.distance {
            Some(distance) => distance,
            // changed from 0.0 to 2.0 to match the distance calculation
            // if the distance is not set, make it the furthest distance
            None => 2.0,
        }
    }
}


/// Filterable implementation for HVector
/// 
/// see helixdb/src/protocol/filterable.rs
/// 
/// NOTE: This could be moved to the protocol module with the node and edges in `helixdb/protocol/items.rs``
impl Filterable for HVector {
    fn type_name(&self) -> FilterableType {
        FilterableType::Vector
    }

    fn id(&self) -> &u128 {
        &self.id
    }

    fn uuid(&self) -> String {
        uuid::Uuid::from_u128(self.id).to_string()
    }

    fn label(&self) -> &str {
        match &self.properties {
            Some(properties) => match properties.get("label") {
                Some(label) => label.as_str(),
                None => "vector",
            },
            None => "vector",
        }
    }

    fn from_node(&self) -> u128 {
        unreachable!()
    }

    fn from_node_uuid(&self) -> String {
        unreachable!()
    }

    fn to_node(&self) -> u128 {
        unreachable!()
    }

    fn to_node_uuid(&self) -> String {
        unreachable!()
    }

    fn properties(self) -> Option<HashMap<String, Value>> {
        let mut properties = match self.properties {
            Some(properties) => properties,
            None => HashMap::new(),
        };
        properties.insert(
            "data".to_string(),
            Value::Array(self.data.iter().map(|f| Value::F64(*f)).collect()),
        );
        Some(properties)
    }

    fn vector_data(&self) -> &[f64] {
        &self.data
    }

    fn score(&self) -> f64 {
        self.get_distance()
    }

    fn properties_mut(&mut self) -> &mut Option<HashMap<String, Value>> {
        &mut self.properties
    }

    fn properties_ref(&self) -> &Option<HashMap<String, Value>> {
        &self.properties
    }

    fn check_property(&self, key: &str) -> Result<&Value, GraphError> {
        match &self.properties {
            Some(properties) => properties
                .get(key)
                .ok_or(GraphError::ConversionError(format!(
                    "Property {} not found",
                    key
                ))),
            None => Err(GraphError::ConversionError(format!(
                "Property {} not found",
                key
            ))),
        }
    }

    fn find_property(
        &self,
        _key: &str,
        _secondary_properties: &HashMap<String, ReturnValue>,
        _property: &mut ReturnValue,
    ) -> Option<&ReturnValue> {
        unreachable!()
    }
}
