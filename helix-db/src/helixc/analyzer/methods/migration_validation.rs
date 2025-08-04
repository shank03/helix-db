use crate::{
    generate_error,
    helixc::{
        analyzer::analyzer::Ctx,
        parser::helix_parser::{
            FieldValue, FieldValueType, Migration, MigrationItem, MigrationItemMapping,
            MigrationPropertyMapping,
        },
    },
};

use paste::paste;

pub(crate) fn validate_migration(ctx: &mut Ctx, migration: &Migration) {
    // check from version exists
    if !ctx
        .all_schemas
        .inner()
        .contains_key(&migration.from_version.1)
    {
        // schema error - version does not exist
    }
    // check to version exists and is 1 greater than from version
    if !ctx
        .all_schemas
        .inner()
        .contains_key(&migration.to_version.1)
    {
        // schema error - version does not exist
    }

    let (from_node_fields, from_edge_fields, from_vector_fields) = ctx
        .all_schemas
        .inner()
        .get(&migration.from_version.1)
        .unwrap();
    let (to_node_fields, to_edge_fields, to_vector_fields) = ctx
        .all_schemas
        .inner()
        .get(&migration.to_version.1)
        .unwrap();

    // for each migration item mapping
    for item in &migration.body {
        // // get from fields and to fields and check they exist in respective versions

        let from_fields = match match &item.from_item {
            (_, MigrationItem::Node(node)) => from_node_fields.get(node.as_str()),
            (_, MigrationItem::Edge(edge)) => from_edge_fields.get(edge.as_str()),
            (_, MigrationItem::Vector(vector)) => from_vector_fields.get(vector.as_str()),
        } {
            Some(fields) => fields,
            None => {
                // schema error - item does not exist
                continue;
            }
        };

        let to_fields = match match &item.to_item {
            (_, MigrationItem::Node(node)) => to_node_fields.get(node.as_str()),
            (_, MigrationItem::Edge(edge)) => to_edge_fields.get(edge.as_str()),
            (_, MigrationItem::Vector(vector)) => to_vector_fields.get(vector.as_str()),
        } {
            Some(fields) => fields,
            None => {
                // schema error - item does not exist
                continue;
            }
        };

        // for now assert that from and to fields are the same type
        // TODO: add support for migrating actual item types
        if item.from_item.1 != item.to_item.1 {
            // schema error - item types do not match
        }

        for MigrationPropertyMapping {
            property_name,
            property_value,
            default,
            cast,
            loc,
        } in &item.remappings
        {
            // check the new property exists in to version schema
            let to_property_field = match to_fields.get(property_name.1.as_str()) {
                Some(field) => field,
                None => {
                    // schema error - property does not exist in to version
                    continue;
                }
            };

            // check the property value is valid for the new field type

            match &property_value.value {
                // if property value is a literal, check it is valid for the new field type
                FieldValueType::Literal(literal) => {
                    if to_property_field.field_type != *literal {
                        // schema error - property value is not valid for the new field type
                        continue;
                    }
                }
                FieldValueType::Identifier(identifier) => {
                    // check the identifier is valid for the new field type
                    if from_fields.get(identifier.as_str()).is_none() {
                        // schema error - identifier does not exist in from version
                        continue;
                    }
                }
                _ => todo!(),
            }

            // check default value is valid for the new field type
            if let Some(default) = &default {
                if to_property_field.field_type != *default {
                    // schema error - default value is not valid for the new field type
                    continue;
                }
            }

            // check the cast is valid for the new field type
            if let Some(cast) = &cast {
                if to_property_field.field_type != cast.cast_to {
                    // schema error - cast is not valid for the new field type
                    continue;
                }
            }

          
          
            // // warnings if name is same
            // // warnings if numeric type cast is smaller than existing type
        }
    }
}
