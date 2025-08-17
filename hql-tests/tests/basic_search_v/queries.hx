N::File7 {
    name: String,
    age: I32,
}

V::File7Vec {
    content: String,
}

E::EdgeFile7 {
    From: File7,
    To: File7,
}


QUERY file7(vec: [F64]) =>
    vecs <- SearchV<File7Vec>(vec, 10)
    // pre_filter <- SearchV<File7Vec>(vec, 10)::PREFILTER(_::{content}::EQ("hello"))
    RETURN "hello"
