pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "panel.inspector.actions.name" => "Name",
        "panel.inspector.actions.tag" => "Tag",
        "panel.inspector.actions.no_tag_chosen_u_this_action_reaches_nobody" => {
            "No tag chosen \u{b7} this action reaches nobody."
        }
        "panel.inspector.actions.that_tag_was_deleted_u_this_action_reaches_nobody" => {
            "That tag was deleted \u{b7} this action reaches nobody."
        }
        "panel.inspector.factory.where" => "Where",
        "panel.inspector.factory.area_m" => "Area",
        "panel.inspector.factory.burst" => "Burst",
        "panel.inspector.factory.max_alive_0_no_limit" => "Max Alive (0 = no limit)",
        "panel.inspector.factory.max_total_0_no_limit" => "Max Total (0 = no limit)",
        "panel.inspector.factory.seed" => "Seed",
        "panel.inspector.factory.no_recipe_u_nothing_to_make_copies_of" => {
            "No recipe \u{2014} nothing to make copies of."
        }
        "panel.inspector.factory.no_signal_u_this_factory_never_fires" => {
            "No signal \u{2014} this factory never fires."
        }
        "panel.inspector.factory.the_clock_is_stopped_u_copies_are_born_while_it_plays" => {
            "The clock is stopped \u{2014} copies are born while it plays."
        }
        "panel.inspector.factory.pick_at_random" => "Pick at random",
        "panel.inspector.factory.aim_from_spawner" => "Aim from spawner",
        "panel.inspector.factory.alive_now" => "{n} alive now",
        "panel.inspector.factory.lifetime_s_0_forever" => "Lifetime (s, 0 = forever)",
        "panel.inspector.factory.off_screen_margin_m" => "Off-screen margin",
        "panel.inspector.factory.nothing_is_born_from_this_object_u_put_this_on_the_recipe_a_factory_makes" => {
            "Nothing is born from this object \u{2014} put this on the recipe a Factory makes."
        }
        "panel.inspector.factory.no_game_camera_u_off_screen_has_no_screen_to_measure" => {
            "No game camera \u{2014} off-screen has no screen to measure."
        }
        "panel.inspector.factory.factory" => "Factory",
        "panel.inspector.factory.editing_the_primary_selection_only" => {
            "Editing the primary selection only."
        }
        "panel.inspector.factory.lifecycle" => "Lifecycle",
        // ⭐⭐⭐ O HUD (TOP-20 #20) — a raiz, o rótulo, o botão e o contador.
        "panel.inspector.hud.hud" => "HUD",
        "panel.inspector.hud.reference_width" => "Reference Width",
        "panel.inspector.hud.reference_height" => "Reference Height",
        "panel.inspector.hud.counter_start" => "Start",
        "panel.inspector.hud.source_name" => "Source Name",
        "panel.inspector.hud.prefix" => "Prefix",
        "panel.inspector.hud.suffix" => "Suffix",
        "panel.inspector.hud.signal" => "Signal",
        "panel.inspector.hud.counter_name" => "Counter Name",
        "panel.inspector.hud.fit" => "Fit",
        "panel.inspector.hud.fit_keep" => "Keep",
        "panel.inspector.hud.fit_stretch" => "Stretch",
        "panel.inspector.hud.source" => "Source",
        "panel.inspector.hud.source_authored" => "Authored",
        "panel.inspector.hud.source_counter" => "Counter",
        "panel.inspector.hud.source_timer" => "Timer",
        "panel.inspector.hud.source_tag" => "Tag",
        "panel.inspector.hud.disabled" => "Disabled",
        "panel.inspector.hud.no_game_camera" => {
            "No game camera \u{2014} the HUD stays where you put it."
        }
        "panel.inspector.hud.showing" => "Showing: {v}",
        "panel.inspector.hud.source_missing" => {
            "That source is not in the scene \u{2014} the authored text is shown."
        }
        "panel.inspector.hud.counter_now" => "Now: {v}",
        "panel.inspector.hud.button_disabled" => "This button refuses the click.",
        "panel.inspector.hud.no_signal" => "No name \u{2014} this button publishes nothing.",
        "panel.inspector.particles.amount" => "Amount",
        "panel.inspector.particles.lifetime_s" => "Lifetime",
        "panel.inspector.particles.lifetime_randomness" => "Lifetime Randomness",
        "panel.inspector.particles.explosiveness" => "Explosiveness",
        "panel.inspector.particles.preprocess_s" => "Preprocess",
        "panel.inspector.particles.speed_scale" => "Speed Scale",
        "panel.inspector.particles.seed" => "Seed",
        "panel.inspector.particles.shape_width_radius_m" => "Shape Width / Radius",
        "panel.inspector.particles.shape_height_m" => "Shape Height",
        "panel.inspector.particles.speed_m_s" => "Speed",
        "panel.inspector.particles.speed_randomness" => "Speed Randomness",
        "panel.inspector.particles.direction_deg" => "Direction",
        "panel.inspector.particles.spread_deg" => "Spread",
        "panel.inspector.particles.gravity_x_m_s_u" => "Gravity X (m/s\u{b2})",
        "panel.inspector.particles.gravity_y_m_s_u" => "Gravity Y (m/s\u{b2})",
        "panel.inspector.particles.damping" => "Damping",
        "panel.inspector.particles.size_m" => "Size",
        "panel.inspector.particles.size_randomness" => "Size Randomness",
        "panel.inspector.particles.size_at_death" => "Size at Death",
        "panel.inspector.particles.the_clock_is_stopped_u_particles_are_born_while_the_clock_plays" => {
            "The clock is stopped \u{2014} particles are born while the clock plays."
        }
        "panel.inspector.particles.nothing_alive_right_now_u_the_emission_ended_or_it_is_switched_off" => {
            "Nothing alive right now \u{2014} the emission ended, or it is switched off."
        }
        "panel.inspector.particles.it_starts_stopped_u_a_signal_switches_it_on" => {
            "It starts stopped \u{2014} a signal switches it on."
        }
        "panel.inspector.particles.emission" => "Emission",
        "panel.inspector.particles.emitting" => "Emitting",
        "panel.inspector.particles.one_shot" => "One Shot",
        "panel.inspector.particles.emission_shape" => "Emission Shape",
        "panel.inspector.particles.point" => "Point",
        "panel.inspector.particles.disc" => "Disc",
        "panel.inspector.particles.ring" => "Ring",
        "panel.inspector.particles.rect" => "Rect",
        "panel.inspector.particles.velocity" => "Velocity",
        "panel.inspector.particles.forces" => "Forces",
        "panel.inspector.particles.look" => "Look",
        "panel.inspector.particles.color" => "Color",
        "panel.inspector.particles.color_at_death" => "Color at Death",
        "panel.inspector.particles.simulation_space" => "Simulation Space",
        "panel.inspector.particles.world" => "World",
        "panel.inspector.particles.local" => "Local",
        "panel.inspector.particles.signals" => "Signals",
        "panel.inspector.particles.switch_on_u" => "switch on\u{2026}",
        "panel.inspector.particles.switch_off_u" => "switch off\u{2026}",
        "panel.inspector.particles.restart_u" => "restart\u{2026}",
        "panel.inspector.particles.shout_when_done_u" => "shout when done\u{2026}",
        "panel.inspector.particles.particles" => "Particles",
        "panel.inspector.particles.editing_the_primary_selection_only" => {
            "Editing the primary selection only."
        }
        "panel.inspector.physics.only_for_tag_u_any" => "Only for tag\u{2026}  (any)",
        "panel.inspector.physics.that_tag_was_deleted_u_this_reaches_nobody" => {
            "That tag was deleted \u{b7} this reaches nobody."
        }
        "panel.inspector.physics.any" => "(any)",
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
        "panel.inspector.projectile.speed_m_s" => "Speed",
        "panel.inspector.projectile.acceleration" => "Acceleration",
        "panel.inspector.projectile.max_speed_0_no_cap" => "Max Speed (0 = no cap)",
        "panel.inspector.projectile.gravity_0_straight" => "Gravity (0 = straight)",
        "panel.inspector.projectile.max_bounces" => "Max Bounces",
        "panel.inspector.projectile.bounciness_1_perfect" => "Bounciness (1 = perfect)",
        "panel.inspector.projectile.range_m_0_forever" => "Range (m, 0 = forever)",
        "panel.inspector.projectile.homing_0_none" => "Homing (0 = none)",
        "panel.inspector.projectile.target_object_name_u" => "target object name\u{2026}",
        "panel.inspector.projectile.no_object_in_the_scene_has_that_name" => {
            "No object in the scene has that name."
        }
        "panel.inspector.projectile.face_velocity" => "Face Velocity",
        "panel.inspector.projectile.projectile_motion" => "Projectile Motion",
        "panel.inspector.projectile.editing_the_primary_selection_only" => {
            "Editing the primary selection only."
        }
        "panel.inspector.script.reset" => "Reset",
        "panel.inspector.script.no_script_file_yet_u_use_browse_to_pick_a_luau_file" => {
            "No script file yet \u{2014} use Browse to pick a .luau file."
        }
        "panel.inspector.script.scripting_is_not_available_in_this_session" => {
            "Scripting is not available in this session."
        }
        "panel.inspector.script.reading_the_file_u" => "Reading the file\u{2026}",
        "panel.inspector.script.that_file_is_gone_u_pick_it_again" => {
            "That file is gone \u{2014} pick it again."
        }
        "panel.inspector.script.the_script_has_an_error" => "The script has an error: {msg}",
        "panel.inspector.script.this_script_offers_no_properties" => {
            "This script offers no properties."
        }
        "panel.inspector.script.this_object_is_also_moved_by_physics_u_the_two_fight" => {
            "This object is also moved by physics \u{2014} the two fight."
        }
        "panel.inspector.script.the_clock_is_stopped_u_scripts_only_run_while_the_scene_plays" => {
            "The clock is stopped \u{2014} scripts only run while the scene plays."
        }
        "panel.inspector.script.script" => "Script",
        "panel.inspector.script.multiple_selected_u_edits_apply_to_the_active_object_only" => {
            "Multiple selected \u{b7} edits apply to the active object only."
        }
        "panel.inspector.script.script_file_u" => "script file\u{2026}",
        "panel.inspector.script.browse" => "Browse",
        "panel.inspector.script.no_longer_in_the_script" => "No longer in the script:",
        "panel.inspector.script.not_in_the_script" => "not in the script",
        "panel.inspector.script.the_script_now_wants_a" => "the script now wants a {tipo}",
        "panel.inspector.script.remove" => "Remove",
        "panel.inspector.statemachine.never_fires" => "(never fires)",
        "panel.inspector.statemachine.state_machine" => "State Machine",
        "panel.inspector.statemachine.multiple_selected_u_edits_apply_to_the_active_object_only" => {
            "Multiple selected \u{b7} edits apply to the active object only."
        }
        "panel.inspector.statemachine.now" => "Now: {nome}",
        "panel.inspector.statemachine.the_clock_is_stopped_u_it_only_thinks_while_the_scene_plays" => {
            "The clock is stopped \u{2014} it only thinks while the scene plays."
        }
        "panel.inspector.statemachine.initial_state" => "Initial state",
        "panel.inspector.statemachine.no_states_yet" => "No states yet.",
        "panel.inspector.statemachine.add_state" => "+ Add State",
        "panel.inspector.statemachine.x_remove_state" => "x Remove State",
        "panel.inspector.statemachine.state_name_u" => "state name\u{2026}",
        "panel.inspector.statemachine.on_enter_signal_name" => "on enter: signal name",
        "panel.inspector.statemachine.on_exit_signal_name" => "on exit: signal name",
        "panel.inspector.statemachine.dead_end_no_transition_leaves_this_state" => {
            "Dead end: no transition leaves this state."
        }
        "panel.inspector.statemachine.add_transition" => "+ Add Transition",
        "panel.inspector.statemachine.x_remove_transition" => "x Remove Transition",
        "panel.inspector.statemachine.from_state" => "From state",
        "panel.inspector.statemachine.to_state" => "To state",
        "panel.inspector.statemachine.on_signal_u" => "on signal\u{2026}",
        "panel.inspector.statemachine.this_transition_never_fires_it_has_no_signal_name" => {
            "This transition never fires: it has no signal name."
        }
        "panel.inspector.tags.tags" => "Tags",
        "panel.inspector.tags.tags_count" => "Tags  ({n})",
        "panel.inspector.tags.multiple_selected_u_tag_edits_apply_to_the_active_object_only" => {
            "Multiple selected \u{b7} tag edits apply to the active object only."
        }
        "panel.inspector.tags.no_tags_yet" => "No tags yet.",
        "panel.inspector.tags.this_object_holds_the_most_tags_the_section_can_show_remove_one_to_add_another" => {
            "This object holds the most tags the section can show. Remove one to add another."
        }
        "panel.inspector.topdown.no_body_u_add_a_rigid_body_for_this_to_move_anything" => {
            "No body \u{2014} add a Rigid Body for this to move anything."
        }
        "panel.inspector.topdown.the_body_must_be_kinematic_u_a_dynamic_body_belongs_to_the_solver" => {
            "The body must be Kinematic \u{2014} a dynamic body belongs to the solver."
        }
        "panel.inspector.topdown.a_platform_player_on_this_object_wins_u_remove_one_of_the_two" => {
            "A Platform Player on this object wins \u{2014} remove one of the two."
        }
        "panel.inspector.topdown.the_clock_is_stopped_u_it_moves_while_the_clock_plays" => {
            "The clock is stopped \u{2014} it moves while the clock plays."
        }
        "panel.inspector.topdown.speed_m_s" => "Speed",
        "panel.inspector.topdown.acceleration_0_instant" => "Acceleration (0 = instant)",
        "panel.inspector.topdown.deceleration_0_instant" => "Deceleration (0 = instant)",
        "panel.inspector.topdown.board_angle_deg" => "Board Angle",
        "panel.inspector.topdown.turn_speed_deg_s_0_instant" => "Turn Speed (deg/s, 0 = instant)",
        "panel.inspector.topdown.min_slide_angle_deg" => "Min Slide Angle",
        "panel.inspector.topdown.max_slides" => "Max Slides",
        "panel.inspector.topdown.directions" => "Directions",
        "panel.inspector.topdown.viewpoint" => "Viewpoint",
        "panel.inspector.topdown.facing" => "Facing",
        "panel.inspector.topdown.default_controls" => "Default Controls",
        "panel.inspector.topdown.off_u_the_keys_don_t_reach_it_something_else_must_drive_it" => {
            "Off \u{2014} the keys don't reach it; something else must drive it."
        }
        "panel.inspector.topdown.top_down_player" => "Top-Down Player",
        "panel.inspector.topdown.editing_the_primary_selection_only" => {
            "Editing the primary selection only."
        }
        "panel.inspector.script.stopped_fix_and_save" => {
            "Stopped: {msg} \u{2014} fix the script and save it."
        }
        "panel.inspector.script.values_kept_until_reload" => {
            "{n} value(s) kept until the script loads again."
        }
        "panel.inspector.statemachine.state_n" => "State {i}",
        "panel.inspector.tags.showing_of_type_to_narrow" => {
            "Showing {n} of {achadas} \u{b7} type to narrow."
        }
        "panel.inspector.tags.create_named" => "+ Create \u{201c}{nome}\u{201d}",
        // ⭐⭐⭐ A CUTSCENE (TOP-20 #19) — o selector e as sete razões de ela não correr.
        "panel.inspector.sequence.sequence" => "Sequence",
        "panel.inspector.sequence.cutscene" => "Cutscene",
        "panel.inspector.sequence.no_cutscenes" => "No cutscenes yet",
        "panel.inspector.sequence.none_chosen" => "None",
        "panel.inspector.sequence.clear" => "Play nothing",
        "panel.inspector.sequence.make_one_in_the_timeline" => {
            "No cutscenes in the timeline yet \u{2014} make one with + Container."
        }
        "panel.inspector.sequence.gone" => {
            "That cutscene is gone: \u{201c}{v}\u{201d} \u{2014} nothing plays."
        }
        "panel.inspector.sequence.nothing_plays" => {
            "No cutscene chosen \u{2014} this object plays nothing."
        }
        "panel.inspector.sequence.no_timer" => {
            "This object has no Timer \u{2014} the cutscene has no clock."
        }
        "panel.inspector.sequence.clock_stopped" => {
            "The clock is stopped \u{2014} a cutscene runs while it plays."
        }
        "panel.inspector.sequence.not_running" => {
            "Not running \u{2014} send this object a Start Timer."
        }
        "panel.inspector.sequence.timer_too_short" => {
            "The timer ends at {t} s and the cutscene is {c} s \u{2014} it never reaches the end."
        }
        "panel.inspector.sequence.timeline_solo" => {
            "The timeline is editing keys \u{2014} cutscenes pause until you go back to Arrange."
        }
        "panel.inspector.sequence.now" => "Now: {t} s of {c}",
        "panel.inspector.sequence.primary_only" => "Editing the primary selection only.",
        // ⭐⭐⭐ A VIGIA DO CONTADOR — as regras «quando o número X chegar a N, diz S».
        "panel.inspector.counter_watch.counter_watch" => "Counter Watch",
        "panel.inspector.counter_watch.title_count" => "Counter Watch ({n})",
        // ⚠️ **O título CONTA as regras partidas**, porque é a única coisa visível com a secção
        // dobrada — que é o estado em que uma regra órfã passa despercebida.
        "panel.inspector.counter_watch.title_count_broken" => {
            "Counter Watch ({n}, {broken} broken)"
        }
        "panel.inspector.counter_watch.no_rules_yet" => "No rules yet",
        "panel.inspector.counter_watch.plus_add_rule" => "+ Add Rule",
        "panel.inspector.counter_watch.x_remove_rule" => "x Remove Rule",
        "panel.inspector.counter_watch.counter_name" => "Counter name",
        "panel.inspector.counter_watch.compare" => "When",
        // ⛔ `at_most`/`at_least`/`exactly` foram APAGADAS em 2026-09-17 (ordem do dono: o chip
        //    mostra `≤ ≥ =`). Um símbolo matemático não é língua, logo não tem chave — e o censo
        //    de dois lados deste painel reprovaria uma chave sem leitor de qualquer maneira.
        "panel.inspector.counter_watch.value" => "Value",
        "panel.inspector.counter_watch.signal_name_empty_mute" => "Signal name (empty = mute)",
        "panel.inspector.counter_watch.only_once" => "Only once",
        "panel.inspector.counter_watch.mute" => "mute",
        "panel.inspector.counter_watch.no_counter_named_yet" => {
            "This rule watches no counter yet \u{2014} name one above."
        }
        "panel.inspector.counter_watch.there_is_no_counter_called" => {
            "There is no counter called \u{201c}{name}\u{201d} \u{2014} this rule never fires."
        }
        "panel.inspector.counter_watch.this_rule_says_nothing" => {
            "This rule says nothing \u{2014} give it a signal name."
        }
        "panel.inspector.counter_watch.the_clock_is_stopped" => {
            "The clock is stopped \u{2014} rules are checked while it plays."
        }
        "panel.inspector.counter_watch.now_value" => "Now: {v}",
        "panel.inspector.counter_watch.multiple_selected_edits_apply" => {
            "Editing the primary selection only."
        }
        // ⭐⭐⭐ O GATILHO — a mão de quem joga (suplente #24).
        "panel.inspector.trigger.trigger" => "Trigger",
        "panel.inspector.trigger.title_count" => "Trigger ({n})",
        // ⚠️ **O título CONTA os gatilhos partidos**, pela mesma razão da vigia: com a secção
        // dobrada ele é a única coisa visível, e é esse o estado em que um órfão passa despercebido.
        "panel.inspector.trigger.title_count_broken" => "Trigger ({n}, {broken} broken)",
        "panel.inspector.trigger.no_triggers_yet" => "No triggers yet",
        "panel.inspector.trigger.plus_add_trigger" => "+ Add Trigger",
        "panel.inspector.trigger.x_remove_trigger" => "x Remove Trigger",
        "panel.inspector.trigger.action_name" => "Action name",
        "panel.inspector.trigger.when" => "When",
        // ⚠️ **Estas TRÊS têm chave e as três da vigia não**, e a diferença não é estilo: `≤` é um
        // símbolo matemático e lê-se igual em qualquer idioma; `Press` é uma palavra.
        "panel.inspector.trigger.on_press" => "Press",
        "panel.inspector.trigger.on_release" => "Release",
        "panel.inspector.trigger.while_held" => "Hold",
        "panel.inspector.trigger.signal_name_empty_mute" => "Signal name (empty = mute)",
        "panel.inspector.trigger.mute" => "mute",
        "panel.inspector.trigger.no_action_named_yet" => {
            "This trigger listens to no action yet \u{2014} name one above."
        }
        // ⚠️⚠️ **A frase mais valiosa das quatro:** a lei CALA uma acção que o mapa não conhece
        // (senão um `Release` sobre ela dispararia em todo quadro), e sem esta linha o artista lê
        // esse silêncio como «o gatilho não funciona».
        "panel.inspector.trigger.there_is_no_action_called" => {
            "There is no action called \u{201c}{name}\u{201d} \u{2014} add it in Settings \u{203a} Input Map."
        }
        // ⭐⭐⭐ **O SEGUNDO silêncio, e a cura dele é OUTRA.** Uma acção declarada e sem ligação
        // nenhuma resolve para `Sample::default()`, logo o gatilho fica tão calado como com um
        // nome errado — e sem esta linha o painel dizia que estava tudo bem. *Duas causas, o mesmo
        // silêncio; um aviso só mandaria metade dos artistas ao sítio errado.*
        "panel.inspector.trigger.the_action_has_no_key" => {
            "\u{201c}{name}\u{201d} has no key yet \u{2014} bind one in Settings \u{203a} Input Map."
        }
        "panel.inspector.trigger.this_trigger_says_nothing" => {
            "This trigger says nothing \u{2014} give it a signal name."
        }
        // ⚠️ **Não é uma cerca de segurança:** as teclas do jogo são as teclas do editor.
        "panel.inspector.trigger.the_clock_is_stopped" => {
            "The clock is stopped \u{2014} triggers listen while it plays."
        }
        "panel.inspector.trigger.multiple_selected_edits_apply" => {
            "Editing the primary selection only."
        }
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
