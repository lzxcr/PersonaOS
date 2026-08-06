//! PersonaOS ships without any pre-configured model providers.
//! Users must add their own providers in `~/.pos/config/config.jsonc`.
//!
//! The constants below exist only for legacy config migration and are not
//! used as defaults for new installations.

/// Legacy provider id preserved for config migration.
pub const OPENCODE_PROVIDER_ID: &str = "opencode";
