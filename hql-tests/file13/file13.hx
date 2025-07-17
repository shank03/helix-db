N::User {
    user_field: String,
}

N::Admin {
    admin_field: String,
}

N::Guest {
    guest_field: String,
}

Enum::UserType {
    NormalUser(User),
    AdminUser(Admin),
    GuestUser(Guest),
}

QUERY GetAdmins() => 
    users <- N<UserType>::MATCH|_|{
        UserType::NormalUser(user) => user::{user_field},
        UserType::AdminUser(admin) => admin::{admin_field},
        UserType::GuestUser(guest) => guest::{guest_field}
    }
    RETURN users