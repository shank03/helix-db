N::User {
    name: String,
}


QUERY addUser(names: [String]) => 
    FOR n IN names {
        AddN<User>({name: n})
    }
    RETURN "success"