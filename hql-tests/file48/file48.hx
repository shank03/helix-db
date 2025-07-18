QUERY countCapitals () =>
    num_capital <- N<City>::WHERE(EXISTS(_::In<Country_to_Capital>))::COUNT
    RETURN num_capital