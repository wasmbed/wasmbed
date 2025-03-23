use crate::DeviceId;
use serde_json::json;
use schemars::{JsonSchema, SchemaGenerator};
use schemars::schema::{InstanceType, Metadata, Schema, SchemaObject, StringValidation};

const UUID_PATTERN: &str =
    r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

impl JsonSchema for DeviceId {
    fn schema_name() -> String {
        "DeviceId".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A unique device identifier represented as a UUID".to_string()
                ),
                examples: vec![
                    json!("123e4567-e89b-12d3-a456-426614174000"),
                    json!("00000000-0000-0000-0000-000000000000"),
                ],
                ..Default::default()
            })),
            instance_type: Some(InstanceType::String.into()),
            format: Some("uuid".to_string()),
            string: Some(Box::new(StringValidation {
                pattern: Some(UUID_PATTERN.to_string()),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
