//! # Common States
//!
//! Shared state definitions for common workflow states used across multiple binaries.
//! This module provides reusable state functions to eliminate code duplication.

use crate::dynamic_state;
use crate::state_machine_dynamic::DynamicState;

/// Parsing configuration state
#[must_use]
pub fn parsing_config() -> DynamicState {
    dynamic_state!("parsing_config", "Parsing Configuration")
}

/// Connecting to `TiDB` state
#[must_use]
pub fn connecting() -> DynamicState {
    dynamic_state!("connecting", "Connecting to TiDB")
}

/// Testing connection state
#[must_use]
pub fn testing_connection() -> DynamicState {
    dynamic_state!("testing_connection", "Testing Connection")
}

/// Verifying database state
#[must_use]
pub fn verifying_database() -> DynamicState {
    dynamic_state!("verifying_database", "Verifying Database")
}

/// Getting server version state
#[must_use]
pub fn getting_version() -> DynamicState {
    dynamic_state!("getting_version", "Getting Server Version")
}

/// Completed state
#[must_use]
pub fn completed() -> DynamicState {
    dynamic_state!("completed", "Completed")
}
