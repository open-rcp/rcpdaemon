//! CLI configuration management
//!
//! This module provides functionality for CLI configuration management.

#[cfg(feature = "cli")]
use crate::cli::error::CliError;
#[cfg(feature = "cli")]
use anyhow::Result;
#[cfg(feature = "cli")]
use std::path::PathBuf;

/// Configure command implementation
#[cfg(feature = "cli")]
pub async fn handle_config_command(
    command: &crate::cli::types::ConfigCommand,
    config_path: Option<PathBuf>,
    formatter: &crate::cli::utils::OutputFormatter,
) -> Result<(), CliError> {
    match command {
        crate::cli::types::ConfigCommand::Get { key } => get_config(Some(key), config_path).await,
        crate::cli::types::ConfigCommand::Set { key, value } => {
            set_config(key, value, config_path).await
        }
        crate::cli::types::ConfigCommand::Show => list_config(config_path).await,
        crate::cli::types::ConfigCommand::Remove { key } => remove_config(key, config_path).await,
    }
}

/// Get configuration value
#[cfg(feature = "cli")]
async fn get_config(key: Option<&str>, config_path: Option<PathBuf>) -> Result<(), CliError> {
    use crate::cli::utils::{load_config, OutputFormatter};

    let config = load_config(config_path)?;
    let formatter = OutputFormatter::new(true, false, false);

    if let Some(key) = key {
        // Get specific config value
        match key {
            "host" => formatter.info(&format!("host = {}", config.service.host)),
            "port" => formatter.info(&format!("port = {}", config.service.port)),
            "use_tls" => formatter.info(&format!("use_tls = {}", config.service.use_tls)),
            "verify_cert" => {
                formatter.info(&format!("verify_cert = {}", config.service.skip_verify))
            }
            "format" => formatter.info(&format!("format = {:?}", config.global.format)),
            "color" => formatter.info(&format!("color = {}", config.global.color)),
            "json" => formatter.info(&format!("json = {}", config.global.json)),
            "quiet" => formatter.info(&format!("quiet = {}", config.global.quiet)),
            "timeout" => formatter.info(&format!("timeout = {}", config.service.timeout)),
            _ => {
                return Err(CliError::ConfigurationError(format!(
                    "Unknown config key: {}",
                    key
                )));
            }
        }
    } else {
        // Return error - need to specify a key
        return Err(CliError::ConfigurationError(
            "No configuration key specified".to_string(),
        ));
    }

    Ok(())
}

/// Set configuration value
#[cfg(feature = "cli")]
async fn set_config(key: &str, value: &str, config_path: Option<PathBuf>) -> Result<(), CliError> {
    use crate::cli::utils::{load_config, save_config, OutputFormatter};

    let mut config = load_config(config_path.clone())?;
    let formatter = OutputFormatter::new(true, false, false);

    // Update config based on key
    match key {
        "host" => config.service.host = value.to_string(),
        "port" => {
            let port = value.parse::<u16>().map_err(|_| {
                CliError::ConfigurationError(
                    "Port must be a valid number between 1-65535".to_string(),
                )
            })?;
            config.service.port = port;
        }
        "use_tls" => {
            let use_tls = value.parse::<bool>().map_err(|_| {
                CliError::ConfigurationError("use_tls must be true or false".to_string())
            })?;
            config.service.use_tls = use_tls;
        }
        "verify_cert" => {
            let verify_cert = value.parse::<bool>().map_err(|_| {
                CliError::ConfigurationError("verify_cert must be true or false".to_string())
            })?;
            config.service.skip_verify = verify_cert;
        }
        "format" => match value.to_lowercase().as_str() {
            "text" | "json" | "yaml" => {
                if value.to_lowercase() == "text" {
                    config.global.format = crate::cli::config::OutputFormat::Text;
                } else if value.to_lowercase() == "json" {
                    config.global.format = crate::cli::config::OutputFormat::Json;
                } else {
                    config.global.format = crate::cli::config::OutputFormat::Yaml;
                }
            }
            _ => {
                return Err(CliError::ConfigurationError(
                    "format must be text, json, or yaml".to_string(),
                ))
            }
        },
        "color" => {
            let color = value.parse::<bool>().map_err(|_| {
                CliError::ConfigurationError("color must be true or false".to_string())
            })?;
            config.global.color = color;
        }
        "json" => {
            let json = value.parse::<bool>().map_err(|_| {
                CliError::ConfigurationError("json must be true or false".to_string())
            })?;
            config.global.json = json;
        }
        "quiet" => {
            let quiet = value.parse::<bool>().map_err(|_| {
                CliError::ConfigurationError("quiet must be true or false".to_string())
            })?;
            config.global.quiet = quiet;
        }
        "timeout" => {
            let timeout = value.parse::<u64>().map_err(|_| {
                CliError::ConfigurationError("timeout must be a valid number".to_string())
            })?;
            config.service.timeout = timeout;
        }
        _ => {
            return Err(CliError::ConfigurationError(format!(
                "Unknown config key: {}",
                key
            )))
        }
    }

    // Save updated config
    save_config(&config, config_path.expect("Config path required to save"))?;
    formatter.success(&format!("Updated {} = {}", key, value));

    Ok(())
}

/// Remove configuration value
#[cfg(feature = "cli")]
async fn remove_config(key: &str, config_path: Option<PathBuf>) -> Result<(), CliError> {
    use crate::cli::utils::{load_config, save_config, OutputFormatter};

    let mut config = load_config(config_path.clone())?;
    let formatter = OutputFormatter::new(true, false, false);

    // Reset config to default based on key
    match key {
        "host" => config.service.host = "127.0.0.1".to_string(),
        "port" => config.service.port = 8716,
        "use_tls" => config.service.use_tls = false,
        "verify_cert" => config.service.skip_verify = false,
        "format" => config.global.format = crate::cli::config::OutputFormat::Text,
        "color" => config.global.color = true,
        "json" => config.global.json = false,
        "quiet" => config.global.quiet = false,
        "timeout" => config.service.timeout = 30,
        _ => {
            return Err(CliError::ConfigurationError(format!(
                "Unknown config key: {}",
                key
            )))
        }
    }

    // Save updated config
    save_config(&config, config_path.expect("Config path required to save"))?;
    formatter.success(&format!("Reset {} to default value", key));

    Ok(())
}

/// List all configuration values
#[cfg(feature = "cli")]
async fn list_config(config_path: Option<PathBuf>) -> Result<(), CliError> {
    use crate::cli::utils::{load_config, OutputFormatter};

    let config = load_config(config_path)?;
    let formatter = OutputFormatter::new(true, false, false);

    // Display connection settings
    formatter.info("Connection settings:");
    formatter.info(&format!("  host = {}", config.service.host));
    formatter.info(&format!("  port = {}", config.service.port));
    formatter.info(&format!("  use_tls = {}", config.service.use_tls));
    formatter.info(&format!("  verify_cert = {}", config.service.skip_verify));

    // Display output settings
    formatter.info("Output settings:");
    formatter.info(&format!("  format = {:?}", config.global.format));
    formatter.info(&format!("  color = {}", config.global.color));
    formatter.info(&format!("  json = {}", config.global.json));
    formatter.info(&format!("  quiet = {}", config.global.quiet));

    // Display other settings
    formatter.info("Other settings:");
    formatter.info(&format!("  timeout = {}", config.service.timeout));

    Ok(())
}
