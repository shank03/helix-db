#[cfg(all(feature = "embed_vectors", feature = "float_vectors"))]
compile_error!("Features \"embed_vectors\" and \"float_vectors\" cannot be enabled at the same time.");

#[cfg(not(any(feature = "embed_vectors", feature = "float_vectors")))]
compile_error!("Either feature \"embed_vectors\" or \"float_vectors\" must be enabled.");

#[cfg(feature = "embed_vectors")]
pub type Vector = String;

#[cfg(feature = "float_vectors")]
pub type Vector = Vec<f64>;

