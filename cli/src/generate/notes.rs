//! Notes added to prompts when retrying or avoiding messages.

use crate::message;
use crate::style::Convention;

use super::SanitizeError;

/// How many turned-down subjects the prompt names: each one takes room the
/// diff could use, and the sanitizer rejects every one of them anyway.
pub const MAX_NAMED: usize = 3;

/// Asks for a message other than the subjects of `messages`, naming the
/// [`MAX_NAMED`] most recent different ones, oldest first.
pub fn avoid_note(messages: &[String]) -> Option<String> {
    let mut named: Vec<String> = Vec::new();
    for m in messages.iter().rev() {
        let subject: String = m
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .take(message::SUBJECT_MAX)
            .collect();
        let subject = format!("\"{subject}\"");
        if !named.contains(&subject) {
            named.push(subject);
        }
        if named.len() == MAX_NAMED {
            break;
        }
    }
    if named.is_empty() {
        return None;
    }
    named.reverse();
    let subjects = named;
    Some(format!(
        "The user turned down {}: {}. Write a different message that describes \
         the change better.",
        if subjects.len() == 1 {
            "this message"
        } else {
            "these messages"
        },
        subjects.join("; ")
    ))
}

/// Tells the model why its previous reply was rejected.
pub fn retry_note(error: &SanitizeError, convention: Convention) -> String {
    let reason = match error {
        SanitizeError::Empty => "it contained no commit message".to_owned(),
        SanitizeError::NotConventional(subject) => {
            let subject: String = subject.chars().take(message::SUBJECT_MAX).collect();
            format!("\"{subject}\" does not follow `type(scope): description`")
        }
        SanitizeError::CopiedHistory(subject) => {
            let subject: String = subject.chars().take(message::SUBJECT_MAX).collect();
            format!(
                "\"{subject}\" repeats an earlier commit of this repository; \
                 describe what this diff changes instead"
            )
        }
        SanitizeError::TurnedDown(subject) => {
            let subject: String = subject.chars().take(message::SUBJECT_MAX).collect();
            format!("\"{subject}\" is a message the user turned down; write a different one")
        }
        SanitizeError::CutOff(subject) => {
            let subject: String = subject.chars().take(message::SUBJECT_MAX).collect();
            format!("\"{subject}\" stops in the middle of a sentence; finish it")
        }
    };
    let format = match convention {
        Convention::Conventional => " in the `type(scope): description` format",
        _ => "",
    };
    format!(
        "Your previous reply was rejected because {reason}. \
         Reply again with only the commit message{format}."
    )
}
