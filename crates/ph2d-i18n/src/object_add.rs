//! **O MENU ADD DE OBJECTOS** — o modal que o `+` da Hierarquia e o `Shift+A` abrem
//! (`docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`, F1).
//!
//! ⚠️ Cada rótulo de item leva o NOME DO TIPO (`Vector Object`, e não `Object`): a busca da paleta
//! só lê o rótulo do item, e *«vector»* tem de achar o vetor.

/// A tradução de uma chave deste menu, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // W18: o objecto Macro (`ph2d_app_macros::object_add`).
        "object_add.macro" => "Macro",
        // GameEngineFuncional N1: os objectos do jogo de plataforma (`ph2d_app_game::object_add`).
        "object_add.game.sprite" => "Sprite",
        "object_add.game.animated_sprite" => "Animated Sprite",
        "object_add.game.static_body" => "Static Body",
        "object_add.game.character" => "Character",
        "object_add.game.collision_shape" => "Collision Shape",
        // As dicas ilustradas do menu (05_N1_plano.md §4): o que o objecto é, para que serve.
        "object_add.game.sprite.tip" => {
            "Shows a picture in the game world. Move, scale and tint it like any object. It does not collide or move by itself \u{2014} use it for scenery, signs and decorations."
        }
        "object_add.game.animated_sprite.tip" => {
            "A picture that plays animations from a sprite sheet. Each animation is a range of frames on the sheet (idle, run, jump\u{2026}), chosen in the Inspector. It comes with a little ready-made character, so you see it move as soon as the game runs."
        }
        "object_add.game.static_body.tip" => {
            "A solid block that never moves: floors, platforms and walls. Characters and falling bodies stop on it. Scale it to make it longer or taller \u{2014} the collision follows the picture."
        }
        "object_add.game.character.tip" => {
            "The hero the player controls. The arrow keys (or A and D) walk, Up or Z jumps, with gravity built in. Its animation changes by itself between idle, run, jump and fall \u{2014} choose which animation plays in each case in the Inspector."
        }
        "object_add.game.collision_shape.tip" => {
            "An invisible shape that things bump into. Drag it inside a Static Body or a Character to give that body one more shape: an invisible wall at the end of a level, a ledge, a wider hit area."
        }
        "object_add.game.camera.tip" => {
            "The game\u{2019}s view: what the player sees when the game runs. Add Camera Follow to make it follow a character, and Camera Limits to keep it inside the level."
        }
        "object_add.game.body.tip" => {
            "An object that falls, bounces and gets pushed \u{2014} a crate, a ball, a rock. Gravity and collisions move it while the game runs."
        }
        "object_add.game.sound.tip" => {
            "Plays a sound in the scene: music, an effect or an ambience. It can loop and start by itself when the game starts, and it fades out with distance."
        }
        "object_add.game.hud.tip" => {
            "A layer drawn on top of the game and fixed to the screen: lives, score, timers and buttons stay in place while the camera moves."
        }
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
        "object_add.motion" => "Motion",
        "object_add.vector.drawing" => "Vector Drawing",
        "object_add.game.camera" => "Camera",
        "object_add.game.body" => "Physics Body",
        "object_add.game.sound" => "Sound",
        "object_add.game.hud" => "HUD",
        "object_add.light.point" => "Light",
        "object_add.light.ambient" => "Ambient Light",
        "object_add.added" => "Added {name}",
        "object_add.failed" => "Could not add the object: {e}",
        "object_add.not_born" => "the object did not appear in the scene",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
