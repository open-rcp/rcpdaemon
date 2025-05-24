//! CLI module for rcpdaemon
//!
//! This module provides CLI functionality for the rcpdaemon daemon.

#[cfg(feature = "cli")]
pub mod commands;

#[cfg(feature = "cli")]
pub mod utils;

#[cfg(feature = "cli")]
pub mod config;

#[cfg(feature = "cli")]
pub mod error;

#[cfg(feature = "cli")]
pub mod types;

#[cfg(feature = "cli")]
pub mod service;

#[cfg(feature = "cli")]
use anyhow::Result;
#[cfg(feature = "cli")]
use service::ServiceClient;
#[cfg(feature = "cli")]
use types::{Cli, RcpdaemonCommand};
#[cfg(feature = "cli")]
use utils::OutputFormatter;

/// Main CLI handler function
#[cfg(feature = "cli")]
pub async fn handle_cli(cli: Cli) -> Result<()> {
    // Create output formatter
    let formatter = OutputFormatter::new(cli.json, true, false);

    // Create service client for commands that need it
    let client = ServiceClient::new("127.0.0.1".to_string(), 8716, 30);

    match cli.command {
        Some(RcpdaemonCommand::Daemon { command }) => {
            match command {
                Some(daemon_cmd) => {
                    // Handle daemon commands (start, stop, restart, status)
                    formatter.info(&format!("Daemon command: {:?}", daemon_cmd));
                }
                None => {
                    formatter.info("No daemon subcommand specified");
                }
            }
        }
        Some(RcpdaemonCommand::Server { command }) => {
            commands::server::handle_status(&client, &formatter).await?;
        }
        Some(RcpdaemonCommand::Service { command }) => {
            commands::service::handle_status(&client, &formatter).await?;
        }
        Some(RcpdaemonCommand::App { ref command }) => {
            let mut cli_mut = cli.clone();
            commands::app::handle_app_command(&mut cli_mut, command)
                .await
                .map_err(|e| anyhow::anyhow!("App command error: {}", e))?;
        }
        Some(RcpdaemonCommand::Session { command }) => match command {
            types::SessionCommand::List => {
                commands::session::handle_list(&client, &formatter).await?;
            }
            types::SessionCommand::Info { session_id } => {
                commands::session::handle_info(&session_id, &client, &formatter).await?;
            }
            types::SessionCommand::Close { session_id } => {
                commands::session::handle_disconnect(&session_id, &client, &formatter).await?;
            }
        },
        Some(RcpdaemonCommand::User { command }) => match command {
            types::UserCommand::List => {
                commands::user::handle_list(&client, &formatter).await?;
            }
            _ => {
                formatter.info("User command handling not fully implemented");
            }
        },
        Some(RcpdaemonCommand::Config { command }) => {
            commands::config::handle_config_command(&command, None, &formatter)
                .await
                .map_err(|e| anyhow::anyhow!("Config command error: {}", e))?;
        }
        Some(RcpdaemonCommand::Diag { command }) => match command {
            types::DiagCommand::System => {
                commands::diag::handle_system_diag(&formatter).await?;
            }
            types::DiagCommand::Network => {
                commands::diag::handle_network_diag(&client, &formatter).await?;
            }
            types::DiagCommand::Logs { lines, follow } => {
                commands::diag::handle_logs(lines, follow, &formatter).await?;
            }
        },
        Some(RcpdaemonCommand::Completions { shell }) => {
            commands::completions::handle_completions_command(shell, None)?;
        }
        None => {
            // No command specified, run daemon mode
            formatter.info("Starting rcpdaemon in daemon mode...");
            run_daemon_mode(&cli).await?;
        }
    }

    Ok(())
}

/// Run daemon mode when no command is specified
#[cfg(feature = "cli")]
async fn run_daemon_mode(cli: &Cli) -> Result<()> {
    use crate::config;
    use log::info;
    use std::path::PathBuf;

    // Load configuration
    let config_file = &cli.config;
    let config = match config::ServiceConfig::from_file(config_file) {
        Ok(cfg) => {
            info!("Configuration loaded from {}", config_file);
            cfg
        }
        Err(e) => {
            info!(
                "Failed to load config from {}: {}. Using defaults.",
                config_file, e
            );
            config::ServiceConfig::default()
        }
    };

    #[cfg(feature = "api")]
    info!("Starting rcpdaemon (with API)...");

    #[cfg(not(feature = "api"))]
    info!("Starting rcpdaemon...");

    // Check if we should daemonize
    if !cli.foreground {
        let work_dir = std::env::current_dir()?;
        info!("Daemonizing process in {}", work_dir.display());
        // Note: daemon functionality will be implemented separately
        // daemon::daemonize(&work_dir)?;
    }

    // Start the daemon
    // Note: daemon start functionality will be implemented separately
    // daemon::start(config, std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))

    info!("Daemon mode started successfully");
    Ok(())
}
