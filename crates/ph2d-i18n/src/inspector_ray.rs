//! **O grupo «Filter» do cartão Ray Sensor** (GEF 07, versão C): os modos do «Only see», o título, o resumo
//! fechado e as dicas. Irmão da `inspector_game.rs`, cortado pelo tecto de 700 linhas da workspace.

/// A tradução de uma chave do Filter do raio, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.ray.see_everything" => "Everything",
        "panel.inspector.ray.see_only_tag" => "Only tag",
        "panel.inspector.ray.see_except_tag" => "All except tag",
        // O grupo «Filter» (GEF 07, versão C): o título, o resumo fechado e as dicas.
        "panel.inspector.ray.filter" => "Filter",
        "panel.inspector.ray.filter_ignores_me" => "Ignores me",
        "panel.inspector.ray.filter_only_x" => "Only {tag}",
        "panel.inspector.ray.filter_not_x" => "Not {tag}",
        "panel.inspector.ray.filter_no_tag" => "no tag",
        "panel.inspector.ray.filter_sees_me" => "Sees me",
        "panel.inspector.ray.filter_sees_my_parts" => "Sees my parts",
        "panel.inspector.ray.filter_sees_triggers" => "Sees triggers",
        "panel.inspector.ray.filter_from_inside" => "From inside",
        "panel.inspector.ray.filter_all_hits" => "All hits",
        "panel.inspector.ray.filter_sep" => ", ",
        "panel.inspector.ray.ignore_body_tip" => {
            "On: the ray never sees the body it belongs to. Off: it can hit its own body."
        }
        "panel.inspector.ray.ignore_parts_tip" => {
            "On: the ray skips the other bodies of the same character (arms, sword). Off: it sees them."
        }
        "panel.inspector.ray.only_see_tip" => {
            "Everything: the ray sees anything in its layers. Only tag: just things with the tag. All except tag: anything but them."
        }
        "panel.inspector.ray.see_tag_tip" => "The tag this filter looks for.",
        "panel.inspector.ray.walls_block_tip" => {
            "On: a wall without the tag hides what is behind it. Off: the ray goes through anything without the tag."
        }
        "panel.inspector.ray.hit_triggers_tip" => {
            "On: the ray also sees trigger areas. Off: it passes through them."
        }
        "panel.inspector.ray.starts_inside_tip" => {
            "On: if the ray starts inside something, it sees it at once. Off: it ignores what it starts in."
        }
        "panel.inspector.ray.all_hits_tip" => {
            "On: the ray sees everything along its line, not just the first thing. Each one sends the signals."
        }
        "panel.inspector.ray.and_x_at_y_m" => "and {name}, at {dist}",
        "panel.inspector.ray.and_n_more" => "and {n} more",
        _ => return None,
    })
}
