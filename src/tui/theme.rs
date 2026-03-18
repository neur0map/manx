use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub header_title: Style,
    pub header_subtitle: Style,
    pub result_number: Style,
    pub result_title: Style,
    pub result_title_selected: Style,
    pub result_library: Style,
    pub result_id: Style,
    pub result_url: Style,
    pub result_excerpt: Style,
    pub code_block: Style,
    pub code_keyword: Style,
    pub code_string: Style,
    pub code_comment: Style,
    pub section_heading: Style,
    pub border: Style,
    pub border_focused: Style,
    pub footer_key: Style,
    pub footer_desc: Style,
    pub loading: Style,
    pub success: Style,
    pub error: Style,
    pub warning: Style,
    pub dimmed: Style,
    pub highlight_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            header_title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            header_subtitle: Style::default().fg(Color::DarkGray),
            result_number: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            result_title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            result_title_selected: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            result_library: Style::default().fg(Color::DarkGray),
            result_id: Style::default().fg(Color::Yellow),
            result_url: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED),
            result_excerpt: Style::default().fg(Color::Gray),
            code_block: Style::default().fg(Color::Green),
            code_keyword: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            code_string: Style::default().fg(Color::Green),
            code_comment: Style::default().fg(Color::DarkGray),
            section_heading: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::DarkGray),
            border_focused: Style::default().fg(Color::Cyan),
            footer_key: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            footer_desc: Style::default().fg(Color::DarkGray),
            loading: Style::default().fg(Color::Cyan),
            success: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            error: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            warning: Style::default().fg(Color::Yellow),
            dimmed: Style::default().fg(Color::DarkGray),
            highlight_bg: Color::Rgb(30, 40, 55),
        }
    }
}
