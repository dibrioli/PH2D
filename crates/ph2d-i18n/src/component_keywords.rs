//! ⭐ **AS PALAVRAS DE INTENÇÃO do *Add Component*** (doc ObjectsAndComponents 01 §7.3 e §13, F6).
//!
//! O artista procura pelo que quer que o objecto FAÇA («fall», «jump», «play sound»), e não pelo nome
//! do componente. Cada componente que a paleta oferece tem aqui a lista de verbos e substantivos que o
//! acham, separados por vírgula. Escolha do dono (08/10): **só inglês**, a língua da interface.
//!
//! A chave deriva do nome de exibição (`component.<snake>.name` → `component.<snake>.keywords`,
//! [`ph2d_component_desc::ComponentDesc::keywords_key`]); o gate
//! `every_offered_component_has_intent_words` prova que nenhum componente oferecível ficou sem lista.
//! ⚠️ As palavras nunca se pintam: só a busca as lê.

/// A lista de intenção de uma chave `component.*.keywords`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // Anchors
        "component.anchor_mount.keywords" => "attach, mount, socket, hold, carry, snap to anchor",
        "component.named_anchor_list.keywords" => "anchor, socket, attach point, hand, slot",
        // Animation
        "component.sprite_animations.keywords" => {
            "animate, frames, sprite sheet, flipbook, walk cycle, frame range"
        }
        "component.sprite_animator.keywords" => {
            "animate, play animation, frames, flipbook, walk, run, idle"
        }
        // Audio
        "component.audio_listener_2d.keywords" => "hear, listen, ears, microphone, sound position",
        "component.audio_source_2d.keywords" => {
            "play sound, sound, audio, music, sfx, noise, voice, speaker"
        }
        // Camera
        "component.camera_follow.keywords" => "follow, track, chase camera, keep in view",
        "component.camera_limits.keywords" => "bounds, clamp camera, level edges, stay inside",
        "component.camera_shake.keywords" => "shake, screen shake, tremble, impact, rumble",
        "component.game_camera.keywords" => "camera, view, viewport, look, see, frame the game",
        // Light
        "component.bloqueia_luz.keywords" => "shadow, block light, cast shadow, occluder, wall",
        "component.nao_recebe_luz.keywords" => {
            "unlit, ignore light, always bright, hud, no shading"
        }
        "component.scroll_factor.keywords" => "parallax, depth, background, scroll, layers",
        "component.scroll_limits.keywords" => "parallax bounds, scroll limits, clamp background",
        "component.scroll_motion.keywords" => "drift, scroll, clouds, moving background, conveyor",
        "component.scroll_repeat.keywords" => "repeat, tile, endless, infinite background, loop",
        "component.shake_emitter.keywords" => "shake, explosion, impact, rumble, earthquake",
        // Identity
        "component.tags.keywords" => "tag, label, group, category, find, team",
        // Image
        "component.slice_nine.keywords" => "nine slice, stretch, panel, border, scalable frame",
        // Logic
        "component.counter.keywords" => "count, score, points, coins, number, tally",
        "component.counter_watch.keywords" => "when score, react to count, threshold, win, goal",
        "component.damage.keywords" => "hurt, hit, damage, attack, spikes, kill, harm",
        "component.destroy_outside.keywords" => "remove offscreen, cleanup, despawn, out of bounds",
        "component.factory.keywords" => "spawn, create, generate, clone, emit objects, spawner",
        "component.health_bar.keywords" => "life bar, hp bar, health display, hearts",
        "component.health.keywords" => "life, hp, health, die, hit points, damage, alive",
        "component.lifetime.keywords" => "expire, timeout, disappear, self destroy, fade out",
        "component.path_follow.keywords" => "follow path, patrol, move along, rail, route",
        "component.sequence_player.keywords" => "sequence, cutscene, steps, script steps, play",
        "component.signal_actions.keywords" => "when, on event, react, trigger action, respond",
        "component.signal_on_action.keywords" => "trigger, on input, press, event, fire signal",
        "component.state_machine.keywords" => "states, behavior, ai, idle, walk, attack, mode",
        "component.timers.keywords" => "timer, delay, wait, countdown, every seconds, repeat",
        "component.tweens.keywords" => "tween, ease, animate value, smooth move, lerp, fade",
        "component.ui_button.keywords" => "button, click, press, menu, ui",
        "component.ui_canvas.keywords" => "hud, ui, screen overlay, interface, menu",
        "component.ui_label.keywords" => "text, label, score text, ui text, caption",
        "component.weapon_fire.keywords" => "shoot, fire, gun, weapon, projectile, bullet",
        // Ordering
        "component.clip_children.keywords" => "clip, crop, mask children, cut off, window",
        // Physics
        "component.nav_agent.keywords" => "pathfinding, navigate, walk to, find path, ai move",
        "component.nav_cost_area.keywords" => "slow zone, avoid, pathfinding cost, mud",
        "component.nav_link.keywords" => "jump link, ladder, teleport, connect paths",
        "component.nav_region.keywords" => "walkable area, navmesh, floor, where to walk",
        "component.projectile_motion.keywords" => "bullet, arrow, throw, launch, projectile",
        "component.ray_sensor.keywords" => "raycast, detect, sense, see ahead, line of sight",
        "component.rigid_body.keywords" => {
            "fall, gravity, physics, jump, platformer, bounce, collide, push, weight, solid"
        }
        "component.top_down_player.keywords" => {
            "move, walk, player, top down, controls, keyboard, character"
        }
        // Rendering
        "component.blend_mode.keywords" => "blend, multiply, add, screen, overlay, glow",
        "component.mask_2d.keywords" => "mask, cut out, stencil, hide part, reveal",
        "component.mask_interaction.keywords" => "inside mask, outside mask, masked by",
        "component.on_screen_enabler.keywords" => "only when visible, offscreen pause, optimize",
        "component.particle_emitter.keywords" => {
            "particles, sparks, smoke, fire, dust, rain, snow, effects"
        }
        "component.texture_filter.keywords" => "pixel art, smooth, crisp, nearest, filtering",
        "component.texture_repeat.keywords" => "tile, repeat texture, pattern, wrap",
        "component.uv_transform.keywords" => "scroll texture, offset texture, uv, flow",
        "component.visibility_layer.keywords" => "layer, show only, hide from camera, render layer",
        // Scripting
        "component.luau_script.keywords" => "code, script, program, custom behavior, logic",
        // Skeleton
        "component.bone_limit.keywords" => "joint limit, constrain, angle, elbow, knee",
        "component.bone.keywords" => "bone, rig, joint, skeleton, limb",
        "component.smart_bone.keywords" => "pose driver, corrective, smart action, drive shape",
        // Transform
        "component.transform.keywords" => "position, move, rotate, scale, place",
        _ => return None,
    })
}
