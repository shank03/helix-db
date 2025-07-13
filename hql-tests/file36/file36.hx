QUERY createUser (arr: [I64]) =>
    user <- AddN<User>({arr: arr})
    RETURN user

QUERY getUser (user_id: ID) =>
    user <- N<User>(user_id)
    RETURN user