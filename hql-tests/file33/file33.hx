N::User {
    name: String,
    age: I32
}


QUERY GetOrder() =>
    userByAge <- N<User>::OrderByDesc(_::{age})
    userByName <- N<User>::OrderByAsc(_::{name})
    RETURN userByAge, userByName