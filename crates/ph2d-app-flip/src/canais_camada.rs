//! ⭐⭐ **A família FLIP nos canais** (plano 35 §A.8) — as propriedades da CAMADA de um objecto
//! Flip que a secção Layers do painel edita: [`publicar`] o que o documento vale em `CanaisLidos`,
//! [`aplicar`] o que a curva mandou em `CanaisEscritos` (com o eco).
//!
//! ⭐ **A escrita é a do painel**: cada campo vira o MESMO `PanelEvent` que a linha da camada emite
//! (o `SetValue` do slider, o `SelectOption` da mistura, o `Click` do olho) e passa pelo funil único
//! [`crate::layers::apply_panel_event`] — os mesmos `clamp`, a mesma descodificação do id.
//!
//! ⭐ **A escrita da curva é pré-visualização**: o `FlipDoc` é documento (o undo fotografa-o por
//! diff, o save grava-o), então o valor AUTORADO de cada campo que a curva move fica em
//! [`ConducaoFlip`] — a irmã da `ph2d_app_vec::canais::ConducaoVetor` — e a fotografia vê-o.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::canal::familia::FLIP;
use ph2d_ecs::canal_flip as f;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, Entity, FieldKey, FlipObjectRef, SimWorld};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_flip::{BlendMode, FlipDoc, FlipLayer, FlipObjectId, FlipTarget, LayerId};
use ph2d_panel_flip::ids::{FlipLayerWidget, flip_layer_widget_id};

/// O endereço do campo `campo` da camada `camada` — `None` se o `LayerId` não cabe no `i` (`u16`)
/// do `FieldKey`: o endereço é durável e não se trunca.
#[must_use]
pub fn chave(campo: u16, camada: LayerId) -> Option<FieldKey> {
    Some(FieldKey::new(FLIP, campo, u16::try_from(camada.0).ok()?, 0))
}

/// A camada que `kk` endereça.
#[must_use]
pub fn camada_de(kk: FieldKey) -> LayerId {
    LayerId(u32::from(kk.i))
}

fn bit(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

/// Quanto vale o campo `campo` na camada — `None` fora do vocabulário.
#[must_use]
pub fn ler(l: &FlipLayer, campo: u16) -> Option<f32> {
    Some(match campo {
        f::VISIVEL => bit(l.visible),
        f::OPACIDADE => l.opacity,
        f::MISTURA => f32::from(l.blend.to_u8()),
        f::PROFUNDIDADE => l.depth,
        _ => return None,
    })
}

/// O índice de uma mistura: arredonda e prende à tabela de modos.
fn modo(v: f32) -> u8 {
    let max = ph2d_painter_effects::MAX_BLEND_MODES.saturating_sub(1);
    if v.is_finite() {
        v.round().clamp(0.0, f32::from(max)) as u8
    } else {
        BlendMode::default().to_u8()
    }
}

/// ⭐ **O evento do painel que escreve `v` no campo `campo` da camada `l`** — `None` quando o olho
/// já está como a curva manda (o olho do painel é um INTERRUPTOR: clicá-lo inverte).
#[must_use]
pub fn edicao(l: &FlipLayer, campo: u16, v: f32) -> Option<PanelEvent> {
    let id = |kind| flip_layer_widget_id(u64::from(l.id.0), kind);
    Some(match campo {
        f::VISIVEL => {
            if (v >= 0.5) == l.visible {
                return None;
            }
            PanelEvent::Click(id(FlipLayerWidget::Visibility))
        }
        f::OPACIDADE => PanelEvent::SetValue(id(FlipLayerWidget::Opacity), f64::from(v)),
        f::MISTURA => PanelEvent::SelectOption(id(FlipLayerWidget::Blend), modo(v).to_string()),
        f::PROFUNDIDADE => PanelEvent::SetValue(id(FlipLayerWidget::Depth), f64::from(v)),
        _ => return None,
    })
}

/// Escreve `v` em `kk` no objecto `oid`, pelo funil do painel; devolve `(antes, ficou)`.
fn escrever(doc: &mut FlipDoc, oid: FlipObjectId, kk: FieldKey, v: f32) -> Option<(f32, f32)> {
    let lid = camada_de(kk);
    let l = doc.object(oid)?.layer(lid)?;
    let antes = ler(l, kk.campo)?;
    if let Some(ev) = edicao(l, kk.campo, v) {
        // O alvo é só o objecto (a camada vem no id): a camada activa do artista não muda.
        let mut alvo = FlipTarget::on(oid, None);
        crate::layers::apply_panel_event(
            &ev,
            doc,
            &mut alvo,
            &ph2d_core::Playhead::default(),
            false,
        );
    }
    let ficou = ler(doc.object(oid)?.layer(lid)?, kk.campo)?;
    Some((antes, ficou))
}

/// O objecto Flip que a entidade `e` representa.
#[must_use]
pub fn objeto_de(sim: &SimWorld, e: u64) -> Option<FlipObjectId> {
    let r = sim
        .world()
        .get::<FlipObjectRef>(Entity::try_from_bits(e)?)?;
    Some(FlipObjectId(r.0))
}

/// ⭐ **Publica o que cada camada vale AGORA** (através do eco), esquecido antes o que cada
/// entidade publicou desta família (a camada apagada deixa de ter campos).
pub fn publicar(sim: &mut SimWorld, doc: &FlipDoc, entidades: &[u64]) {
    let mut lotes: Vec<(u64, FieldKey, f32)> = Vec::new();
    for &e in entidades {
        let Some(obj) = objeto_de(sim, e).and_then(|oid| doc.object(oid)) else {
            continue;
        };
        for l in obj.layers() {
            for campo in f::CAMPOS {
                if let (Some(kk), Some(v)) = (chave(campo, l.id), ler(l, campo)) {
                    lotes.push((e, kk, v));
                }
            }
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &e in entidades {
        lidos.esquecer(e, FLIP);
    }
    for (e, kk, v) in lotes {
        lidos.publicar(e, kk, v);
    }
}

/// ⭐ **Aplica às camadas o que a curva mandou neste quadro**, regista o eco e o autorado.
/// Devolve `true` se alguma camada mudou.
pub fn aplicar(sim: &mut SimWorld, doc: &mut FlipDoc, conducao: &mut ConducaoFlip) -> bool {
    let Some(escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(FLIP))
    else {
        return false;
    };
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for (e, kk, v) in escritas {
        let Some(oid) = objeto_de(sim, e) else {
            continue;
        };
        if let Some((antes, ficou)) = escrever(doc, oid, kk, v) {
            if antes != ficou {
                conducao.conduziu(e, kk, antes);
                mudou = true;
            }
            ecos.push((e, kk, v, ficou));
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (e, kk, v, ficou) in ecos {
        lidos.eco(e, kk, v, ficou);
    }
    mudou
}

/// **A condução das camadas Flip pela curva** — o `PreviewDrive` deste documento: guarda o valor
/// AUTORADO de cada campo que a curva está a mover, para que o undo e o save o fotografem em vez
/// do instante da reprodução. A semântica é a da `ConducaoVetor`: só se declara o que MUDOU neste
/// quadro; [`Self::settle`] esquece quem não foi declarado.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoFlip {
    autorado: BTreeMap<(u64, FieldKey), f32>,
    neste_quadro: BTreeSet<(u64, FieldKey)>,
}

impl ConducaoFlip {
    /// A curva moveu `(e, kk)`; `antes` era o valor do documento (o primeiro de uma corrida fica).
    pub fn conduziu(&mut self, e: u64, kk: FieldKey, antes: f32) {
        self.autorado.entry((e, kk)).or_insert(antes);
        self.neste_quadro.insert((e, kk));
    }

    /// Esquece quem não foi conduzido neste quadro (chamado onde o `PreviewDrive::settle` é).
    pub fn settle(&mut self) {
        let neste = std::mem::take(&mut self.neste_quadro);
        self.autorado.retain(|k, _| neste.contains(k));
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.autorado.is_empty()
    }

    /// Repõe o AUTORADO no documento para a fotografia; devolve o vivo para [`Self::repor`].
    pub fn substituir(&self, sim: &SimWorld, doc: &mut FlipDoc) -> Vec<(u64, FieldKey, f32)> {
        let mut vivo = Vec::with_capacity(self.autorado.len());
        for (&(e, kk), &v) in &self.autorado {
            let Some(oid) = objeto_de(sim, e) else {
                continue;
            };
            if let Some((antes, _)) = escrever(doc, oid, kk, v) {
                vivo.push((e, kk, antes));
            }
        }
        vivo
    }

    /// Devolve o vivo que [`Self::substituir`] tirou.
    pub fn repor(&self, sim: &SimWorld, doc: &mut FlipDoc, vivo: Vec<(u64, FieldKey, f32)>) {
        for (e, kk, v) in vivo {
            if let Some(oid) = objeto_de(sim, e) {
                let _ = escrever(doc, oid, kk, v);
            }
        }
    }
}

#[cfg(test)]
#[path = "canais_camada_tests.rs"]
mod tests;
