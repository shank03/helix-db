N::UserFile12 {
    name: String,
    age: U32,
    email: String,
    created_at: Date DEFAULT NOW,
    updated_at: Date DEFAULT NOW, 
}


QUERY CreatedUser(name: String, age: U32, email: String) => 
    user <- AddN<UserFile12>({name: name, age: age, email: email})
    RETURN user

QUERY GetUsers() => 
    users <- N<UserFile12>
    RETURN users