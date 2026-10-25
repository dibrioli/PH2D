//! A forma GRAVADA do `Health` até ao `PROJECT_SCHEMA` 194 (a resistência sem `item`) — só para o
//! degrau `194 → 195` (`ph2d_ecs::item_id::migra_v194`). ⛔ Congelada: não acompanhe o tipo vivo.

use serde::Deserialize;

use super::health::{Health, Resistance};

#[derive(Deserialize)]
pub(crate) struct ResistanceV194 {
    kind: String,
    rate: f32,
    absorbs: bool,
}

#[derive(Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct HealthV194 {
    max: f32,
    start: f32,
    invincible_s: f32,
    overheal: bool,
    regen: f32,
    regen_delay_s: f32,
    shield_start: f32,
    shield_max: f32,
    shield_duration_s: f32,
    shield_regen: f32,
    shield_regen_delay_s: f32,
    shield_blocks_excess: bool,
    armor_flat: f32,
    armor_percent: f32,
    dodge: f32,
    team: String,
    on_damage: String,
    on_heal: String,
    on_death: String,
    seed: u64,
    death_hitstop_s: f32,
    blink_s: f32,
    knockback_taken: f32,
    numbers: bool,
    numbers_color: [f32; 4],
    numbers_size: f32,
    resistances: Vec<ResistanceV194>,
}

impl From<HealthV194> for Health {
    fn from(h: HealthV194) -> Self {
        Self {
            max: h.max,
            start: h.start,
            invincible_s: h.invincible_s,
            overheal: h.overheal,
            regen: h.regen,
            regen_delay_s: h.regen_delay_s,
            shield_start: h.shield_start,
            shield_max: h.shield_max,
            shield_duration_s: h.shield_duration_s,
            shield_regen: h.shield_regen,
            shield_regen_delay_s: h.shield_regen_delay_s,
            shield_blocks_excess: h.shield_blocks_excess,
            armor_flat: h.armor_flat,
            armor_percent: h.armor_percent,
            dodge: h.dodge,
            team: h.team,
            on_damage: h.on_damage,
            on_heal: h.on_heal,
            on_death: h.on_death,
            seed: h.seed,
            death_hitstop_s: h.death_hitstop_s,
            blink_s: h.blink_s,
            knockback_taken: h.knockback_taken,
            numbers: h.numbers,
            numbers_color: h.numbers_color,
            numbers_size: h.numbers_size,
            resistances: (h.resistances.into_iter())
                .map(|r| Resistance {
                    kind: r.kind,
                    rate: r.rate,
                    absorbs: r.absorbs,
                    item: ph2d_ecs::ItemId::NONE,
                })
                .collect(),
        }
    }
}

ph2d_ecs::com_itens!(Health: resistances);
