use crate::ui::app::GraphUnit;
use egui::{Stroke, TextStyle, Ui};
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints, Polygon};

use crate::{battle::BattleContext, models::misc::Avatar};

use super::{app::App, helpers};

pub struct PieSegment {
    pub points: Vec<[f64; 2]>,
    pub value: f64,
}

impl App {
    pub fn show_damage_distribution_widget(&mut self, ui: &mut Ui) {
        let battle_context = BattleContext::get_instance();
        let available = ui.available_size();
        Plot::new("damage_pie")
            //优化饼图显示位置，默认状态下不会被图例遮挡
            .set_margin_fraction(egui::Vec2 { x: (0.), y: (0.05) })
            .default_x_bounds(-13., 13.)
            .default_y_bounds(-10.5, 20.)
            .legend(
                Legend::default()
                    .position(egui_plot::Corner::RightTop)
                    .text_style(TextStyle::Name("Legend".into())),
            )
            .show_x(false)
            .show_y(false)
            .height(available.y)
            .width(available.x)
            .data_aspect(1.0)
            .clamp_grid(true)
            .show_grid(false)
            .show_background(false)
            .show_axes([false; 2])
            // .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .show(ui, |plot_ui: &mut egui_plot::PlotUi<'_>| {
                let total_damage = battle_context.total_damage;
                if total_damage > 0.0 {
                    let segments = create_pie_segments(
                        &battle_context.real_time_damages,
                        &battle_context.avatar_lineup,
                    );
                    for (avatar, segment, i) in segments {
                        let color = helpers::get_character_color(i);
                        let percentage = segment.value / total_damage * 100.0;

                        let plot_points = PlotPoints::new(segment.points);
                        // 不同的Polygon修改后拥有不同的唯一识别符，避免点击图例后饼图全部消失
                        let polygon = Polygon::new(
                            format!(
                                "Damage Pie {index} For {avatar_name}", 
                                avatar_name = avatar.name, 
                                index = i
                            ), 
                            plot_points)
                            .stroke(Stroke::new(1.8, color))
                            .highlight(true)
                            .name(format!(//优化图例的格式化文本，一般情况下能对齐
                                "{name:<}:{damage:>6}#{percentage:>5.1}%",
                                name = avatar.name,
                                damage = helpers::format_damage(segment.value),
                                percentage = percentage
                            ));

                        plot_ui.polygon(polygon);
                    }
                }
            });
    }

    pub fn show_damage_bar_widget(&mut self, ui: &mut Ui) {
        let battle_context = BattleContext::get_instance();
        let available = ui.available_size();
        Plot::new("damage_bars")
            .legend(Legend::default())
            .show_x(false)
            //.show_y(false)
            .height(available.y)
            .width(available.x)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .show_background(false)
            .y_axis_formatter(|y, _| helpers::format_damage(y.value))
            .x_axis_formatter(|x, _| {
                let index = x.value.floor() as usize;
                battle_context
                    .avatar_lineup
                    .get(index)
                    .map(|avatar| avatar.name.clone())
                    .unwrap_or_default()
            })
            .show(ui, |plot_ui| {
                let bars_data = create_bar_data(
                    &battle_context.real_time_damages,
                    &battle_context.avatar_lineup,
                );
                let bars: Vec<Bar> = bars_data
                    .iter()
                    .enumerate()
                    .map(|(pos, (avatar, value, color_idx))| {
                        Bar::new(pos as f64, *value)
                            .name(&avatar.name)
                            .fill(helpers::get_character_color(*color_idx))
                            .width(0.7)
                            .stroke(
                                Stroke::new(1.8, 
                                helpers::get_character_color(*color_idx))
                            )
                            
                    })
                    .collect();

                plot_ui.bar_chart(BarChart::new("", bars));
            });
    }

    pub fn show_turn_damage_plot(&mut self, ui: &mut Ui) {
        let battle_context = BattleContext::get_instance();
        let available = ui.available_size();
        Plot::new("turn_damage_plot")
            .legend(
                Legend::default()
                    .position(egui_plot::Corner::RightTop)
                    .text_style(TextStyle::Name("Legend".into())),
            )
            .show_x(false)
            .show_y(false)
            .height(available.y)
            .width(available.x)
            .include_y(0.0)
            .x_axis_label(t!("Turn"))
            .y_axis_label(t!("Damage"))
            .y_axis_formatter(|y, _| helpers::format_damage(y.value))
            .show(ui, |plot_ui| {
                for (i, avatar) in battle_context.avatar_lineup.iter().enumerate() {
                    let color = helpers::get_character_color(i);
                    let points = battle_context
                        .turn_history
                        .iter()
                        .enumerate()
                        .map(|(turn_idx, turn)| {
                            [turn_idx as f64 + 1.0, turn.avatars_turn_damage[i]]
                        })
                        .collect::<Vec<[f64; 2]>>();

                    if !points.is_empty() {
                        plot_ui.line(
                            Line::new(&avatar.name, PlotPoints::from(points))
                                .color(color)
                                .width(2.0),
                        );
                    }
                }
            });
    }

    pub fn show_av_damage_plot(&mut self, ui: &mut Ui) {
        let battle_context = BattleContext::get_instance();
        let available = ui.available_size();
        Plot::new("av_damage_plot")
            .legend(
                Legend::default()
                    .position(egui_plot::Corner::RightTop)
                    .text_style(TextStyle::Name("Legend".into())),
            )
            .show_x(false)
            .show_y(false)
            .height(available.y)
            .width(available.x)
            .include_y(0.0)
            .x_axis_label(t!("Action Value"))
            .y_axis_label(t!("Damage"))
            .y_axis_formatter(|y, _| helpers::format_damage(y.value))
            .show(ui, |plot_ui| {
                for (i, avatar) in battle_context.avatar_lineup.iter().enumerate() {
                    let color = helpers::get_character_color(i);
                    let points = battle_context
                        .av_history
                        .iter()
                        .map(|turn| [turn.action_value, turn.avatars_turn_damage[i]])
                        .collect::<Vec<[f64; 2]>>();

                    if !points.is_empty() {
                        plot_ui.line(
                            Line::new(&avatar.name, PlotPoints::from(points))
                                .color(color)
                                .width(2.0),
                        );
                    }
                }
            });
    }

    pub fn show_real_time_damage_graph(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.graph_x_unit, GraphUnit::Turn, t!("Turn"));
                ui.radio_value(
                    &mut self.graph_x_unit,
                    GraphUnit::ActionValue,
                    t!("Action Value"),
                );
            });
            ui.add_space(8.0);

            match self.graph_x_unit {
                GraphUnit::Turn => self.show_turn_damage_plot(ui),
                GraphUnit::ActionValue => self.show_av_damage_plot(ui),
            }
        });
    }

    pub fn show_av_metrics(&mut self, ui: &mut Ui) {
        let battle_context = BattleContext::get_instance();
        ui.horizontal(|ui| {
            ui.label(t!("Total Elapsed AV:"));
            ui.label(format!(
                "{:.2}",
                battle_context.current_turn_info.action_value
            ));
        });
        ui.label(t!("Current Turn"));
        ui.horizontal(|ui| {
            ui.label(t!("AV:"));
            ui.label(format!(
                "{:.2}",
                battle_context.current_turn_info.action_value
            ));
        });
        ui.horizontal(|ui| {
            ui.label(t!("Total Damage:"));
            ui.label(helpers::format_damage(battle_context.total_damage));
        });
        ui.horizontal(|ui| {
            ui.label(t!("DpAV:"));
            if battle_context.action_value > 0.0 {
                ui.label(format!(
                    "{:.2}",
                    battle_context.total_damage / battle_context.action_value
                ));
            } else {
                ui.label(format!("{:.2}", battle_context.total_damage / 1.0));
            }
        });
    }

    pub fn show_enemy_stats(&mut self, ui: &mut Ui) {
        let battle_context = BattleContext::get_instance();
        let enemy_lineup = battle_context.enemy_lineup.clone();

        ui.vertical(|ui| {
            for enemy in &enemy_lineup {
                if let Some(i) = battle_context
                    .battle_enemies
                    .iter()
                    .enumerate()
                    .find(|(_i, x)| x.entity == *enemy)
                    .map(|(i, _x)| i)
                {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} HP: ", &battle_context.enemies[i].name));
                        ui.label(format!(
                            "{:.2}",
                            battle_context.battle_enemies[i].battle_stats.hp
                        ));
                    });
                }
            }
        });
    }
}

fn create_bar_data(real_time_damages: &[f64], avatars: &[Avatar]) -> Vec<(Avatar, f64, usize)> {
    let mut bar_data = Vec::new();
    for (i, avatar) in avatars.iter().enumerate() {
        bar_data.push((avatar.clone(), real_time_damages[i], i));
    }
    bar_data
}

fn create_pie_segments(
    real_time_damages: &[f64],
    avatars: &[Avatar],
) -> Vec<(Avatar, PieSegment, usize)> {
    let total_damage = real_time_damages.iter().sum::<f64>();
    let mut segments = Vec::new();
    let mut start_angle = -std::f64::consts::FRAC_PI_2;

    for (i, avatar) in avatars.iter().enumerate() {
        let damage = real_time_damages[i];
        let fraction = damage / total_damage;
        let angle = fraction * std::f64::consts::TAU;
        let end_angle = start_angle + angle;

        segments.push((
            avatar.clone(),
            PieSegment {
                points: create_pie_slice(start_angle, end_angle),
                value: damage,
            },
            i,
        ));

        start_angle = end_angle;
    }

    segments
}

fn create_pie_slice(start_angle: f64, end_angle: f64) -> Vec<[f64; 2]> {
    let center = [0.0, 0.0];
    let radius = 10.;
    let mut points = vec![center];

    let steps = 50;
    let p = (end_angle - start_angle) / (steps as f64);
    for i in 0..=steps {
        let angle = start_angle + p * i as f64;
        let (sin, cos) = angle.sin_cos();
        points.push([cos * radius, sin * radius]);
    }
    points.push(center);

    points
}
