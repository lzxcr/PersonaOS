use base64::Engine;

include!(concat!(env!("OUT_DIR"), "/builtin_prompts.rs"));

pub const PLAN_REMINDER: &str = include_str!("prompts/plan.md");
pub const CHAT_REMINDER: &str = include_str!("prompts/chat.md");
pub const MEME_DESCRIPTION_PROMPT: &str = include_str!("prompts/meme-description.md");
pub const INPUT_METHOD_DIAGNOSIS_PROMPT: &str =
    include_str!("prompts/linux-input-method-diagnose.md");
pub const GAME_COMPATIBILITY_PROMPT: &str = include_str!("prompts/linux-game-compatibility.md");
pub const COMPACT_SYSTEM_PROMPT: &str = include_str!("prompts/compact.md");

// ── Builtin persona registry ──────────────────────────────────────────

/// Metadata for one builtin persona. The framework never hardcodes persona
/// names — all persona-specific behaviour is driven by this registry.
pub struct BuiltinPersona {
    /// Canonical kebab-case name matching the `builtin-<name>.md` stem.
    pub canonical_name: &'static str,
    /// Human-readable name for TUI / CLI menus.
    pub display_name: &'static str,
    /// English display name fallback (used when i18n is English).
    pub display_name_en: &'static str,
    /// Whether this is the factory-default persona (empty active_persona).
    pub is_default: bool,
    /// Optional WebUI panel title shown on the landing page.
    pub board_title: Option<&'static str>,
    /// Optional WebUI panel subtitle.
    pub board_subtitle: Option<&'static str>,
    /// Optional starter prompts shown on the WebUI landing page.
    pub starter_prompts: &'static [&'static str],
    /// Optional asset path for the persona avatar (WebUI).
    pub avatar_asset: Option<&'static str>,
    /// Optional asset path for the WebUI board background image.
    pub board_asset: Option<&'static str>,
    /// Meme library name override; `None` uses `canonical_name`.
    pub meme_library: Option<&'static str>,
}

/// Builtin persona registry. Empty in production: PersonaOS ships without any
/// preloaded personality. Users register personas here (or via prompt files)
/// before the assistant can run.
#[cfg(not(test))]
pub static BUILTIN_PERSONAS: &[BuiltinPersona] = &[];

/// Test-only default persona so unit tests have a stable builtin to resolve.
#[cfg(test)]
pub static BUILTIN_PERSONAS: &[BuiltinPersona] = &[BuiltinPersona {
    canonical_name: "test",
    display_name: "Test",
    display_name_en: "Test",
    is_default: true,
    board_title: None,
    board_subtitle: None,
    starter_prompts: &[],
    avatar_asset: None,
    board_asset: None,
    meme_library: Some("test"),
}];

// ── Registry queries ───────────────────────────────────────────────────

pub fn builtin_personas() -> &'static [BuiltinPersona] {
    BUILTIN_PERSONAS
}

pub fn builtin_persona(name: &str) -> Option<&'static BuiltinPersona> {
    let canonical = name.trim().to_ascii_lowercase();
    BUILTIN_PERSONAS
        .iter()
        .find(|p| p.canonical_name == canonical)
}

pub fn default_builtin_persona() -> anyhow::Result<&'static BuiltinPersona> {
    BUILTIN_PERSONAS
        .iter()
        .find(|p| p.is_default)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no builtin persona is registered: add an entry to BUILTIN_PERSONAS in src/prompts.rs, or configure a custom persona in the prompts directory"
            )
        })
}

pub fn is_builtin_canonical(name: &str) -> bool {
    builtin_persona(name).is_some()
}

/// Returns the persona file name used for `active_persona` when a
/// non-default builtin is activated (e.g. `Default.md`).
pub fn builtin_persona_file(name: &str) -> String {
    let name = name.trim();
    let stem = name.strip_suffix(".md").unwrap_or(name);
    // Capitalize first letter to match convention.
    let mut file = String::with_capacity(stem.len() + 3);
    let mut chars = stem.chars();
    if let Some(first) = chars.next() {
        file.push(first.to_ascii_uppercase());
    }
    file.push_str(chars.as_str());
    file.push_str(".md");
    file
}

// ── Prompt loading ─────────────────────────────────────────────────────

#[cfg(not(test))]
pub fn default_system_prompt() -> anyhow::Result<String> {
    let default = default_builtin_persona()?;
    builtin_persona_prompt(default.canonical_name)
}

#[cfg(test)]
pub fn default_system_prompt() -> anyhow::Result<String> {
    Ok("You are the default test persona of PersonaOS.".to_string())
}

/// Load a builtin persona's embedded system prompt by canonical name.
pub fn builtin_persona_prompt(name: &str) -> anyhow::Result<String> {
    let canonical = name.trim().to_ascii_lowercase();
    for (key, encoded) in BUILTIN_PROMPTS {
        if *key == canonical {
            return Ok(decode_obfuscated(encoded));
        }
    }
    anyhow::bail!("unknown builtin persona: {name}")
}

fn decode_obfuscated(encoded: &str) -> String {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .expect("embedded default prompt must be valid base64");
    let decoded = bytes
        .into_iter()
        .enumerate()
        .map(|(index, byte)| byte ^ PROMPT_MASK[index % PROMPT_MASK.len()])
        .collect::<Vec<_>>();
    String::from_utf8(decoded).expect("embedded default prompt must be valid utf-8")
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_test_default_in_test_build() {
        // In test builds the registry carries a single "test" persona so the
        // framework has something to resolve. Production builds keep it empty.
        assert!(builtin_persona("test").is_some());
        assert!(builtin_persona("pos").is_none());
        assert!(builtin_persona("alice").is_none());
        assert!(builtin_persona("nobody").is_none());

        let default = default_builtin_persona().unwrap();
        assert_eq!(default.canonical_name, "test");
        assert!(default.is_default);
        assert!(default_system_prompt().is_ok());
    }

    #[test]
    fn unknown_builtin_persona_is_rejected() {
        assert!(builtin_persona_prompt("nobody").is_err());
        assert!(builtin_persona_prompt("pos").is_err());
    }

    #[test]
    fn builtin_persona_file_names() {
        assert_eq!(builtin_persona_file("alice"), "Alice.md");
        assert_eq!(builtin_persona_file("alice"), "Alice.md");
        // Already with .md suffix.
        assert_eq!(builtin_persona_file("Alice.md"), "Alice.md");
    }
}
