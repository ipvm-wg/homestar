#![allow(dead_code)]

//! OpenRPC API document generator
//!
//! OpenRPC spec: https://github.com/open-rpc/spec
//! Module adapted from: https://github.com/austbot/rust-open-rpc-macros/tree/master/open-rpc-schema

use schemars::{gen::SchemaSettings, schema::RootSchema, JsonSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

extern crate serde_json;

#[derive(Serialize, Deserialize, Clone)]
pub enum Openrpc {
    #[serde(rename = "1.2.6")]
    V26,
    #[serde(rename = "1.2.5")]
    V25,
    #[serde(rename = "1.2.4")]
    V24,
    #[serde(rename = "1.2.3")]
    V23,
    #[serde(rename = "1.2.2")]
    V22,
    #[serde(rename = "1.2.1")]
    V21,
    #[serde(rename = "1.2.0")]
    V20,
    #[serde(rename = "1.1.12")]
    V112,
    #[serde(rename = "1.1.11")]
    V111,
    #[serde(rename = "1.1.10")]
    V110,
    #[serde(rename = "1.1.9")]
    V19,
    #[serde(rename = "1.1.8")]
    V18,
    #[serde(rename = "1.1.7")]
    V17,
    #[serde(rename = "1.1.6")]
    V16,
    #[serde(rename = "1.1.5")]
    V15,
    #[serde(rename = "1.1.4")]
    V14,
    #[serde(rename = "1.1.3")]
    V13,
    #[serde(rename = "1.1.2")]
    V12,
    #[serde(rename = "1.1.1")]
    V11,
    #[serde(rename = "1.1.0")]
    V10,
    #[serde(rename = "1.0.0")]
    V00,
    #[serde(rename = "1.0.0-rc1")]
    V00Rc1,
    #[serde(rename = "1.0.0-rc0")]
    V00Rc0,
}

pub type InfoObjectProperties = String;
pub type InfoObjectDescription = String;
pub type InfoObjectTermsOfService = String;
pub type InfoObjectVersion = String;
pub type ContactObjectName = String;
pub type ContactObjectEmail = String;
pub type ContactObjectUrl = String;
pub type SpecificationExtension = serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct ContactObject {
    pub name: Option<ContactObjectName>,
    pub email: Option<ContactObjectEmail>,
    pub url: Option<ContactObjectUrl>,
}

pub type LicenseObjectName = String;
pub type LicenseObjectUrl = String;

#[derive(Serialize, Deserialize, Clone)]
pub struct LicenseObject {
    pub name: Option<LicenseObjectName>,
    pub url: Option<LicenseObjectUrl>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InfoObject {
    pub title: InfoObjectProperties,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<InfoObjectDescription>,
    #[serde(rename = "termsOfService")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<InfoObjectTermsOfService>,
    pub version: InfoObjectVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<ContactObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<LicenseObject>,
}

pub type ExternalDocumentationObjectDescription = String;
pub type ExternalDocumentationObjectUrl = String;

/// ExternalDocumentationObject
///
/// information about external documentation
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ExternalDocumentationObject {
    pub description: Option<ExternalDocumentationObjectDescription>,
    pub url: ExternalDocumentationObjectUrl,
}

pub type ServerObjectUrl = String;
pub type ServerObjectName = String;
pub type ServerObjectDescription = String;
pub type ServerObjectSummary = String;
pub type ServerObjectVariableDefault = String;
pub type ServerObjectVariableDescription = String;
pub type ServerObjectVariableEnumItem = String;
pub type ServerObjectVariableEnum = Vec<ServerObjectVariableEnumItem>;

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerObjectVariable {
    pub default: ServerObjectVariableDefault,
    pub description: Option<ServerObjectVariableDescription>,
    #[serde(rename = "enum")]
    pub variable_enum: Option<ServerObjectVariableEnum>,
}

pub type ServerObjectVariables = HashMap<String, Option<serde_json::Value>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerObject {
    pub url: ServerObjectUrl,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ServerObjectName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<ServerObjectDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ServerObjectSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<ServerObjectVariables>,
}

pub type Servers = Vec<ServerObject>;
/// MethodObjectName
///
/// The cannonical name for the method. The name MUST be unique within the methods array.
///
pub type MethodObjectName = String;
/// MethodObjectDescription
///
/// A verbose explanation of the method behavior. GitHub Flavored Markdown syntax MAY be used for rich text representation.
///
pub type MethodObjectDescription = String;
/// MethodObjectSummary
///
/// A short summary of what the method does.
///
pub type MethodObjectSummary = String;
pub type TagObjectName = String;
pub type TagObjectDescription = String;

#[derive(Serialize, Deserialize, Clone)]
pub struct TagObject {
    pub name: TagObjectName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<TagObjectDescription>,
    #[serde(rename = "externalDocs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocumentationObject>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReferenceObject {
    #[serde(rename = "$ref")]
    pub reference: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum TagOrReference {
    TagObject(TagObject),
    ReferenceObject(ReferenceObject),
}

pub type MethodObjectTags = Vec<TagOrReference>;

/// MethodObjectParamStructure
///
/// Format the server expects the params. Defaults to 'either'.
///
/// # Default
///
/// either
///
#[derive(Serialize, Deserialize, Clone)]
pub enum MethodObjectParamStructure {
    #[serde(rename = "by-position")]
    ByPosition,
    #[serde(rename = "by-name")]
    ByName,
    #[serde(rename = "either")]
    Either,
}

pub type ContentDescriptorObjectName = String;
pub type ContentDescriptorObjectDescription = String;
pub type ContentDescriptorObjectSummary = String;
pub type Id = String;
pub type Schema = String;
pub type Comment = String;
pub type Title = String;
pub type Description = String;
type AlwaysTrue = serde_json::Value;
pub type ReadOnly = bool;
pub type Examples = Vec<AlwaysTrue>;
pub type MultipleOf = f64;
pub type Maximum = f64;
pub type ExclusiveMaximum = f64;
pub type Minimum = f64;
pub type ExclusiveMinimum = f64;
pub type NonNegativeInteger = i64;
pub type NonNegativeIntegerDefaultZero = i64;
pub type Pattern = String;
pub type SchemaArray = Vec<JSONSchema>;

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Items {
    JSONSchema(JSONSchema),
    SchemaArray(SchemaArray),
}

pub type UniqueItems = bool;
pub type StringDoaGddGA = String;
/// StringArray
///
/// # Default
///
/// []
///
pub type StringArray = Vec<StringDoaGddGA>;
/// Definitions
///
/// # Default
///
/// {}
///
pub type Definitions = HashMap<String, Option<serde_json::Value>>;
/// Properties
///
/// # Default
///
/// {}
///
pub type Properties = HashMap<String, Option<serde_json::Value>>;
/// PatternProperties
///
/// # Default
///
/// {}
///
pub type PatternProperties = HashMap<String, Option<serde_json::Value>>;

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum DependenciesSet {
    JSONSchema(JSONSchema),
    StringArray(StringArray),
}

pub type Dependencies = HashMap<String, Option<serde_json::Value>>;
pub type Enum = Vec<AlwaysTrue>;
pub type SimpleTypes = serde_json::Value;
pub type ArrayOfSimpleTypes = Vec<SimpleTypes>;

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Type {
    SimpleTypes(SimpleTypes),
    ArrayOfSimpleTypes(ArrayOfSimpleTypes),
}

pub type Format = String;
pub type ContentMediaType = String;
pub type ContentEncoding = String;

/// JSONSchemaBoolean
///
/// Always valid if true. Never valid if false. Is constant.
///
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum JSONSchema {
    JsonSchemaObject(RootSchema),
    JSONSchemaBoolean(bool),
}

pub type ContentDescriptorObjectRequired = bool;
pub type ContentDescriptorObjectDeprecated = bool;

#[derive(Serialize, Deserialize, Clone)]
pub struct ContentDescriptorObject {
    pub name: ContentDescriptorObjectName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<ContentDescriptorObjectDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ContentDescriptorObjectSummary>,
    pub schema: JSONSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<ContentDescriptorObjectRequired>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<ContentDescriptorObjectDeprecated>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ContentDescriptorOrReference {
    ContentDescriptorObject(ContentDescriptorObject),
    ReferenceObject(ReferenceObject),
}

pub type MethodObjectParams = Vec<ContentDescriptorOrReference>;

/// ErrorObjectCode
///
/// A Number that indicates the error type that occurred. This MUST be an integer. The error codes from and including -32768 to -32000 are reserved for pre-defined errors. These pre-defined errors SHOULD be assumed to be returned from any JSON-RPC api.
///
pub type ErrorObjectCode = i64;
/// ErrorObjectMessage
///
/// A String providing a short description of the error. The message SHOULD be limited to a concise single sentence.
///
pub type ErrorObjectMessage = String;
/// ErrorObjectData
///
/// A Primitive or Structured value that contains additional information about the error. This may be omitted. The value of this member is defined by the Server (e.g. detailed error information, nested errors etc.).
///
pub type ErrorObjectData = serde_json::Value;

/// ErrorObject
///
/// Defines an application level error.
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ErrorObject {
    pub code: ErrorObjectCode,
    pub message: ErrorObjectMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ErrorObjectData>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ErrorOrReference {
    ErrorObject(ErrorObject),
    ReferenceObject(ReferenceObject),
}

/// MethodObjectErrors
///
/// Defines an application level error.
///
pub type MethodObjectErrors = Vec<ErrorOrReference>;
pub type LinkObjectName = String;
pub type LinkObjectSummary = String;
pub type LinkObjectMethod = String;
pub type LinkObjectDescription = String;
pub type LinkObjectParams = serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct LinkObjectServer {
    pub url: ServerObjectUrl,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ServerObjectName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<ServerObjectDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ServerObjectSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<ServerObjectVariables>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LinkObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<LinkObjectName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<LinkObjectSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<LinkObjectMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LinkObjectDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<LinkObjectParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<LinkObjectServer>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LinkOrReference {
    LinkObject(LinkObject),
    ReferenceObject(ReferenceObject),
}

pub type MethodObjectLinks = Vec<LinkOrReference>;
pub type ExamplePairingObjectName = String;
pub type ExamplePairingObjectDescription = String;
pub type ExampleObjectSummary = String;
pub type ExampleObjectValue = serde_json::Value;
pub type ExampleObjectDescription = String;
pub type ExampleObjectName = String;

#[derive(Serialize, Deserialize, Clone)]
pub struct ExampleObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ExampleObjectSummary>,
    pub value: ExampleObjectValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<ExampleObjectDescription>,
    pub name: ExampleObjectName,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ExampleOrReference {
    ExampleObject(ExampleObject),
    ReferenceObject(ReferenceObject),
}

pub type ExamplePairingObjectParams = Vec<ExampleOrReference>;

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ExamplePairingObjectResult {
    ExampleObject(ExampleObject),
    ReferenceObject(ReferenceObject),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExamplePairingObject {
    pub name: ExamplePairingObjectName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<ExamplePairingObjectDescription>,
    pub params: ExamplePairingObjectParams,
    pub result: ExamplePairingObjectResult,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ExamplePairingOrReference {
    ExampleObject(ExampleObject),
    ReferenceObject(ReferenceObject),
}

pub type MethodObjectExamples = Vec<ExamplePairingOrReference>;
pub type MethodObjectDeprecated = bool;

#[derive(Serialize, Deserialize, Clone)]
pub struct MethodObject {
    pub name: MethodObjectName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<MethodObjectDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<MethodObjectSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Servers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<MethodObjectTags>,
    #[serde(rename = "paramStructure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_structure: Option<MethodObjectParamStructure>,
    pub params: MethodObjectParams,
    pub result: ContentDescriptorOrReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<MethodObjectErrors>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<MethodObjectLinks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<MethodObjectExamples>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<MethodObjectDeprecated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentationObject>,
}

pub type Methods = Vec<MethodObject>;
pub type SchemaComponents = HashMap<String, Option<serde_json::Value>>;
pub type LinkComponents = HashMap<String, Option<serde_json::Value>>;
pub type ErrorComponents = HashMap<String, Option<serde_json::Value>>;
pub type ExampleComponents = HashMap<String, Option<serde_json::Value>>;
pub type ExamplePairingComponents = HashMap<String, Option<serde_json::Value>>;
pub type ContentDescriptorComponents = HashMap<String, Option<serde_json::Value>>;
pub type TagComponents = HashMap<String, Option<serde_json::Value>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Components {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<SchemaComponents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<LinkComponents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<ErrorComponents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<ExampleComponents>,
    #[serde(rename = "examplePairings")]
    pub example_pairings: Option<ExamplePairingComponents>,
    #[serde(rename = "contentDescriptors")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_descriptors: Option<ContentDescriptorComponents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<TagComponents>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenrpcDocument {
    pub openrpc: Openrpc,
    pub info: InfoObject,
    #[serde(rename = "externalDocs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocumentationObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Servers>,
    pub methods: Methods,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
}

impl Default for OpenrpcDocument {
    fn default() -> Self {
        OpenrpcDocument {
            openrpc: Openrpc::V26,
            info: InfoObject {
                title: "".to_string(),
                description: None,
                terms_of_service: None,
                version: "".to_string(),
                contact: None,
                license: None,
            },
            external_docs: None,
            servers: None,
            methods: vec![],
            components: None,
        }
    }
}

impl OpenrpcDocument {
    pub fn set_info(mut self, info: InfoObject) -> Self {
        self.info = info;
        self
    }
    pub fn add_object_method(&mut self, method: MethodObject) {
        self.methods.push(method)
    }
}

impl ContentDescriptorOrReference {
    pub fn new_content_descriptor<T: ?Sized + JsonSchema>(
        name: ContactObjectName,
        description: Option<Description>,
    ) -> Self {
        let mut setting = SchemaSettings::draft07();
        setting.inline_subschemas = true;
        let schema = schemars::gen::SchemaGenerator::new(setting).into_root_schema_for::<T>();
        let json_schema = JSONSchema::JsonSchemaObject(schema);
        ContentDescriptorOrReference::ContentDescriptorObject(ContentDescriptorObject {
            name,
            description,
            summary: None,
            schema: json_schema,
            required: None,
            deprecated: None,
        })
    }
}

impl MethodObject {
    pub fn new(name: MethodObjectName, description: Option<Description>) -> Self {
        Self {
            name,
            description,
            summary: None,
            servers: None,
            tags: None,
            param_structure: None,
            params: vec![],
            result: ContentDescriptorOrReference::ReferenceObject(ReferenceObject {
                reference: "".to_string(),
            }),
            errors: None,
            links: None,
            examples: None,
            deprecated: None,
            external_docs: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(JsonSchema)]
    pub struct MyType([u8; 8]);

    #[derive(JsonSchema)]
    pub struct MyParam {
        pub my_int: i32,
        pub my_bool: bool,
        pub my_type: Box<MyType>,
    }

    #[derive(JsonSchema)]
    pub struct MyRet {
        pub success: Box<bool>,
    }

    #[test]
    fn test_openrpc_document() {
        let mut document = OpenrpcDocument::default();
        let mut method = MethodObject::new("method1".to_string(), None);
        let param = ContentDescriptorOrReference::new_content_descriptor::<MyParam>(
            "first_param".to_string(),
            Some("no desc".to_string()),
        );
        method.params.push(param);
        method.result =
            ContentDescriptorOrReference::new_content_descriptor::<MyRet>("ret".to_string(), None);
        document.add_object_method(method);
        let j = serde_json::to_string_pretty(&document).unwrap();
        println!("{}", j);
    }
}
