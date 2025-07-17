use eframe::egui;
use ytcli::application::DownloadVideoUseCase;
use ytcli::domain::Video;
use ytcli::infrastructure::YtDlpDownloader;

use std::sync::{Arc, Mutex};
use std::thread;

struct App {
    url: String,
    output: String,
    quality: String,
    audio_only: bool,
    file_type: String,
    status: String,
    progress: Arc<Mutex<f32>>,
    downloading: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            url: String::new(),
            output: String::new(),
            quality: "Best (Default)".to_string(),
            audio_only: false,
            file_type: "mp4".to_string(),
            status: String::new(),
            progress: Arc::new(Mutex::new(0.0)),
            downloading: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set a custom style for better appearance
        // macOS-inspired light theme
        let mut visuals = egui::Visuals::light();
        visuals.widgets.inactive.rounding = egui::Rounding::same(12.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(14.0);
        visuals.widgets.active.rounding = egui::Rounding::same(14.0);
        visuals.widgets.open.rounding = egui::Rounding::same(14.0);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(245, 245, 247); // macOS light background
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(220, 220, 230);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(180, 200, 255);
        visuals.window_fill = egui::Color32::from_rgb(248, 249, 250);
        visuals.extreme_bg_color = egui::Color32::from_rgb(255, 255, 255);
        visuals.faint_bg_color = egui::Color32::from_rgb(240, 240, 243);
        visuals.panel_fill = egui::Color32::from_rgb(255, 255, 255);
        visuals.override_text_color = Some(egui::Color32::from_rgb(30, 30, 30));
        visuals.selection.bg_fill = egui::Color32::from_rgb(180, 200, 255);
        visuals.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 120, 255));
        visuals.window_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 220, 220));

        
        ctx.set_visuals(visuals);
        // Use default font definitions (proportional by default)
        // Custom font loading can be done here if needed
        ctx.set_style({
            let mut style = (*ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(16.0, 18.0);
            style.spacing.button_padding = egui::vec2(18.0, 10.0);
            style.spacing.window_margin = egui::Margin::symmetric(32.0, 32.0);
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::proportional(32.0)),
                (egui::TextStyle::Body, egui::FontId::proportional(18.0)),
                (egui::TextStyle::Button, egui::FontId::proportional(18.0)),
                (egui::TextStyle::Small, egui::FontId::proportional(14.0)),
            ].into();
            style
        });

        egui::CentralPanel::default().frame(egui::Frame::none().fill(egui::Color32::from_rgb(248, 249, 250))).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.heading(egui::RichText::new(" YouTube Downloader")
                    .font(egui::FontId::proportional(36.0))
                    .color(egui::Color32::from_rgb(0, 122, 255)));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("A clean, simple YouTube downloader for macOS").size(18.0).color(egui::Color32::from_gray(120)));
                ui.add_space(20.0);
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(12.0);
            
            // URL input with icon
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔗 URL:").size(16.0).strong());
                ui.add_space(10.0);
                let width = (ui.available_width() - 120.0).clamp(180.0, 700.0);
                ui.add_sized([
                    width, 32.0], egui::TextEdit::singleline(&mut self.url)
                    .hint_text("Paste a YouTube URL...")
                    .font(egui::TextStyle::Body)
                    .margin(egui::Margin::symmetric(8.0, 8.0))
                );
            });
            
            ui.add_space(10.0);
            
            // Output filename with icon
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📁 Output:").size(16.0).strong());
                ui.add_space(10.0);
                let width = (ui.available_width() - 120.0).clamp(180.0, 700.0);
                ui.add_sized([
                    width, 32.0], egui::TextEdit::singleline(&mut self.output)
                    .hint_text("Optional: custom filename (without extension)")
                    .font(egui::TextStyle::Body)
                    .margin(egui::Margin::symmetric(8.0, 8.0))
                );
            });
            
            ui.add_space(10.0);
            
            // Quality dropdown
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚙️ Quality:").size(16.0).strong());
                ui.add_space(10.0);
                let width = (ui.available_width() - 120.0).clamp(120.0, 700.0);
                egui::ComboBox::from_id_source("quality")
                    .selected_text(&self.quality)
                    .width(width)
                    .wrap(true)
                    .show_ui(ui, |ui| {
                        let qualities = [
                            "Best (Default)",
                            "Worst",
                            "720p",
                            "480p",
                            "360p",
                            "240p",
                            "144p",
                        ];
                        for quality in qualities {
                            ui.selectable_value(&mut self.quality, quality.to_string(), quality);
                        }
                    });
            });
            
            ui.add_space(10.0);
            
            // File type dropdown with better styling
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📄 File Type:").size(16.0).strong());
                ui.add_space(10.0);
                let width = (ui.available_width() - 120.0).clamp(120.0, 700.0);
                egui::ComboBox::from_id_source("file_type")
                    .selected_text(&self.file_type.to_uppercase())
                    .width(width)
                    .wrap(true)
                    .show_ui(ui, |ui| {
                        for ft in ["mp4", "webm", "mp3", "m4a", "wav"] {
                            ui.selectable_value(&mut self.file_type, ft.to_string(), ft.to_uppercase());
                        }
                    });
            });
            
            ui.add_space(10.0);
            
            // Audio only checkbox with better styling
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.audio_only, egui::RichText::new("🎵 Audio Only").size(16.0));
            });
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(15.0);
            
            // Download button with custom styling
            ui.vertical_centered(|ui| {
                let btn_width = ui.available_width().clamp(120.0, 350.0);
                let download_button = egui::Button::new(
                    egui::RichText::new("⬇️ Download")
                        .size(20.0)
                        .color(egui::Color32::WHITE)
                )
                .fill(egui::Color32::from_rgb(0, 122, 255))
                .min_size(egui::vec2(btn_width, 48.0));
                if ui.add(download_button).clicked() && !self.downloading {
                    self.status = "⏳ Starting download...".to_string();
                    self.downloading = true;
                    let progress = self.progress.clone();
                    *progress.lock().unwrap() = 0.0;
                    let url = self.url.clone();
                    let output = self.output.clone();
                    let audio_only = self.audio_only;
                    let file_type = self.file_type.clone();
                    let status_arc = Arc::new(Mutex::new(String::new()));
                    let status_arc_clone = status_arc.clone();
                    // Convert display quality to yt-dlp format OUTSIDE the thread
                    let quality_value = match self.quality.as_str() {
                        "Best (Default)" => "bv*+ba/b",
                        "Worst" => "worst",
                        other => other,
                    };
                    let quality_value = quality_value.to_string();
                    thread::spawn(move || {
                        let video = Video {
                            url,
                            output: if output.is_empty() { None } else { Some(output) },
                            quality: quality_value,
                            audio_only,
                            file_type: Some(file_type),
                        };
                        let downloader = YtDlpDownloader;
                        let progress = progress.clone();
                        let result = downloader.download_with_progress(&video, move |p| {
                            let mut prog = progress.lock().unwrap();
                            *prog = p;
                        });
                        let mut status = status_arc_clone.lock().unwrap();
                        *status = match result {
                            Ok(()) => "✅ Download complete!".to_string(),
                            Err(e) => format!("❌ Download failed: {}", e),
                        };
                    });
                    self.status = "⏳ Downloading...".to_string();
                }
            });
            
            ui.add_space(15.0);
            
            // Progress bar
            let progress = *self.progress.lock().unwrap();
            if self.downloading {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    let pb_width = ui.available_width().clamp(120.0, 700.0);
                    ui.add(egui::ProgressBar::new(progress)
                        .desired_width(pb_width)
                        .show_percentage()
                        .text(format!("{:.1}%", progress * 100.0))
                        .fill(egui::Color32::from_rgb(0, 122, 255))
                    );
                });
                if progress >= 1.0 {
                    self.downloading = false;
                    self.status = "✅ Download complete!".to_string();
                }
            }
            // Status message with styling
            if !self.status.is_empty() {
                ui.vertical_centered(|ui| {
                    let color = if self.status.contains("✅") {
                        egui::Color32::from_rgb(40, 180, 60)
                    } else if self.status.contains("❌") {
                        egui::Color32::from_rgb(220, 20, 60)
                    } else {
                        egui::Color32::from_rgb(0, 122, 255)
                    };
                    ui.label(egui::RichText::new(&self.status)
                        .size(18.0)
                        .color(color)
                        .strong());
                });
            }
            
            ui.add_space(20.0);
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_min_inner_size([500.0, 400.0])
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "YouTube Downloader GUI",
        native_options,
        Box::new(|_cc| Box::new(App::default())),
    )
}
