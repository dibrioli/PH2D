//! ⭐⭐⭐ **A família VETOR nos canais** (plano 35 §3.0) — o lado do quadro do
//! [`crate::canais_campos`]: [`publicar`] o que o documento vale em `CanaisLidos`, [`aplicar`] o que
//! a curva mandou em `CanaisEscritos` (com o eco da quantização), e a [`ConducaoVetor`]: a lei do
//! `ph2d_preview_drive` para o documento vetorial — **o que a curva escreve a tocar é
//! pré-visualização**, o undo e o save fotografam o AUTORADO; parado, a curva vira documento.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld, VecPathRef, VecShape};
use ph2d_vec_scene::VecScene;

use crate::canais_campos::{campos, escrever, ler, prioridade};

fn caminho(sim: &SimWorld, e: u64) -> Option<(u64, Option<VecShape>)> {
    let ent = Entity::try_from_bits(e)?;
    let id = sim.world().get::<VecPathRef>(ent)?.0;
    Some((id, sim.world().get::<VecShape>(ent).cloned()))
}

/// ⭐ **Publica o que o documento de cada entidade vale AGORA** (através do eco).
pub fn publicar(sim: &mut SimWorld, scene: &VecScene, entidades: &[u64]) {
    let mut lotes: Vec<(u64, Vec<(FieldKey, f32)>)> = Vec::with_capacity(entidades.len());
    for &e in entidades {
        let Some((id, shape)) = caminho(sim, e) else {
            continue;
        };
        let Some(p) = scene.path(id) else { continue };
        let valores = campos(p, shape.as_ref())
            .into_iter()
            .filter_map(|kk| Some((kk, ler(p, shape.as_ref(), kk)?)))
            .collect();
        lotes.push((e, valores));
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (e, valores) in lotes {
        lidos.esquecer(e);
        for (kk, v) in valores {
            lidos.publicar(e, kk, v);
        }
    }
}

/// ⭐ **Aplica ao documento o que a curva mandou neste quadro** e regista o eco. Devolve `true` se o
/// documento mudou.
pub fn aplicar(sim: &mut SimWorld, scene: &mut VecScene, conducao: &mut ConducaoVetor) -> bool {
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(VETOR))
    else {
        return false;
    };
    if escritas.is_empty() {
        return false;
    }
    escritas.sort_by_key(|&(e, kk, _)| (e, prioridade(kk), kk));
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    let mut i = 0;
    while i < escritas.len() {
        let e = escritas[i].0;
        let fim = escritas[i..]
            .iter()
            .position(|w| w.0 != e)
            .map_or(escritas.len(), |n| i + n);
        if let Some((id, mut shape)) = caminho(sim, e)
            && let Some(p) = scene.path_mut(id)
        {
            let shape_antes = shape.clone();
            for &(_, kk, v) in &escritas[i..fim] {
                let antes = ler(p, shape.as_ref(), kk);
                escrever(p, shape.as_mut(), kk, v);
                let ficou = ler(p, shape.as_ref(), kk);
                if let (Some(a), Some(f)) = (antes, ficou) {
                    if a != f {
                        conducao.conduziu(e, kk, a);
                        mudou = true;
                    }
                    ecos.push((e, kk, v, f));
                }
            }
            if shape != shape_antes
                && let Some(sh) = shape
            {
                if let VecShape::Param { .. } = sh {
                    crate::shape_live::recook_into(scene, id, &sh);
                }
                if let Some(ent) = Entity::try_from_bits(e) {
                    sim.world_mut().entity_mut(ent).insert(sh);
                }
            }
        }
        i = fim;
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (e, kk, v, f) in ecos {
        lidos.eco(e, kk, v, f);
    }
    mudou
}

/// **A condução do documento vetorial pela curva** — o `ph2d_preview_drive::PreviewDrive` deste
/// documento: guarda o valor AUTORADO de cada campo que a curva está a mover, para que o undo e o
/// save o fotografem em vez do instante da reprodução.
///
/// ⚠️ A semântica é a do ledger irmão: só se declara o que MUDOU neste quadro; [`Self::settle`]
/// esquece quem não foi declarado — parado num quadro, a curva já não muda nada e o valor dela
/// passa a ser documento (a lei do `Transform` animado).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoVetor {
    autorado: BTreeMap<(u64, FieldKey), f32>,
    neste_quadro: BTreeSet<(u64, FieldKey)>,
}

impl ConducaoVetor {
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
    pub fn substituir(
        &self,
        sim: &mut SimWorld,
        scene: &mut VecScene,
    ) -> Vec<(u64, FieldKey, f32)> {
        let vivo = self.valores(sim, scene);
        self.escrever_todos(
            sim,
            scene,
            self.autorado.iter().map(|(&(e, kk), &v)| (e, kk, v)),
        );
        vivo
    }

    /// Devolve o vivo que [`Self::substituir`] tirou.
    pub fn repor(&self, sim: &mut SimWorld, scene: &mut VecScene, vivo: Vec<(u64, FieldKey, f32)>) {
        self.escrever_todos(sim, scene, vivo.into_iter());
    }

    fn valores(&self, sim: &SimWorld, scene: &VecScene) -> Vec<(u64, FieldKey, f32)> {
        self.autorado
            .keys()
            .filter_map(|&(e, kk)| {
                let (id, shape) = caminho(sim, e)?;
                Some((e, kk, ler(scene.path(id)?, shape.as_ref(), kk)?))
            })
            .collect()
    }

    fn escrever_todos(
        &self,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        valores: impl Iterator<Item = (u64, FieldKey, f32)>,
    ) {
        let mut v: Vec<_> = valores.collect();
        v.sort_by_key(|&(e, kk, _)| (e, prioridade(kk), kk));
        for (e, kk, x) in v {
            let Some((id, mut shape)) = caminho(sim, e) else {
                continue;
            };
            let Some(p) = scene.path_mut(id) else {
                continue;
            };
            escrever(p, shape.as_mut(), kk, x);
            if let (Some(sh), Some(ent)) = (shape, Entity::try_from_bits(e)) {
                if let VecShape::Param { .. } = sh {
                    crate::shape_live::recook_into(scene, id, &sh);
                }
                sim.world_mut().entity_mut(ent).insert(sh);
            }
        }
    }
}

#[cfg(test)]
#[path = "canais_tests.rs"]
mod tests;
