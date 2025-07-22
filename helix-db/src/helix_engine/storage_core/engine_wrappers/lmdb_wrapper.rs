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
impl<'a, 'env, K: heed3::BytesEncode<'a>, V: heed3::BytesDecode<'a>> Storage<'a> for heed3::Database<K, V> {
    fn get_data(&self, txn: RTxn<'a, 'env>, key: &K) -> Result<Option<Vec<u8>>, GraphError> {
        Ok(self.get(txn.txn, key)?.map(|v| v.to_vec()))
    }

    fn put_data(&self, txn: &mut WTxn, key: &[u8], value: &[u8]) -> Result<(), GraphError> {
        Ok(self.put(txn, key, value)?)
    }

    fn delete_data(&self, txn: &mut WTxn, key: &[u8]) -> Result<(), GraphError> {
        Ok(self.delete(txn, key)?)
    }

    fn iter_data<M>(
        &self,
        txn: &'a RTxn<'a>,
    ) -> Result<HelixIterator<&'a [u8], &'a [u8], M>, GraphError> {
        Ok(HelixIterator {
            iter: txn.txn.iterator_cf(self, rocksdb::IteratorMode::Start),
            _phantom: PhantomData,
        })
    }
}