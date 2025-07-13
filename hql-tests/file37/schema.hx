
N::Entity {
    INDEX entity_name: String
}

E::Relationship {
    From: Entity,
    To: Entity,
    Properties: {
        edge_name: String
    }
}