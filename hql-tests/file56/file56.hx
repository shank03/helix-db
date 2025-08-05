QUERY CreateUserBioEmbedding(userId: String, bioText: String, lastUpdated: String) =>
    embedding <- AddV<UserEmbedding>(Embed(bioText), {
        userId: userId,
        dataType: "bio",
        metadata: "{}",
        lastUpdated: lastUpdated
    })
    RETURN embedding

QUERY SearchSimilarUsers(queryText?: String, k: I64, dataType: String) =>
    search_results <- SearchV<UserEmbedding>(Embed(queryText), k)
    RETURN search_results

V::UserEmbedding {
    userId: String,
    dataType: String,
    metadata: String DEFAULT "{}",
    lastUpdated: String,
    createdAt: Date DEFAULT NOW
}

N::User {
    name: String,
}

N::OtherUser {
    name?: String,
    age: I64,
}

QUERY UpdateUser(id: ID, user: OtherUser) =>
    updated_user <- N<OtherUser>(id)::UPDATE(user)
    RETURN updated_user

QUERY GetUser(userId: String) =>
    user <- N<User>(userId)::ID
    RETURN user