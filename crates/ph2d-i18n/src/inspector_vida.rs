//! **As palavras das secções HEALTH e DAMAGE do Inspector** (plano 28, W3).
//!
//! ⚠️ **Um ficheiro próprio e não o `inspector_game`**: aquele está a `673` linhas de um tecto de
//! `700`, e estas são uma família nova — o corte por responsabilidade que a casa prescreve.
//!
//! ⚠️ **Os rótulos das linhas são CURTOS de propósito**: a varredura das elisões mede-os em quatro
//! larguras de painel, e a explicação longa mora no aviso ou no `placeholder`, que quebram a linha.

/// A tradução de uma chave `panel.inspector.vida.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.vida.health" => "Health",
        "panel.inspector.vida.damage" => "Damage",
        "panel.inspector.vida.max_0_no_max" => "Max (0 = none)",
        "panel.inspector.vida.start" => "Start",
        "panel.inspector.vida.invincible" => "Invincible",
        "panel.inspector.vida.regen" => "Regen / s",
        "panel.inspector.vida.regen_delay" => "Regen Delay",
        "panel.inspector.vida.armor" => "Armor",
        "panel.inspector.vida.armor_fraction" => "Armor (0\u{2013}1)",
        "panel.inspector.vida.dodge" => "Dodge (0\u{2013}1)",
        "panel.inspector.vida.seed" => "Seed",
        "panel.inspector.vida.shield_max" => "Shield Max",
        "panel.inspector.vida.shield_start" => "Shield",
        "panel.inspector.vida.shield_duration_0_forever" => "Shield Time",
        "panel.inspector.vida.shield_regen" => "Shield Regen",
        "panel.inspector.vida.shield_regen_delay" => "Shield Delay",
        "panel.inspector.vida.overheal" => "Overheal",
        "panel.inspector.vida.shield_blocks_excess" => "Absorb Rest",
        "panel.inspector.vida.team_hint" => "Team \u{2014} the same team does not hurt it\u{2026}",
        "panel.inspector.vida.on_damage_hint" => "Signal when it takes damage\u{2026}",
        "panel.inspector.vida.on_heal_hint" => "Signal when it is healed\u{2026}",
        "panel.inspector.vida.on_death_hint" => "Signal when it dies\u{2026}",
        "panel.inspector.vida.now" => "Now: {now}",
        "panel.inspector.vida.now_of_max" => "Now: {now} of {max}",
        "panel.inspector.vida.shield_now" => "Shield now: {shield}",
        "panel.inspector.vida.starts_at" => {
            "Starts at {start} \u{b7} it comes alive when the clock plays."
        }
        "panel.inspector.vida.clock_stopped" => {
            "The clock is stopped \u{b7} hits only land while it plays."
        }
        "panel.inspector.vida.no_body" => {
            "No body \u{b7} add a Physics Body so it can hit or be hit."
        }
        "panel.inspector.vida.dead" => "Dead \u{b7} it stays dead; rewind to bring it back.",
        "panel.inspector.vida.hurts_nobody" => "Amount 0 \u{b7} it hurts nobody.",
        "panel.inspector.vida.editing_the_primary_selection_only" => {
            "Editing the primary selection only."
        }
        "panel.inspector.vida.amount" => "Amount",
        "panel.inspector.vida.amount_per_second" => "Amount / s",
        "panel.inspector.vida.per_second" => "Per Second",
        "panel.inspector.vida.ignores_shield" => "Pierce Shield",
        "panel.inspector.vida.ignores_armor" => "Pierce Armor",
        "panel.inspector.vida.vanish" => "Vanish on Hit",
        "panel.inspector.vida.damage_team_hint" => {
            "Team \u{2014} it does not hurt its own team\u{2026}"
        }
        "panel.inspector.vida.health_bar" => "Health Bar",
        "panel.inspector.vida.bar_width" => "Width",
        "panel.inspector.vida.bar_height" => "Height",
        "panel.inspector.vida.bar_offset_x" => "Offset X",
        "panel.inspector.vida.bar_offset_y" => "Offset Y",
        "panel.inspector.vida.bar_trail_delay" => "Trail Delay",
        "panel.inspector.vida.bar_trail_speed" => "Trail / s",
        "panel.inspector.vida.bar_fill" => "Fill",
        "panel.inspector.vida.bar_trail" => "Trail",
        "panel.inspector.vida.bar_back" => "Back",
        // ⚠️ **`If` e não `When`**: a varredura das elisões mediu `Hide When Full` cortado no degrau
        // estreito, e a regra da casa é *um nome perde a explicação antes de perder letras*.
        "panel.inspector.vida.bar_hide_when_full" => "Hide If Full",
        "panel.inspector.vida.bar_target_hint" => {
            "Target \u{2014} empty shows this object\u{2019}s own health\u{2026}"
        }
        "panel.inspector.vida.bar_shows" => "Shows {now} of {max}",
        "panel.inspector.vida.bar_no_target" => {
            "No object is named \u{201c}{name}\u{201d} \u{b7} the bar draws nothing."
        }
        "panel.inspector.vida.bar_target_no_health" => {
            "\u{201c}{name}\u{201d} has no Health \u{b7} the bar draws nothing."
        }
        "panel.inspector.vida.bar_own_no_health" => {
            "This object has no Health \u{b7} add Health, or name another object in Target."
        }
        _ => return None,
    })
}
