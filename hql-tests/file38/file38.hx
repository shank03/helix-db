V::User {
    name: String,
    age: I64,
    email: String,
}


QUERY addUser(name: String, age: I64, email: String) => 
    user <- AddV<User>(Embed(name), {
        name: name,
        age: age,
        email: email
    })
    RETURN user