use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState};

use super::Theme;

/// A search result item for display in the TUI
#[derive(Clone, Debug)]
pub struct ResultItem {
    pub index: usize,
    pub title: String,
    pub library: String,
    pub id: String,
    pub url: Option<String>,
    pub excerpt: String,
    pub full_content: String,
}

pub struct ResultsView {
    pub items: Vec<ResultItem>,
    pub state: ListState,
    pub scrollbar_state: ScrollbarState,
}

impl ResultsView {
    pub fn new(items: Vec<ResultItem>) -> Self {
        let len = items.len();
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self {
            items,
            state,
            scrollbar_state: ScrollbarState::new(len),
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    pub fn selected(&self) -> Option<&ResultItem> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = self.state.selected() == Some(i);
                render_result_item(item, is_selected, theme)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Results ")
                    .title_style(theme.header_title)
                    .borders(Borders::ALL)
                    .border_style(theme.border_focused),
            )
            .highlight_style(
                Style::default()
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.state);

        // Scrollbar
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut self.scrollbar_state,
        );
    }
}

fn render_result_item<'a>(item: &ResultItem, _is_selected: bool, theme: &Theme) -> ListItem<'a> {
    let number = Span::styled(
        format!("[{}] ", item.index + 1),
        theme.result_number,
    );
    let title = Span::styled(
        item.title.clone(),
        theme.result_title,
    );
    let library = Span::styled(
        format!("  ({})", item.library),
        theme.result_library,
    );

    let title_line = Line::from(vec![number, title, library]);

    let id_line = Line::from(vec![
        Span::styled("    ID: ", theme.dimmed),
        Span::styled(item.id.clone(), theme.result_id),
    ]);

    // Truncate excerpt to ~80 chars
    let excerpt_text = if item.excerpt.len() > 80 {
        format!("    {}...", &item.excerpt[..77])
    } else {
        format!("    {}", &item.excerpt)
    };
    let excerpt_line = Line::from(Span::styled(excerpt_text, theme.result_excerpt));

    let spacer = Line::from("");

    ListItem::new(vec![title_line, id_line, excerpt_line, spacer])
}
