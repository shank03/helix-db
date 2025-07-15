use std::{borrow::Cow, ops::Deref};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub enum Format {
    #[default]
    Json,
}

impl Format {
    pub fn serialize<T: Serialize>(&self, val: &T) -> Cow<[u8]> {
        match self {
            Format::Json => sonic_rs::to_string(val).unwrap().into_bytes().into(),
        }
    }

    pub fn deserialize<'a, T: Deserialize<'a>>(&self, val: &'a [u8]) -> MaybeOwned<'a, T> {
        match self {
            Format::Json => MaybeOwned::Owned(sonic_rs::from_slice::<T>(val).unwrap()),
        }
    }
}

pub enum MaybeOwned<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<'a, T> Deref for MaybeOwned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwned::Owned(v) => &v,
            MaybeOwned::Borrowed(v) => *v,
        }
    }
}
