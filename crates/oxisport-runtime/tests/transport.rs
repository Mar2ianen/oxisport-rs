//! Integration tests for the shared transport using wiremock.

use std::time::Duration;

use bytes::Bytes;
use futures_util::TryStreamExt;
use oxisport_runtime::{Client, ContentLength, Error, MediaType, MultipartForm, UploadBody};
use serde_json::{Value, json};
use wiremock::matchers::{body_bytes, body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client() -> Client {
    Client::builder()
        .user_agent("oxisport-runtime-test")
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn get_returns_json_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .mount(&server)
        .await;

    let url = format!("{}/ping", server.uri()).parse().unwrap();
    let response = client().get(url).send().await.expect("request succeeds");

    assert_eq!(response.status(), 200);
    let value: Value = response.json().await.expect("body parses as json");
    assert_eq!(value, json!({ "ok": true }));
}

#[tokio::test]
async fn sends_bearer_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/private"))
        .and(header("authorization", "Bearer secret-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let url = format!("{}/private", server.uri()).parse().unwrap();
    client()
        .get(url)
        .bearer_token("secret-token")
        .send()
        .await
        .expect("request succeeds");
}

#[tokio::test]
async fn not_found_is_classified() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&server)
        .await;

    let url = format!("{}/missing", server.uri()).parse().unwrap();
    let error = client().get(url).send().await.expect_err("fails");

    assert!(matches!(error, Error::NotFound(body) if body == "nope"));
}

#[tokio::test]
async fn rate_limited_keeps_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/busy"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "17")
                .set_body_string("slow down"),
        )
        .mount(&server)
        .await;

    let url = format!("{}/busy", server.uri()).parse().unwrap();
    let error = client().get(url).send().await.expect_err("fails");

    match error {
        Error::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(Duration::from_secs(17)));
        }
        other => panic!("expected RateLimited, got {other:?}"),
    }
}

#[tokio::test]
async fn server_error_is_preserved_as_remote() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/boom"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal"))
        .mount(&server)
        .await;

    let url = format!("{}/boom", server.uri()).parse().unwrap();
    let error = client().get(url).send().await.expect_err("fails");

    match error {
        Error::Remote(remote) => {
            assert_eq!(remote.status, Some(500));
            assert_eq!(remote.body.as_deref(), Some("internal"));
        }
        other => panic!("expected Remote, got {other:?}"),
    }
}

#[tokio::test]
async fn download_streams_body_and_metadata() {
    let payload = b"0123456789abcdef".repeat(128);
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/file"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/octet-stream")
                .set_body_bytes(payload.clone()),
        )
        .mount(&server)
        .await;

    let url = format!("{}/file", server.uri()).parse().unwrap();
    let response = client().get(url).send().await.expect("request succeeds");
    let download = response.download();

    assert_eq!(
        download.content_length(),
        ContentLength::new(payload.len() as u64)
    );
    assert_eq!(
        download.media_type(),
        Some(&MediaType::new("application/octet-stream"))
    );

    let collected: Vec<Bytes> = download
        .into_stream()
        .try_collect()
        .await
        .expect("stream succeeds");
    assert_eq!(collected.concat().as_slice(), payload.as_slice());
}

#[tokio::test]
async fn downloads_file_from_disk() {
    let content = b"hello from disk".to_vec();
    let file_path = temp_file("upload-file", &content).await;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/upload"))
        .and(body_bytes(content))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    let url = format!("{}/upload", server.uri()).parse().unwrap();
    client()
        .post(url)
        .body(UploadBody::from_file(&file_path))
        .send()
        .await
        .expect("upload succeeds");

    let _ = tokio::fs::remove_file(&file_path).await;
}

#[tokio::test]
async fn uploads_byte_stream_chunked() {
    let chunks = vec![Bytes::from_static(b"part1-"), Bytes::from_static(b"part2")];
    let stream = futures_util::stream::iter(chunks.into_iter().map(Ok::<_, std::io::Error>));

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/stream"))
        .and(body_bytes(b"part1-part2"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    let url = format!("{}/stream", server.uri()).parse().unwrap();
    client()
        .post(url)
        .body(UploadBody::from_stream(stream))
        .send()
        .await
        .expect("upload succeeds");
}

#[tokio::test]
async fn uploads_multipart_form() {
    let file = temp_file("multipart", b"fit-bytes").await;
    let form = MultipartForm::new()
        .text("name", "Morning ride")
        .file("file", &file)
        .await
        .expect("file opens");

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/upload"))
        .and(body_string_contains("fit-bytes"))
        .and(body_string_contains("name"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    let url = format!("{}/upload", server.uri()).parse().unwrap();
    let response = client()
        .post(url)
        .multipart(form)
        .send()
        .await
        .expect("multipart upload succeeds");
    assert_eq!(response.status(), 201);

    let received = server.received_requests().await.expect("request received");
    let content_type = received[0]
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .expect("content-type header present");
    assert!(
        content_type.starts_with("multipart/form-data; boundary="),
        "unexpected content type: {content_type}"
    );

    let _ = tokio::fs::remove_file(file).await;
}

async fn temp_file(name: &str, content: &[u8]) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "oxisport-runtime-test-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    tokio::fs::write(&path, content)
        .await
        .expect("writes temp file");
    path
}
