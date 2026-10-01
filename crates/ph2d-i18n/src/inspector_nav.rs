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
        "panel.inspector.nav.radius" => "Radius",
        "panel.inspector.nav.arrive_at" => "Arrive At",
        "panel.inspector.nav.repath_after" => "Repath At",
        "panel.inspector.nav.stuck_after" => "Stuck After",
        "panel.inspector.nav.active" => "Active",
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
        _ => return None,
    })
}
