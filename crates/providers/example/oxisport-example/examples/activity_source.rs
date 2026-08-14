//! Demonstrates the example provider as an `ActivitySource`.
//!
//! The mock service at the default base URL (`https://example.invalid`) is
//! unreachable, so this example fails at runtime unless it is pointed at a
//! mock server. Point it at a wiremock instance running on `127.0.0.1:8000`
//! with:
//!
//! ```text
//! BASE_URL=http://127.0.0.1:8000 cargo run --example activity_source
//! ```

use futures_util::TryStreamExt;
use oxisport_core::ActivitySource;
use oxisport_example::{ExampleConfig, ExampleProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| ExampleConfig::default().base_url);
    let provider = ExampleProvider::new(ExampleConfig {
        base_url,
        user_agent: Some("oxisport-example-demo".to_string()),
    })?;

    println!("provider: {}", provider.provider_id());

    let mut activities = provider.activities(&Default::default()).await?;
    while let Some(activity) = activities.try_next().await? {
        println!(
            "{}: {} {:?} ({:?})",
            activity.id, activity.sport, activity.distance, activity.duration
        );
    }
    Ok(())
}
