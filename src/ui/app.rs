use edio11::{input::InputResult, Overlay, WindowMessage, WindowProcessOptions};
use egui::Key;
use egui::KeyboardShortcut;
use egui::Label;
use egui::Modifiers;
use egui::Stroke;
use egui::TextEdit;
use egui::{
    epaint::text::{FontInsert, InsertFontFamily},
    CentralPanel, Color32, Context, Frame, Slider, Window,
};
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::{Input::KeyboardAndMouse::VK_MENU, WindowsAndMessaging::WM_KEYDOWN},
};

use egui::{FontId, FontFamily, TextStyle,};
use std::collections::BTreeMap;


#[derive(Default, PartialEq)]
pub enum GraphUnit {
    #[default]
    Turn,
    ActionValue,
}

#[derive(Default)]
pub struct App {
    pub show_menu: bool,
    pub show_console: bool,
    show_damage_distribution: bool,
    show_damage_bars: bool,
    show_real_time_damage: bool,
    show_enemy_stats: bool,
    show_av_metrics: bool,
    widget_opacity: f32,
    pub graph_x_unit: GraphUnit,
    pub should_hide: bool,
    streamer_mode: bool,
    streamer_msg: String,
}

pub const HIDE_UI: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::H);
pub const SHOW_MENU: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::M);

impl Overlay for App {
    fn update(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_shortcut(&HIDE_UI)) {
            self.should_hide = !self.should_hide;
        }

        if self.streamer_mode {
            egui::TopBottomPanel::bottom("statusbar")
                .resizable(true)
                .show(ctx, |ui| {
                    let label = Label::new(&self.streamer_msg).selectable(false);

                    ui.add(label);
                    ui.allocate_space(ui.available_size())
                });
        }

        if !self.should_hide {
            if self.show_menu {
                CentralPanel::default()
                    .frame(Frame {
                        fill: Color32::GRAY.gamma_multiply(0.25),
                        ..Default::default()
                    })
                    .show(ctx, |_ui: &mut egui::Ui| {
                        Window::new(t!("Overlay Menu"))
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.heading(t!("Widget Controls"));

                                    ui.checkbox(&mut self.streamer_mode, t!("Streamer Mode"));
                                    ui.checkbox(&mut self.show_console, t!("Show Logs"));
                                    ui.checkbox(
                                        &mut self.show_damage_distribution,
                                        t!("Show Damage Distribution"),
                                    );
                                    ui.checkbox(&mut self.show_damage_bars, t!("Show Damage Bars"));
                                    ui.checkbox(
                                        &mut self.show_real_time_damage,
                                        t!("Show Real-Time Damage"),
                                    );
                                    ui.checkbox(&mut self.show_enemy_stats, t!("Show Enemy Stats"));

                                    ui.checkbox(&mut self.show_av_metrics, t!("Show AV Metrics"));

                                    ui.separator();
                                    ui.label(t!("Window Opacity"));
                                    ui.add_sized(
                                        [ui.available_width()*0.8, ui.spacing().interact_size.y], 
                                        Slider::new(&mut self.widget_opacity, 0.0..=1.0).text("")
                                    );

                                    ui.separator();
                                    ui.label(t!("Streamer Message"));
                                    ui.add(TextEdit::singleline(&mut self.streamer_msg));

                                    ui.separator();
                                    ui.label(t!("DamageLogger Panel"));
                                    ui.add(TextEdit::multiline(&mut t!("http://localhost:1305/static/")));
                                    
                                    ui.separator();
                                    ui.label(t!("Battle Timeline Panel"));
                                    ui.add(TextEdit::multiline(&mut t!("http://localhost:1305/static/battle_timeline.html")));

                                    ui.separator();
                                    if ui.button(t!("Close Menu")).clicked() {
                                        self.show_menu = false;
                                    }
                                });
                            });
                    });
            }

            if self.show_console {
                egui::Window::new(t!("Log"))
                    .resizable(true)
                    .default_height(300.0)
                    .default_width(400.0)
                    .min_width(200.0)
                    .min_height(100.0)
                    .show(ctx, |ui| {
                        let available = ui.available_size();
                        ui.set_min_size(available);
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            egui_logger::logger_ui().show(ui);
                        });
                    });
            }

            let opacity = self.widget_opacity.clamp(0.0, 1.0);
            let window_frame = egui::Frame::new()
                .fill(Color32::from_black_alpha((255.0 * opacity) as u8))
                .stroke(Stroke::new(0.5, Color32::WHITE))
                .inner_margin(8.0)
                .corner_radius(10.0);

            let transparent_frame = egui::Frame::new()
                .stroke(Stroke::new(0.5, Color32::WHITE))
                .inner_margin(8.0)
                .corner_radius(10.0);

            if self.show_damage_distribution {
                // 窗口title未设置，设置为Damage Ratio
                egui::containers::Window::new("Damage Ratio")
                    .frame(transparent_frame)
                    .resizable(true)
                    .min_width(200.0)
                    .min_height(200.0)
                    .show(ctx, |ui| {
                        self.show_damage_distribution_widget(ui);
                    });
            }

            if self.show_damage_bars {
                egui::containers::Window::new(t!("Damage by Character"))
                    .frame(window_frame)
                    .resizable(true)
                    .min_width(200.0)
                    .min_height(200.0)
                    .show(ctx, |ui| {
                        self.show_damage_bar_widget(ui);
                    });
            }

            if self.show_real_time_damage {
                egui::containers::Window::new(t!("Real-Time Damage"))
                    .frame(window_frame)
                    .resizable(true)
                    .min_width(200.0)
                    .min_height(200.0)
                    .show(ctx, |ui| {
                        self.show_real_time_damage_graph(ui);
                    });
            }

            if self.show_av_metrics {
                egui::containers::Window::new(t!("Action Value Metrics"))
                    .frame(window_frame)
                    .resizable(true)
                    .min_width(200.0)
                    .min_height(150.0)
                    .show(ctx, |ui| {
                        self.show_av_metrics(ui);
                    });
            }

            if self.show_enemy_stats {
                egui::containers::Window::new(t!("Enemy Stats"))
                    .frame(window_frame)
                    .resizable(true)
                    .min_width(200.0)
                    .min_height(150.0)
                    .show(ctx, |ui| {
                        self.show_enemy_stats(ui);
                    });
            }
        }
    }

    fn window_process(
        &mut self,
        input: &InputResult,
        input_events: &Vec<egui::Event>,
    ) -> Option<WindowProcessOptions> {
        // Refactor later
        if let InputResult::Key = input {
            for e in input_events {
                if let egui::Event::Key {
                    key,
                    physical_key: _,
                    pressed,
                    repeat: _,
                    modifiers,
                } = e
                {
                    if modifiers.matches_exact(SHOW_MENU.modifiers)
                        && *key == SHOW_MENU.logical_key
                        && *pressed
                    {
                        self.show_menu = !self.show_menu;

                        return Some(WindowProcessOptions {
                            // Simulate alt to get cursor
                            window_message: Some(WindowMessage {
                                msg: WM_KEYDOWN,
                                wparam: WPARAM(VK_MENU.0 as _),
                                lparam: LPARAM(0),
                            }),
                            ..Default::default()
                        });
                    }
                }
            }
        };

        if self.show_menu {
            Some(WindowProcessOptions {
                should_capture_all_input: true,
                ..Default::default()
            })
        } else {
            Some(WindowProcessOptions::default())
        }
    }
}

impl App {
    pub fn new(ctx: Context) -> Self {
        let path = r"StarRail_Data\StreamingAssets\MiHoYoSDKRes\HttpServerResources\font\zh-cn.ttf";
        match std::fs::read(path) {
            Ok(font) => {
                // Start with the default fonts (we will be adding to them rather than replacing them).
                ctx.add_font(FontInsert::new(
                    "game_font",
                    egui::FontData::from_owned(font),
                    vec![
                        InsertFontFamily {
                            family: egui::FontFamily::Proportional,
                            priority: egui::epaint::text::FontPriority::Highest,
                        },
                        InsertFontFamily {
                            family: egui::FontFamily::Monospace,
                            priority: egui::epaint::text::FontPriority::Lowest,
                        },
                    ],
                ));
            }
            Err(e) => log::warn!("{e} : Could not locate {path}. Defaulting to default font."),
        }

        ctx.style_mut(|style| {
            style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
        });

        // 更改全局字体样式的大小
        let custom_fontsize_style = egui::Style {
            text_styles: BTreeMap::from([
                (TextStyle::Heading, FontId::new(30.0, FontFamily::Proportional)),
                (TextStyle::Name("Heading2".into()), FontId::new(26.0, FontFamily::Proportional)),
                (TextStyle::Name("Context".into()), FontId::new(24.0, FontFamily::Proportional)),
                (TextStyle::Name("Legend".into()), FontId::new(22.0, FontFamily::Monospace)),
                (TextStyle::Body, FontId::new(24.0, FontFamily::Proportional)),
                (TextStyle::Monospace, FontId::new(24.0, FontFamily::Monospace)),
                (TextStyle::Button, FontId::new(22.0, FontFamily::Proportional)),
                (TextStyle::Small, FontId::new(18.0, FontFamily::Proportional)),
            ]),
            ..ctx.style().as_ref().clone() 
        };
        ctx.set_style(custom_fontsize_style);

        Self {
            widget_opacity: 0.15,
            // 默认不启用Steamer Mode
            streamer_mode: false,
            ..Default::default()
        }
    }
}
