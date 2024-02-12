//! JSON Schema generation for DAG-JSON encoded Ipld.

use const_format::formatcp;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use std::{borrow::Cow, collections::BTreeMap, module_path};

/// Ipld stub for JSON Schema generation
#[derive(Debug)]
#[doc(hidden)]
pub struct IpldStub();

// The Ipld stub exists solely to implement a JSON Schema
// represenation of Ipld. Should libipld provide an implementation
// in the future, this can be removed.
impl JsonSchema for IpldStub {
    fn schema_name() -> String {
        "ipld".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(formatcp!("{}::IpldStub", module_path!()))
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
        let array_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Array.into())),
            ..Default::default()
        };
        let object_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            ..Default::default()
        };

        schema.subschemas().one_of = Some(vec![
            <()>::json_schema(gen),
            <bool>::json_schema(gen),
            Schema::Object(number_schema),
            <String>::json_schema(gen),
            gen.subschema_for::<IpldBytesStub>(),
            Schema::Object(array_schema),
            Schema::Object(object_schema),
            gen.subschema_for::<IpldLinkStub>(),
        ]);

        schema.into()
    }
}

/// Ipld link stub for JSON Schema generation
#[derive(Debug)]
#[doc(hidden)]
pub struct IpldLinkStub();

impl JsonSchema for IpldLinkStub {
    fn schema_name() -> String {
        "ipld_link".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(formatcp!("{}::IpldLinkStub", module_path!()))
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([('/'.to_string(), <String>::json_schema(gen))]),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                title: Some("IPLD link".to_string()),
                description: Some("CID link that points to some IPLD data".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        schema.into()
    }
}

/// Ipld bytes stub for JSON Schema generation
#[derive(Debug)]
#[doc(hidden)]
pub struct IpldBytesStub();

impl JsonSchema for IpldBytesStub {
    fn schema_name() -> String {
        "ipld_bytes".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(formatcp!("{}::IpldBytesStub", module_path!()))
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("IPLD bytes".to_string()),
                description: Some("Base64 encoded binary".to_string()),
                ..Default::default()
            })),
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
            ..Default::default()
        };

        schema.into()
    }
}
