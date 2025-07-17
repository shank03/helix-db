N::Company {
    INDEX company_number: String
}

QUERY AddCompany (company_number: String) =>
    c <- AddN<Company> ({company_number: company_number})
    RETURN c

QUERY HasCompany (company_number: String) =>
    c <- N<Company> ({company_number: company_number})
    RETURN c