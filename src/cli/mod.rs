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
