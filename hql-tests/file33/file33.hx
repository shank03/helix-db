N::User {
    name: String,
    age: I32
}


QUERY GetOrder() =>
    userByAge <- N<User>::ORDER<Desc>(_::{age})
    userByName <- N<User>::ORDER<Asc>(_::{name})
    RETURN userByAge, userByName