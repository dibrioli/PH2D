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
        "object_add.game.background_layer" => "Background Layer",
        // As dicas ilustradas do menu (05_N1_plano.md §4): o que o objecto é, para que serve.
        "object_add.game.sprite.tip" => {
            "Shows a picture in the game world. Move, scale and tint it like any object. It does not collide or move by itself \u{2014} use it for scenery, signs and decorations."
        }
        "object_add.game.animated_sprite.tip" => {
            "A picture that plays animations from a sprite sheet. Each animation is a range of frames on the sheet (idle, run, jump\u{2026}), chosen in the Inspector. It comes with a little ready-made character, so you see it move as soon as the game runs."
        }
        "object_add.game.static_body.tip" => {
            "A solid block that never moves: floors, platforms and walls. Characters and falling bodies stop on it. To make it longer, set Use to Repeated and give it a Width \u{2014} the collision follows the picture."
        }
        "object_add.game.character.tip" => {
            "The hero the player controls. The arrow keys (or A and D) walk, Up or Z jumps, with gravity built in. Its animation changes by itself between idle, run, jump and fall \u{2014} choose which animation plays in each case in the Inspector."
        }
        "object_add.game.collision_shape.tip" => {
            "An invisible shape that things bump into. Select a body first and the shape goes inside it as one more shape of that body: an invisible wall at the end of a level, a ledge, a wider hit area. With nothing selected it is a fixed wall of its own."
        }
        "object_add.game.background_layer.tip" => {
            "Scenery far away: sky, clouds, hills. When the camera moves it stays behind and moves more slowly than the level, which makes the world look deep. Set how far away it is in the Inspector (Parallax) \u{2014} 0 stays still on the screen, 1 moves with the level. It repeats sideways, so it never ends."
        }
        "object_add.game.camera.tip" => {
            "The game\u{2019}s view: what the player sees when the game runs. Add Camera Follow to make it follow a character, and Camera Limits to keep it inside the level."
        }
        "object_add.game.sound.tip" => {
            "Plays a sound in the scene: music, an effect or an ambience. It can loop and start by itself when the game starts, and it fades out with distance."
        }
        "object_add.game.hud.tip" => {
            "A layer drawn on top of the game and fixed to the screen: lives, score, timers and buttons stay in place while the camera moves."
        }
        // Plano 24 §2.6 (dono 07/11): a dica das entradas que não a tinham.
        "object_add.image.tip" => {
            "A blank picture to paint on with the Painter brushes. You choose its size first (Ctrl+N does the same), and it appears in the scene ready to paint."
        }
        "object_add.pixel_sprite.tip" => {
            "A small picture made of square pixels, for retro and pixel-art games. Paint it pixel by pixel, with layers, frames and a palette of colors."
        }
        "object_add.vector.drawing.tip" => {
            "Shapes made of points and curves that stay sharp at any size. Draw with the pen or with ready-made shapes, then change the fill and the outline. Good for clean art, icons and backgrounds."
        }
        "object_add.vector.text.tip" => {
            "Words in the scene: titles, signs and labels. Type to change them, and choose the font, size and color in the Inspector."
        }
        "object_add.flip.tip" => {
            "Hand-drawn animation, frame by frame, as on paper. The frames before and after show in light colors (onion skin), so you can draw the next one."
        }
        "object_add.skeleton.tip" => {
            "Bones that move a drawing like a puppet. Join the bones, attach the art to them, and pose them to animate an arm, a leg or a whole character."
        }
        "object_add.motion.tip" => {
            "Shapes that move by themselves: they fall, bounce and collide, or follow rules you build with nodes. Good for effects, particles and lively backgrounds."
        }
        "object_add.light.point.tip" => {
            "A lamp that lights the scene and casts shadows behind solid things. Change its color, brightness and reach in the Inspector. Set Type to Ambient to light the whole scene evenly."
        }
        "object_add.empty.tip" => {
            "An object with nothing in it, only a position. Use it as a folder that groups other objects and moves them together, or add components to give it a job."
        }
        "object_add.macro.tip" => {
            "Records what you do and plays it back: a few brush strokes, a series of steps. Use it to repeat the same work on many pictures or objects."
        }
        // Plano 24 §2.5: as palavras de busca (inglês, vírgula; a lei de `component_keywords`).
        "object_add.image.keywords" => "new image, canvas, paint, draw, blank, picture, painting",
        "object_add.game.sprite.keywords" => {
            "picture, image, png, decoration, scenery, prop, sign, art"
        }
        "object_add.game.animated_sprite.keywords" => {
            "animation, sprite sheet, frames, animated, walk cycle, run cycle"
        }
        "object_add.pixel_sprite.keywords" => "pixel art, retro, 8-bit, pixels, tiles, tilemap",
        "object_add.game.background_layer.keywords" => {
            "background, sky, clouds, hills, parallax, far away, backdrop, scenery"
        }
        "object_add.vector.drawing.keywords" => {
            "vector, shapes, lines, curves, illustration, svg, pen"
        }
        "object_add.vector.text.keywords" => "words, title, label, letters, font, write, caption",
        "object_add.flip.keywords" => {
            "frame by frame, hand drawn, 2d animation, sketch, onion skin, flipbook"
        }
        "object_add.skeleton.keywords" => "bones, rig, puppet, joints, cutout, arm, leg",
        "object_add.motion.keywords" => {
            "simulation, particles, physics shapes, nodes, procedural, effects"
        }
        "object_add.game.character.keywords" => {
            "player, hero, protagonist, main character, avatar, platformer, jump, walk, run"
        }
        "object_add.game.static_body.keywords" => {
            "wall, floor, ground, platform, block, solid, terrain, ceiling, obstacle"
        }
        "object_add.game.collision_shape.keywords" => {
            "collider, hitbox, invisible wall, barrier, bounds, sensor, area"
        }
        "object_add.game.camera.keywords" => "view, viewport, screen, follow, look",
        "object_add.game.sound.keywords" => {
            "music, audio, sfx, sound effect, ambience, song, noise"
        }
        "object_add.game.hud.keywords" => {
            "ui, interface, score, lives, overlay, menu, timer, display"
        }
        "object_add.light.point.keywords" => {
            "lamp, torch, glow, shadow, lighting, bright, ambient, sun"
        }
        "object_add.empty.keywords" => "folder, group, container, parent, pivot, node",
        "object_add.macro.keywords" => "record, repeat, automation, replay, batch",
        // Plano 24 §2.5: a busca que não acha ensina de que peças a coisa se monta.
        "object_add.teach.start" => "Start with the {name}",
        "object_add.teach.coin.words" => {
            "coin, coins, money, gold, gem, gems, pickup, pickups, collectible, collectibles"
        }
        "object_add.teach.coin" => {
            "A coin is built from pieces: a Sprite for the picture, plus the components Collision Shape (to be touched), Counter (the score) and Signal Actions (what happens on touch)."
        }
        "object_add.teach.enemy.words" => {
            "enemy, enemies, monster, monsters, foe, villain, bad guy"
        }
        "object_add.teach.enemy" => {
            "An enemy is built from pieces: an Animated Sprite for the picture, plus the components Physics Body, Path Follow (to patrol) and Damage (to hurt the player)."
        }
        "object_add.teach.door.words" => "door, doors, exit, gate, portal",
        "object_add.teach.door" => {
            "A door or an exit is built from pieces: a Sprite for the picture, plus the components Collision Shape and Signal Actions (what happens when the player reaches it)."
        }
        "object_add.teach.trap.words" => "trap, traps, spike, spikes, lava, hazard, saw",
        "object_add.teach.trap" => {
            "A trap is built from pieces: a Static Body for the solid part, plus the component Damage to hurt whoever touches it."
        }
        "object_add.teach.checkpoint.words" => {
            "checkpoint, checkpoints, save point, flag, goal, finish, finish line"
        }
        "object_add.teach.checkpoint" => {
            "A checkpoint or a goal is built from pieces: a Sprite for the picture, plus the components Collision Shape and Signal Actions (what happens when the player touches it)."
        }
        "object_add.teach.bullet.words" => {
            "bullet, bullets, projectile, projectiles, arrow, shot, shoot, fireball"
        }
        "object_add.teach.bullet" => {
            "A shot is built from pieces: a Sprite, plus the components Projectile Motion and Lifetime. To fire it, give the shooter the component Weapon."
        }
        "object_add.teach.moving_platform.words" => "moving platform, elevator, lift, moving block",
        "object_add.teach.moving_platform" => {
            "A moving platform is a Static Body with the component Path Follow."
        }
        "object_add.teach.health.words" => "health, life, hp, hearts",
        "object_add.teach.health" => {
            "Health is a component, not an object: add a Character, then use Add Component to give it Health. Health Bar shows it on screen."
        }
        // ph2d-migrar-texto:begin
        "object_add.title" => "Add Object",
        "object_add.group.recent" => "Recent",
        "object_add.group.pictures" => "Pictures",
        "object_add.group.drawing" => "Drawing",
        "object_add.group.game" => "Game",
        "object_add.group.empty" => "Empty",
        "object_add.empty" => "Empty",
        "object_add.image" => "Paint a new image\u{2026}",
        "object_add.flip" => "Flip Drawing",
        "object_add.flip.first_layer" => "Layer",
        "object_add.skeleton" => "Skeleton",
        "object_add.motion" => "Motion",
        "object_add.vector.drawing" => "Vector Drawing",
        "object_add.vector.text" => "Text",
        "object_add.vector.text.placeholder" => "Text",
        "object_add.game.camera" => "Camera",
        "object_add.game.sound" => "Sound",
        "object_add.game.hud" => "HUD",
        "object_add.light.point" => "Light",
        "object_add.added" => "Added {name}",
        "object_add.failed" => "Could not add the object: {e}",
        "object_add.not_born" => "the object did not appear in the scene",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
