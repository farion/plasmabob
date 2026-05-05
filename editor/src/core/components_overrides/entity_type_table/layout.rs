use bevy_egui::egui;

pub(super) struct TableLayout {
    pub name_col_w: f32,
    pub middle_col_w: f32,
    pub clear_col_w: f32,
    pub button_w: f32,
}

pub(super) fn compute_layout(
    ui: &egui::Ui,
    widths: &crate::core::ColumnWidths,
    max_name_chars: usize,
) -> TableLayout {
    let button_padding = 8.0f32;
    let button_w = 24.0f32;
    let clear_col_w = button_padding * 2.0 + button_w;
    let mut name_col_w = widths.widths.get(0).cloned().unwrap_or(80.0);
    let col_spacing = ui.spacing().item_spacing.x * 2.0 + 4.0;

    let ppp = ui.ctx().pixels_per_point();
    let avg_char_w = 7.0 * ppp;
    let desired_name_w = (max_name_chars as f32) * avg_char_w + 12.0;
    let max_allowed = (ui.available_width() - clear_col_w - col_spacing - 40.0).max(40.0);
    name_col_w = name_col_w.max(desired_name_w).min(max_allowed);

    let middle_col_w = (ui.available_width() - name_col_w - clear_col_w - col_spacing).max(40.0);

    TableLayout {
        name_col_w,
        middle_col_w,
        clear_col_w,
        button_w,
    }
}
