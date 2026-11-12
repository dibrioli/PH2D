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
        // O cartão do Inspector (GEF 05, versão A): rótulos do More, resumos e dicas de hover.
        "panel.inspector.topdown.acceleration_hint" => {
            "How fast it reaches top speed once a key is held. 0 = instant."
        }
        "panel.inspector.topdown.deceleration_hint" => {
            "How fast it stops once the keys are let go. 0 = instant."
        }
        "panel.inspector.topdown.turn_speed_hint" => {
            "How fast the picture turns, in degrees per second. 0 = instant."
        }
        "panel.inspector.topdown.more" => "More",
        "panel.inspector.topdown.sum_no_easing" => "No easing",
        "panel.inspector.topdown.sum_eased" => "Eased",
        "panel.inspector.topdown.sum_view" => "{name} view",
        "panel.inspector.topdown.turn_on" => "Turn on",
        "panel.inspector.topdown.tip_more" => {
            "Speed up and slow down, the viewpoint, hit recovery, wall sliding and the keyboard."
        }
        "panel.inspector.topdown.tip_speed" => "Top speed while a key is held.",
        "panel.inspector.topdown.tip_dir_free" => {
            "Free: moves in any direction the keys add up to."
        }
        "panel.inspector.topdown.tip_dir_8" => "8 directions: straight and diagonal.",
        "panel.inspector.topdown.tip_dir_4" => "4 directions: up, down, left and right only.",
        "panel.inspector.topdown.tip_dir_x" => "Left and right only.",
        "panel.inspector.topdown.tip_dir_y" => "Up and down only.",
        "panel.inspector.topdown.tip_face_still" => "Don't turn: the picture keeps its angle.",
        "panel.inspector.topdown.tip_face_move" => {
            "Face movement: the picture turns toward where it walks."
        }
        "panel.inspector.topdown.tip_face_4" => {
            "Snap to 4: the picture turns to the nearest of up, down, left and right."
        }
        "panel.inspector.topdown.tip_face_8" => {
            "Snap to 8: the picture turns to the nearest of eight directions."
        }
        "panel.inspector.topdown.tip_view_top" => {
            "Changes where the Up key leads. Top-Down: Up goes straight up the screen."
        }
        "panel.inspector.topdown.tip_view_iso21" => {
            "Changes where the Up key leads. Iso 2:1: Up follows the diamond grid of 2:1 tiles."
        }
        "panel.inspector.topdown.tip_view_iso30" => {
            "Changes where the Up key leads. Iso 30°: Up follows a true isometric grid."
        }
        "panel.inspector.topdown.tip_view_custom" => {
            "Changes where the Up key leads. Custom: Up follows the tilt set in Board Angle."
        }
        "panel.inspector.topdown.tip_board_angle" => {
            "How far the board tilts away from you, in degrees. Changes where the Up key leads."
        }
        "panel.inspector.topdown.tip_hit_recovery" => {
            "After a hit pushes it, how fast it stops sliding. 0 = instant."
        }
        "panel.inspector.topdown.tip_min_slide" => {
            "Meeting a wall closer to head-on than this, it stops instead of sliding along."
        }
        "panel.inspector.topdown.tip_max_slides" => {
            "How many walls it can slide along in one step."
        }
        "panel.inspector.topdown.tip_default_controls" => {
            "On: the arrow keys from the Input Map drive it. Off: something else must drive it."
        }
        "panel.inspector.topdown.tip_turn_on" => "Let the arrow keys drive it again.",
        _ => return None,
    })
}
