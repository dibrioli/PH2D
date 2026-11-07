//! **O VOCABULÁRIO DO CARTÃO «ANIMATION»** (plano GameEngineFuncional 07) — as chaves
//! `panel.inspector.anim_card.…`: a lista com ▶, «quem escolhe» e as regras (plano 08). Irmão do
//! `inspector.rs` por ASSUNTO (o tecto de 700 linhas). Os nomes dos quatro estados, «Run From» e
//! «Flip» são os do catálogo (`component.field.character_animations.*`).

/// O editor de animações (`anim_editor.*`) responde por esta mesma porta: as duas tabelas são da
/// mesma família, e o `lib.rs` (tecto de 700 linhas) conhece uma só.
#[path = "anim_editor.rs"]
mod editor;

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    if let Some(t) = editor::tr(key) {
        return Some(t);
    }
    Some(match key {
        "panel.inspector.anim_card.play" => "\u{25B6}",
        "panel.inspector.anim_card.play_tip" => "Play or pause this animation.",
        "panel.inspector.anim_card.open_editor" => "Edit Animations\u{2026}",
        "panel.inspector.anim_card.open_editor_tip" => {
            "Open the animation editor: pick frames on the sheet, set their timing and how they loop."
        }
        "panel.inspector.anim_card.frames" => "{n} {n?frame|frames}",
        "panel.inspector.anim_card.who_picks" => "Who Picks",
        "panel.inspector.anim_card.who_picks_live" => "Who Picks · LIVE",
        "panel.inspector.anim_card.me" => "Auto",
        "panel.inspector.anim_card.script" => "Script",
        "panel.inspector.anim_card.rules" => "Rules",
        "panel.inspector.anim_card.me_hint" => {
            "It starts by itself when the game runs, with the animation in Starts with."
        }
        "panel.inspector.anim_card.script_hint" => {
            "Nothing starts by itself: a script picks with ph2d.set(self.id, \"animator.animation\", \"run\") and starts it with ph2d.set(self.id, \"animator.playing\", true)."
        }
        "panel.inspector.anim_card.rules_hint" => {
            "The rule on top wins. Add rules for keys, damage or signals; a script can still pick for the hard cases."
        }
        "panel.inspector.anim_card.rule_add" => "+ Rule",
        "panel.inspector.anim_card.rule_grip_tip" => {
            "Drag to reorder \u{2014} the rule on top wins"
        }
        "panel.inspector.anim_card.rule_del_tip" => "Remove this rule",
        "panel.inspector.anim_card.rule_when_tip" => "When this happens\u{2026}",
        "panel.inspector.anim_card.rule_play_tip" => "\u{2026}play this animation",
        "panel.inspector.anim_card.rules_hidden" => {
            "{n} more rules do not fit here; they still count, in order, below these."
        }
        "panel.inspector.anim_card.when_pose_0" => "Standing still",
        "panel.inspector.anim_card.when_pose_1" => "Running",
        "panel.inspector.anim_card.when_pose_2" => "Going up",
        "panel.inspector.anim_card.when_pose_3" => "Falling",
        "panel.inspector.anim_card.when_pose_4" => "Crouching",
        "panel.inspector.anim_card.when_pose_5" => "Crouch-walking",
        "panel.inspector.anim_card.when_life_0" => "Took damage",
        "panel.inspector.anim_card.when_life_1" => "Died",
        "panel.inspector.anim_card.when_life_2" => "Got healed",
        "panel.inspector.anim_card.when_life_3" => "Dodged",
        "panel.inspector.anim_card.when_action" => "Key held: {name}",
        "panel.inspector.anim_card.when_signal" => "Signal: {name}",
        "panel.inspector.anim_card.run_from_tip" => {
            "Walking faster than this, in m/s against the ground, plays Run."
        }
        "panel.inspector.anim_card.flip_tip" => "Mirror the picture when he faces left.",
        "panel.inspector.anim_card.missing" => "{name} (not in the list)",
        "panel.inspector.anim_card.no_animations" => "Add an animation first.",
        "panel.inspector.anim_card.pause" => "\u{23F8}",
        "panel.inspector.anim_card.rewind_tip" => "Back to the first frame.",
        "panel.inspector.anim_card.playback" => "Playback",
        "panel.inspector.anim_card.frames_chosen" => "{n} {n?frame|frames} chosen in the editor",
        "panel.inspector.anim_card.fps" => "Frames/s",
        "panel.inspector.anim_card.cycle" => "Cycle",
        "panel.inspector.anim_card.cycle_repeat" => "Repeat",
        "panel.inspector.anim_card.cycle_once" => "Once",
        "panel.inspector.anim_card.cycle_ping_pong" => "Ping-pong",
        "panel.inspector.anim_card.own_times" => "\u{2022} some frames have their own time",
        "panel.inspector.anim_card.repeat_count" => "Repeat count",
        "panel.inspector.anim_card.advanced" => "Advanced",
        "panel.inspector.anim_card.more" => "More",
        "panel.inspector.anim_card.starts_with" => "Starts with",
        "panel.inspector.anim_card.starts_with_tip" => {
            "The animation this object starts with when the game runs. Playing one here is only a preview."
        }
        "panel.inspector.anim_card.more_summary" => "Hold \u{00b7} signals",
        "panel.inspector.anim_card.selected" => "Selected Animation",
        "panel.inspector.anim_card.add_tip" => "Add a new animation to the list.",
        "panel.inspector.anim_card.remove_tip" => "Remove the selected animation from the list.",
        "panel.inspector.anim_card.speed_tip" => {
            "How fast it plays: 2 = twice as fast. A negative number plays it backward."
        }
        "panel.inspector.anim_card.fps_tip" => "How many frames play each second.",
        "panel.inspector.anim_card.cycle_repeat_tip" => {
            "Play over and over, always from the start."
        }
        "panel.inspector.anim_card.cycle_once_tip" => "Play one time and stop on the last frame.",
        "panel.inspector.anim_card.cycle_ping_pong_tip" => {
            "Play to the end, then back to the start, over and over."
        }
        "panel.inspector.anim_card.hold_tip" => "Extra time on the last frame of each cycle.",
        "panel.inspector.anim_card.delay_tip" => "Extra pause before the next cycle starts.",
        "panel.inspector.anim_card.from_tip" => "The first cell of the sheet this animation uses.",
        "panel.inspector.anim_card.to_tip" => "The last cell of the sheet this animation uses.",
        "panel.inspector.anim_card.signal_finish_tip" => {
            "Send a signal with this name when the animation ends. Empty = no signal."
        }
        "panel.inspector.anim_card.signal_loop_tip" => {
            "Send a signal with this name each time the animation loops. Empty = no signal."
        }
        "panel.inspector.anim_card.direction_tip" => {
            "This animation plays in reverse, usually because it was imported that way. Choose Forward or Ping-pong to use Cycle instead; this row then goes away."
        }
        "panel.inspector.anim_card.direction_override_tip" => {
            "This object forces a direction on every animation it plays. Choose Inherit to turn that off; this row then goes away."
        }
        "panel.inspector.anim_card.loop_override_tip" => {
            "This object forces looping on or off for every animation it plays. Choose Inherit to turn that off; this row then goes away."
        }
        // A troca de imagem (plano GEF 12).
        "app.sprite_anim.image_choice.not_an_image" => "That asset is not an image",
        "app.sprite_anim.image_choice.not_a_sprite" => "This object has no image to change",
        _ => return None,
    })
}
