#[cfg(feature = "lmdb")]
use heed3::iteration_method::MoveThroughDuplicateValues;

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
impl<'a> Storage<'a, &'a [u8], &'a [u8]> for heed3::Database<Bytes, Bytes> {
    fn get_data(&self, txn: &'a RTxn<'a>, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError> {
        Ok(self.get(txn.get_txn(), key)?.map(|v| v.to_vec()))
    }

    fn put_data(&self, txn: &'a mut WTxn<'a>, key: &[u8], value: &[u8]) -> Result<(), GraphError> {
        Ok(self.put(txn.get_txn(), key, value)?)
    }

    fn delete_data(&self, txn: &'a mut WTxn<'a>, key: &[u8]) -> Result<(), GraphError> {
        self.delete(txn.get_txn(), key)?;
        Ok(())
    }

    fn iter_data<M>(
        &self,
        txn: &'a RTxn<'a>,
    ) -> Result<HelixIterator<&'a [u8], &'a [u8], M>, GraphError> {
        Ok(HelixIterator {
            iter: self.lazily_decode_data().iter(txn.get_txn()).map_err(|e| GraphError::from(e))?,
            _phantom: PhantomData,
        })
    }
}
