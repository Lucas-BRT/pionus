//! The modes: one choice of what to prioritise, resolved into the settings it
//! stands for.
//!
//! A mode fills in `model.name` and `model.max_prompt_tokens` only where
//! nobody set them (a file, the environment or a flag): a setting written
//! explicitly always wins, so a mode is a default, never an override. The
//! presets are starting points to be measured, not promises of quality or speed.

use super::{Origin, Value};

/// A mode's settings.
pub struct Preset {
    pub name: &'static str,
    pub model: &'static str,
    pub max_prompt_tokens: usize,
    pub about: &'static str,
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "fast",
        model: "pionus-0.5b-v0.3",
        max_prompt_tokens: 1024,
        about: "the smallest fine-tune: the least time and memory per message",
    },
    Preset {
        name: "balanced",
        model: "pionus-1.5b-v0.3",
        max_prompt_tokens: 1024,
        about: "the 1.5B fine-tune: the best messages at the default prompt size",
    },
    Preset {
        name: "quality",
        model: "pionus-1.5b-v0.3",
        max_prompt_tokens: 2048,
        about: "the 1.5B fine-tune with twice the prompt: sees more of a large diff, slower",
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
    vec![
        (
            "model.name",
            Value::Text(preset.model.to_owned()),
            origin.clone(),
        ),
        (
            "model.max_prompt_tokens",
            Value::Count(preset.max_prompt_tokens),
            origin,
        ),
    ]
}
