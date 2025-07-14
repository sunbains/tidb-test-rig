//! # Configuration Extensions
//!
//! Dynamic configuration extensions for test-specific options.
//! Provides a plugin system for extending CLI arguments and configuration building.

use crate::config::AppConfig;
use clap::{ArgMatches, Command};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Trait for extending the configuration generator with test-specific options
pub trait ConfigExtension: Send + Sync {
    /// Add CLI arguments specific to this extension
    fn add_cli_args(&self, app: Command) -> Command;

    /// Build configuration from command line arguments
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be built.
    fn build_config(
        &self,
        args: &ArgMatches,
        config: &mut AppConfig,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Get the name of this extension
    fn get_extension_name(&self) -> &'static str;

    /// Get help text for this extension
    fn get_help_text(&self) -> &'static str;
}

/// Global registry for configuration extensions
static EXTENSIONS: OnceLock<Mutex<HashMap<String, Box<dyn ConfigExtension>>>> = OnceLock::new();

/// Register a configuration extension
pub fn register_config_extension(extension: Box<dyn ConfigExtension>) {
    let extensions = EXTENSIONS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut extensions) = extensions.lock() {
        extensions.insert(extension.get_extension_name().to_string(), extension);
    }
}

/// Get all registered extensions
pub fn get_extensions() -> Option<&'static Mutex<HashMap<String, Box<dyn ConfigExtension>>>> {
    EXTENSIONS.get()
}

/// Apply all registered extensions to a CLI command
#[must_use]
pub fn apply_extensions_to_command(mut app: Command) -> Command {
    if let Some(extensions) = get_extensions()
        && let Ok(extensions) = extensions.lock()
    {
        for extension in extensions.values() {
            app = extension.add_cli_args(app);
        }
    }
    app
}

/// Apply configuration extensions to the main configuration
///
/// # Errors
///
/// Returns an error if any extension fails to apply.
pub fn apply_extensions_to_config(
    args: &ArgMatches,
    config: &mut AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(extensions) = get_extensions()
        && let Ok(extensions) = extensions.lock()
    {
        for extension in extensions.values() {
            extension.build_config(args, config)?;
        }
    }
    Ok(())
}

/// Print help for all registered extensions
pub fn print_extensions_help() {
    if let Some(extensions) = get_extensions()
        && let Ok(extensions) = extensions.lock()
        && !extensions.is_empty()
    {
        println!("\nTest-Specific Configuration Options:");
        for extension in extensions.values() {
            println!(
                "  {}: {}",
                extension.get_extension_name(),
                extension.get_help_text()
            );
        }
    }
}
