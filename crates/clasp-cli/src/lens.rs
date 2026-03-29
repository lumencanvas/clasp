//! CLI handlers for WASM lens module commands.

use anyhow::{Context, Result};
use clasp_lens::LensHost;
use colored::Colorize;
use std::path::PathBuf;

use crate::LensAction;

pub fn handle_lens_command(action: LensAction) -> Result<()> {
    match action {
        LensAction::Validate { file } => validate(&file),
        LensAction::Info { file } => info(&file),
        LensAction::Test {
            file,
            input,
            params,
            inverse,
        } => test(&file, &input, params.as_deref(), inverse),
    }
}

fn load_wasm(file: &PathBuf) -> Result<(Vec<u8>, LensHost)> {
    let bytes =
        std::fs::read(file).with_context(|| format!("Failed to read: {}", file.display()))?;
    let host = LensHost::new(&bytes)
        .with_context(|| format!("Invalid WASM lens module: {}", file.display()))?;
    Ok((bytes, host))
}

fn validate(file: &PathBuf) -> Result<()> {
    let (_bytes, _host) = load_wasm(file)?;
    println!(
        "{} {} is a valid lens module",
        "OK".green().bold(),
        file.display()
    );
    Ok(())
}

fn info(file: &PathBuf) -> Result<()> {
    let (bytes, host) = load_wasm(file)?;
    println!("{}: {}", "File".cyan(), file.display());
    println!("{}: {} bytes", "Size".cyan(), bytes.len());
    println!("{}: alloc, transform", "Required exports".cyan());
    println!(
        "{}: {}",
        "Has inverse".cyan(),
        if host.has_inverse() { "yes" } else { "no" }
    );
    Ok(())
}

fn test(file: &PathBuf, input: &str, params: Option<&str>, inverse: bool) -> Result<()> {
    let (_bytes, mut host) = load_wasm(file)?;

    let input_val: serde_json::Value =
        serde_json::from_str(input).context("Failed to parse --input as JSON")?;

    if let Some(p) = params {
        let params_val: serde_json::Value =
            serde_json::from_str(p).context("Failed to parse --params as JSON")?;
        host.set_params(params_val);
    }

    let result = if inverse {
        host.inverse(&input_val)
            .context("Inverse transform failed")?
    } else {
        host.transform(&input_val)
            .context("Forward transform failed")?
    };

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
