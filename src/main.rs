use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

use anyhow::{Context as AnyhowContext, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::{
    egui::{
        self, pos2, vec2, Align, CentralPanel, Color32, ComboBox, Context as EguiContext, FontData,
        FontDefinitions, FontFamily, Pos2, Rect, RichText, Shape, Slider, Stroke, TopBottomPanel,
    },
    CreationContext, NativeOptions,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use viboscop::audio::load_audio_file;
use viboscop::scope::TraceHistory;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ThemePreset {
    Graphite,
    Neon,
    Green,
    Amber,
    Monochrome,
}

impl ThemePreset {
    fn next(self) -> Self {
        match self {
            Self::Graphite => Self::Neon,
            Self::Neon => Self::Green,
            Self::Green => Self::Amber,
            Self::Amber => Self::Monochrome,
            Self::Monochrome => Self::Graphite,
        }
    }

    fn palette(self) -> ThemePalette {
        match self {
            Self::Graphite => ThemePalette {
                bg: Color32::from_rgb(14, 18, 22),
                panel: Color32::from_rgb(25, 30, 35),
                text: Color32::from_rgb(233, 239, 245),
                muted: Color32::from_rgb(154, 166, 175),
                trace: Color32::from_rgb(255, 180, 82),
                axis: Color32::from_rgb(150, 170, 182),
                grid: Color32::from_rgb(51, 62, 70),
                accent: Color32::from_rgb(104, 208, 162),
            },
            Self::Neon => ThemePalette {
                bg: Color32::from_rgb(7, 10, 18),
                panel: Color32::from_rgb(17, 22, 35),
                text: Color32::from_rgb(224, 246, 255),
                muted: Color32::from_rgb(104, 167, 188),
                trace: Color32::from_rgb(58, 255, 170),
                axis: Color32::from_rgb(88, 207, 255),
                grid: Color32::from_rgb(25, 46, 57),
                accent: Color32::from_rgb(255, 91, 214),
            },
            Self::Green => ThemePalette {
                bg: Color32::from_rgb(8, 16, 10),
                panel: Color32::from_rgb(17, 31, 20),
                text: Color32::from_rgb(210, 255, 220),
                muted: Color32::from_rgb(130, 198, 140),
                trace: Color32::from_rgb(104, 255, 150),
                axis: Color32::from_rgb(152, 214, 171),
                grid: Color32::from_rgb(27, 56, 38),
                accent: Color32::from_rgb(175, 255, 194),
            },
            Self::Amber => ThemePalette {
                bg: Color32::from_rgb(20, 14, 8),
                panel: Color32::from_rgb(37, 25, 16),
                text: Color32::from_rgb(255, 240, 210),
                muted: Color32::from_rgb(214, 171, 88),
                trace: Color32::from_rgb(255, 180, 71),
                axis: Color32::from_rgb(255, 201, 133),
                grid: Color32::from_rgb(85, 53, 23),
                accent: Color32::from_rgb(255, 144, 70),
            },
            Self::Monochrome => ThemePalette {
                bg: Color32::from_rgb(12, 12, 12),
                panel: Color32::from_rgb(26, 26, 26),
                text: Color32::from_rgb(236, 236, 236),
                muted: Color32::from_rgb(170, 170, 170),
                trace: Color32::from_rgb(210, 210, 210),
                axis: Color32::from_rgb(154, 154, 154),
                grid: Color32::from_rgb(58, 58, 58),
                accent: Color32::from_rgb(191, 191, 191),
            },
        }
    }
}

impl std::fmt::Display for ThemePreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Graphite => write!(f, "Graphite"),
            Self::Neon => write!(f, "Neon"),
            Self::Green => write!(f, "Green"),
            Self::Amber => write!(f, "Amber"),
            Self::Monochrome => write!(f, "Monochrome"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum TraceConnectionMode {
    Continuous,
    Adaptive,
    Points,
    Segmented,
}

impl std::fmt::Display for TraceConnectionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continuous => write!(f, "Continuous"),
            Self::Adaptive => write!(f, "Adaptive"),
            Self::Points => write!(f, "Points"),
            Self::Segmented => write!(f, "Segmented"),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct AppSettings {
    theme: ThemePreset,
    invert_x: bool,
    invert_y: bool,
    trace_connection: TraceConnectionMode,
    gap_threshold: f32,
    trace_thickness: f32,
    show_settings: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemePreset::Graphite,
            invert_x: false,
            invert_y: false,
            trace_connection: TraceConnectionMode::Adaptive,
            gap_threshold: 0.18,
            trace_thickness: 1.7,
            show_settings: false,
        }
    }
}

#[derive(Clone, Copy)]
struct ThemePalette {
    bg: Color32,
    panel: Color32,
    text: Color32,
    muted: Color32,
    trace: Color32,
    axis: Color32,
    grid: Color32,
    accent: Color32,
}

#[derive(Clone, Debug)]
struct LoadedTrack {
    _path: PathBuf,
    title: String,
    sample_rate: u32,
    samples: Vec<f32>,
}

struct PlaybackHandle {
    _stream: cpal::Stream,
    samples: Arc<[f32]>,
    playhead: Arc<AtomicUsize>,
    playing: Arc<AtomicBool>,
    looping: Arc<AtomicBool>,
    gain: Arc<std::sync::atomic::AtomicU32>,
    _history: Arc<Mutex<TraceHistory>>,
    sample_rate: u32,
}

impl PlaybackHandle {
    fn new(
        samples: Vec<f32>,
        sample_rate: u32,
        gain: f32,
        history: Arc<Mutex<TraceHistory>>,
    ) -> Result<Self> {
        let host = cpal::default_host();
        let output = host
            .default_output_device()
            .context("no audio output device was found")?;

        let stream_cfg = cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(sample_rate.max(8000)),
            buffer_size: cpal::BufferSize::Default,
        };

        let samples: Arc<[f32]> = Arc::from(samples);
        let playhead = Arc::new(AtomicUsize::new(0));
        let playing = Arc::new(AtomicBool::new(true));
        let looping = Arc::new(AtomicBool::new(false));
        let gain_bits = Arc::new(std::sync::atomic::AtomicU32::new(
            gain.clamp(0.0, 1.0).to_bits(),
        ));

        let playhead_cb = Arc::clone(&playhead);
        let playing_cb = Arc::clone(&playing);
        let looping_cb = Arc::clone(&looping);
        let gain_cb = Arc::clone(&gain_bits);
        let samples_cb = Arc::clone(&samples);
        let history_cb = Arc::clone(&history);

        let stream = output.build_output_stream(
            &stream_cfg,
            move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let gain_value = f32::from_bits(gain_cb.load(Ordering::Relaxed));
                let should_play = playing_cb.load(Ordering::Relaxed);
                let loop_enabled = looping_cb.load(Ordering::Relaxed);
                let total_frames = samples_cb.len() / 2;
                let mut current = playhead_cb.load(Ordering::Relaxed);
                let mut written = 0usize;

                while written < out.len() / 2 {
                    let offset = current * 2;
                    let left = samples_cb.get(offset).copied().unwrap_or(0.0);
                    let right = samples_cb.get(offset + 1).copied().unwrap_or(0.0);
                    let idx = written * 2;

                    out[idx] = if should_play { left * gain_value } else { 0.0 };
                    out[idx + 1] = if should_play { right * gain_value } else { 0.0 };

                    if should_play && left.is_finite() && right.is_finite() {
                        if let Ok(mut trace) = history_cb.lock() {
                            trace.push((left, right));
                        }
                    }

                    current += 1;
                    written += 1;

                    if current >= total_frames {
                        if loop_enabled {
                            current = 0;
                        } else {
                            playing_cb.store(false, Ordering::Relaxed);
                            current = total_frames;
                            break;
                        }
                    }
                }

                playhead_cb.store(current, Ordering::Relaxed);
            },
            |err| eprintln!("audio error: {err}"),
            None,
        )?;

        stream.play()?;

        Ok(Self {
            _stream: stream,
            samples,
            playhead,
            playing,
            looping,
            gain: gain_bits,
            _history: history,
            sample_rate,
        })
    }

    fn toggle_play(&self) {
        let current = self.playing.load(Ordering::Relaxed);
        self.playing.store(!current, Ordering::Relaxed);
    }

    fn set_gain(&self, gain: f32) {
        self.gain
            .store(gain.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    fn set_looping(&self, looping: bool) {
        self.looping.store(looping, Ordering::Relaxed);
    }

    fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    fn elapsed_seconds(&self) -> f32 {
        (self.playhead.load(Ordering::Relaxed) as f32) / self.sample_rate as f32
    }

    fn seek(&self, fraction: f32) {
        let total_frames = (self.samples.len() / 2).max(1);
        let target = (fraction.clamp(0.0, 1.0) * total_frames as f32) as usize;
        self.playhead.store(
            target.min(total_frames.saturating_sub(1)),
            Ordering::Relaxed,
        );
    }

    fn reset(&self) {
        self.playhead.store(0, Ordering::Relaxed);
        self.playing.store(true, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsTab {
    Appearance,
    Visualization,
    Tutorial,
    About,
}

fn settings_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/viboscop/settings.json"))
}

fn load_settings() -> AppSettings {
    let Some(path) = settings_path() else {
        return AppSettings::default();
    };
    let Ok(raw) = fs::read(path) else {
        return AppSettings::default();
    };
    match serde_json::from_slice(&raw) {
        Ok(value) => value,
        Err(_) => AppSettings::default(),
    }
}

fn save_settings(settings: &AppSettings) -> Result<()> {
    let Some(path) = settings_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let encoded = serde_json::to_string_pretty(settings)?;
    fs::write(path, encoded)?;
    Ok(())
}

fn configure_fonts(ctx: &EguiContext) {
    let mut fonts = FontDefinitions::default();
    let candidates = [
        "/usr/share/fonts/truetype/google-sans/GoogleSans-Regular.ttf",
        "/usr/share/fonts/truetype/GoogleSans-Regular.ttf",
        "/usr/share/fonts/TTF/GoogleSans-Regular.ttf",
        "/usr/share/fonts/opentype/google-sans/GoogleSans-Regular.otf",
    ];

    for candidate in candidates {
        if std::path::Path::new(candidate).exists() {
            let data = fs::read(candidate).unwrap_or_default();
            fonts.font_data.insert(
                "google_sans".to_owned(),
                Arc::new(FontData::from_owned(data)),
            );
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "google_sans".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

struct ViboscopApp {
    settings: AppSettings,
    settings_tab: SettingsTab,
    history: Arc<Mutex<TraceHistory>>,
    playback: Option<PlaybackHandle>,
    audio_name: String,
    file_error: Option<String>,
    is_loading: bool,
    load_rx: Option<mpsc::Receiver<Result<LoadedTrack, String>>>,
    sample_rate: u32,
    duration_seconds: f32,
    volume: f32,
    playing: bool,
    fullscreen: bool,
    x_scale: f32,
    y_scale: f32,
}

impl ViboscopApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let settings = load_settings();
        configure_fonts(&cc.egui_ctx);
        Self {
            settings,
            settings_tab: SettingsTab::Appearance,
            history: Arc::new(Mutex::new(TraceHistory::new(4096))),
            playback: None,
            audio_name: String::from("No track loaded"),
            file_error: None,
            is_loading: false,
            load_rx: None,
            sample_rate: 44100,
            duration_seconds: 0.0,
            volume: 0.3,
            playing: false,
            fullscreen: false,
            x_scale: 1.0,
            y_scale: 1.0,
        }
    }

    fn persist(&self) {
        if let Err(err) = save_settings(&self.settings) {
            eprintln!("could not save settings: {err}");
        }
    }

    fn open_file_dialog(&mut self) {
        let file = FileDialog::new()
            .add_filter("Audio", &["wav", "flac"])
            .set_directory(".")
            .pick_file();

        if let Some(path) = file {
            self.start_loading(path);
        }
    }

    fn start_loading(&mut self, path: PathBuf) {
        self.file_error = None;
        self.is_loading = true;
        self.audio_name = String::from("Loading…");
        self.playback = None;

        let (tx, rx) = mpsc::channel();
        let path_for_thread = path.clone();
        std::thread::spawn(move || {
            let result = load_audio_file(&path_for_thread)
                .map(|file| LoadedTrack {
                    _path: path_for_thread.clone(),
                    title: path_for_thread
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("track")
                        .to_owned(),
                    sample_rate: file.sample_rate,
                    samples: file.samples,
                })
                .map_err(|err| err.to_string());
            let _ = tx.send(result);
        });

        self.load_rx = Some(rx);
    }

    fn handle_loaded_track(&mut self, result: Result<LoadedTrack, String>) {
        self.is_loading = false;
        self.load_rx = None;

        match result {
            Ok(track) => {
                self.history.lock().unwrap().clear();
                self.audio_name = track.title.clone();
                self.sample_rate = track.sample_rate;
                self.duration_seconds = (track.samples.len() / 2) as f32 / track.sample_rate as f32;
                let history = Arc::clone(&self.history);
                self.playback =
                    PlaybackHandle::new(track.samples, track.sample_rate, self.volume, history)
                        .ok();
                if self.playback.is_some() {
                    self.playing = true;
                    self.file_error = None;
                    if let Some(playback) = &self.playback {
                        playback.set_looping(false);
                    }
                }
            }
            Err(err) => {
                self.file_error = Some(err);
                self.audio_name = String::from("No track loaded");
            }
        }
    }

    fn toggle_playback(&mut self) {
        if let Some(playback) = &self.playback {
            playback.toggle_play();
            self.playing = playback.is_playing();
        }
    }

    fn set_volume(&mut self, value: f32) {
        self.volume = value.clamp(0.0, 1.0);
        if let Some(playback) = &self.playback {
            playback.set_gain(self.volume);
        }
    }

    fn draw_scope(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect().shrink(12.0);
        let painter = ui.painter();
        let palette = self.settings.theme.palette();
        painter.rect_filled(rect, 12.0, palette.panel);

        let center = rect.center();
        painter.line_segment(
            [pos2(rect.left(), center.y), pos2(rect.right(), center.y)],
            Stroke::new(1.0_f32, palette.axis),
        );
        painter.line_segment(
            [pos2(center.x, rect.top()), pos2(center.x, rect.bottom())],
            Stroke::new(1.0_f32, palette.axis),
        );

        for i in 0..=6 {
            let t = i as f32 / 6.0;
            let x = rect.left() + t * rect.width();
            let y = rect.top() + t * rect.height();
            painter.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(0.5_f32, palette.grid),
            );
            painter.line_segment(
                [pos2(rect.left(), y), pos2(rect.right(), y)],
                Stroke::new(0.5_f32, palette.grid),
            );
        }

        let points = {
            let guard = self.history.lock().unwrap();
            let trace = guard.snapshot();
            let mut mapped = Vec::new();
            let half_w = rect.width() * 0.42;
            let half_h = rect.height() * 0.42;

            for (left, right) in trace {
                if !left.is_finite() || !right.is_finite() {
                    continue;
                }
                let x_value = if self.settings.invert_x { -left } else { left };
                let y_value = if self.settings.invert_y {
                    -right
                } else {
                    right
                };
                let px = center.x + x_value * self.x_scale * half_w;
                let py = center.y - y_value * self.y_scale * half_h;
                mapped.push(Pos2::new(
                    px.clamp(rect.left() + 2.0, rect.right() - 2.0),
                    py.clamp(rect.top() + 2.0, rect.bottom() - 2.0),
                ));
            }
            mapped
        };

        if self.settings.trace_connection == TraceConnectionMode::Points {
            for point in &points {
                painter.circle_filled(*point, 1.5_f32, palette.trace);
            }
        } else {
            let segments = build_segments(
                &points,
                self.settings.gap_threshold,
                self.settings.trace_connection,
                rect,
            );
            for (a, b) in segments {
                painter.add(Shape::line(
                    vec![a, b],
                    Stroke::new(self.settings.trace_thickness, palette.trace),
                ));
            }
        }

        if self.playback.is_none() && !self.is_loading {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Add an audio file to begin.",
                egui::FontId::proportional(20.0),
                palette.muted,
            );
        } else if self.is_loading {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Loading audio…",
                egui::FontId::proportional(20.0),
                palette.accent,
            );
        }
    }

    fn render_settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings.show_settings {
            return;
        }

        egui::Window::new("Settings").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for tab in [
                    SettingsTab::Appearance,
                    SettingsTab::Visualization,
                    SettingsTab::Tutorial,
                    SettingsTab::About,
                ] {
                    let selected = self.settings_tab == tab;
                    if ui.selectable_label(selected, format!("{tab:?}")).clicked() {
                        self.settings_tab = tab;
                    }
                }
            });

            ui.separator();
            match self.settings_tab {
                SettingsTab::Appearance => {
                    ui.label("Appearance");
                    ComboBox::from_label("Theme")
                        .selected_text(self.settings.theme.to_string())
                        .show_ui(ui, |ui| {
                            for theme in [
                                ThemePreset::Graphite,
                                ThemePreset::Amber,
                                ThemePreset::Green,
                                ThemePreset::Neon,
                                ThemePreset::Monochrome,
                            ] {
                                if ui.selectable_value(&mut self.settings.theme, theme, theme.to_string()).clicked() {
                                    self.persist();
                                }
                            }
                        });
                    if ui.button("Apply").clicked() {
                        self.persist();
                    }
                }
                SettingsTab::Visualization => {
                    ui.label("Visualization");
                    ui.checkbox(&mut self.settings.invert_x, "Invert X");
                    ui.checkbox(&mut self.settings.invert_y, "Invert Y");
                    ComboBox::from_label("Trace connection")
                        .selected_text(self.settings.trace_connection.to_string())
                        .show_ui(ui, |ui| {
                            for mode in [
                                TraceConnectionMode::Continuous,
                                TraceConnectionMode::Adaptive,
                                TraceConnectionMode::Points,
                                TraceConnectionMode::Segmented,
                            ] {
                                ui.selectable_value(&mut self.settings.trace_connection, mode, mode.to_string());
                            }
                        });
                    ui.add(Slider::new(&mut self.settings.gap_threshold, 0.0..=1.0).text("Gap threshold"));
                    ui.add(Slider::new(&mut self.settings.trace_thickness, 0.5..=4.0).text("Trace thickness"));
                    if ui.button("Apply").clicked() {
                        self.persist();
                    }
                }
                SettingsTab::Tutorial => {
                    ui.label("Tutorial");
                    ui.indent("tutorial", |ui| {
                        ui.monospace("1. Load audio: click Add File and choose a WAV or FLAC recording.");
                        ui.monospace("2. Play and seek: use the playback controls and progress bar.");
                        ui.monospace("3. XY mode: left channel drives X and right channel drives Y.");
                        ui.monospace("4. Invert axes: use Invert X and Invert Y in Visualization settings.");
                        ui.monospace("5. Adjust trace fidelity with Continuous, Adaptive, Points, or Segmented modes.");
                        ui.monospace("6. Switch theme presets to tune the dark interface while keeping the amber trace.");
                        ui.monospace("7. Use Fullscreen for a larger scope view.");
                    });
                }
                SettingsTab::About => {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Viboscop").size(24.0));
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.label("Stereo XY oscilloscope for WAV and FLAC files.");
                        ui.separator();
                        ui.label("Features: native desktop interface, WAV/FLAC audio playback, live XY scope, theme presets, fullscreen support.");
                        ui.label("License: MIT");
                    });
                }
            }
        });
    }
}

fn build_segments(
    points: &[Pos2],
    gap_threshold: f32,
    mode: TraceConnectionMode,
    rect: Rect,
) -> Vec<(Pos2, Pos2)> {
    let mut segments = Vec::new();
    if points.len() < 2 {
        return segments;
    }

    let max_gap = gap_threshold.clamp(0.0, 1.0) * rect.width().max(rect.height()) * 0.8 + 10.0;

    let mut previous = points[0];
    for current in points.iter().skip(1) {
        let dx = current.x - previous.x;
        let dy = current.y - previous.y;
        let dist = (dx * dx + dy * dy).sqrt();
        match mode {
            TraceConnectionMode::Continuous => segments.push((previous, *current)),
            TraceConnectionMode::Adaptive => {
                if dist <= max_gap {
                    segments.push((previous, *current));
                }
            }
            TraceConnectionMode::Points => {}
            TraceConnectionMode::Segmented => {
                if dist <= max_gap {
                    segments.push((previous, *current));
                }
            }
        }
        previous = *current;
    }
    segments
}

impl eframe::App for ViboscopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.load_rx {
            if let Ok(result) = rx.try_recv() {
                self.handle_loaded_track(result);
            }
        }

        if let Some(playback) = &self.playback {
            self.playing = playback.is_playing();
        }

        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Viboscop").size(22.0));
                ui.separator();
                if ui.button("Add File").clicked() {
                    self.open_file_dialog();
                }
                if ui
                    .button(if self.playing { "Pause" } else { "Play" })
                    .clicked()
                {
                    self.toggle_playback();
                }
                if ui.button("Restart").clicked() {
                    if let Some(playback) = &self.playback {
                        playback.reset();
                    }
                }
                if ui.button("Theme").clicked() {
                    self.settings.theme = self.settings.theme.next();
                    self.persist();
                }
                if ui.button("Settings").clicked() {
                    self.settings.show_settings = !self.settings.show_settings;
                    self.persist();
                }
                if ui
                    .button(if self.fullscreen {
                        "Window"
                    } else {
                        "Fullscreen"
                    })
                    .clicked()
                {
                    self.fullscreen = !self.fullscreen;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
                }
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    ui.add(Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
                    self.set_volume(self.volume);
                });
            });
        });

        TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let elapsed = self
                    .playback
                    .as_ref()
                    .map(|p| p.elapsed_seconds())
                    .unwrap_or(0.0);
                let total = self.duration_seconds.max(0.01);
                ui.label(format_duration(elapsed));

                let mut progress = (elapsed / total).clamp(0.0, 1.0);
                if ui.add(Slider::new(&mut progress, 0.0..=1.0)).changed() {
                    if let Some(playback) = &self.playback {
                        playback.seek(progress);
                    }
                }

                ui.label(format_duration(total));
                if ui.button("Loop").clicked() {
                    if let Some(playback) = &self.playback {
                        playback.set_looping(!playback.is_playing());
                    }
                }
                if ui.button("Clear Trace").clicked() {
                    self.history.lock().unwrap().clear();
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            let palette = self.settings.theme.palette();
            ui.set_min_height(380.0);
            ui.painter().rect_filled(ui.max_rect(), 16.0, palette.bg);
            self.draw_scope(ui);

            let title = RichText::new(&self.audio_name)
                .size(16.0)
                .color(palette.text);
            let info = RichText::new(format!(
                "{} Hz • {} • volume {}%",
                self.sample_rate,
                format_duration(self.duration_seconds),
                (self.volume * 100.0).round() as u32,
            ))
            .size(11.0)
            .color(palette.muted);

            let top_left = ui.max_rect().left_top() + vec2(16.0, 12.0);
            ui.painter().text(
                top_left,
                egui::Align2::LEFT_TOP,
                title.text().to_owned(),
                egui::FontId::proportional(16.0),
                palette.text,
            );
            ui.painter().text(
                top_left + vec2(0.0, 18.0),
                egui::Align2::LEFT_TOP,
                info.text().to_owned(),
                egui::FontId::proportional(11.0),
                palette.muted,
            );

            if let Some(err) = &self.file_error {
                ui.painter().text(
                    ui.max_rect().right_bottom() - vec2(16.0, 18.0),
                    egui::Align2::RIGHT_BOTTOM,
                    err.clone(),
                    egui::FontId::proportional(12.0),
                    Color32::from_rgb(255, 110, 110),
                );
            }
        });

        self.render_settings_window(ctx);
        ctx.request_repaint_after(Duration::from_millis(32));
    }
}

fn format_duration(total_seconds: f32) -> String {
    let total = total_seconds.max(0.0) as u64;
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Viboscop")
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([760.0, 420.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Viboscop",
        native_options,
        Box::new(|cc| Ok(Box::new(ViboscopApp::new(cc)))),
    )
}
