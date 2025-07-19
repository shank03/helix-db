QUERY createContinent (name: String) =>
    continent <- AddN<Continent>({name: name})
    RETURN continent

QUERY createCountry (continent_id: ID, name: String, currency: String, population: I64, gdp: F64) =>
    country <- AddN<Country>({name: name, currency: currency, population: population, gdp: gdp})
    continent <- N<Continent>(continent_id)
    continent_country <- AddE<Continent_to_Country>()::From(continent)::To(country)
    RETURN country

QUERY setCapital (country_id: ID, city_id: ID) =>
    country <- N<Country>(country_id)
    city <- N<City>(city_id)
    country_capital <- AddE<Country_to_Capital>()::From(country)::To(city)
    RETURN country_capital

QUERY getCountriesWithCapitals () =>
    countries <- N<Country>::WHERE(EXISTS(_::Out<Country_to_Capital>))
    RETURN countries