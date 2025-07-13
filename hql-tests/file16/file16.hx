N::File16 {
    INDEX name: String,
    age: I32,
}

QUERY file16(name: String) =>
    node <- N<File16>({name: name})
    RETURN node
