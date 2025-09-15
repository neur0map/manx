use console::Style;
use dialoguer::theme::ColorfulTheme;

pub fn create_theme() -> ColorfulTheme {
    ColorfulTheme {
        defaults_style: Style::new().cyan().bright(),
        prompt_style: Style::new().bold(),
        prompt_prefix: Style::new().cyan().bold().apply_to("?".to_string()),
        prompt_suffix: Style::new().bold().apply_to(":".to_string()),
        success_prefix: Style::new().green().bold().apply_to("✓".to_string()),
        success_suffix: Style::new().bold().apply_to(":".to_string()),
        error_prefix: Style::new().red().bold().apply_to("✗ Error:".to_string()),
        error_style: Style::new().red(),
        hint_style: Style::new().dim(),
        values_style: Style::new().green(),
        active_item_style: Style::new().cyan().bold(),
        inactive_item_style: Style::new(),
        active_item_prefix: Style::new().cyan().bold().apply_to("●".to_string()),
        inactive_item_prefix: Style::new().apply_to("○".to_string()),
        checked_item_prefix: Style::new().green().bold().apply_to("✓".to_string()),
        unchecked_item_prefix: Style::new().apply_to("○".to_string()),
        picked_item_prefix: Style::new().green().bold().apply_to("✓".to_string()),
        unpicked_item_prefix: Style::new().apply_to(" ".to_string()),
    }
}
