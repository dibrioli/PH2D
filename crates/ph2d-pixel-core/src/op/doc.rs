//! As Ops do dado da P13 (rampas, ciclos, guias, som, efeitos de camada), partidas de `op.rs`.
//! O inverso de cada uma é o estado de antes, inteiro (`SetRamps`, `SetGuias`, …): exacto.

use super::{Op, OpError};
use crate::audio::FaixaDeAudio;
use crate::cycle::Cycle;
use crate::efeito::EfeitoDeCamada;
use crate::guia::Guias;
use crate::ramp::{PaletteRamps, Ramp, RampId, check_ramps};
use crate::sprite::{LayerId, Sprite};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocOp {
    /// Palette ▸ New Ramp: o id sai do contador do documento.
    AddRamp {
        name: String,
        indices: Vec<u8>,
    },
    /// Troca a rampa de mesmo id (nome e índices).
    SetRamp(Ramp),
    /// Apaga a rampa e o ciclo dela.
    RemoveRamp(RampId),
    /// O ciclo da rampa (`None` = sem ciclo).
    SetCycle {
        ramp: RampId,
        cycle: Option<Cycle>,
    },
    /// As rampas e os ciclos inteiros (o inverso das quatro de cima).
    SetRamps(PaletteRamps),
    SetGuias(Guias),
    SetAudio(Vec<FaixaDeAudio>),
    /// Os efeitos de uma camada que não é grupo.
    SetLayerEffects {
        layer: LayerId,
        efeitos: Vec<EfeitoDeCamada>,
    },
}

impl DocOp {
    /// Os bytes que o op segura (a régua do tecto do histórico).
    #[must_use]
    pub fn bytes(&self) -> usize {
        let ramp = |r: &Ramp| r.name.len() + r.indices.len() + size_of::<Ramp>();
        size_of::<Self>()
            + match self {
                Self::AddRamp { name, indices } => name.len() + indices.len(),
                Self::SetRamp(r) => ramp(r),
                Self::SetRamps(p) => {
                    p.ramps.iter().map(ramp).sum::<usize>() + p.cycles.len() * size_of::<Cycle>()
                }
                Self::SetAudio(v) => v
                    .iter()
                    .map(|a| a.asset.len() + size_of::<FaixaDeAudio>())
                    .sum(),
                Self::SetLayerEffects { efeitos, .. } => {
                    efeitos.len() * size_of::<EfeitoDeCamada>()
                }
                Self::RemoveRamp(_) | Self::SetCycle { .. } | Self::SetGuias(_) => 0,
            }
    }
}

pub(super) fn apply_doc(s: &mut Sprite, op: DocOp) -> Result<Op, OpError> {
    let bad = OpError::BadRamps;
    let mut ramps = s.ramps.clone();
    match op {
        DocOp::AddRamp { name, indices } => {
            let id = RampId(s.next_id);
            ramps.ramps.push(Ramp { id, name, indices });
            check_ramps(&ramps, s.palette.len(), s.next_id + 1).map_err(bad)?;
            s.next_id += 1;
        }
        DocOp::SetRamp(r) => {
            let slot = ramps.ramps.iter_mut().find(|x| x.id == r.id);
            *slot.ok_or(bad("ramp.id"))? = r;
        }
        DocOp::RemoveRamp(id) => {
            let n = ramps.ramps.len();
            ramps.ramps.retain(|r| r.id != id);
            if ramps.ramps.len() == n {
                return Err(bad("ramp.id"));
            }
            ramps.cycles.retain(|c| c.ramp != id);
        }
        DocOp::SetCycle { ramp, cycle } => {
            if cycle.is_some_and(|c| c.ramp != ramp) {
                return Err(bad("cycle.ramp"));
            }
            ramps.cycles.retain(|c| c.ramp != ramp);
            ramps.cycles.extend(cycle);
        }
        DocOp::SetRamps(r) => ramps = r,
        DocOp::SetGuias(g) => return Ok(doc(DocOp::SetGuias(std::mem::replace(&mut s.guias, g)))),
        DocOp::SetAudio(a) => return Ok(doc(DocOp::SetAudio(std::mem::replace(&mut s.audio, a)))),
        DocOp::SetLayerEffects { layer, efeitos } => {
            let l = s.layers.iter_mut().find(|l| l.id == layer);
            let l = l.ok_or(OpError::UnknownLayer(layer))?;
            if l.is_group() {
                return Err(OpError::BadLayers("effects_group"));
            }
            let efeitos = std::mem::replace(&mut l.efeitos, efeitos);
            return Ok(doc(DocOp::SetLayerEffects { layer, efeitos }));
        }
    }
    check_ramps(&ramps, s.palette.len(), s.next_id).map_err(bad)?;
    Ok(doc(DocOp::SetRamps(std::mem::replace(&mut s.ramps, ramps))))
}

fn doc(op: DocOp) -> Op {
    Op::Doc(Box::new(op))
}
