//! JSON Schema generation for DAG-JSON encoded Ipld.

use libipld::Ipld;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use std::{borrow::Cow, collections::BTreeMap};

/// Ipld stub for JSON Schema generation
#[derive(Debug)]
#[doc(hidden)]
pub struct IpldStub(Ipld);

// The Ipld stub exists solely to implement a JSON Schema
// represenation of Ipld. Should libipld provide an implementation
// in the future, this can be removed.
impl JsonSchema for IpldStub {
    fn schema_name() -> String {
        "ipld".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-invocation::ipld::schema::IpldSchema")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject {
            instance_type: None,
            metadata: Some(Box::new(Metadata {
                title: Some("Ipld".to_string()),
                description: Some("DAG-JSON encoded IPLD: https://github.com/ipld/ipld/blob/master/specs/codecs/dag-json/spec.md".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let number_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Number.into())),
            ..Default::default()
        };
        let bytes_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([(
                    '/'.to_string(),
                    Schema::Object(SchemaObject {
                        instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
                        object: Some(Box::new(ObjectValidation {
                            properties: BTreeMap::from([(
                                "bytes".to_string(),
                                <String>::json_schema(gen),
                            )]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                )]),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                description: Some("Base64 encoded binary".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };
        let array_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Array.into())),
            ..Default::default()
        };
        let object_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            ..Default::default()
        };
        let link_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([('/'.to_string(), <String>::json_schema(gen))]),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                description: Some("CID link that points to some IPLD data".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        schema.subschemas().one_of = Some(vec![
            <()>::json_schema(gen),
            <bool>::json_schema(gen),
            Schema::Object(number_schema),
            <String>::json_schema(gen),
            Schema::Object(bytes_schema),
            Schema::Object(array_schema),
            Schema::Object(object_schema),
            Schema::Object(link_schema),
        ]);

        schema.into()
    }
}
