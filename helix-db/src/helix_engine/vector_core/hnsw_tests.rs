// MAKE SURE TO --release
use crate::{
    helix_engine::vector_core::{
        hnsw::HNSW,
        vector::HVector,
        vector_core::{HNSWConfig, VectorCore},
    },
};
use heed3::{Env, EnvOpenOptions, RoTxn};
use rand::Rng;

type Filter = fn(&HVector, &RoTxn) -> bool;

fn setup_temp_env() -> Env {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();

    unsafe {
        EnvOpenOptions::new()
            .map_size(2 * 1024 * 1024 * 1024) // 2 GB
            .max_dbs(10)
            .open(path)
            .unwrap()
    }
}

/// Higher values of similarity make the vectors more similar
fn gen_sim_vecs(n: usize, dim: usize, similarity: f64) -> Vec<Vec<f64>> {
    let mut rng = rand::rng();
    let mut vectors = Vec::with_capacity(n);
    let similarity = 1.0 - similarity;

    let base: Vec<f64> = (0..dim).map(|_| rng.random_range(-1.0..1.0)).collect();

    for _ in 0..n {
        let mut vec = base.clone();
        for v in vec.iter_mut() {
            *v += rng.random_range(-similarity..similarity);
            *v = v.clamp(-1.0, 1.0);
        }
        vectors.push(vec);
    }

    vectors
}

#[test]
fn tests_hnsw_config_build() {
    let env = setup_temp_env();
    let mut txn = env.write_txn().unwrap();

    let config = HNSWConfig::new(
        Some(69),
        Some(69),
        Some(69),
    );

    let index = VectorCore::new(&env, &mut txn, config).unwrap();
    assert_eq!(index.config.m, 69, "m is set");
    assert_eq!(index.config.ef_construct, 69, "ef_construct is set");
    assert_eq!(index.config.ef, 69, "ef is set");

    let config = HNSWConfig::new(
        Some(6969),
        Some(6969),
        Some(6969),
    );
    assert_eq!(config.m, 48);
    assert_eq!(config.ef_construct, 512);
    assert_eq!(config.ef, 512);
}

#[test]
fn test_hnsw_insert() {
    let env = setup_temp_env();
    let mut txn = env.write_txn().unwrap();
    let index = VectorCore::new(&env, &mut txn, HNSWConfig::new(None, None, None)).unwrap();

    let n_base = 1_000;
    let dims = 750;
    let vectors = gen_sim_vecs(n_base, dims, 0.8);

    for data in vectors {
        let vec = index.insert::<Filter>(&mut txn, &data, None).unwrap();
        assert_eq!(vec.data, data);
        assert!(vec.properties.is_none());
    }

    assert!(index.num_inserted_vectors(&txn).unwrap() > n_base as u64);
}

#[test]
fn test_get_vector() {
    let env = setup_temp_env();
    let mut txn = env.write_txn().unwrap();
    let index = VectorCore::new(&env, &mut txn, HNSWConfig::new(None, None, None)).unwrap();

    let n_base = 1_000;
    let dims = 750;
    let vectors = gen_sim_vecs(n_base, dims, 0.8);

    let mut all_vectors: Vec<HVector> = Vec::with_capacity(n_base);
    for data in vectors {
        all_vectors.push(index.insert::<Filter>(&mut txn, &data, None).unwrap());
    }

    for inserted_vec in all_vectors {
        let got_vec = match index.get_vector(&txn, inserted_vec.id, 0, true) {
            Ok(vec) => vec,
            Err(_) => panic!("couldn't find the vector"),
        };

        assert_eq!(got_vec.get_level(), 0);
        assert_eq!(got_vec.id, inserted_vec.id);
        assert_eq!(got_vec.data, inserted_vec.data);
    }
}

#[test]
fn test_hnsw_search() {
}

#[test]
fn test_hnsw_delete() {
}

