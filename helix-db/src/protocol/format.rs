use std::{borrow::Cow, collections::HashMap, error::Error, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;

#[derive(Debug, Default, Clone, Copy)]
pub enum Format {
    #[default]
    Json,
}

impl Format {
    pub fn serialize<T: Serialize>(self, val: &T) -> Cow<[u8]> {
        match self {
            Format::Json => sonic_rs::to_string(val).unwrap().into_bytes().into(),
        }
    }

    pub async fn serialize_to_async<T: Serialize>(
        self,
        val: &T,
        writer: &mut BufWriter<impl AsyncWrite + Unpin>,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            Format::Json => {
                let encoded = sonic_rs::to_vec(val)?;
                writer.write_all(&encoded).await?;
            }
        }
        Ok(())
    }

    pub fn deserialize<'a, T: Deserialize<'a>>(
        self,
        val: &'a [u8],
    ) -> Result<MaybeOwned<'a, T>, Box<dyn Error>> {
        match self {
            Format::Json => Ok(MaybeOwned::Owned(sonic_rs::from_slice::<T>(val)?)),
        }
    }

    pub fn from_headers(headers: HashMap<String, String>) -> (Format, Format) {
        let content_type = headers
            .iter()
            .find_map(|(k, v)| {
                if k.to_ascii_lowercase() == "content-type" {
                    Some(Format::from_str(v).unwrap_or_default())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let accept = headers
            .iter()
            .find_map(|(k, v)| {
                if k.to_ascii_lowercase() == "accept" {
                    Some(Format::from_str(v).unwrap_or(content_type))
                } else {
                    None
                }
            })
            .unwrap_or(content_type);

        (content_type, accept)
    }
}

impl FromStr for Format {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "application/json" => Ok(Format::Json),
            _ => Err(()),
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
