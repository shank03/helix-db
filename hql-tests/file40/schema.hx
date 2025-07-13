N::Professor {
    name: String,
    title: String,
    page: String,
    bio: String,
}

V::ResearchAreaAndDescriptionEmbedding {
    areas_and_descriptions: String,
}

E::HasResearchAreaAndDescriptionEmbedding {
    From: Professor,
    To: ResearchAreaAndDescriptionEmbedding,
    Properties: {
        areas_and_descriptions: String,
    }
}

E::HasUniversity {
    From: Professor,
    To: University,
}

E::HasDepartment {
    From: Professor,
    To: Department,
}

N::University {
    name: String,
}

N::Department {
    name: String,
}