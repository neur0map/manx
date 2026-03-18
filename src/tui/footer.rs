use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::Theme;

pub struct KeyHint {
    pub key: &'static str,
    pub desc: &'static str,
}

pub fn render_footer(frame: &mut Frame, area: Rect, hints: &[KeyHint], theme: &Theme) {
    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, hint)| {
            let mut v = vec![
                Span::styled(format!(" {} ", hint.key), theme.footer_key),
                Span::styled(format!(" {} ", hint.desc), theme.footer_desc),
            ];
            if i < hints.len() - 1 {
                v.push(Span::styled("  ", theme.dimmed));
            }
            v
        })
        .collect();

    let bar = Paragraph::new(Line::from(spans));
    frame.render_widget(bar, area);
}

pub fn results_hints() -> Vec<KeyHint> {
    vec![
        KeyHint { key: "j/k", desc: "navigate" },
        KeyHint { key: "Enter", desc: "expand" },
        KeyHint { key: "/", desc: "filter" },
        KeyHint { key: "s", desc: "save" },
        KeyHint { key: "q", desc: "quit" },
    ]
}

pub fn detail_hints() -> Vec<KeyHint> {
    vec![
        KeyHint { key: "j/k", desc: "scroll" },
        KeyHint { key: "Esc", desc: "back" },
        KeyHint { key: "s", desc: "save" },
        KeyHint { key: "o", desc: "open URL" },
        KeyHint { key: "q", desc: "quit" },
    ]
}
