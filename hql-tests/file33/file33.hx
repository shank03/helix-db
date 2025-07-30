N::User {
    name: String,
    age: I32,
    created_at: Date,
}

E::Knows {
    From: User,
    To: User,
    Properties: {
        since: Date,
    }
}


QUERY GetOrder() =>
    userByAge <- N<User>::ORDER<Desc>(_::{age})
    userByName <- N<User>::ORDER<Asc>(_::{name})
    userByCreatedAt <- N<User>::ORDER<Desc>(_::{created_at})::OutE<Knows>
    RETURN userByAge, userByName