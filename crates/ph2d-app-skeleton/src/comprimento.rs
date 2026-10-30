//! ⭐⭐ **O comprimento EMPURRA os filhos** (dono 30/10) — a lei mora na folha
//! ([`ph2d_skeleton_live::comprimento`]); dois escritores: a mão ([`aplica`]) e a curva
//! (`canais_osso::aplicar`, que declara cada filho à pré-visualização).

use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_ecs::Bone;
pub use ph2d_skeleton_live::comprimento::empurra_filhos;

use crate::knobs::{BoneKnob, apply};

/// ⭐ **A MÃO escreve um número do osso `e`** — o [`apply`] e, no comprimento, o empurrão dos filhos
/// (dado autorado: a pose autorada dos filhos anda junto). `false` sem osso.
pub fn aplica(sim: &mut SimWorld, e: Entity, knob: BoneKnob, v: f64) -> bool {
    let Some(mut osso) = sim.world_mut().get_mut::<Bone>(e) else {
        return false;
    };
    let l0 = osso.length;
    apply(&mut osso, knob, v);
    let l1 = osso.length;
    empurra_filhos(sim, e, l0, l1);
    true
}

#[cfg(test)]
#[path = "comprimento_tests.rs"]
mod tests;
