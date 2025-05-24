use crate::{config::ServiceConfig, error::ServiceError, manager::ServiceManager};
use anyhow::Result;
use log::{error, info};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Service daemon that runs in the background
pub struct ServiceDaemon {
    /// Configuration
    config: ServiceConfig,

    /// Working directory
    work_dir: PathBuf,

    /// Shutdown channel receiver
    shutdown_rx: mpsc::Receiver<()>,
}

impl ServiceDaemon {
    /// Create a new service daemon
    pub fn new(config: ServiceConfig, work_dir: PathBuf, shutdown_rx: mpsc::Receiver<()>) -> Self {
        Self {
            config,
            work_dir,
            shutdown_rx,
        }
    }

    /// Start the daemon
    pub async fn start(&mut self) -> Result<(), ServiceError> {
        info!("Starting service daemon");

        // Create service manager with all required parameters
        let (shutdown_tx, _) = mpsc::channel::<()>(1);
        let mut service_manager =
            ServiceManager::new(self.work_dir.clone(), self.config.clone(), shutdown_tx);

        // Start the manager
        service_manager.start().await?;

        // Wait for shutdown signal
        self.wait_for_shutdown().await;

        Ok(())
    }

    /// Wait for shutdown signal
    async fn wait_for_shutdown(&mut self) {
        if let Some(_) = self.shutdown_rx.recv().await {
            info!("Shutdown signal received");
        }
    }
}

/// Daemonize the current process (Unix only)
#[cfg(unix)]
pub fn daemonize(work_dir: &PathBuf) -> Result<()> {
    use std::fs::File;

    info!("Daemonizing process");

    let pid_file = std::env::temp_dir().join("rcpdaemon.pid");
    let log_file = std::env::temp_dir().join("rcpdaemon.log");

    let daemonize = daemonize::Daemonize::new()
        .pid_file(pid_file)
        .chown_pid_file(true)
        .working_directory(work_dir)
        .stdout(File::create(&log_file).unwrap())
        .stderr(File::create(&log_file).unwrap());

    daemonize.start().map_err(|e| {
        error!("Error starting daemon: {}", e);
        anyhow::anyhow!("Failed to start daemon: {}", e)
    })?;

    Ok(())
}

/// Windows service implementation (placeholder)
#[cfg(windows)]
pub fn daemonize(_work_dir: &PathBuf) -> Result<()> {
    info!("Windows service mode - daemonize not needed");
    Ok(())
}

/// Start the daemon service
pub fn start(config: ServiceConfig, work_dir: PathBuf) -> Result<()> {
    info!("Starting daemon service");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        setup_signal_handlers(shutdown_tx).await?;

        let mut daemon = ServiceDaemon::new(config, work_dir, shutdown_rx);
        daemon
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Daemon error: {}", e))
    })?;

    Ok(())
}

/// Setup signal handlers (Unix)
#[cfg(unix)]
async fn setup_signal_handlers(shutdown_tx: mpsc::Sender<()>) -> Result<()> {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::spawn(async move {
        tokio::select! {
            _ = sigterm.recv() => {
                info!("SIGTERM received, shutting down");
                let _ = shutdown_tx.send(()).await;
            }
            _ = sigint.recv() => {
                info!("SIGINT received, shutting down");
                let _ = shutdown_tx.send(()).await;
            }
        }
    });

    Ok(())
}

/// Setup signal handlers (Windows)
#[cfg(windows)]
async fn setup_signal_handlers(shutdown_tx: mpsc::Sender<()>) -> Result<()> {
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Ctrl+C received, shutting down");
                let _ = shutdown_tx.send(()).await;
            }
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    Ok(())
}

/// Get daemon status
pub fn status() -> Result<String> {
    let pid_file = std::env::temp_dir().join("rcpdaemon.pid");

    if !pid_file.exists() {
        return Ok("Not running".to_string());
    }

    let pid_data = std::fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_data.trim().parse()?;

    if is_process_running(pid) {
        Ok(format!("Running (PID: {})", pid))
    } else {
        Ok("Not running (stale PID file)".to_string())
    }
}

/// Check if a process is running (Unix)
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    let status = unsafe { libc::kill(pid as i32, 0) };
    status == 0
}

/// Check if a process is running (Windows)
#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    use std::process::Command;

    let output = Command::new("tasklist")
        .args(&["/FI", &format!("PID eq {}", pid)])
        .output();

    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains(&pid.to_string())
        }
        Err(_) => false,
    }
}

/// Stop the daemon
pub fn stop() -> Result<()> {
    info!("Stopping daemon");

    let pid_file = std::env::temp_dir().join("rcpdaemon.pid");

    if !pid_file.exists() {
        return Err(anyhow::anyhow!("Daemon not running (no PID file)"));
    }

    let pid_data = std::fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_data.trim().parse()?;

    terminate_process(pid)?;

    std::fs::remove_file(&pid_file)?;

    info!("Daemon stopped");
    Ok(())
}

/// Terminate a process (Unix)
#[cfg(unix)]
fn terminate_process(pid: u32) -> Result<()> {
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
    Ok(())
}

/// Terminate a process (Windows)
#[cfg(windows)]
fn terminate_process(pid: u32) -> Result<()> {
    use std::process::Command;

    let output = Command::new("taskkill")
        .args(&["/PID", &pid.to_string(), "/F"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to terminate process {}", pid));
    }

    Ok(())
}
