//! YAML spec frontend: serde structs and conversion into the IR.

use std::collections::HashSet;

use serde::Deserialize;
use thiserror::Error;

use crate::ir::{Api, Endpoint, FieldDef, HttpMethod, TypeDef, WireType};

/// Errors produced while parsing and validating a spec.
#[derive(Debug, Error)]
pub enum Error {
    /// The YAML could not be parsed.
    #[error("invalid YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// The spec is semantically invalid.
    #[error("invalid spec: {0}")]
    Spec(String),
}

/// The result type for spec parsing.
pub type Result<T> = std::result::Result<T, Error>;

/// Raw YAML shape of a spec.
#[derive(Debug, Deserialize)]
pub struct RawSpec {
    /// Service name.
    pub service: String,
    /// Base URL.
    pub base_url: String,
    /// Wire model types.
    #[serde(default)]
    pub types: Vec<RawType>,
    /// HTTP endpoints.
    #[serde(default)]
    pub endpoints: Vec<RawEndpoint>,
}

/// Raw YAML shape of a wire model type.
#[derive(Debug, Deserialize)]
pub struct RawType {
    /// Struct name.
    pub name: String,
    /// Fields.
    pub fields: Vec<RawField>,
}

/// Raw YAML shape of a field.
#[derive(Debug, Deserialize)]
pub struct RawField {
    /// Field name.
    pub name: String,
    /// Field type string.
    #[serde(rename = "type")]
    pub ty: String,
    /// Whether the field is optional.
    #[serde(default)]
    pub optional: bool,
}

/// Raw YAML shape of an endpoint.
#[derive(Debug, Deserialize)]
pub struct RawEndpoint {
    /// Method name.
    pub name: String,
    /// HTTP method.
    pub method: String,
    /// Path template.
    pub path: String,
    /// Path parameters.
    #[serde(default)]
    pub path_params: Vec<RawField>,
    /// Response type string.
    #[serde(default)]
    pub response: Option<String>,
}

/// Parses a YAML spec into the internal IR.
pub fn parse(yaml: &str) -> Result<Api> {
    let raw: RawSpec = serde_yaml::from_str(yaml)?;
    convert(raw)
}

fn convert(raw: RawSpec) -> Result<Api> {
    let service = raw.service.trim().to_string();
    if service.is_empty() {
        return Err(Error::Spec("`service` must not be empty".to_string()));
    }
    if raw.base_url.trim().is_empty() {
        return Err(Error::Spec("`base_url` must not be empty".to_string()));
    }

    let known_types: HashSet<String> = raw.types.iter().map(|t| t.name.clone()).collect();

    let mut types = Vec::with_capacity(raw.types.len());
    for t in &raw.types {
        validate_identifier(&t.name)?;
        let mut fields = Vec::with_capacity(t.fields.len());
        for f in &t.fields {
            validate_identifier(&f.name)?;
            fields.push(FieldDef {
                name: f.name.clone(),
                ty: parse_wire_type(&f.ty, &known_types)?,
                optional: f.optional,
            });
        }
        types.push(TypeDef {
            name: t.name.clone(),
            fields,
        });
    }

    let mut endpoints = Vec::with_capacity(raw.endpoints.len());
    for e in &raw.endpoints {
        validate_identifier(&e.name)?;
        let method = parse_method(&e.method)?;
        let mut path_params = Vec::with_capacity(e.path_params.len());
        for p in &e.path_params {
            validate_identifier(&p.name)?;
            if !e.path.contains(&format!("{{{}}}", p.name)) {
                return Err(Error::Spec(format!(
                    "path parameter `{}` of endpoint `{}` does not appear in path `{}`",
                    p.name, e.name, e.path
                )));
            }
            path_params.push(FieldDef {
                name: p.name.clone(),
                ty: parse_wire_type(&p.ty, &known_types)?,
                optional: p.optional,
            });
        }
        let response = match &e.response {
            Some(r) => Some(parse_wire_type(r, &known_types)?),
            None => None,
        };
        endpoints.push(Endpoint {
            name: e.name.clone(),
            method,
            path: e.path.clone(),
            path_params,
            response,
        });
    }

    Ok(Api {
        service,
        base_url: raw.base_url,
        types,
        endpoints,
    })
}

fn parse_method(method: &str) -> Result<HttpMethod> {
    match method.trim().to_ascii_uppercase().as_str() {
        "GET" => Ok(HttpMethod::Get),
        "POST" => Ok(HttpMethod::Post),
        "PUT" => Ok(HttpMethod::Put),
        "PATCH" => Ok(HttpMethod::Patch),
        "DELETE" => Ok(HttpMethod::Delete),
        "HEAD" => Ok(HttpMethod::Head),
        other => Err(Error::Spec(format!("unsupported HTTP method `{other}`"))),
    }
}

fn parse_wire_type(value: &str, known: &HashSet<String>) -> Result<WireType> {
    let value = value.trim();
    let simple = match value {
        "string" => Some(WireType::String),
        "bool" => Some(WireType::Bool),
        "i16" | "int16" => Some(WireType::I16),
        "i32" | "int32" => Some(WireType::I32),
        "i64" | "int64" => Some(WireType::I64),
        "u16" | "uint16" => Some(WireType::U16),
        "u32" | "uint32" => Some(WireType::U32),
        "u64" | "uint64" => Some(WireType::U64),
        "f32" => Some(WireType::F32),
        "f64" => Some(WireType::F64),
        "json" => Some(WireType::Json),
        _ => None,
    };
    if let Some(simple) = simple {
        return Ok(simple);
    }
    if let Some(inner) = value
        .strip_prefix("list<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return Ok(WireType::List(Box::new(parse_wire_type(inner, known)?)));
    }
    if known.contains(value) {
        return Ok(WireType::Model(value.to_string()));
    }
    Err(Error::Spec(format!("unknown type `{value}`")))
}

fn validate_identifier(value: &str) -> Result<()> {
    let mut chars = value.chars();
    let valid = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !valid {
        return Err(Error::Spec(format!(
            "`{value}` is not a valid snake_case identifier"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::ir::{HttpMethod, WireType};

    #[test]
    fn parses_minimal_spec() {
        let api = parse(
            r#"
service: example
base_url: https://example.invalid
types:
  - name: ActivityResponse
    fields:
      - name: id
        type: string
      - name: laps
        type: list<LapResponse>
      - name: metadata
        type: json
        optional: true
  - name: LapResponse
    fields:
      - name: seconds
        type: uint32
endpoints:
  - name: get_activity
    method: GET
    path: /activities/{id}
    path_params:
      - name: id
        type: string
    response: ActivityResponse
  - name: list_activities
    method: GET
    path: /activities
    response: list<ActivityResponse>
"#,
        )
        .expect("spec parses");

        assert_eq!(api.service, "example");
        assert_eq!(api.base_url, "https://example.invalid");
        assert_eq!(api.types.len(), 2);
        assert_eq!(api.endpoints.len(), 2);

        let activity = &api.types[0];
        assert_eq!(activity.name, "ActivityResponse");
        assert_eq!(activity.fields[0].ty, WireType::String);
        assert_eq!(
            activity.fields[1].ty,
            WireType::List(Box::new(WireType::Model("LapResponse".into())))
        );
        assert!(activity.fields[2].optional);

        let get = &api.endpoints[0];
        assert_eq!(get.method, HttpMethod::Get);
        assert_eq!(get.path, "/activities/{id}");
        assert_eq!(get.path_params[0].name, "id");
        assert_eq!(
            get.response,
            Some(WireType::Model("ActivityResponse".into()))
        );

        let list = &api.endpoints[1];
        assert_eq!(
            list.response,
            Some(WireType::List(Box::new(WireType::Model(
                "ActivityResponse".into()
            ))))
        );
    }

    #[test]
    fn rejects_unknown_type() {
        let error = parse(
            r#"
service: example
base_url: https://example.invalid
types:
  - name: A
    fields:
      - name: x
        type: bogus
endpoints: []
"#,
        )
        .expect_err("spec is invalid");
        assert!(error.to_string().contains("unknown type"));
    }

    #[test]
    fn rejects_undeclared_path_parameter() {
        let error = parse(
            r#"
service: example
base_url: https://example.invalid
types: []
endpoints:
  - name: get_thing
    method: GET
    path: /things/{id}
    path_params:
      - name: other
        type: string
"#,
        )
        .expect_err("spec is invalid");
        assert!(error.to_string().contains("does not appear in path"));
    }

    #[test]
    fn rejects_invalid_identifiers() {
        let error = parse(
            r#"
service: example
base_url: https://example.invalid
types:
  - name: Not Valid
    fields: []
endpoints: []
"#,
        )
        .expect_err("spec is invalid");
        assert!(error.to_string().contains("identifier"));
    }

    #[test]
    fn rejects_invalid_yaml() {
        let error = parse("service: [unclosed").expect_err("yaml is invalid");
        assert!(error.to_string().contains("invalid YAML"));
    }
}
