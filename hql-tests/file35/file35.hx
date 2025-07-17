N::User {
    name: String,
    age: I64,
    email: String,
}


QUERY addUser(name: String, age: I64, email: String) => 
    user <- AddN<User>({
        name: name,
        age: age,
        email: email
    })
    RETURN user


QUERY updateUser(id: ID, name: String) => 
    u <- N<User>(id)::UPDATE({
        name: name
    })
    RETURN "success"


QUERY getUser(id: ID) => 
    u <- N<User>(id)
    RETURN u

