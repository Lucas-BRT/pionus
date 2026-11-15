//! The modes: one choice of what to prioritise, resolved into the settings it
//! stands for.
//!
//! A mode fills in `model.name`, and `model.max_prompt_tokens` for the modes
//! that need a prompt of their own, only where nobody set them (a file, the environment or a flag): a setting written
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

/// The names of every mode, `off` (no preset) first.
pub const NAMES: &[&str] = &["off", "fast", "balanced", "quality"];

/// The settings `mode` stands for, each with the origin that says why.
pub fn settings(mode: &str) -> Vec<(&'static str, Value, Origin)> {
    let Some(preset) = PRESETS.iter().find(|p| p.name == mode) else {
        return Vec::new();
    };
    let origin = Origin::Mode(preset.name);
    let mut settings = vec![(
        "model.name",
        Value::Text(preset.model.to_owned()),
        origin.clone(),
    )];
    if let Some(tokens) = preset.max_prompt_tokens {
        settings.push(("model.max_prompt_tokens", Value::Count(tokens), origin));
    }
    settings
}
