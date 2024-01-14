use schemars::{
    gen::SchemaGenerator,
    schema::{Schema, SchemaObject},
    JsonSchema,
};
use serde_json::json;

pub(crate) trait TaggedSchema {
    fn tag() -> String;

    fn make_tag_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema: SchemaObject = <String>::json_schema(gen).into();
        schema.const_value = Some(json!(Self::tag()));
        schema.into()
    }
}
