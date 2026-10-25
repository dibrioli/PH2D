//! **As palavras das CONSTRAINTS** — o cartão do Inspector (plano
//! `docs/BlenderConstraints/01_plano.md` §4). Os nomes dos tipos são os do Blender: o artista que vem
//! de lá reconhece-os.

/// A tradução de uma chave `panel.inspector.constraint.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.constraint.child_of" => "Child Of",
        "panel.inspector.constraint.copy_location" => "Copy Location",
        "panel.inspector.constraint.copy_rotation" => "Copy Rotation",
        "panel.inspector.constraint.copy_scale" => "Copy Scale",
        "panel.inspector.constraint.copy_transforms" => "Copy Transforms",
        "panel.inspector.constraint.look_at" => "Look At",
        "panel.inspector.constraint.limit_rotation" => "Limit Rotation",
        "panel.inspector.constraint.limit_distance" => "Limit Distance",
        "panel.inspector.constraint.add" => "Add Constraint",
        "panel.inspector.constraint.target" => "Target",
        "panel.inspector.constraint.no_target" => "None",
        "panel.inspector.constraint.pick" => "Pick",
        "panel.inspector.constraint.picking" => "Click a target…",
        "panel.inspector.constraint.influence" => "Influence",
        "panel.inspector.constraint.enabled" => "Enabled",
        "panel.inspector.constraint.delete" => "Delete",
        "panel.inspector.constraint.up" => "Move Up",
        "panel.inspector.constraint.down" => "Move Down",
        "panel.inspector.constraint.empty" => "No constraints yet: add one with +.",
        "panel.inspector.constraint.full" => "The card is full.",
        "panel.inspector.constraint.state.off" => "Off",
        "panel.inspector.constraint.state.no_influence" => "Influence 0: does nothing",
        "panel.inspector.constraint.state.no_target" => "Pick a target",
        "panel.inspector.constraint.state.self_target" => "It cannot target itself",
        "panel.inspector.constraint.state.cycle" => "Cycle: two constraints follow each other",
        "panel.inspector.constraint.state.dynamic_body" => "Physics moves this body: use a joint",
        "panel.inspector.constraint.state.collapsed" => "A parent has scale 0",
        _ => return None,
    })
}
