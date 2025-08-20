QUERY create_preferences(preferences: [String]) =>
    FOR preference IN preferences {
        AddV<Preference>(Embed(preference), { preference: preference })
    }
    RETURN "Success"