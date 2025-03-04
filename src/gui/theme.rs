use eframe::egui::{Color32, Visuals, Stroke, Rounding, Style};

// Define color theme for the application
pub struct AppTheme {
    pub background: Color32,
    pub accent: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub button_text: Color32,
    pub button_normal: Color32,
    pub button_hovered: Color32,
    pub button_active: Color32,
    pub button_selected: Color32,
    pub error: Color32,
    pub success: Color32,
    pub tab_active: Color32,
    pub tab_inactive: Color32,
    pub separator: Color32,
    pub header_bg: Color32,
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme {
            background: Color32::from_rgb(248, 248, 248), // Off-white background
            accent: Color32::from_rgb(255, 140, 0),       // Orange accent (#FF8C00)
            text_primary: Color32::from_rgb(20, 20, 20),  // Near black text
            text_secondary: Color32::from_rgb(100, 100, 100), // Gray text
            button_text: Color32::from_rgb(240, 240, 255), // Light text for buttons that's easier to read
            button_normal: Color32::from_rgb(30, 144, 255), // Blue buttons (#1E90FF)
            button_hovered: Color32::from_rgb(255, 140, 0), // Orange when hovered (#FF8C00)
            button_active: Color32::from_rgb(0, 84, 195), // Darker blue when clicked
            button_selected: Color32::from_rgb(255, 165, 0), // Brighter orange for selected state
            error: Color32::from_rgb(220, 50, 50),        // Red for errors
            success: Color32::from_rgb(50, 180, 50),      // Green for success
            tab_active: Color32::from_rgb(255, 140, 0),   // Orange for active tab
            tab_inactive: Color32::from_rgb(200, 200, 200), // Light gray for inactive tab
            separator: Color32::from_rgb(220, 220, 220),  // Light gray for separators
            header_bg: Color32::from_rgb(240, 240, 240),  // Slightly darker background for headers
        }
    }
}

impl AppTheme {
    // Apply theme to egui context
    pub fn apply_to_context(&self, ctx: &eframe::egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Set visuals
        let mut visuals = Visuals::light();
        visuals.override_text_color = Some(self.text_primary);
        visuals.widgets.noninteractive.bg_fill = self.background;
        visuals.widgets.inactive.bg_fill = self.button_normal;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.button_text);
        visuals.widgets.hovered.bg_fill = self.button_hovered;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, self.button_text);
        visuals.widgets.active.bg_fill = self.button_active;
        visuals.widgets.active.fg_stroke = Stroke::new(2.0, self.button_text);
        
        // Set button rounding
        style.visuals.widgets.noninteractive.rounding = Rounding::same(5.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(5.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(5.0);
        style.visuals.widgets.active.rounding = Rounding::same(5.0);
        
        // Apply the style
        ctx.set_style(style);
    }
}
