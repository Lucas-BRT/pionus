//! The modes: one choice of what to prioritise, resolved into the settings it
//! stands for.
//!
//! A mode fills in `model.name` only where nobody set it, and may ask for a
//! longer prompt on the llama.cpp devices (`prompt_tokens`) (a file, the environment or a flag): a setting written
//! explicitly always wins, so a mode is a default, never an override. The
//! presets are starting points to be measured, not promises of quality or speed.

use super::{Origin, Value};

/// A mode's settings.
pub struct Preset {
    pub name: &'static str,
    pub model: &'static str,
    /// `None` leaves the prompt budget to the device's default.
    pub max_prompt_tokens: Option<usize>,
    pub about: &'static str,
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "fast",
        model: "pionus-0.5b-v0.3",
        max_prompt_tokens: None,
        about: "the smallest fine-tune: the least time and memory per message",
    },
    Preset {
        name: "balanced",
        model: "pionus-1.5b-v0.3",
        max_prompt_tokens: None,
        about: "the 1.5B fine-tune: the best messages at the device's default prompt size",
    },
    Preset {
        name: "quality",
        model: "pionus-1.5b-v0.3",
        max_prompt_tokens: Some(4096),
        about: "the 1.5B fine-tune with a prompt of 4096 tokens: sees more of a large diff, slower",
    },
];

/// How many replies a mode asks for before the fallback: `fast` takes the
/// first one, as the cheapest way to a message.
pub fn max_attempts(mode: &str) -> usize {
    if mode == "fast" {
        1
    } else {
        crate::backend::DEFAULT_MAX_ATTEMPTS
    }
}

/// The prompt budget a mode asks for on the llama.cpp devices (`vulkan`,
/// `cpu`), which size their context for it. It is not a setting: the NPU
/// compiles its graph for one length, so a mode's longer prompt is not its.
pub fn prompt_tokens(mode: &str) -> Option<usize> {
    PRESETS
        .iter()
        .find(|p| p.name == mode)
        .and_then(|p| p.max_prompt_tokens)
}

/// The names of every mode, `off` (no preset) first.
pub const NAMES: &[&str] = &["off", "fast", "balanced", "quality"];

/// The settings `mode` stands for, each with the origin that says why.
pub fn settings(mode: &str) -> Vec<(&'static str, Value, Origin)> {
    let Some(preset) = PRESETS.iter().find(|p| p.name == mode) else {
        return Vec::new();
    };
    let origin = Origin::Mode(preset.name);
    let settings = vec![(
        "model.name",
        Value::Text(preset.model.to_owned()),
        origin.clone(),
    )];
    settings
}
