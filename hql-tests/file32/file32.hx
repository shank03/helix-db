// ─── node helpers ─────────────────────────────────────────────

QUERY HasCompany(company_number: String) =>
    company <- N<Company>({company_number: company_number})
    RETURN company

QUERY AddCompany(company_number: String, number_of_filings: I32) =>
    company <- AddN<Company>({
        company_number: company_number, 
        number_of_filings: number_of_filings
    })
    RETURN company

QUERY HasDocumentEmbeddings(company_number: String) =>
    c <- N<Company>({company_number: company_number})
    embeddings <- c::Out<DocumentEdge>
    RETURN embeddings

QUERY GetCompanies() =>
    companies <- N<Company>
    RETURN companies

QUERY DeleteCompany(company_number: String) =>
    DROP N<Company>({company_number: company_number})::Out<DocumentEdge>
    DROP N<Company>({company_number: company_number})
    RETURN "success"

// ─── filing / embedding helpers ───────────────────────────────

QUERY AddEmbeddingsToCompany(
    company_number: String, 
    embeddings_data: [{
        vector: [F64],
        text: String,
        chunk_id: String,
        page_number: I32,
        reference: String,
        filing_id: String,
        category: String,
        subcategory: String,
        date1: String,
        date2: String,
        source: String,
        description: String
    }]
) =>
    c <- N<Company>({company_number: company_number})
    FOR { vector, text, chunk_id, page_number, reference, filing_id, category, subcategory, date1, date2, source, description } IN embeddings_data {
        embedding <- AddV<DocumentEmbedding>(
            vector, {
                text: text,
                chunk_id: chunk_id,
                page_number: page_number,
                reference: reference,
                source_link: source,
                source_date: date1
        })

            edges <- AddE<DocumentEdge>({
                filing_id: filing_id,
                category: category,
                subcategory: subcategory,
                date: date2,
                description: description
            })::From(c)::To(embedding)
    }
    RETURN "success"



//  tmp function for testing helix
QUERY AddVector(vector: [F64], text: String, chunk_id: String, page_number: I32, reference: String) =>
    embedding <- AddV<DocumentEmbedding>(vector, {text: text, chunk_id: chunk_id, page_number: page_number, reference: reference})
    RETURN embedding

//  tmp function for testing helix
QUERY SearchVector(query: [F64], k: I32) =>
    embedding_search <- SearchV<DocumentEmbedding>(query, k)
    RETURN embedding_search


QUERY GetAllCompanyEmbeddings(company_number: String) =>
    // get company node
    c <- N<Company>({company_number: company_number})
    // get all embeddings
    embeddings <- c::Out<DocumentEdge>

    // return vector data
    RETURN embeddings

QUERY CompanyEmbeddingSearch(company_number: String, query: [F64], k: I32) =>
    c <- N<Company>({company_number: company_number})::OutE<DocumentEdge>::ToV
    embedding_search <- c::SearchV<DocumentEmbedding>(query, k)
    RETURN embedding_search

QUERY DeleteAll() =>
    DROP N<Company>
    RETURN "success"