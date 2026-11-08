//! **O ADD COMPONENT em 11 assuntos** (`docs/GameEngineFuncional/25_add_component_plano.md`): os títulos dos
//! grupos, a frase de cada dica (Plain English, para qualquer país), o porquê de cada peça que vem junto,
//! o recibo e o ensino da busca. Os dados moram em `ph2d_game_dicas::componente`.

/// A tradução de uma chave deste painel, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "component_add.group.move" => "Move and collide",
        "component_add.group.path" => "Find a path",
        "component_add.group.rules" => "Game rules",
        "component_add.group.screen" => "On screen",
        "component_add.group.camera" => "Camera",
        "component_add.group.background" => "Background",
        "component_add.group.time" => "Change over time",
        "component_add.group.look" => "Look",
        "component_add.group.sound" => "Sound",
        "component_add.group.bones" => "Bones",
        "component_add.group.organize" => "Organize",
        // A dica: a frase, depois as palavras da busca, depois o que vem junto.
        "component_add.plus_n" => "+{n}",
        "component_add.tip.words" => "Words: {words}.",
        "component_add.tip.also_adds" => "Also adds: {names}.",
        // Move and collide
        "component_add.tip.rigid_body" => {
            "It falls, bumps into things and gets pushed, with gravity built in."
        }
        "component_add.tip.top_down_player" => {
            "The player walks in every direction with the arrow keys, in a game seen from above."
        }
        "component_add.tip.projectile_motion" => {
            "It flies by itself once it starts: a bullet, an arrow, a thrown stone. It can fall, bounce or chase."
        }
        "component_add.tip.path_follow" => {
            "It travels along a drawn path: a lift, a moving platform, a guard on patrol."
        }
        "component_add.tip.motion_input" => {
            "Sends a number from the game (health, damage, or a value you set) into a Motion graph, so the effect reacts to it."
        }
        "component_add.tip.ray_sensor" => {
            "An invisible line that asks \u{201c}is something there?\u{201d}. Enemies use it to find a wall or the edge of a floor."
        }
        // Find a path
        "component_add.tip.nav_agent" => "Walks to a goal on its own, going around walls.",
        "component_add.tip.nav_region" => "Marks the floor where walkers can go.",
        "component_add.tip.nav_cost_area" => {
            "A slow zone, like mud: walkers avoid it when there is a faster way."
        }
        "component_add.tip.nav_link" => {
            "Joins two places a walker cannot reach on foot: a ladder, a jump, a door."
        }
        // Game rules
        "component_add.tip.signal_actions" => {
            "The rules of the object: \u{201c}when this signal is heard, do that\u{201d}."
        }
        "component_add.tip.signal_on_action" => {
            "Listens for a player action you set in the Input Map, and sends a signal when it is pressed."
        }
        "component_add.tip.counter" => "Counts something with a name: score, coins, lives.",
        "component_add.tip.counter_watch" => {
            "Watches a counter and sends a signal when it reaches a number, like \u{201c}10 coins: you win\u{201d}."
        }
        "component_add.tip.timers" => {
            "A clock that sends a signal after a wait, once or again and again."
        }
        "component_add.tip.lifetime" => {
            "Copies made during play disappear by themselves after some time."
        }
        "component_add.tip.destroy_outside" => {
            "Copies made during play are removed when they leave the camera\u{2019}s view."
        }
        "component_add.tip.factory" => {
            "Makes copies of an object when it hears a signal: bullets, enemies, coins."
        }
        "component_add.tip.weapon_fire" => {
            "The rhythm and the ammo of a shot: how fast it fires and how many shots are left."
        }
        "component_add.tip.damage" => "Hurts what it touches.",
        "component_add.tip.health" => {
            "Life points. At zero it stops taking hits and sends its death signal."
        }
        // On screen
        "component_add.tip.ui_canvas" => {
            "A layer stuck to the game view, for the score, the menu and the buttons."
        }
        "component_add.tip.ui_label" => "Text on the screen, like the score or a message.",
        "component_add.tip.ui_button" => "A button the player clicks, which sends a signal.",
        "component_add.tip.health_bar" => "Shows the health of an object as a bar.",
        "component_add.tip.named_anchor_list" => {
            "Named points on the object, like a hand or a hat spot, where other objects can attach."
        }
        "component_add.tip.anchor_mount" => {
            "Sticks this object, as a child, to a named point on its parent, like a sword in a hand."
        }
        "component_add.tip.clip_children" => {
            "Hides the parts of its children that fall outside the shape of this object\u{2019}s picture."
        }
        // Camera
        "component_add.tip.game_camera" => "What the player sees when the game runs.",
        "component_add.tip.camera_follow" => {
            "The camera goes after an object and keeps it in view."
        }
        "component_add.tip.camera_limits" => {
            "The camera stays inside the level and never shows past its edges."
        }
        "component_add.tip.camera_shake" => "Lets the screen shake, for hits and explosions.",
        "component_add.tip.shake_emitter" => {
            "When a signal is heard, it tells nearby cameras with Camera Shake to shake, more when it is close."
        }
        // Background
        "component_add.tip.scroll_factor" => {
            "Far away things move slower than the camera, so the scene looks deep."
        }
        "component_add.tip.scroll_limits" => "Stops the background from scrolling past a point.",
        "component_add.tip.scroll_motion" => "The background drifts by itself, like clouds.",
        "component_add.tip.scroll_repeat" => {
            "The background repeats without end as the camera moves."
        }
        // Change over time
        "component_add.tip.animation" => {
            "Animate this object in the Timeline: keys for position, color and more."
        }
        "component_add.tip.sprite_animations" => {
            "The animations of a sprite sheet: which frames make idle, run, jump."
        }
        "component_add.tip.sprite_animator" => {
            "Plays one of the sprite\u{2019}s animations and switches between them."
        }
        "component_add.tip.state_machine" => {
            "The object is in one state at a time (idle, walk, attack), and rules say when it changes."
        }
        "component_add.tip.tweens" => "Changes a value smoothly over time: slide, grow, fade.",
        "component_add.tip.sequence_player" => {
            "Plays a list of steps one after another, like a small cutscene."
        }
        "component_add.tip.luau_script" => "Your own behavior, written in Luau code.",
        // Look
        "component_add.tip.blend_mode" => {
            "How its colors mix with what is behind: add for glow, multiply for shade."
        }
        "component_add.tip.mask_2d" => {
            "Its picture becomes a shape. Every object with Mask Interaction is shown inside it or outside it."
        }
        "component_add.tip.mask_interaction" => {
            "Shows this object only inside, or only outside, a mask."
        }
        "component_add.tip.slice_nine" => {
            "Stretches the picture without stretching its corners, for panels and frames."
        }
        "component_add.tip.texture_filter" => {
            "Crisp pixels for pixel art, or smooth for painted art."
        }
        "component_add.tip.texture_repeat" => {
            "The picture repeats like tiles to fill a bigger size."
        }
        "component_add.tip.uv_transform" => {
            "Slides the picture inside the object, like water flowing."
        }
        "component_add.tip.particle_emitter" => {
            "Throws out many small pieces: sparks, smoke, dust, rain."
        }
        "component_add.tip.bloqueia_luz" => "Blocks the 2D lights and casts a shadow.",
        "component_add.tip.nao_recebe_luz" => {
            "Ignores the 2D lights, so the lights do not change how it looks."
        }
        // Sound
        "component_add.tip.audio_source_2d" => {
            "Plays a sound from this object. It sounds louder when the listener is near."
        }
        "component_add.tip.audio_listener_2d" => {
            "The ears of the game: sounds are heard from here."
        }
        // Bones
        "component_add.tip.bone" => {
            "Makes this object a bone of a skeleton, so it can bend other parts."
        }
        "component_add.tip.bone_limit" => {
            "Stops a bone from turning past an angle, like an elbow or a knee."
        }
        "component_add.tip.smart_bone" => {
            "Moves other things when a bone turns, like a muscle that bulges."
        }
        // Organize
        "component_add.tip.transform" => "Position, rotation and size.",
        "component_add.tip.tags" => {
            "Names that group objects, like \u{201c}enemy\u{201d} or \u{201c}coin\u{201d}, so rules can find them."
        }
        "component_add.tip.constraints" => {
            "Ties this object to another: follow it, look at it, stay at a distance."
        }
        "component_add.tip.visibility_layer" => "Chooses which cameras can see this object.",
        "component_add.tip.on_screen_enabler" => {
            "Pauses the object while it is off screen, to save work."
        }
        // O porquê de cada peça que vem junto.
        "component_add.why.shape" => "a body needs a shape",
        "component_add.why.body_moves" => "it moves as a body",
        "component_add.why.body_flies" => "it flies as a body",
        "component_add.why.body_touches" => {
            "it hurts by touching, and a hit needs a body to touch with"
        }
        "component_add.why.body_is_hit" => "only a body can be hit",
        "component_add.why.agent_walks" => {
            "it moves with the top-down walker, which it steers itself"
        }
        "component_add.why.clock" => "it needs a clock",
        "component_add.why.ammo" => "it counts the ammo",
        "component_add.why.parallax" => "it works on top of Parallax",
        // O recibo.
        "component_add.receipt" => "{name} added. Also added: {list}.",
        "component_add.receipt.because" => "{name}, because {why}",
        "component_add.receipt.undo" => "Undo",
        // O ensino da busca que não acha.
        "component_add.teach.platform_player" => {
            "To make it walk and jump, add Physics Body, then press Make Platform Player on its card."
        }
        "component_add.teach.platform_player.words" => {
            "platform player, platformer player, platform character"
        }
        "component_add.teach.collision_shape" => {
            "A shape comes with Physics Body. Add the body, or use Add Object \u{25b8} Collision Shape."
        }
        "component_add.teach.collision_shape.words" => "collision shape, collider, hitbox",
        "component_add.teach.start" => "Start with {name}",
        _ => return None,
    })
}
