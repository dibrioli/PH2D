//! **O instantâneo da ARMA e o dreno das edições dela.**
//!
//! Irmão do [`crate::ray_inspector`]: a shell publica o que o painel mostra, e o painel devolve
//! edições que voltam por aqui.
//!
//! # ⭐⭐ A munição VIVA é o que faz esta secção valer a pena
//!
//! Quantas balas ela tem **agora** e se está a recarregar não vêm de campos: vêm do
//! [`ph2d_ecs::CounterRuntime`] e do [`ph2d_ecs::WeaponRuntime`], que é onde a corrida vive. É a
//! mesma razão pela qual a secção do raio mostra *o que ele vê agora*, e é o que transforma oito
//! números numa ferramenta que se afina a olhar.
//!
//! # ⚠️ O pente é lido AQUI com a mesma lente da PONTE
//!
//! A [`crate::weapon_bridge`] lê o [`ph2d_ecs::Counter`] **desta entidade**; se o painel usasse o
//! [`ph2d_ecs::counter::soma`] (que soma todos os do nome) ele mostraria um número e a arma gastaria
//! outro. ⇒ os dois perguntam a mesma coisa, e é a discordância que a queixa `PenteAusente` conta ao
//! artista.

use ph2d_ecs::{Counter, CounterRuntime, Entity, SimWorld, WeaponFire, WeaponRuntime};
use ph2d_editor_core::weapon_edits::{InspectorWeaponInfo, WeaponFieldEdit as E};

/// **O que o painel mostra da arma deste objecto**, ou `None` se ele não tiver o componente.
#[must_use]
pub fn build_info(
    sim: &SimWorld,
    bits: u64,
    clock_playing: bool,
    selected_count: usize,
) -> Option<InspectorWeaponInfo> {
    let e = Entity::from_bits(bits);
    let w = sim.world().get::<WeaponFire>(e)?;

    // ⚠️ A MESMA lente da ponte: o contador desta entidade, com o nome que a arma pede.
    let alvo = w.ammo_counter.trim();
    let pente = sim
        .world()
        .get::<Counter>(e)
        .filter(|c| c.name.trim() == alvo && !alvo.is_empty());
    let municao = pente
        .and(sim.world().get::<CounterRuntime>(e))
        .map(|r| r.value);

    Some(InspectorWeaponInfo {
        entity_bits: bits,
        on_signal: w.on_signal.clone(),
        cooldown_ms: w.cooldown_ms,
        ammo_counter: w.ammo_counter.clone(),
        reload_ms: w.reload_ms,
        reload_on: w.reload_on.clone(),
        on_fire: w.on_fire.clone(),
        on_empty: w.on_empty.clone(),
        on_reloaded: w.on_reloaded.clone(),
        municao,
        pente: pente.map_or(0, |c| c.start),
        recarregando: sim
            .world()
            .get::<WeaponRuntime>(e)
            .is_some_and(|r| r.reloading),
        clock_playing,
        selected_count,
    })
}

/// Uma edição. `true` = o documento mudou.
fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = Entity::from_bits(bits);
    let Some(mut w) = sim.world_mut().get_mut::<WeaponFire>(e) else {
        return false;
    };
    // ⚠️ **Escrever o mesmo valor não é uma mudança** — o `get_mut` do bevy marca o componente como
    // alterado por ser PEDIDO, e re-escolher o que já lá estava entraria no undo.
    macro_rules! poe {
        ($campo:ident, $v:expr) => {{
            if w.$campo == *$v {
                return false;
            }
            w.$campo = $v.clone();
            true
        }};
    }
    match edit {
        E::OnSignal(v) => poe!(on_signal, v),
        E::CooldownMs(v) => poe!(cooldown_ms, v),
        E::AmmoCounter(v) => poe!(ammo_counter, v),
        E::ReloadMs(v) => poe!(reload_ms, v),
        E::ReloadOn(v) => poe!(reload_on, v),
        E::OnFire(v) => poe!(on_fire, v),
        E::OnEmpty(v) => poe!(on_empty, v),
        E::OnReloaded(v) => poe!(on_reloaded, v),
    }
}

/// Todas as edições de um quadro. `true` = alguma mudou.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mudou = false;
    for (bits, edit) in edits {
        mudou |= apply(sim, *bits, edit);
    }
    mudou
}

#[cfg(test)]
#[path = "weapon_inspector_tests.rs"]
mod tests;
