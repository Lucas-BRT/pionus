//! Choosing what to prioritise: a mode (`model.mode`), each with the model it
//! uses and what it is for. Shown from the setup and from the model picker.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use super::{hints, is_interrupt, theme};
use crate::config::mode::PRESETS;

/// What the user chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    None,
    /// This mode: one of [`crate::config::mode::NAMES`].
    Chosen(&'static str),
    /// Leave things as they are.
    Cancel,
}

pub struct ModeView {
    /// The mode in use, marked, which the highlight starts on.
    current: &'static str,
    selected: usize,
}

impl ModeView {
    /// `current` is the mode in use, one of [`crate::config::mode::NAMES`] (`off`
    /// otherwise).
    pub fn new(current: &str) -> Self {
        let names = Self::names();
        let current = names
            .iter()
            .copied()
            .find(|n| *n == current)
            .unwrap_or("off");
        Self {
            current,
            selected: names.iter().position(|n| *n == current).unwrap_or(0),
        }
    }

    /// The modes in screen order: the presets, then `off`, no preset (every
    /// one of [`crate::config::mode::NAMES`]).
    fn names() -> Vec<&'static str> {
        PRESETS
            .iter()
            .map(|p| p.name)
            .chain(std::iter::once("off"))
            .collect()
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Outcome {
        let names = Self::names();
        if is_interrupt(&key) {
            return Outcome::Cancel;
        }
        match key.code {
            KeyCode::Enter => Outcome::Chosen(names[self.selected]),
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.checked_sub(1).unwrap_or(names.len() - 1);
                Outcome::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % names.len();
                Outcome::None
            }
            KeyCode::Char(digit @ '1'..='9') => names
                .get(digit as usize - '1' as usize)
                .map_or(Outcome::None, |name| Outcome::Chosen(name)),
            KeyCode::Char('q') | KeyCode::Esc => Outcome::Cancel,
            _ => Outcome::None,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let names = Self::names();
        let [question, list, detail, keys] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(names.len() as u16 + 2),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        frame.render_widget(
            Paragraph::new(vec![
                Line::styled(" What matters most?", theme::title()),
                Line::styled(
                    " A mode picks the model; what you set yourself still wins.",
                    theme::dim(),
                ),
            ]),
            question,
        );

        let lines: Vec<Line> = names
            .iter()
            .enumerate()
            .map(|(n, name)| {
                let model = PRESETS
                    .iter()
                    .find(|p| p.name == *name)
                    .map_or("nothing is chosen for you", |p| p.model);
                let mark = if *name == self.current {
                    " (in use)"
                } else {
                    ""
                };
                let text = format!(" {} {:<9} {}{mark}", n + 1, name, model);
                if n == self.selected {
                    Line::from(Span::styled(text, theme::selected()))
                } else {
                    Line::raw(text)
                }
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).block(Block::bordered()), list);

        let about = PRESETS
            .iter()
            .find(|p| p.name == names[self.selected])
            .map_or(
                "no preset: model.name and the prompt budget stay as configured",
                |p| p.about,
            );
        frame.render_widget(
            Paragraph::new(vec![Line::styled(format!(" {about}"), theme::dim())])
                .wrap(ratatui::widgets::Wrap { trim: true }),
            detail,
        );

        frame.render_widget(
            Paragraph::new(hints(&[
                ("Enter", "choose"),
                ("1-4", "pick"),
                ("↑/↓", "move"),
                ("q", "cancel"),
            ])),
            keys,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::testing::{char, draw, key};
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn lists_the_modes_with_their_models_and_marks_the_one_in_use() {
        insta::assert_snapshot!(draw(78, 13, |f| ModeView::new("balanced").render(f)));
    }

    #[test]
    fn the_highlight_starts_on_the_mode_in_use_and_enter_chooses_it() {
        assert_eq!(
            ModeView::new("quality").on_key(key(KeyCode::Enter)),
            Outcome::Chosen("quality")
        );
        assert_eq!(
            ModeView::new("off").on_key(key(KeyCode::Enter)),
            Outcome::Chosen("off")
        );
        // Anything else is `off`, the mode of no preset.
        assert_eq!(
            ModeView::new("turbo").on_key(key(KeyCode::Enter)),
            Outcome::Chosen("off")
        );
    }

    #[test]
    fn the_highlight_moves_round_the_list() {
        let mut view = ModeView::new("fast");
        assert_eq!(view.on_key(key(KeyCode::Up)), Outcome::None);
        assert_eq!(view.on_key(key(KeyCode::Enter)), Outcome::Chosen("off"));
        view.on_key(key(KeyCode::Down));
        view.on_key(char('j'));
        assert_eq!(
            view.on_key(key(KeyCode::Enter)),
            Outcome::Chosen("balanced")
        );
        let screen = draw(78, 13, |f| view.render(f));
        assert!(screen.contains("the 1.5B fine-tune"), "{screen}");
    }

    #[test]
    fn a_digit_chooses_its_line_at_once_and_a_missing_one_does_nothing() {
        assert_eq!(
            ModeView::new("off").on_key(char('1')),
            Outcome::Chosen("fast")
        );
        assert_eq!(
            ModeView::new("off").on_key(char('3')),
            Outcome::Chosen("quality")
        );
        assert_eq!(
            ModeView::new("off").on_key(char('4')),
            Outcome::Chosen("off")
        );
        assert_eq!(ModeView::new("off").on_key(char('5')), Outcome::None);
        assert_eq!(ModeView::new("off").on_key(char('0')), Outcome::None);
    }

    #[test]
    fn q_escape_and_interrupt_give_up() {
        assert_eq!(ModeView::new("fast").on_key(char('q')), Outcome::Cancel);
        assert_eq!(
            ModeView::new("fast").on_key(key(KeyCode::Esc)),
            Outcome::Cancel
        );
        let interrupt = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(ModeView::new("fast").on_key(interrupt), Outcome::Cancel);
        assert_eq!(ModeView::new("fast").on_key(char('x')), Outcome::None);
    }
}
