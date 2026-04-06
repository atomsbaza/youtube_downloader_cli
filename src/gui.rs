use eframe::egui;
use ytcli::domain::Video;
use ytcli::infrastructure::{DownloadProgress, YtDlpDownloader};

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if yt-dlp is installed and responds to --version.
fn check_ytdlp_available() -> bool {
    Command::new("yt-dlp")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Validate a URL before handing it to yt-dlp.
fn validate_url(url: &str) -> Result<(), String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("Please enter a YouTube URL.".to_string());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }
    let known = ["youtube.com", "youtu.be"];
    if !known.iter().any(|d| url.contains(d)) {
        return Err(
            "URL does not look like a YouTube URL.\nSupported domains: youtube.com, youtu.be"
                .to_string(),
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

struct App {
    url: String,
    output: String,
    save_dir: String,
    quality: String,
    audio_only: bool,
    file_type: String,
    playlist_items: String,
    status: String,
    /// 0.0..=1.0, written by download thread
    progress: Arc<Mutex<f32>>,
    /// Written by download thread when it finishes (Ok or Err)
    download_result: Arc<Mutex<Option<Result<(), String>>>>,
    /// Set to true to cancel; reset to false when a new download starts
    cancel_flag: Arc<AtomicBool>,
    /// Live speed string from yt-dlp, e.g. "1.23MiB/s"
    speed: Arc<Mutex<String>>,
    /// Live ETA string from yt-dlp, e.g. "00:30"
    eta: Arc<Mutex<String>>,
    downloading: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            url: String::new(),
            output: String::new(),
            save_dir: String::new(),
            quality: "Best (Default)".to_string(),
            audio_only: false,
            file_type: "mp4".to_string(),
            playlist_items: String::new(),
            status: String::new(),
            progress: Arc::new(Mutex::new(0.0)),
            download_result: Arc::new(Mutex::new(None)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            speed: Arc::new(Mutex::new(String::new())),
            eta: Arc::new(Mutex::new(String::new())),
            downloading: false,
        }
    }
}

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll the download thread result
        if self.downloading {
            if let Ok(mut guard) = self.download_result.try_lock() {
                if let Some(result) = guard.take() {
                    self.downloading = false;
                    self.status = match result {
                        Ok(()) => "✅ Download complete!".to_string(),
                        Err(ref e) if e == "Download cancelled" => "⏹ Download cancelled.".to_string(),
                        Err(e) => format!("❌ {}", e),
                    };
                }
            }
            ctx.request_repaint();
        }

        // Theme
        let mut visuals = egui::Visuals::light();
        visuals.widgets.inactive.rounding = egui::Rounding::same(12.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(14.0);
        visuals.widgets.active.rounding = egui::Rounding::same(14.0);
        visuals.widgets.open.rounding = egui::Rounding::same(14.0);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(245, 245, 247);
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
            ]
            .into();
            style
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(248, 249, 250)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.heading(
                        egui::RichText::new(" YouTube Downloader")
                            .font(egui::FontId::proportional(36.0))
                            .color(egui::Color32::from_rgb(0, 122, 255)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("A clean, simple YouTube downloader")
                            .size(18.0)
                            .color(egui::Color32::from_gray(120)),
                    );
                    ui.add_space(20.0);
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(12.0);

                // URL
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔗 URL:").size(16.0).strong());
                    ui.add_space(10.0);
                    let width = (ui.available_width() - 120.0).clamp(180.0, 700.0);
                    ui.add_sized(
                        [width, 32.0],
                        egui::TextEdit::singleline(&mut self.url)
                            .hint_text("Paste a YouTube URL...")
                            .font(egui::TextStyle::Body)
                            .margin(egui::Margin::symmetric(8.0, 8.0)),
                    );
                });

                ui.add_space(10.0);

                // Save directory
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📂 Save to:").size(16.0).strong());
                    ui.add_space(10.0);
                    let width = (ui.available_width() - 200.0).clamp(100.0, 560.0);
                    ui.add_sized(
                        [width, 32.0],
                        egui::TextEdit::singleline(&mut self.save_dir)
                            .hint_text("Save directory (default: current folder)")
                            .font(egui::TextStyle::Body)
                            .margin(egui::Margin::symmetric(8.0, 8.0)),
                    );
                    if ui.button("Browse…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.save_dir = path.to_string_lossy().to_string();
                        }
                    }
                });

                ui.add_space(10.0);

                // Output filename
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📁 Filename:").size(16.0).strong());
                    ui.add_space(10.0);
                    let width = (ui.available_width() - 120.0).clamp(180.0, 700.0);
                    ui.add_sized(
                        [width, 32.0],
                        egui::TextEdit::singleline(&mut self.output)
                            .hint_text("Optional: custom filename (without extension)")
                            .font(egui::TextStyle::Body)
                            .margin(egui::Margin::symmetric(8.0, 8.0)),
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
                        .show_ui(ui, |ui| {
                            for q in ["Best (Default)", "Worst", "720p", "480p", "360p", "240p", "144p"] {
                                ui.selectable_value(&mut self.quality, q.to_string(), q);
                            }
                        });
                });

                ui.add_space(10.0);

                // File type dropdown
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📄 File Type:").size(16.0).strong());
                    ui.add_space(10.0);
                    let width = (ui.available_width() - 120.0).clamp(120.0, 700.0);
                    egui::ComboBox::from_id_source("file_type")
                        .selected_text(self.file_type.to_uppercase())
                        .width(width)
                        .show_ui(ui, |ui| {
                            for ft in ["mp4", "webm", "mp3", "m4a", "wav"] {
                                ui.selectable_value(
                                    &mut self.file_type,
                                    ft.to_string(),
                                    ft.to_uppercase(),
                                );
                            }
                        });
                });

                ui.add_space(10.0);

                // Audio only checkbox
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.audio_only,
                        egui::RichText::new("🎵 Audio Only").size(16.0),
                    );
                });

                ui.add_space(10.0);

                // Playlist items
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📋 Playlist Items:").size(16.0).strong());
                    ui.add_space(10.0);
                    let width = (ui.available_width() - 120.0).clamp(180.0, 700.0);
                    ui.add_sized(
                        [width, 32.0],
                        egui::TextEdit::singleline(&mut self.playlist_items)
                            .hint_text("e.g. 1,3,5-7  (leave blank for full playlist/single video)")
                            .font(egui::TextStyle::Body)
                            .margin(egui::Margin::symmetric(8.0, 8.0)),
                    );
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(15.0);

                // Download / Cancel buttons
                ui.vertical_centered(|ui| {
                    let btn_width = ui.available_width().clamp(120.0, 350.0);

                    let download_button = egui::Button::new(
                        egui::RichText::new("⬇️ Download")
                            .size(20.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(0, 122, 255))
                    .min_size(egui::vec2(btn_width, 48.0));

                    if ui.add_enabled(!self.downloading, download_button).clicked() {
                        match validate_url(&self.url) {
                            Err(msg) => {
                                self.status = format!("❌ {}", msg);
                            }
                            Ok(()) => {
                                self.status = "⏳ Starting download...".to_string();
                                self.downloading = true;

                                // Reset shared state
                                *self.progress.lock().expect("progress lock poisoned") = 0.0;
                                *self.download_result.lock().expect("result lock poisoned") = None;
                                *self.speed.lock().expect("speed lock poisoned") = String::new();
                                *self.eta.lock().expect("eta lock poisoned") = String::new();
                                self.cancel_flag.store(false, Ordering::Relaxed);

                                let progress = self.progress.clone();
                                let download_result = self.download_result.clone();
                                let cancel_flag = self.cancel_flag.clone();
                                let speed_arc = self.speed.clone();
                                let eta_arc = self.eta.clone();
                                let url = self.url.clone();
                                let output = self.output.clone();
                                let save_dir = self.save_dir.clone();
                                let audio_only = self.audio_only;
                                let file_type = self.file_type.clone();
                                let playlist_items = self.playlist_items.clone();
                                let quality_value = match self.quality.as_str() {
                                    "Best (Default)" => "bv*+ba/b",
                                    "Worst" => "worst",
                                    other => other,
                                }
                                .to_string();

                                thread::spawn(move || {
                                    let video = Video {
                                        url,
                                        output: if output.is_empty() { None } else { Some(output) },
                                        save_dir: if save_dir.is_empty() { None } else { Some(save_dir) },
                                        quality: quality_value,
                                        audio_only,
                                        file_type: Some(file_type),
                                        playlist_start: None,
                                        playlist_end: None,
                                        playlist_items: if playlist_items.is_empty() {
                                            None
                                        } else {
                                            Some(playlist_items)
                                        },
                                    };
                                    let downloader = YtDlpDownloader;
                                    let result = downloader.download_with_progress(
                                        &video,
                                        cancel_flag,
                                        move |p: DownloadProgress| {
                                            *progress.lock().expect("progress lock poisoned") = p.percent;
                                            *speed_arc.lock().expect("speed lock poisoned") = p.speed;
                                            *eta_arc.lock().expect("eta lock poisoned") = p.eta;
                                        },
                                    );
                                    *download_result.lock().expect("result lock poisoned") =
                                        Some(result);
                                });
                            }
                        }
                    }

                    // Cancel button — only visible while downloading
                    if self.downloading {
                        ui.add_space(10.0);
                        let cancel_button = egui::Button::new(
                            egui::RichText::new("⏹ Cancel")
                                .size(18.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(200, 40, 40))
                        .min_size(egui::vec2(btn_width * 0.5, 38.0));

                        if ui.add(cancel_button).clicked() {
                            self.cancel_flag.store(true, Ordering::Relaxed);
                            self.status = "⏳ Cancelling...".to_string();
                        }
                    }
                });

                ui.add_space(15.0);

                // Progress bar + speed/ETA
                if self.downloading {
                    let progress = *self.progress.lock().expect("progress lock poisoned");
                    let speed = self.speed.lock().expect("speed lock poisoned").clone();
                    let eta = self.eta.lock().expect("eta lock poisoned").clone();

                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        let pb_width = ui.available_width().clamp(120.0, 700.0);
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .desired_width(pb_width)
                                .show_percentage()
                                .fill(egui::Color32::from_rgb(0, 122, 255)),
                        );

                        if !speed.is_empty() || !eta.is_empty() {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if !speed.is_empty() {
                                    ui.label(
                                        egui::RichText::new(format!("⚡ {}", speed))
                                            .size(14.0)
                                            .color(egui::Color32::from_gray(100)),
                                    );
                                }
                                if !eta.is_empty() {
                                    ui.label(
                                        egui::RichText::new(format!("⏱ ETA {}", eta))
                                            .size(14.0)
                                            .color(egui::Color32::from_gray(100)),
                                    );
                                }
                            });
                        }
                    });
                }

                // Status message
                if !self.status.is_empty() {
                    ui.add_space(12.0);
                    ui.vertical_centered(|ui| {
                        let color = if self.status.contains("✅") {
                            egui::Color32::from_rgb(40, 180, 60)
                        } else if self.status.contains("❌") {
                            egui::Color32::from_rgb(220, 20, 60)
                        } else if self.status.contains("⏹") {
                            egui::Color32::from_rgb(150, 100, 0)
                        } else {
                            egui::Color32::from_rgb(0, 122, 255)
                        };
                        ui.label(
                            egui::RichText::new(&self.status)
                                .size(16.0)
                                .color(color)
                                .strong(),
                        );
                    });
                }

                ui.add_space(20.0);
            });
    }
}

fn main() -> Result<(), eframe::Error> {
    // Guard: yt-dlp must be available before we launch the UI
    if !check_ytdlp_available() {
        rfd::MessageDialog::new()
            .set_title("yt-dlp not found")
            .set_description(
                "yt-dlp is not installed or not in PATH.\n\n\
                Install it with:\n\
                  pip install yt-dlp\n\n\
                Then relaunch this application.",
            )
            .set_level(rfd::MessageLevel::Error)
            .show();
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 640.0])
            .with_min_inner_size([520.0, 520.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "YouTube Downloader",
        native_options,
        Box::new(|_cc| Box::new(App::default())),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_empty() {
        assert!(validate_url("").is_err());
        assert!(validate_url("   ").is_err());
    }

    #[test]
    fn test_validate_url_no_scheme() {
        assert!(validate_url("youtube.com/watch?v=abc").is_err());
    }

    #[test]
    fn test_validate_url_non_youtube() {
        assert!(validate_url("https://vimeo.com/123456").is_err());
        assert!(validate_url("https://example.com").is_err());
    }

    #[test]
    fn test_validate_url_valid_youtube() {
        assert!(validate_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_ok());
        assert!(validate_url("https://youtu.be/dQw4w9WgXcQ").is_ok());
        assert!(validate_url("https://youtube.com/playlist?list=PL123").is_ok());
        assert!(validate_url("https://music.youtube.com/watch?v=abc").is_ok());
    }

    #[test]
    fn test_validate_url_http_allowed() {
        assert!(validate_url("http://youtube.com/watch?v=abc").is_ok());
    }
}
