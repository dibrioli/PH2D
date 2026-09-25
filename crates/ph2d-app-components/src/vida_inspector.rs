//! **As secções HEALTH e DAMAGE: o instantâneo e o dreno** (plano 28, W3).
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — o molde do [`super::projectile_inspector`]:
//! quem lê o mundo para o painel e quem escreve a edição de volta fazem a MESMA tradução, e
//! separá-los seria a porta pela qual as duas divergem.
//!
//! # ⚠️ As CERCAS são as da LEI, escritas também aqui
//!
//! O painel já prende cada número na faixa dele — e o campo é alcançável por outra rota (um script,
//! um ficheiro). ⇒ o dreno prende de novo: uma percentagem de armadura fora de `0..1` daria dano
//! NEGATIVO (curaria), e uma esquiva acima de `1` esquivaria sempre.

use ph2d_ecs::{Entity, SimWorld, World};
use ph2d_editor_core::vida_edits::{
    BarraAlvo, InspectorBarInfo, InspectorDamageInfo, InspectorHealthInfo, InspectorVidaInfo,
    VidaAgora, VidaFieldEdit as E,
};
use ph2d_physics_ecs::{Damage, Health, HealthBar, HealthNow, OnHit, RigidBody};

fn health_info(h: &Health, agora: Option<&HealthNow>) -> InspectorHealthInfo {
    InspectorHealthInfo {
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
        team: h.team.clone(),
        on_damage: h.on_damage.clone(),
        on_heal: h.on_heal.clone(),
        on_death: h.on_death.clone(),
        seed: h.seed,
        death_hitstop_s: h.death_hitstop_s,
        blink_s: h.blink_s,
        knockback_taken: h.knockback_taken,
        numbers: h.numbers,
        numbers_color: h.numbers_color,
        numbers_size: h.numbers_size,
        agora: agora.map(|a| VidaAgora {
            pontos: a.pontos,
            escudo: a.escudo,
            morta: a.morta,
        }),
    }
}

fn damage_info(d: &Damage) -> InspectorDamageInfo {
    InspectorDamageInfo {
        amount: d.amount,
        team: d.team.clone(),
        per_second: d.per_second,
        ignores_shield: d.ignores_shield,
        ignores_armor: d.ignores_armor,
        vanish: d.on_hit == OnHit::Vanish,
        hitstop_s: d.hitstop_s,
        knockback: d.knockback,
        knockback_lift: d.knockback_lift,
    }
}

/// **A barra e o que ela encontrou** (plano 28, W4) — o alvo e a vida saem das MESMAS portas que a
/// ponte usa ao desenhar ([`crate::health_bar_bridge::alvo_da_barra`] e
/// [`crate::health_bar_bridge::vida_da_barra`]).
fn bar_info(world: &World, dono: Entity, b: &HealthBar) -> InspectorBarInfo {
    use crate::health_bar_bridge::{alvo_da_barra, vida_da_barra};
    let alvo = match alvo_da_barra(world, dono, &b.target) {
        None => BarraAlvo::SemAlvo,
        Some(a) => vida_da_barra(world, a).map_or(BarraAlvo::SemVida, |(pontos, max)| {
            BarraAlvo::Mostra { pontos, max }
        }),
    };
    InspectorBarInfo {
        target: b.target.clone(),
        width: b.width,
        height: b.height,
        offset_x: b.offset_x,
        offset_y: b.offset_y,
        fill: b.fill,
        trail: b.trail,
        back: b.back,
        trail_delay_s: b.trail_delay_s,
        trail_speed: b.trail_speed,
        hide_when_full: b.hide_when_full,
        alvo,
    }
}

/// **O instantâneo.** `None` para quem não tem vida, nem dano, nem barra (ADR-0166).
#[must_use]
pub fn build_vida_info(
    world: &World,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
) -> Option<InspectorVidaInfo> {
    let e = Entity::from_bits(bits);
    let health = world
        .get::<Health>(e)
        .map(|h| health_info(h, world.get::<HealthNow>(e)));
    let damage = world.get::<Damage>(e).map(damage_info);
    let bar = world.get::<HealthBar>(e).map(|b| bar_info(world, e, b));
    if health.is_none() && damage.is_none() && bar.is_none() {
        return None;
    }
    Some(InspectorVidaInfo {
        entity_bits: bits,
        health,
        damage,
        bar,
        has_body: world.get::<RigidBody>(e).is_some(),
        clock_playing,
        selected_count,
    })
}

/// Uma fracção em `0..=1`, e um número não finito vale `0` (um `NaN` que passasse pelo `clamp`
/// ficaria `NaN`, e a lei leria-o como «sem armadura» por acaso).
fn fraccao(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Um número `≥ 0`, com o não-finito a valer `0`.
fn positivo(v: f32) -> f32 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

/// **Uma edição da VIDA.** `true` = tocou no mundo.
fn apply_health(h: &mut Health, edit: &E) -> bool {
    match edit {
        E::Max(v) => h.max = positivo(*v),
        E::Start(v) => h.start = positivo(*v),
        E::InvincibleS(v) => h.invincible_s = positivo(*v),
        E::Overheal(b) => h.overheal = *b,
        E::Regen(v) => h.regen = positivo(*v),
        E::RegenDelayS(v) => h.regen_delay_s = positivo(*v),
        E::ShieldStart(v) => h.shield_start = positivo(*v),
        E::ShieldMax(v) => h.shield_max = positivo(*v),
        // ⚠️ **`≤ 0` é «nunca expira»** — a lei do alvo; o painel só produz `≥ 0`, e o zero é a
        // forma autorável de dizer «para sempre».
        E::ShieldDurationS(v) => h.shield_duration_s = positivo(*v),
        E::ShieldRegen(v) => h.shield_regen = positivo(*v),
        E::ShieldRegenDelayS(v) => h.shield_regen_delay_s = positivo(*v),
        E::ShieldBlocksExcess(b) => h.shield_blocks_excess = *b,
        E::ArmorFlat(v) => h.armor_flat = positivo(*v),
        E::ArmorPercent(v) => h.armor_percent = fraccao(*v),
        E::Dodge(v) => h.dodge = fraccao(*v),
        E::Team(t) => h.team = t.trim().to_string(),
        E::OnDamage(t) => h.on_damage = t.trim().to_string(),
        E::OnHeal(t) => h.on_heal = t.trim().to_string(),
        E::OnDeath(t) => h.on_death = t.trim().to_string(),
        E::Seed(n) => h.seed = *n,
        E::DeathHitstopS(v) => h.death_hitstop_s = positivo(*v),
        E::BlinkS(v) => h.blink_s = positivo(*v),
        E::KnockbackTaken(v) => h.knockback_taken = positivo(*v),
        E::Numbers(b) => h.numbers = *b,
        E::NumbersColor(c) => h.numbers_color = cor(*c),
        E::NumbersSize(v) => h.numbers_size = positivo(*v),
        _ => return false,
    }
    true
}

/// **Uma edição do DANO.** `true` = tocou no mundo.
fn apply_damage(d: &mut Damage, edit: &E) -> bool {
    match edit {
        E::DamageAmount(v) => d.amount = positivo(*v),
        E::DamageTeam(t) => d.team = t.trim().to_string(),
        E::PerSecond(b) => d.per_second = *b,
        E::IgnoresShield(b) => d.ignores_shield = *b,
        E::IgnoresArmor(b) => d.ignores_armor = *b,
        E::Vanish(b) => d.on_hit = if *b { OnHit::Vanish } else { OnHit::Stay },
        E::HitstopS(v) => d.hitstop_s = positivo(*v),
        // ⚠️ **O empurrão é LIVRE**: negativo PUXA para quem bate (um íman, um gancho) — a lei do
        // empurrão aceita o sinal, e só o não-finito cai a zero.
        E::Knockback(v) => d.knockback = if v.is_finite() { *v } else { 0.0 },
        E::KnockbackLift(v) => d.knockback_lift = if v.is_finite() { *v } else { 0.0 },
        _ => return false,
    }
    true
}

/// Uma cor com os quatro canais em `0..=1` — a amostra só produz isso, e outra rota (um script, um
/// ficheiro) não pode pôr um `NaN` no tinte de uma faixa.
fn cor(c: [f32; 4]) -> [f32; 4] {
    c.map(fraccao)
}

/// **Uma edição da BARRA.** `true` = tocou no mundo.
///
/// ⚠️ **Os deslocamentos são LIVRES** (uma barra pode ir abaixo ou ao lado) — só o não-finito cai
/// a zero; o tamanho, o atraso e a velocidade são `≥ 0`.
fn apply_bar(b: &mut HealthBar, edit: &E) -> bool {
    let livre = |v: f32| if v.is_finite() { v } else { 0.0 };
    match edit {
        E::BarTarget(t) => b.target = t.trim().to_string(),
        E::BarWidth(v) => b.width = positivo(*v),
        E::BarHeight(v) => b.height = positivo(*v),
        E::BarOffsetX(v) => b.offset_x = livre(*v),
        E::BarOffsetY(v) => b.offset_y = livre(*v),
        E::BarFill(c) => b.fill = cor(*c),
        E::BarTrail(c) => b.trail = cor(*c),
        E::BarBack(c) => b.back = cor(*c),
        E::BarTrailDelayS(v) => b.trail_delay_s = positivo(*v),
        E::BarTrailSpeed(v) => b.trail_speed = positivo(*v),
        E::BarHideWhenFull(on) => b.hide_when_full = *on,
        _ => return false,
    }
    true
}

/// **O dreno de UMA edição.** `true` = tocou no mundo.
///
/// ⚠️ Cada edição só tem sujeito num dos dois componentes: a do dano sobre um objecto sem `Damage`
/// não faz nada, e é o `false` que o diz.
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = Entity::from_bits(bits);
    let w = sim.world_mut();
    if w.get_entity(e).is_err() {
        return false;
    }
    if let Some(mut h) = w.get_mut::<Health>(e)
        && apply_health(&mut h, edit)
    {
        return true;
    }
    if let Some(mut d) = w.get_mut::<Damage>(e)
        && apply_damage(&mut d, edit)
    {
        return true;
    }
    w.get_mut::<HealthBar>(e)
        .is_some_and(|mut b| apply_bar(&mut b, edit))
}

/// **O dreno do quadro.** `true` = alguma tocou no mundo.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        mudou |= apply(sim, *bits, edit);
    }
    mudou
}

#[cfg(test)]
#[path = "vida_inspector_tests.rs"]
mod tests;
