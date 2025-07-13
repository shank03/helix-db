QUERY get_professor_research_areas_with_descriptions_v1(professor_id: ID) =>
    research_areas <- N<Professor>(professor_id)::Out<HasResearchAreaAndDescriptionEmbedding>
    RETURN research_areas::{areas_and_descriptions}


QUERY get_professor_research_areas_with_descriptions_v2(professor_id: ID) =>
    research_areas <- N<Professor>(professor_id)::Out<HasResearchAreaAndDescriptionEmbedding>::{areas_and_descriptions}
    RETURN research_areas::{areas_and_descriptions: areas_and_descriptions}

QUERY get_professor_research_areas_with_descriptions(professor_id: ID) =>
    research_areas <- N<Professor>(professor_id)::Out<HasResearchAreaAndDescriptionEmbedding>::{areas_and_descriptions}
    RETURN research_areas

QUERY get_professors_by_university_and_department_name(university_name: String, department_name: String) =>
    professors <- N<Professor>::WHERE(AND(
        EXISTS(_::Out<HasUniversity>::WHERE(_::{name}::EQ(university_name))),
        EXISTS(_::Out<HasDepartment>::WHERE(_::{name}::EQ(department_name)))
    ))
    RETURN professors