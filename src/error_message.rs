pub struct ErrorInfo {
    error_title: String,
    error_message: String,
    severity: ErrorSeverity,
}

#[derive(Debug)]
pub enum ErrorSeverity {
    Minimal,  // Can continue
    Critical, // Cannot continue
    None,     // Shit is fucked
}

impl Default for ErrorInfo {
    fn default() -> Self {
        Self {
            error_title: "Unknown Error!".to_owned(),
            error_message: "Error determining the error message!\nSeverity: None".to_owned(),
            severity: ErrorSeverity::None,
        }
    }
}

impl ErrorInfo {
    pub fn new(error_title: String, error_message: String, severity: ErrorSeverity) -> Self {
        Self {
            error_title,
            error_message,
            severity,
        }
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        let window = egui::Window::new(format!("Error: {}", &self.error_title))
            .title_bar(true)
            .enabled(true)
            .id(egui::Id::new(format!(
                "{}{}",
                self.error_title, self.error_message
            )));
        window.show(ctx, |ui| self.ui(ctx, ui));
    }

    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(&self.error_message);
            ui.label(format!("Severity: {:?}", &self.severity));
            if ui.button("Close Program").clicked() {
                match &self.severity {
                    _ => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                }
            }
        });
    }
}
