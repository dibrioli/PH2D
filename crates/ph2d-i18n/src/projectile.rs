//! As palavras do cartão Projectile Motion (GEF 06, versão B): avisos, caixas, a dobra e as dicas de hover.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.projectile.no_body_u_add_a_rigid_body_for_this_to_move_anything" => {
            "No body \u{2014} add a Rigid Body for this to move anything."
        }
        "panel.inspector.projectile.the_body_must_be_kinematic_u_a_dynamic_body_belongs_to_the_solver" => {
            "The body must be Kinematic \u{2014} a dynamic body belongs to the solver."
        }
        "panel.inspector.projectile.the_flight_is_over_u_rewind_to_launch_it_again" => {
            "The flight is over \u{2014} rewind to launch it again."
        }
        "panel.inspector.projectile.the_clock_is_stopped_u_it_flies_while_the_clock_plays" => {
            "The clock is stopped \u{2014} it flies while the clock plays."
        }
        "panel.inspector.projectile.max_speed_hint" => "0 = no cap.",
        "panel.inspector.projectile.gravity_hint" => "0 = flies straight.",
        "panel.inspector.projectile.bounciness_hint" => "1 = a perfect bounce.",
        "panel.inspector.projectile.range_hint" => "0 = flies forever.",
        "panel.inspector.projectile.homing_hint" => {
            "How hard it turns toward the Target. Higher = tighter turns."
        }
        "panel.inspector.projectile.target_object_name_u" => "target object name\u{2026}",
        "panel.inspector.projectile.no_object_in_the_scene_has_that_name" => {
            "No object in the scene has that name."
        }
        "panel.inspector.projectile.bounce" => "Bounce",
        "panel.inspector.projectile.homing" => "Homing",
        "panel.inspector.projectile.speed_change" => "Speed change",
        "panel.inspector.projectile.sum_none" => "None",
        "panel.inspector.projectile.sum_speeds_up" => "Speeds up",
        "panel.inspector.projectile.sum_slows_down" => "Slows down",
        "panel.inspector.projectile.sum_capped" => "Capped",
        "panel.inspector.projectile.sum_and_capped" => "{change} \u{b7} capped",
        "panel.inspector.projectile.tip_speed" => {
            "How fast it leaves, toward where the object faces."
        }
        "panel.inspector.projectile.tip_faces_path" => {
            "On: the picture turns to point where it flies, like an arrow. Off: it keeps its angle, like a ball."
        }
        "panel.inspector.projectile.tip_bounce" => {
            "On: it bounces off walls before it stops. Off: it stops at the first thing it hits."
        }
        "panel.inspector.projectile.tip_max_bounces" => {
            "How many times it bounces before it stops."
        }
        "panel.inspector.projectile.tip_homing" => {
            "On: it curves toward the Target while it flies."
        }
        "panel.inspector.projectile.tip_target" => {
            "The object it chases, by name. Type it, or pick it with the eyedropper."
        }
        "panel.inspector.projectile.tip_target_pick" => {
            "Pick the Target: click this, then click an object on the canvas. Esc cancels."
        }
        "panel.inspector.projectile.tip_acceleration" => {
            "Speed it gains every second while it flies. Below 0 it slows down. 0 = steady."
        }
        "panel.inspector.projectile.tip_speed_change" => {
            "Make it speed up or slow down in flight, with an optional top speed."
        }
        _ => return None,
    })
}
