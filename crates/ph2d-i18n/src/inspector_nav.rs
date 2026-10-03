//! **As palavras das secções NAV REGION e NAV AGENT do Inspector** (plano 30, W4).
//!
//! ⚠️ **Um ficheiro próprio**, pela razão do [`super::inspector_vida`]: uma família nova, e o
//! `inspector_game` perto do tecto.
//!
//! ⚠️ **Os rótulos das linhas são CURTOS de propósito**: a varredura das elisões mede-os em quatro
//! larguras de painel, e a explicação longa mora no aviso ou no `placeholder`, que quebram a linha.

/// A tradução de uma chave `panel.inspector.nav.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.nav.nav_region" => "Nav Region",
        "panel.inspector.nav.nav_agent" => "Nav Agent",
        "panel.inspector.nav.half_size" => "Half Size",
        "panel.inspector.nav.blocking_layers" => "Wall Layers",
        "panel.inspector.nav.region_has_no_size" => {
            "The region has no size \u{2014} no agent can walk in it."
        }
        "panel.inspector.nav.no_layer_blocks" => {
            "No layer is a wall \u{2014} agents walk straight through everything."
        }
        "panel.inspector.nav.target" => "Target",
        "panel.inspector.nav.target_none" => "None",
        "panel.inspector.nav.target_object" => "Object",
        "panel.inspector.nav.target_point" => "Point",
        "panel.inspector.nav.object_name_u" => "object name\u{2026}",
        "panel.inspector.nav.target_tag" => "Tag",
        "panel.inspector.nav.target_patrol" => "Patrol",
        "panel.inspector.nav.shape" => "Shape",
        "panel.inspector.nav.shape_name_u" => "shape name\u{2026}",
        "panel.inspector.nav.pick_a_tag_u" => "pick a tag\u{2026}",
        "panel.inspector.nav.that_tag_was_deleted" => {
            "That tag was deleted \u{2014} it chases nobody."
        }
        "panel.inspector.nav.shape_lost" => {
            "No drawn shape has that name \u{2014} write the name of a shape drawn with the pen."
        }
        "panel.inspector.nav.radius" => "Radius",
        "panel.inspector.nav.arrive_at" => "Arrive At",
        "panel.inspector.nav.repath_after" => "Repath At",
        "panel.inspector.nav.stuck_after" => "Stuck After",
        "panel.inspector.nav.active" => "Active",
        "panel.inspector.nav.avoidance" => "Avoid Others",
        "panel.inspector.nav.on_arrive" => "On Arrive",
        "panel.inspector.nav.on_no_path" => "On No Path",
        "panel.inspector.nav.on_stuck" => "On Stuck",
        "panel.inspector.nav.signal_when_it_arrives_u" => "signal when it arrives\u{2026}",
        "panel.inspector.nav.signal_when_there_is_no_way_u" => {
            "signal when there is no way\u{2026}"
        }
        "panel.inspector.nav.signal_when_it_gets_stuck_u" => "signal when it gets stuck\u{2026}",
        // ⚠️ **As QUEIXAS, da mais específica para a mais geral** — a ordem vive na porta
        // `InspectorNavAgent::queixa`; estas são só a língua dela.
        "panel.inspector.nav.no_body" => {
            "No body \u{2014} add a Rigid Body; only bodies are steered."
        }
        "panel.inspector.nav.no_mover" => {
            "No mover \u{2014} add a Top-Down Player: the agent asks, the mover walks."
        }
        "panel.inspector.nav.mover_reads_keys" => {
            "The Top-Down Player also reads the keys \u{2014} turn off its Default Controls."
        }
        "panel.inspector.nav.platformer_wins" => {
            "A Platform Player on this object wins \u{2014} remove one of the two."
        }
        "panel.inspector.nav.switched_off" => "Switched off \u{2014} it stands still.",
        // ⭐ A ORDEM de um verbo manda mais que a caixa — o painel diz quem manda AGORA.
        "panel.inspector.nav.stopped_by_an_action" => {
            "Stopped by an action \u{2014} a Start sets it going again."
        }
        "panel.inspector.nav.started_by_an_action" => "Started by an action.",
        "panel.inspector.nav.started_by_an_action_after_x" => {
            "Started by an action \u{2014} after {name}."
        }
        "panel.inspector.nav.no_target" => "No target \u{2014} it stands still.",
        "panel.inspector.nav.target_lost" => {
            "Nobody has that name any more \u{2014} write the name of an object in the scene."
        }
        "panel.inspector.nav.outside_regions" => {
            "Not inside any Nav Region \u{2014} there is no map where it stands."
        }
        // ⭐ A LEITURA VIVA — o que ele faz AGORA, do `NavNow` que a ponte publica.
        "panel.inspector.nav.idle" => "Idle.",
        "panel.inspector.nav.moving_x_m_to_go" => "Moving \u{b7} {dist} m to go",
        "panel.inspector.nav.cant_reach_x_m" => {
            "Can\u{2019}t reach it \u{b7} going to the nearest point, {dist} m"
        }
        "panel.inspector.nav.arrived" => "Arrived.",
        "panel.inspector.nav.no_way_there" => "No way there.",
        "panel.inspector.nav.radius_x_m_from_the_collider" => "Radius {r} m, from the collider",
        "panel.inspector.nav.the_clock_is_stopped" => {
            "The clock is stopped \u{2014} it walks while the clock plays."
        }
        // ⭐ (W7) O que FERE, o que CUSTA e os ATALHOS.
        "panel.inspector.nav.avoid_harm" => "Avoid Harm",
        "panel.inspector.nav.no_health_nothing_hurts_it" => {
            "No Health \u{2014} nothing hurts it, so it avoids nothing."
        }
        "panel.inspector.nav.nav_cost_area" => "Nav Cost Area",
        "panel.inspector.nav.cost" => "Cost",
        "panel.inspector.nav.forbidden" => "Forbidden",
        "panel.inspector.nav.no_agent_enters" => "No agent enters.",
        "panel.inspector.nav.area_needs_a_collider" => {
            "Needs a Collider \u{2014} the area is its shape."
        }
        "panel.inspector.nav.area_body_moves" => {
            "A Dynamic body moves \u{2014} make it Static so the area stays put."
        }
        "panel.inspector.nav.nav_link" => "Nav Link",
        "panel.inspector.nav.link_exit" => "Exit",
        "panel.inspector.nav.exit_object_name_u" => "exit object name\u{2026}",
        "panel.inspector.nav.teleport" => "Teleport",
        "panel.inspector.nav.two_way" => "Both Ways",
        "panel.inspector.nav.extra_cost" => "Extra Cost",
        "panel.inspector.nav.on_cross" => "On Cross",
        "panel.inspector.nav.signal_when_an_agent_crosses_u" => {
            "signal when an agent crosses\u{2026}"
        }
        "panel.inspector.nav.link_has_no_exit" => {
            "No exit \u{2014} write the name of the object where it comes out."
        }
        "panel.inspector.nav.link_exit_lost" => {
            "Nobody has that name any more \u{2014} the link leads nowhere."
        }
        _ => return None,
    })
}
