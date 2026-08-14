//! Development tasks for OxiSport.
//!
//! Run through the workspace alias:
//!
//! ```text
//! cargo xtask codegen
//! cargo xtask codegen example
//! cargo xtask check-generated
//! ```

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

/// (spec path, generated output path) pairs relative to the workspace root.
const TARGETS: &[(&str, &str)] = &[(
    "specs/examples/example.yaml",
    "crates/providers/example/oxisport-example-raw/src/generated.rs",
)];

#[derive(Parser)]
#[command(name = "xtask", about = "OxiSport development tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerates raw clients from specs.
    Codegen {
        /// Optional target name; when given, only matching specs are generated.
        target: Option<String>,
    },
    /// Verifies that committed generated sources are up to date.
    CheckGenerated,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Codegen { target } => codegen(target.as_deref()),
        Command::CheckGenerated => check_generated(),
    }
}

fn codegen(filter: Option<&str>) -> Result<()> {
    let root = workspace_root();
    for (spec, output) in TARGETS {
        if let Some(filter) = filter {
            let matches_filter = Path::new(spec)
                .file_stem()
                .is_some_and(|stem| stem.eq_ignore_ascii_case(filter) || spec.contains(filter));
            if !matches_filter {
                continue;
            }
        }
        let spec_path = root.join(spec);
        let yaml = std::fs::read_to_string(&spec_path)
            .with_context(|| format!("reading spec {spec_path:?}"))?;
        let code = rustfmt(&oxisport_codegen::codegen_from_yaml(&yaml, spec)?)?;

        let output_path = root.join(output);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {parent:?}"))?;
        }
        if std::fs::read_to_string(&output_path).ok().as_deref() == Some(code.as_str()) {
            println!("unchanged: {spec} -> {output}");
        } else {
            std::fs::write(&output_path, &code)
                .with_context(|| format!("writing {output_path:?}"))?;
            println!("generated: {spec} -> {output}");
        }
    }
    Ok(())
}

fn check_generated() -> Result<()> {
    let root = workspace_root();
    let mut outdated = 0;
    for (spec, output) in TARGETS {
        let yaml = std::fs::read_to_string(root.join(spec))
            .with_context(|| format!("reading spec {spec}"))?;
        let expected = rustfmt(&oxisport_codegen::codegen_from_yaml(&yaml, spec)?)?;
        let committed = std::fs::read_to_string(root.join(output)).unwrap_or_default();
        if committed == expected {
            println!("ok: {output}");
        } else {
            println!("outdated: {output} (run `cargo xtask codegen`)");
            outdated += 1;
        }
    }
    if outdated > 0 {
        bail!("{outdated} generated file(s) are outdated");
    }
    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives directly under the workspace root")
        .to_path_buf()
}

/// Formats Rust source with rustfmt so generated files pass
/// `cargo fmt --check` without carrying an unstable `rustfmt::skip`.
fn rustfmt(code: &str) -> Result<String> {
    let mut child = std::process::Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawning rustfmt; is the rustfmt component installed?")?;
    child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(code.as_bytes())
        .context("writing source to rustfmt")?;
    let output = child.wait_with_output().context("waiting for rustfmt")?;
    if !output.status.success() {
        bail!(
            "rustfmt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout).context("rustfmt output is not UTF-8")
}
