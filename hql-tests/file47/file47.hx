QUERY createContinent (name: String) =>
    continent <- AddN<Continent>({name: name})
    RETURN continent

QUERY createCountry (continent_id: ID, name: String, currency: String, population: I64, gdp: F64) =>
    country <- AddN<Country>({name: name, currency: currency, population: population, gdp: gdp})
    continent <- N<Continent>(continent_id)
    continent_country <- AddE<Continent_to_Country>()::From(continent)::To(country)
    RETURN country

QUERY getContinentCities (continent_name: String, k: I64) =>
    continent <- N<Continent>::WHERE(_::{name}::EQ(continent_name))
    countries <- continent::Out<Continent_to_Country>
    cities <- countries::Out<Country_to_City>::RANGE(0, k)
    RETURN cities