N::User{
    name: String,
    age: I32,
}

QUERY bigRemap(id: ID) => 
    user <- N<User>(id)::|u|{
        userdata: u::{name, age}
        
    }
    RETURN user