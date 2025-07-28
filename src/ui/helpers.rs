pub fn format_damage(value: f64) -> String {
    if value >= 100_000_000_000.0 {
        //100.0G-Inf, 大于999.9G将会导致图例不对齐，这游戏应该不会膨胀到这种程度吧？ 
        let m = value / 1_000_000_000.0;
        format!("{m:.1}G")
    } else if value >= 1_000_000_000.0 {
        //1000M-99999M
        let m = value / 1_000_000.0;
        format!("{m:.0}M")
    } else if value >= 1_000_000.0 {
        //1.0M-999.9M
        let m = value / 1_000_000.0;
        format!("{m:.1}M")
    } else if value >= 1_000.0 {
        format!("{}K", (value / 1_000.0).floor())
    } else {
        format!("{}", value.floor())
    }
}

pub fn get_character_color(index: usize) -> egui::Color32 {
    const COLORS: &[egui::Color32] = &[
        egui::Color32::from_rgb(255, 99, 132),
        egui::Color32::from_rgb(54, 162, 235),
        egui::Color32::from_rgb(255, 206, 86),
        egui::Color32::from_rgb(75, 192, 192),
        egui::Color32::from_rgb(153, 102, 255),
        egui::Color32::from_rgb(255, 159, 64),
        egui::Color32::from_rgb(231, 233, 237),
        egui::Color32::from_rgb(102, 255, 102),
    ];

    COLORS[index % COLORS.len()]
}
