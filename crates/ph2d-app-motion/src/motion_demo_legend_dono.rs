//! ⭐ **A legenda segue a ORIGEM do dono** (onda 4, item 8; doc ObjectsAndComponents 01 §13).
//!
//! O `world` de uma [`Caption`] é do espaço do GRAFO, o mesmo dos sinks: com o objecto Motion fora
//! da origem, o que a cena gera sai na afim dele e a ficha ficava a apontar para o vazio. A cura é a
//! lei dos sinks, não uma cópia dela: a ficha prende-se a um NÓ da cena ([`Caption::no`]) e a afim é
//! a do dono desse nó ([`crate::motion_objeto::origem::afim_do_no`]) — vários donos ao mesmo tempo,
//! cada ficha com a sua.
//!
//! ⚠️ **O nó, e não o dono:** quando a cena se monta os nós ainda não têm dono (a migração do
//! `motion_objeto` adopta-os no quadro seguinte), e o dono pode mudar depois. O nó fica; o dono
//! resolve-se a cada quadro.

use ph2d_nodegraph::graph::NodeId;

use super::{Caption, LEGEND, captions};
use crate::motion_objeto::IDENTIDADE;
use crate::motion_objeto::origem::{afim_do_no, dono_do_no, ponto};
use crate::motion_state::MotionState;

/// A PORTA por onde uma cena de demo se monta: o roteador e, a seguir, a legenda que ela publicou
/// presa ao 1.º sink dela.
pub fn monta_cena(
    level: Option<&str>,
    doc: &mut ph2d_motion_doc::MotionDoc,
    reg: &ph2d_node_registry::NodeRegistry,
) -> Vec<NodeId> {
    let sinks = crate::motion_state::demo_router::build_level(level, doc, reg);
    if let Ok(mut slot) = LEGEND.lock() {
        ancora_em(&mut slot, &sinks);
    }
    sinks
}

/// Prende ao 1.º sink as fichas que ainda não nomeiam um nó.
pub fn ancora_em(captions: &mut [Caption], sinks: &[NodeId]) {
    let Some(&sink) = sinks.first() else {
        return;
    };
    for c in captions.iter_mut().filter(|c| c.no.is_none()) {
        c.no = Some(sink);
    }
}

/// As fichas publicadas, no MUNDO deste quadro — o que o passe de pintura leva à tela pela câmara.
#[must_use]
pub fn no_mundo(motion: &MotionState) -> Vec<Caption> {
    no_mundo_de(&captions(), motion)
}

/// A lei de [`no_mundo`] sem o global: `world' = afim do dono × world`; dono escondido pelo olho ⇒ a
/// ficha sai com o que ela explica. ⭐ Sem dono deslocado, a ficha de sempre, bit a bit.
#[must_use]
pub fn no_mundo_de(captions: &[Caption], motion: &MotionState) -> Vec<Caption> {
    captions
        .iter()
        .filter(|c| {
            c.no.and_then(|n| dono_do_no(motion, n))
                .is_none_or(|d| d.visible)
        })
        .map(|c| {
            let afim = c.no.map_or(IDENTIDADE, |n| afim_do_no(motion, n));
            let mut c = c.clone();
            if afim != IDENTIDADE {
                c.world = ponto(&afim, c.world);
            }
            c
        })
        .collect()
}

#[cfg(test)]
#[path = "motion_demo_legend_dono_tests.rs"]
mod tests;
