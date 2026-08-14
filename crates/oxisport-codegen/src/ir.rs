//! Internal IR between spec parsers and Rust generation.

/// A parsed API specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Api {
    /// Service name, used for the generated client name.
    pub service: String,
    /// Base URL for the service.
    pub base_url: String,
    /// Wire model types.
    pub types: Vec<TypeDef>,
    /// HTTP endpoints.
    pub endpoints: Vec<Endpoint>,
}

/// A wire model type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDef {
    /// Struct name.
    pub name: String,
    /// Struct fields.
    pub fields: Vec<FieldDef>,
}

/// A struct field or path parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDef {
    /// Field name.
    pub name: String,
    /// Field type.
    pub ty: WireType,
    /// Whether the field is optional.
    pub optional: bool,
}

/// An HTTP endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// Method name.
    pub name: String,
    /// HTTP method.
    pub method: HttpMethod,
    /// Path template with `{param}` placeholders.
    pub path: String,
    /// Path parameters.
    pub path_params: Vec<FieldDef>,
    /// Query parameters.
    pub query_params: Vec<FieldDef>,
    /// Response type, when the endpoint returns a body.
    pub response: Option<WireType>,
}

/// An HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET.
    Get,
    /// POST.
    Post,
    /// PUT.
    Put,
    /// PATCH.
    Patch,
    /// DELETE.
    Delete,
    /// HEAD.
    Head,
}

impl HttpMethod {
    /// Returns the HTTP method name.
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
        }
    }
}

/// A wire type reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireType {
    /// `string`
    String,
    /// `bool`
    Bool,
    /// `i16`
    I16,
    /// `i32`
    I32,
    /// `i64`
    I64,
    /// `u16`
    U16,
    /// `u32`
    U32,
    /// `u64`
    U64,
    /// `f32`
    F32,
    /// `f64`
    F64,
    /// `json` -> `serde_json::Value`
    Json,
    /// A list of another type.
    List(Box<WireType>),
    /// A reference to a model defined in the spec.
    Model(String),
}
