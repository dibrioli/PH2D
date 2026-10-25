//! **O VOCABULÁRIO DO CARTÃO «ANIMATION»** (plano GameEngineFuncional 07) — as chaves
//! `panel.inspector.anim_card.…`: a lista com ▶, «quem escolhe» e os estados do personagem. Irmão do
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
        "panel.inspector.anim_card.play_tip" => "Play this animation.",
        "panel.inspector.anim_card.open_editor" => "Edit Animations\u{2026}",
        "panel.inspector.anim_card.open_editor_tip" => {
            "Open the animation editor: pick frames on the sheet, set their timing and how they loop."
        }
        "panel.inspector.anim_card.frames" => "{n} frames",
        "panel.inspector.anim_card.who_picks" => "Who Picks",
        "panel.inspector.anim_card.me" => "Me",
        "panel.inspector.anim_card.script" => "Script",
        "panel.inspector.anim_card.character" => "The Character",
        "panel.inspector.anim_card.me_hint" => {
            "The animation you play here starts by itself when the game runs."
        }
        "panel.inspector.anim_card.script_hint" => {
            "Nothing starts by itself: a script picks with ph2d.set(self.id, \"animator.animation\", \"run\") and starts it with ph2d.set(self.id, \"animator.playing\", true)."
        }
        "panel.inspector.anim_card.character_hint" => {
            "The character picks by what he does: standing, running, jumping, falling."
        }
        "panel.inspector.anim_card.needs_player" => {
            "The character can pick only with a Platform Player on this object."
        }
        "panel.inspector.anim_card.state_tip" => "Which animation plays in this state.",
        "panel.inspector.anim_card.run_from_tip" => {
            "Walking faster than this, in m/s against the ground, plays Run."
        }
        "panel.inspector.anim_card.flip_tip" => "Mirror the picture when he faces left.",
        "panel.inspector.anim_card.missing" => "{name} (not in the list)",
        "panel.inspector.anim_card.no_animations" => "Add an animation first.",
        "panel.inspector.anim_card.more" => "More",
        _ => return None,
    })
}
