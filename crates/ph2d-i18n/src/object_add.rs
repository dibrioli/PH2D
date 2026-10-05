//! **O MENU ADD DE OBJECTOS** — o modal que o `+` da Hierarquia e o `Shift+A` abrem
//! (`docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`, F1).
//!
//! ⚠️ Cada rótulo de item leva o NOME DO TIPO (`Vector Object`, e não `Object`): a busca da paleta
//! só lê o rótulo do item, e *«vector»* tem de achar o vetor.

/// A tradução de uma chave deste menu, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "object_add.title" => "Add Object",
        "object_add.group.two_d" => "2D",
        "object_add.group.game" => "Game",
        "object_add.group.empty" => "Empty",
        "object_add.empty" => "Empty",
        "object_add.image" => "Image\u{2026}",
        "object_add.flip" => "Flip Drawing",
        "object_add.flip.first_layer" => "Layer",
        "object_add.skeleton" => "Skeleton",
        "object_add.vector.drawing" => "Vector Drawing",
        "object_add.game.camera" => "Camera",
        "object_add.game.body" => "Physics Body",
        "object_add.game.sound" => "Sound",
        "object_add.game.hud" => "HUD",
        "object_add.added" => "Added {name}",
        "object_add.failed" => "Could not add the object: {e}",
        "object_add.not_born" => "the object did not appear in the scene",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
