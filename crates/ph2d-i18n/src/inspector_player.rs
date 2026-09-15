//! **O VOCABULÁRIO DA §14 PLATFORM PLAYER** — as chaves `panel.inspector.player.…`.
//!
//! ⚠️ **Irmão por SECÇÃO do `inspector.rs`**, cortado pelo tecto de 700 linhas da workspace (a tabela
//! do Inspector nasceu com 861): a §14 é a maior secção do painel — 165 chaves, quase todas dicas
//! de hover com frase inteira — e a que mais cresce, uma wave por card. Consultado na mesma cadeia do
//! [`crate::tr`]; os braços entre os marcadores `ph2d-migrar-texto` são do script, os de fora são à
//! mão. ⛔ Uma chave não mora nas DUAS tabelas: o gate do painel
//! (`every_inspector_key_lives_in_exactly_one_table`) reprova-o.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ── À MÃO (fora dos marcadores): frases com peças do código e leituras em minúsculas ──────
        "panel.inspector.player.fit_needs" => "Fit to Collider (needs > {min} m)",
        "panel.inspector.player.fit_crouch_needs" => "Fit Crouch to Collider (needs > {min} m)",
        "panel.inspector.player.clear_run" => "Clear Recorded Run ({s} s)",
        "panel.inspector.player.restore_run" => "Restore Discarded Run ({s} s)",
        "panel.inspector.player.posture_air" => "air",
        "panel.inspector.player.posture_steep" => "steep",
        "panel.inspector.player.posture_ground" => "ground",
        "panel.inspector.player.facing_left" => "left",
        "panel.inspector.player.facing_right" => "right",
        // ph2d-migrar-texto:begin
        "panel.inspector.player.the_floating_capsule_impulses_a" => {
            "The floating capsule: impulses, a spring leg, the solver owns the pose."
        }
        "panel.inspector.player.the_controller_the_pose_is" => {
            "The controller: the pose is written, the world only says how much fit."
        }
        "panel.inspector.player.classic_platformer_the_same_controller" => {
            "Classic platformer: the same controller, but the physical world is \
         scenery. Everything stops him and he moves nothing."
        }
        "panel.inspector.player.nobody_hears_him_he_lands" => {
            "Nobody hears him: he lands and jumps in silence."
        }
        "panel.inspector.player.publish_what_he_does_as" => {
            "Publish what he does as signals (player.landed, player.jumped.wall, ...)."
        }
        "panel.inspector.player.jump_height_is_measured_against" => {
            "Jump height is measured against the PLATFORM: a rising lift launches \
         him higher, a descending one almost cancels the jump."
        }
        "panel.inspector.player.a_rising_platform_still_launches" => {
            "A rising platform still launches him; a descending one stops stealing \
         the jump. The authored height is delivered in the world."
        }
        "panel.inspector.player.the_platform_never_changes_the" => {
            "The platform never changes the jump: the authored height is always \
         measured against the world."
        }
        "panel.inspector.player.he_walks_off_ledges_like" => {
            "He walks off ledges, like every character before this option existed."
        }
        "panel.inspector.player.he_stops_at_the_edge" => {
            "He stops at the edge instead of walking off it. Jumping off still \
         works, and so does being carried off by a platform or a belt. A gap \
         wider than his leg can span reads as a ledge, so he stops there too."
        }
        "panel.inspector.player.crouching_does_not_change_it" => {
            "Crouching does not change it: he walks off ledges if standing does."
        }
        "panel.inspector.player.crouched_he_stops_at_the" => {
            "Crouched, he stops at the edge -- the sneak-to-the-brink move. It only \
         tightens: it cannot give back what standing already refuses."
        }
        "panel.inspector.player.turn_this_body_into_a" => {
            "Turn this body into a walking, jumping character."
        }
        "panel.inspector.player.set_float_height_from_the" => {
            "Set Float Height from the collider, so he really hovers."
        }
        "panel.inspector.player.give_the_behaviour_back_it" => {
            "Give the behaviour back: it becomes a plain body again."
        }
        "panel.inspector.player.throw_away_the_recorded_run" => {
            "Throw away the recorded run. Playing with Physics on records a new one."
        }
        "panel.inspector.player.set_crouch_height_to_the" => {
            "Set Crouch Height to the lowest this body can really float at."
        }
        "panel.inspector.player.platform_player" => "Platform Player",
        "panel.inspector.player.body" => "Body",
        "panel.inspector.player.dynamic" => "Dynamic",
        "panel.inspector.player.kinematic" => "Kinematic",
        "panel.inspector.player.pure" => "Pure",
        "panel.inspector.player.emit_signals" => "Emit Signals",
        "panel.inspector.player.off" => "Off",
        "panel.inspector.player.on" => "On",
        "panel.inspector.player.platform_lift" => "Platform Lift",
        "panel.inspector.player.full" => "Full",
        "panel.inspector.player.up_only" => "Up Only",
        "panel.inspector.player.none" => "None",
        "panel.inspector.player.walk_off_ledges" => "Walk Off Ledges",
        "panel.inspector.player.yes" => "Yes",
        "panel.inspector.player.stop_at_edge" => "Stop At Edge",
        "panel.inspector.player.when_crouching" => "  ...When Crouching",
        "panel.inspector.player.fit_to_collider" => "Fit to Collider",
        "panel.inspector.player.fit_crouch_to_collider" => "Fit Crouch to Collider",
        "panel.inspector.player.make_platform_player" => "Make Platform Player",
        "panel.inspector.player.remove_platform_player" => "Remove Platform Player",
        "panel.inspector.player.live" => "Live",
        "panel.inspector.player.not_simulating" => "not simulating",
        "panel.inspector.player.posture" => "Posture",
        "panel.inspector.player.facing" => "Facing",
        "panel.inspector.player.speed" => "Speed",
        "panel.inspector.player.float_height_m" => "Float Height",
        "panel.inspector.player.how_high_the_character_hovers" => {
            "How high the character hovers above the ground."
        }
        "panel.inspector.player.cling_distance_m" => "Cling Distance",
        "panel.inspector.player.how_far_above_rest_the" => {
            "How far above rest the leg still grips: steps, not jumps."
        }
        "panel.inspector.player.leg_stiffness" => "Leg Stiffness",
        "panel.inspector.player.how_hard_the_leg_pushes" => {
            "How hard the leg pushes back. Higher is a firmer stance."
        }
        "panel.inspector.player.leg_damping" => "Leg Damping",
        "panel.inspector.player.how_fast_the_bounce_dies" => {
            "How fast the bounce dies out. Above 1 he pops. Lower it for a bouncier \
         landing, then raise World > Sub-steps to stop him creeping up ramps."
        }
        "panel.inspector.player.foot_rays" => "Foot Rays",
        "panel.inspector.player.how_many_rays_the_leg" => {
            "How MANY rays the leg casts. Odd; the middle one breaks ties. \
         1 sinks over gaps your body could span."
        }
        "panel.inspector.player.foot_ray_spread" => "Foot Ray Spread",
        "panel.inspector.player.where_the_outer_feet_sit" => {
            "Where the OUTER feet sit, as a fraction of your half-width. 1 = the box edge, \
         0 collapses back to a single ray."
        }
        "panel.inspector.player.cruising_speed_measured_relative_to" => {
            "Cruising speed, measured relative to the ground."
        }
        "panel.inspector.player.acceleration" => "Acceleration",
        "panel.inspector.player.how_quickly_he_reaches_cruising" => {
            "How quickly he reaches cruising speed on the ground."
        }
        "panel.inspector.player.air_acceleration" => "Air Acceleration",
        "panel.inspector.player.steering_while_airborne_0_keeps" => {
            "Steering while airborne. 0 keeps the jump arc intact."
        }
        "panel.inspector.player.brake" => "Brake",
        "panel.inspector.player.how_much_of_that_acceleration" => {
            "How much of that acceleration he spends STOPPING, once you let go of the \
         stick. 1 stops as hard as he starts. 0 is ice: he keeps the speed. \
         Airborne is unaffected -- Air Acceleration already answers that."
        }
        "panel.inspector.player.max_slope_deg" => "Max Slope",
        "panel.inspector.player.steepest_ramp_he_stands_on" => {
            "Steepest ramp he stands on and walks up, in DEGREES."
        }
        "panel.inspector.player.jump_height_m" => "Jump Height",
        "panel.inspector.player.how_high_a_full_jump" => "How high a full jump reaches, in metres.",
        "panel.inspector.player.air_jumps" => "Air Jumps",
        "panel.inspector.player.extra_jumps_after_leaving_the" => {
            "Extra jumps after leaving the ground. 0 turns it off; they refill on landing."
        }
        "panel.inspector.player.air_jump_height_m" => "Air Jump Height",
        "panel.inspector.player.how_high_an_air_jump" => {
            "How high an AIR jump reaches, in metres. Same as above is the Celeste feel; \
         lower is Hollow Knight."
        }
        "panel.inspector.player.takeoff_gravity" => "Takeoff Gravity",
        "panel.inspector.player.gravity_while_rising_fast_1" => {
            "Gravity while rising fast. 1 is the world's."
        }
        "panel.inspector.player.rising_faster_than_this_uses" => {
            "Rising faster than this uses Takeoff Gravity."
        }
        "panel.inspector.player.peak_gravity" => "Peak Gravity",
        "panel.inspector.player.gravity_near_the_top_below" => {
            "Gravity near the top. Below 1 he hangs longer."
        }
        "panel.inspector.player.fall_gravity" => "Fall Gravity",
        "panel.inspector.player.gravity_while_falling_above_1" => {
            "Gravity while falling. Above 1 he drops faster than he rose."
        }
        "panel.inspector.player.cut_gravity" => "Cut Gravity",
        "panel.inspector.player.gravity_while_rising_with_the" => {
            "Gravity while rising with the button RELEASED."
        }
        "panel.inspector.player.coyote_time_s" => "Coyote Time",
        "panel.inspector.player.grace_after_leaving_the_ground" => {
            "Grace after leaving the ground. 0 turns it off."
        }
        "panel.inspector.player.jump_buffer_s" => "Jump Buffer",
        "panel.inspector.player.a_press_this_early_still" => {
            "A press this early still fires on landing."
        }
        "panel.inspector.player.corner_reach_m" => "Corner Reach",
        "panel.inspector.player.slide_sideways_up_to_this" => {
            "Slide sideways up to this to clear a ledge you clipped. In METRES."
        }
        "panel.inspector.player.corner_rays" => "Corner Rays",
        "panel.inspector.player.how_many_rays_scan_the" => {
            "How MANY rays scan the ceiling profile. More = a finer ledge edge."
        }
        "panel.inspector.player.corner_look_ahead" => "Corner Look-ahead",
        "panel.inspector.player.how_many_ticks_ahead_the" => {
            "How many TICKS ahead the ceiling profile looks. 0 = no anticipation."
        }
        "panel.inspector.player.lift_momentum_s" => "Lift Momentum",
        "panel.inspector.player.keep_a_moving_platform_s" => {
            "Keep a moving platform's speed for this long after leaving it."
        }
        "panel.inspector.player.weight_on_ground" => "Weight on Ground",
        "panel.inspector.player.how_much_of_his_weight" => {
            "How much of his weight presses the ground down."
        }
        "panel.inspector.player.push_on_ground" => "Push on Ground",
        "panel.inspector.player.how_much_of_his_walking" => {
            "How much of his walking shoves the ground back."
        }
        "panel.inspector.player.push_on_bodies" => "Push on Bodies",
        "panel.inspector.player.how_hard_he_shoves_what" => {
            "How hard he shoves what he walks into. A dynamic body already pushes through the solver."
        }
        "panel.inspector.player.slide_down_a_wall_at" => {
            "Slide DOWN a wall at this speed while pushing into it. 0 = off."
        }
        "panel.inspector.player.wall_jump_m" => "Wall Jump",
        "panel.inspector.player.how_high_a_jump_off" => "How high a jump off a wall goes. 0 = off.",
        "panel.inspector.player.how_hard_a_wall_jump" => {
            "How hard a wall jump throws you AWAY from the wall."
        }
        "panel.inspector.player.wall_lockout_s" => "Wall Lockout",
        "panel.inspector.player.air_control_stays_quiet_this" => {
            "Air control stays quiet this long after a wall jump."
        }
        "panel.inspector.player.wall_reach_m" => "Wall Reach",
        "panel.inspector.player.how_far_past_your_own" => {
            "How far past your own width the wall sensor looks."
        }
        "panel.inspector.player.wall_rays" => "Wall Rays",
        "panel.inspector.player.how_many_rays_the_flank" => {
            "How MANY rays the flank casts. Odd; the middle one breaks ties."
        }
        "panel.inspector.player.wall_ray_spread" => "Wall Ray Spread",
        "panel.inspector.player.where_the_outer_rays_sit" => {
            "Where the OUTER rays sit, as a fraction of your half-height. 1 = the box edge."
        }
        "panel.inspector.player.wall_grab_s" => "Wall Grab",
        "panel.inspector.player.hold_r_against_a_wall" => {
            "Hold R against a wall to stick instead of sliding, for this long. 0 = off."
        }
        "panel.inspector.player.how_fast_the_dash_carries" => {
            "How fast the dash carries him. 0 = off."
        }
        "panel.inspector.player.dash_time_s" => "Dash Time",
        "panel.inspector.player.how_long_it_lasts_speed" => {
            "How long it lasts. Speed x Time is the DISTANCE it covers."
        }
        "panel.inspector.player.dash_cooldown_s" => "Dash Cooldown",
        "panel.inspector.player.recovery_after_it_ends_before" => {
            "Recovery after it ENDS, before he can dash again."
        }
        "panel.inspector.player.crouch_height_m" => "Crouch Height",
        "panel.inspector.player.how_low_he_floats_while" => {
            "How low he floats while holding DOWN. 0 = off."
        }
        "panel.inspector.player.how_fast_he_walks_while" => {
            "How fast he walks while crouched. 0 means duck in place."
        }
        "panel.inspector.player.how_fast_he_swims_in" => {
            "How fast he swims, in any direction. 0 = off."
        }
        "panel.inspector.player.authority_against_the_water_low" => {
            "Authority against the water. Low: he floats up on his own."
        }
        "panel.inspector.player.swim_line_weights" => "Swim Line (weights)",
        "panel.inspector.player.buoyancy_he_swims_at_and" => {
            "Buoyancy he swims at, and rests at. 1 = the water holds him."
        }
        "panel.inspector.player.ledge_grab_m" => "Ledge Grab",
        "panel.inspector.player.how_far_ahead_the_sensor" => {
            "How far ahead the sensor looks for a lip. 0 = off."
        }
        "panel.inspector.player.grab_window_m" => "Grab Window",
        "panel.inspector.player.how_tall_the_catch_window" => {
            "How TALL the catch window is, above and below."
        }
        "panel.inspector.player.grab_span_m" => "Grab Span",
        "panel.inspector.player.how_wide_the_sensor_is" => {
            "How wide the sensor is. 0 = a single ray."
        }
        "panel.inspector.player.grab_offset_y_m" => "Grab Offset Y",
        "panel.inspector.player.slides_the_sensor_up_or" => {
            "Slides the sensor up or down without resizing it."
        }
        "panel.inspector.player.how_fast_he_settles_into" => {
            "How fast he settles into the hang, and climbs over."
        }
        "panel.inspector.player.top_descent_speed_while_holding" => {
            "Top descent speed while holding jump in a fall. 0 = off."
        }
        "panel.inspector.player.terminal_speed_the_fall_never" => {
            "Terminal speed: the fall never gets faster than this. 0 = no cap."
        }
        "panel.inspector.player.leg" => "LEG",
        "panel.inspector.player.walk" => "WALK",
        "panel.inspector.player.jump" => "JUMP",
        "panel.inspector.player.forgiveness" => "FORGIVENESS",
        "panel.inspector.player.reaction" => "REACTION",
        "panel.inspector.player.walls" => "WALLS",
        "panel.inspector.player.dash" => "DASH",
        "panel.inspector.player.crouch" => "CROUCH",
        "panel.inspector.player.swim" => "SWIM",
        "panel.inspector.player.ledge" => "LEDGE",
        "panel.inspector.player.glide" => "GLIDE",
        "panel.inspector.player.fall" => "FALL",
        "panel.inspector.player.speed_m_s" => "Speed",
        "panel.inspector.player.takeoff_above_m_s" => "Takeoff Above",
        "panel.inspector.player.peak_window_m_s" => "Peak Window",
        "panel.inspector.player.how_wide_that_slow_top" => "How wide that slow top is, in m/s.",
        "panel.inspector.player.wall_slide_m_s" => "Wall Slide",
        "panel.inspector.player.wall_push_m_s" => "Wall Push",
        "panel.inspector.player.dash_speed_m_s" => "Dash Speed",
        "panel.inspector.player.crouch_speed_m_s" => "Crouch Speed",
        "panel.inspector.player.swim_speed_m_s" => "Swim Speed",
        "panel.inspector.player.swim_accel_m_s2" => "Swim Accel",
        "panel.inspector.player.ledge_speed_m_s" => "Ledge Speed",
        "panel.inspector.player.glide_fall_m_s" => "Glide Fall",
        "panel.inspector.player.max_fall_m_s" => "Max Fall",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
