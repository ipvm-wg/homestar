//! Standalone binary to generate OpenRPC API docs and
//! JSON Schemas for method params and notifications.

use homestar_runtime::{Health, NetworkNotification};
use schemars::{schema::RootSchema, schema_for};
use std::{fs, io::Write};

mod openrpc;
use openrpc::document::{
    ContactObject, ContentDescriptorObject, ContentDescriptorOrReference,
    ExternalDocumentationObject, InfoObject, JSONSchema, LicenseObject, MethodObject,
    MethodObjectParamStructure, Openrpc, OpenrpcDocument,
};

// Generate docs with `cargo run --bin schemas`
fn main() {
    let health_schema = schema_for!(Health);
    let _ = fs::File::create("schemas/docs/health.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&health_schema).unwrap());

    let network_schema = schema_for!(NetworkNotification);
    let _ = fs::File::create("schemas/docs/network.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&network_schema).unwrap());

    let api_doc = generate_api_doc(network_schema);
    let _ = fs::File::create("schemas/docs/api.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&api_doc).unwrap());
}

// Spec: https://github.com/open-rpc/spec/blob/1.2.6/spec.md
fn generate_api_doc(network_schema: RootSchema) -> OpenrpcDocument {
    let network: MethodObject = MethodObject {
        name: "network".to_string(),
        description: None,
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::ByName),
        params: vec![],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "network".to_string(),
            summary: None,
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(schema_for!(String)),
            deprecated: Some(false),
        }),
        external_docs: None,
        errors: None,
        links: None,
        examples: None,
        deprecated: Some(false),
        x_messages: Some(ContentDescriptorObject {
            name: "network subscription messages".to_string(),
            summary: None,
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(network_schema),
            deprecated: Some(false),
        }),
    };

    OpenrpcDocument {
        openrpc: Openrpc::V26, // TODO Should we upgrade to latest spec at 1.3.2?
        info: InfoObject {
            title: "homestar".to_string(),
            description: Some(env!("CARGO_PKG_DESCRIPTION").into()),
            terms_of_service: None,
            version: "0.10.0".to_string(),
            contact: Some(ContactObject {
                name: None,
                url: Some(env!("CARGO_PKG_REPOSITORY").into()),
                email: None,
            }),
            license: Some(LicenseObject {
                name: Some(env!("CARGO_PKG_LICENSE").into()),
                url: None,
            }),
        },
        external_docs: Some(ExternalDocumentationObject {
            description: None,
            url: "https://docs.everywhere.computer/homestar/what-is-homestar/".to_string(),
        }),
        servers: None,
        methods: vec![network],
        components: None,
    }
}
