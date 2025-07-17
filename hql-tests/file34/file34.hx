N::User { 
  name: String,
  age: I64,
  email: String,
  phone: String,
  address: String,
  city: String,
  state: String,
  zip: String,
  country: String,
  created_at: Date,
  updated_at: Date,
}


QUERY addUser(
    name: String,
    age: I64,
    email: String,
    phone: String,
    address: String,
    city: String,
    state: String,
    zip: String,
    country: String,
    created_at: Date,
    updated_at: Date
) =>
    AddN<User>({
        name: name,
        age: age,
        email: email,
        phone: phone,
        address: address,
        city: city,
        state: state,
        zip: zip,
        country: country,
        created_at: created_at,
        updated_at: updated_at
    })
    RETURN "success"
