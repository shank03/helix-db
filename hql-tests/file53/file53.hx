QUERY getEpisodeEdgesbyGroup (group_id: String) =>
    episode_edges <- E<Episode_Entity>::WHERE(_::{group_id}::EQ(group_id))
    RETURN episode_edges


QUERY getEpisodeEdgesbyGroupLimit (group_id: String, limit: I64) =>
    episode_edges <- E<Episode_Entity>::WHERE(_::{group_id}::EQ(group_id))::RANGE(0, limit)
    RETURN episode_edges