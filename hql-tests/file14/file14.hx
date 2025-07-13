N::File14 {
    name: String,
    age: I32,
}

QUERY file14() =>
    res <- SearchBM25<File14>("John", 10)
    RETURN res
