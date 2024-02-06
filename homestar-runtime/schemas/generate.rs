//! Standalone binary to generate OpenRPC API docs and
//! JSON Schemas for method params and notifications.

use homestar_invocation::Receipt;
use homestar_runtime::{
    Health, NetworkNotification, NodeInfo, PrometheusData, ReceiptNotification,
};
use homestar_workflow::Workflow;
use schemars::{
    schema::{RootSchema, SchemaObject},
    schema_for,
};
use std::{fs, io::Write};

mod openrpc;
use openrpc::document::{
    ContactObject, ContentDescriptorObject, ContentDescriptorOrReference,
    ExternalDocumentationObject, InfoObject, JSONSchema, LicenseObject, MethodObject,
    MethodObjectParamStructure, Openrpc, OpenrpcDocument,
};

fn main() {
    let health_schema = schema_for!(Health);
    let _ = fs::File::create("schemas/docs/health.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&health_schema).unwrap());

    let metrics_schema = schema_for!(PrometheusData);
    let _ = fs::File::create("schemas/docs/metrics.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&metrics_schema).unwrap());

    let node_info_schema = schema_for!(NodeInfo);
    let _ = fs::File::create("schemas/docs/node_info.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&node_info_schema).unwrap());

    let network_schema = schema_for!(NetworkNotification);
    let _ = fs::File::create("schemas/docs/network.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&network_schema).unwrap());

    let workflow_schema = schema_for!(Workflow<'static, ()>);
    let _ = fs::File::create("schemas/docs/workflow.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&workflow_schema).unwrap());

    let receipt_schema = schema_for!(Receipt<()>);
    let _ = fs::File::create("schemas/docs/receipt.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&receipt_schema).unwrap());

    let receipt_notification_schema = schema_for!(ReceiptNotification);
    let _ = fs::File::create("schemas/docs/receipt_notification.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&receipt_notification_schema).unwrap());

    let api_doc = generate_api_doc(
        health_schema,
        metrics_schema,
        node_info_schema,
        network_schema,
        workflow_schema,
        receipt_notification_schema,
    );
    let _ = fs::File::create("schemas/docs/api.json")
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&api_doc).unwrap());
}

// Spec: https://github.com/open-rpc/spec/blob/1.2.6/spec.md
fn generate_api_doc(
    health_schema: RootSchema,
    metrics_schema: RootSchema,
    node_info_schema: RootSchema,
    network_schema: RootSchema,
    workflow_schema: RootSchema,
    receipt_notification_schema: RootSchema,
) -> OpenrpcDocument {
    let discover: MethodObject = MethodObject {
        name: "rpc.discover".to_string(),
        description: Some("OpenRPC schema as a description of this service".to_string()),
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::Either),
        params: vec![],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "OpenRPC Schema".to_string(),
            summary: None,
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(RootSchema {
                schema: SchemaObject {
                  reference: Some("https://github.com/ipvm-wg/homestar/blob/main/homestar-runtime/schemas/docs/api.json".to_string()),
                  ..Default::default()
                },
                ..Default::default()
            }),
            deprecated: Some(false),
        }),
        external_docs: None,
        errors: None,
        links: None,
        examples: None,
        deprecated: Some(false),
        x_messages: None,
    };

    let health: MethodObject = MethodObject {
        name: "health".to_string(),
        description: None,
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::ByName),
        params: vec![],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "health".to_string(),
            summary: None,
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(health_schema),
            deprecated: Some(false),
        }),
        external_docs: None,
        errors: None,
        links: None,
        examples: None,
        deprecated: Some(false),
        x_messages: None,
    };

    let metrics: MethodObject = MethodObject {
        name: "metrics".to_string(),
        description: None,
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::ByName),
        params: vec![],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "metrics".to_string(),
            summary: None,
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(metrics_schema),
            deprecated: Some(false),
        }),
        external_docs: None,
        errors: None,
        links: None,
        examples: None,
        deprecated: Some(false),
        x_messages: None,
    };

    let node_info: MethodObject = MethodObject {
        name: "node".to_string(),
        description: None,
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::ByName),
        params: vec![],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "node_info".to_string(),
            summary: None,
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(node_info_schema),
            deprecated: Some(false),
        }),
        external_docs: None,
        errors: None,
        links: None,
        examples: None,
        deprecated: Some(false),
        x_messages: None,
    };

    let network: MethodObject = MethodObject {
        name: "subscribe_network_events".to_string(),
        description: None,
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::ByName),
        params: vec![],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "subscription_id".to_string(),
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

    let network_unsubscribe: MethodObject = MethodObject {
        name: "unsubscribe_network_events".to_string(),
        description: None,
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::ByName),
        params: vec![],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "unsubscribe result".to_string(),
            summary: None,
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(schema_for!(bool)),
            deprecated: Some(false),
        }),
        external_docs: None,
        errors: None,
        links: None,
        examples: None,
        deprecated: Some(false),
        x_messages: None,
    };

    let workflow: MethodObject = MethodObject {
        name: "subscribe_run_workflow".to_string(),
        description: None,
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::ByName),
        params: vec![ContentDescriptorOrReference::ContentDescriptorObject(
            ContentDescriptorObject {
                name: "workflow".to_string(),
                summary: None,
                description: None,
                required: Some(true),
                schema: JSONSchema::JsonSchemaObject(workflow_schema),
                deprecated: Some(false),
            },
        )],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "subscription_id".to_string(),
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
            name: "workflow subscription messages".to_string(),
            summary: Some("receipt notifications from a running workflow".to_string()),
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(receipt_notification_schema),
            deprecated: Some(false),
        }),
    };

    let workflow_unsubscribe: MethodObject = MethodObject {
        name: "unsubscribe_run_workflow".to_string(),
        description: None,
        summary: None,
        servers: None,
        tags: None,
        param_structure: Some(MethodObjectParamStructure::ByName),
        params: vec![],
        result: ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name: "unsubscribe result".to_string(),
            summary: None,
            description: None,
            required: Some(true),
            schema: JSONSchema::JsonSchemaObject(schema_for!(bool)),
            deprecated: Some(false),
        }),
        external_docs: None,
        errors: None,
        links: None,
        examples: None,
        deprecated: Some(false),
        x_messages: None,
    };

    OpenrpcDocument {
        openrpc: Openrpc::V26,
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
        methods: vec![
            discover,
            health,
            metrics,
            node_info,
            network,
            network_unsubscribe,
            workflow,
            workflow_unsubscribe,
        ],
        components: None,
    }
}
