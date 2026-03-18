use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;

use super::detail_view::DetailView;
use super::footer;
use super::header;
use super::results_view::{ResultItem, ResultsView};
use super::Theme;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Results,
    Detail,
}

pub struct App {
    pub view: View,
    pub query: String,
    pub library: String,
    pub results: ResultsView,
    pub detail: DetailView,
    pub theme: Theme,
    pub should_quit: bool,
}

impl App {
    pub fn new(library: &str, query: &str, items: Vec<ResultItem>) -> Self {
        Self {
            view: View::Results,
            query: query.to_string(),
            library: library.to_string(),
            results: ResultsView::new(items),
            detail: DetailView::new(),
            theme: Theme::default(),
            should_quit: false,
        }
    }

    /// Launch the TUI. Call this after results have been fetched.
    pub fn run(mut self) -> io::Result<()> {
        if self.results.items.is_empty() {
            // Nothing to show interactively
            eprintln!("No results found.");
            return Ok(());
        }

        let mut terminal = ratatui::init();
        let result = self.main_loop(&mut terminal);
        ratatui::restore();
        result
    }

    fn main_loop(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if self.should_quit {
                break;
            }

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                self.handle_key(key.code, key.modifiers);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        // Global keys
        match code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            _ => {}
        }

        match self.view {
            View::Results => self.handle_results_key(code),
            View::Detail => self.handle_detail_key(code),
        }
    }

    fn handle_results_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Down | KeyCode::Char('j') => self.results.next(),
            KeyCode::Up | KeyCode::Char('k') => self.results.previous(),
            KeyCode::Enter => {
                if self.results.selected().is_some() {
                    self.detail.reset_scroll();
                    self.view = View::Detail;
                }
            }
            KeyCode::Home => {
                if !self.results.items.is_empty() {
                    self.results.state.select(Some(0));
                }
            }
            KeyCode::End => {
                if !self.results.items.is_empty() {
                    self.results
                        .state
                        .select(Some(self.results.items.len() - 1));
                }
            }
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Backspace => {
                self.view = View::Results;
            }
            KeyCode::Down | KeyCode::Char('j') => self.detail.scroll_down(1),
            KeyCode::Up | KeyCode::Char('k') => self.detail.scroll_up(1),
            KeyCode::PageDown => self.detail.scroll_down(10),
            KeyCode::PageUp => self.detail.scroll_up(10),
            KeyCode::Home => self.detail.reset_scroll(),
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Fill(1),  // main content
            Constraint::Length(1), // footer
        ])
        .split(area);

        // Header
        header::render_compact_header(
            frame,
            layout[0],
            &self.query,
            &self.library,
            self.results.items.len(),
            &self.theme,
        );

        // Main content
        match self.view {
            View::Results => {
                self.render_results_layout(frame, layout[1]);
            }
            View::Detail => {
                if let Some(item) = self.results.selected().cloned() {
                    self.detail.render(frame, layout[1], &item, &self.theme);
                }
            }
        }

        // Footer
        let hints = match self.view {
            View::Results => footer::results_hints(),
            View::Detail => footer::detail_hints(),
        };
        footer::render_footer(frame, layout[2], &hints, &self.theme);
    }

    fn render_results_layout(&mut self, frame: &mut Frame, area: Rect) {
        // Split: narrow list on left, wide preview on right
        let chunks = Layout::horizontal([
            Constraint::Percentage(35),
            Constraint::Percentage(65),
        ])
        .split(area);

        // Results list
        self.results.render(frame, chunks[0], &self.theme);

        // Preview pane for selected item
        if let Some(item) = self.results.selected().cloned() {
            self.detail
                .render_preview(frame, chunks[1], &item, &self.theme);
        }
    }
}

/// Convert client::SearchResult into TUI ResultItem
impl From<(usize, &crate::client::SearchResult)> for ResultItem {
    fn from((index, result): (usize, &crate::client::SearchResult)) -> Self {
        Self {
            index,
            title: crate::render::Renderer::strip_emojis(&result.title),
            library: result.library.clone(),
            id: result.id.clone(),
            url: result.url.clone(),
            excerpt: crate::render::Renderer::strip_emojis(&result.excerpt),
            full_content: crate::render::Renderer::strip_emojis(
                result
                    .full_content
                    .as_deref()
                    .unwrap_or(&result.excerpt),
            ),
        }
    }
}
