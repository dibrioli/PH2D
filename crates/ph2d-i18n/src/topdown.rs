pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "panel.topdown.free" => "Free",
        "panel.topdown.8_dir" => "8 Dir",
        "panel.topdown.4_dir" => "4 Dir",
        "panel.topdown.x_only" => "X Only",
        "panel.topdown.y_only" => "Y Only",
        "panel.topdown.top_down" => "Top-Down",
        "panel.topdown.iso_2_1" => "Iso 2:1",
        "panel.topdown.iso_30" => "Iso 30°",
        "panel.topdown.custom" => "Custom",
        "panel.topdown.don_t_turn" => "Don't Turn",
        "panel.topdown.face_move" => "Face Move",
        "panel.topdown.face_4" => "Face 4",
        "panel.topdown.face_8" => "Face 8",
        "panel.topdown.dir_x" => "X",
        "panel.topdown.dir_y" => "Y",
        "panel.topdown.facing_still" => "Still",
        "panel.topdown.facing_move" => "Move",
        "panel.topdown.facing_4" => "4",
        "panel.topdown.facing_8" => "8",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
