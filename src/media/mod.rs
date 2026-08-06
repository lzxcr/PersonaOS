//! Media capability interfaces — unified, extensible abstractions for
//! speech synthesis and recognition. No built-in implementations; users
//! configure providers via config.

use anyhow::Result;

/// Synthesised audio data returned by a TTS provider.
#[derive(Clone, Debug)]
pub struct SynthesisedAudio {
    /// Raw audio bytes (MP3, OGG, or WAV).
    pub data: Vec<u8>,
    /// MIME type, e.g. "audio/ogg" or "audio/mpeg".
    pub mime: String,
    /// Duration in seconds (best-effort estimate).
    pub duration_secs: u64,
}

/// Text-to-speech synthesis provider.
///
/// Implementations convert text to audio. The framework does not ship a
/// default implementation — users configure a provider via config.
pub trait SpeechSynthesis: Send + Sync {
    /// Convert text to spoken audio.
    fn synthesise(&self, text: &str, language: Option<&str>) -> Result<SynthesisedAudio>;
}

/// Speech-to-text recognition provider.
///
/// Implementations transcribe audio to text. The framework does not ship a
/// default implementation.
pub trait SpeechRecognition: Send + Sync {
    /// Transcribe audio bytes to text.
    fn transcribe(&self, audio: &[u8], mime: &str) -> Result<String>;
}

// ── Config ─────────────────────────────────────────────────────────────

/// TTS provider configuration.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TtsConfig {
    pub enabled: bool,
    /// Provider identifier (e.g. "openai", "edge", "piper").
    pub provider: String,
    /// Provider-specific settings.
    #[serde(default)]
    pub settings: serde_json::Value,
}

/// STT provider configuration.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SttConfig {
    pub enabled: bool,
    /// Provider identifier (e.g. "openai-whisper", "whisper-local").
    pub provider: String,
    /// Provider-specific settings.
    #[serde(default)]
    pub settings: serde_json::Value,
}
