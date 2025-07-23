#[cfg(feature = "lmdb")]
use heed3::iteration_method::{MoveOnCurrentKeyDuplicates, MoveThroughDuplicateValues};

#[cfg(feature = "rocksdb")]
use crate::helix_engine::storage_core::engine_wrapper::{Database, Table};
use crate::helix_engine::storage_core::engine_wrapper::{HelixIterator, RTxn, Storage, WTxn};
use crate::helix_engine::types::GraphError;
use std::marker::PhantomData;

#[cfg(feature = "lmdb")]
pub type U128 = heed3::types::U128<heed3::byteorder::BE>;

#[cfg(feature = "rocksdb")]
pub enum Bytes {}

#[cfg(feature = "lmdb")]
pub type Bytes = heed3::types::Bytes;

#[cfg(feature = "lmdb")]
impl<'a> RTxn<'a> {
    pub fn get_txn(&'a self) -> &'a heed3::RoTxn<'a> {
        return &self.txn;
    }
}

#[cfg(feature = "lmdb")]
impl Storage for heed3::Database<Bytes, Bytes> {
    type Key<'a> = &'a [u8];
    type Value<'a> = &'a [u8];
    type BasicIter<'a> = heed3::RoIter<'a, Bytes, heed3::types::LazyDecode<Bytes>>;
    type PrefixIter<'a> = heed3::RoPrefix<'a, Bytes, heed3::types::LazyDecode<Bytes>>;
    type DuplicateIter<'a> =
        heed3::RoIter<'a, Bytes, heed3::types::LazyDecode<Bytes>, MoveOnCurrentKeyDuplicates>;

    fn get_data<'a>(&self, txn: &'a RTxn<'a>, key: Self::Key<'a>) -> Result<Option<Vec<u8>>, GraphError> {
        Ok(self.get(txn.get_txn(), key)?.map(|v| v.to_vec()))
    }

    fn put_data<'a>(
        &self,
        txn: &'a mut WTxn<'a>,
        key: Self::Key<'a>,
        value: Self::Value<'a>,
    ) -> Result<(), GraphError> {
        Ok(self.put(txn.get_txn(), key, value)?)
    }

    fn delete_data<'a>(&self, txn: &'a mut WTxn<'a>, key: Self::Key<'a>) -> Result<(), GraphError> {
        self.delete(txn.get_txn(), key)?;
        Ok(())
    }

    fn iter_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
    ) -> Result<HelixIterator<'a, Self::BasicIter<'a>>, GraphError> {
        Ok(HelixIterator {
            iter: self
                .lazily_decode_data()
                .iter(txn.get_txn())
                .map_err(|e| GraphError::from(e))?,
            _phantom: PhantomData,
        })
    }

    fn prefix_iter_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
        prefix: Self::Key<'a>,
    ) -> Result<HelixIterator<'a, Self::PrefixIter<'a>>, GraphError> {
        Ok(HelixIterator {
            iter: self
                .lazily_decode_data()
                .prefix_iter(txn.get_txn(), prefix)?,
            _phantom: PhantomData,
        })
    }

    fn get_duplicate_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
        key: Self::Key<'a>,
    ) -> Result<HelixIterator<'a, Self::DuplicateIter<'a>>, GraphError> {
        let duplicate_iter = match self
            .lazily_decode_data()
            .get_duplicates(txn.get_txn(), key)?
        {
            Some(iter) => iter,
            None => return Err(GraphError::from("No duplicates found")),
        };

        Ok(HelixIterator {
            iter: duplicate_iter,
            _phantom: PhantomData,
        })
    }
}
