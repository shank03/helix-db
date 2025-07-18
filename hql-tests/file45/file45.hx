N::Country {
    name: String,
    currency: String,
    population: U64,
    gdp: F64
}

QUERY getCountriesByPopulation (max_population: U64) =>
    countries <- N<Country>::WHERE(_::{population}::LT(max_population))
    RETURN countries