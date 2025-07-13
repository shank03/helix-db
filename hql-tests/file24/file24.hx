QUERY create_user(name: String, age: U32, email: String, now: I32) =>
    user <- AddN<User>({name: name, age: age, email: email, created_at: now, updated_at: now})
    RETURN user

QUERY create_follow(follower_id: ID, followed_id: ID, now: I32) =>
    follower <- N<User>(follower_id)
    followed <- N<User>(followed_id)
    AddE<Follows>({since: now})::From(follower)::To(followed)
    RETURN "success"

QUERY get_followed_users(user_id: ID) =>
    followed <- N<User>(user_id)::Out<Follows>
    RETURN followed

QUERY create_post(user_id: ID, content: String, now: I32) =>
    user <- N<User>(user_id)
    post <- AddN<Post>({content: content, created_at: now, updated_at: now})
    AddE<Created>({created_at: now})::From(user)::To(post)
    RETURN post
    

// property access

QUERY find_users_access() =>
    users <- N<User>
    RETURN users::{ name: name, age:age }

// property addition


QUERY get_user_details_addition() =>
    users <- N<User>
   RETURN users::{name: name, followerCount: _::In<Follows>::COUNT}

// property exclusion

// QUERY find_users_exclusion() =>
//    users <- N<User>
//    RETURN users::!{name, email}

// property remapping

QUERY get_name_remapping_simple() =>
    users <- N<User>
    RETURN users::{
        givenName: _::{name}
    }

QUERY find_user_posts_with_creator_details(userID: ID) =>
    user <- N<User>(userID)
    posts <- user::Out<Created>
    RETURN user::|creator|{
        creatorName: name,
        createdPosts: posts::{
            postContent: content,
            createdAt: created_at,
            updatedAt: updated_at,
        }
    }