//! ⭐⭐⭐ **A família VETOR nos canais** (plano 35 §3.0) — o lado do quadro do
//! [`crate::canais_campos`]: [`publicar`] o que o documento vale em `CanaisLidos`, [`aplicar`] o que
//! a curva mandou em `CanaisEscritos` (com o eco da quantização), e a [`ConducaoVetor`]: a lei do
//! `ph2d_preview_drive` para o documento vetorial — **o que a curva escreve a tocar é
//! pré-visualização**, o undo e o save fotografam o AUTORADO; parado, a curva vira documento.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::{
    CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld, TextosDosCanais, VecPathRef, VecShape,
};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::canais_campos::{campos, escrever, ler, prioridade};
use crate::canais_entidade::{self as entidade, Alvo, e_da_entidade};
use crate::canais_textos::{self as textos, e_de_texto};

fn caminho(sim: &SimWorld, e: u64) -> Option<(u64, Option<VecShape>)> {
    let ent = Entity::try_from_bits(e)?;
    let id = sim.world().get::<VecPathRef>(ent)?.0;
    Some((id, sim.world().get::<VecShape>(ent).cloned()))
}

/// A tabela dos textos dos canais, tirada do mundo enquanto se lê e escreve (ler um texto novo
/// acrescenta-o); [`devolver_textos`] repõe-na.
fn tirar_textos(sim: &mut SimWorld) -> TextosDosCanais {
    sim.world_mut()
        .remove_resource::<TextosDosCanais>()
        .unwrap_or_default()
}

fn devolver_textos(sim: &mut SimWorld, t: TextosDosCanais) {
    sim.world_mut().insert_resource(t);
}

/// A ordem das escritas de um quadro: a da entidade ou a do caminho, estrutura primeiro.
fn ordem(kk: FieldKey) -> u8 {
    if e_da_entidade(kk) {
        entidade::prioridade(kk)
    } else {
        prioridade(kk)
    }
}

/// O valor de `kk` na entidade `e`, o caminho `p` e a forma `shape` — a porta única de leitura.
fn ler_campo(
    sim: &SimWorld,
    e: u64,
    p: Option<&VecPath>,
    shape: Option<&VecShape>,
    kk: FieldKey,
    t: &mut TextosDosCanais,
) -> Option<f32> {
    if e_da_entidade(kk) {
        entidade::ler(sim, Entity::try_from_bits(e)?, kk, t)
    } else if e_de_texto(kk) {
        textos::ler(shape, kk, t)
    } else {
        ler(p?, shape, kk)
    }
}

/// Escreve `v` em `kk` (entidade, texto ou caminho).
fn escrever_campo(
    sim: &mut SimWorld,
    e: u64,
    p: Option<&mut VecPath>,
    shape: Option<&mut VecShape>,
    kk: FieldKey,
    v: f32,
    t: &mut TextosDosCanais,
) {
    if e_da_entidade(kk) {
        if let Some(ent) = Entity::try_from_bits(e) {
            entidade::escrever(sim, ent, kk, v, t);
        }
    } else if e_de_texto(kk) {
        textos::escrever(shape, kk, v, t);
    } else if let Some(p) = p {
        escrever(p, shape, kk, v);
    }
}

/// Devolve à entidade a forma que mudou e re-coze a paramétrica.
fn devolver_forma(sim: &mut SimWorld, scene: &mut VecScene, e: u64, id: u64, sh: VecShape) {
    if let VecShape::Param { .. } = sh {
        crate::shape_live::recook_into(scene, id, &sh);
    }
    if let Some(ent) = Entity::try_from_bits(e) {
        sim.world_mut().entity_mut(ent).insert(sh);
    }
}

/// ⭐ **Publica o que o documento de cada entidade vale AGORA** (através do eco).
pub fn publicar(sim: &mut SimWorld, scene: &VecScene, entidades: &[u64]) {
    let mut t = tirar_textos(sim);
    let mut lotes: Vec<(u64, Vec<(FieldKey, f32)>)> = Vec::with_capacity(entidades.len());
    for &e in entidades {
        let Some(ent) = Entity::try_from_bits(e) else {
            continue;
        };
        let mut valores: Vec<(FieldKey, f32)> = Vec::new();
        let mut alvo = Alvo::default();
        if let Some((id, shape)) = caminho(sim, e)
            && let Some(p) = scene.path(id)
        {
            alvo = Alvo {
                caminho: true,
                fechado: p.closed,
            };
            let sh = shape.as_ref();
            valores.extend(
                campos(p, sh)
                    .into_iter()
                    .filter_map(|kk| Some((kk, ler(p, sh, kk)?))),
            );
            for kk in textos::campos(sh) {
                if let Some(v) = textos::ler(sh, kk, &mut t) {
                    valores.push((kk, v));
                }
            }
        }
        for kk in entidade::campos(sim, ent, alvo) {
            if let Some(v) = entidade::ler(sim, ent, kk, &mut t) {
                valores.push((kk, v));
            }
        }
        lotes.push((e, valores));
    }
    devolver_textos(sim, t);
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (e, valores) in lotes {
        lidos.esquecer(e, VETOR);
        for (kk, v) in valores {
            lidos.publicar(e, kk, v);
        }
    }
}

/// O que [`aplicar`] fez neste quadro.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Aplicado {
    /// O documento mudou.
    pub mudou: bool,
    /// Os textos cuja receita (`VecShape::Text`) a curva mudou: `(caminho, entidade)`. O desenho
    /// de um texto coze-se com a fonte do sistema, que mora na shell — quem chama re-coze-os.
    pub textos: Vec<(VecPathId, Entity)>,
}

/// ⭐ **Aplica ao documento o que a curva mandou neste quadro** e regista o eco.
pub fn aplicar(sim: &mut SimWorld, scene: &mut VecScene, conducao: &mut ConducaoVetor) -> Aplicado {
    let mut feito = Aplicado::default();
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(VETOR))
    else {
        return feito;
    };
    if escritas.is_empty() {
        return feito;
    }
    escritas.sort_by_key(|&(e, kk, _)| (e, ordem(kk), kk));
    let mut t = tirar_textos(sim);
    let mut ecos = Vec::with_capacity(escritas.len());
    let mut i = 0;
    while i < escritas.len() {
        let e = escritas[i].0;
        let fim = escritas[i..]
            .iter()
            .position(|w| w.0 != e)
            .map_or(escritas.len(), |n| i + n);
        let (id, mut shape) = caminho(sim, e).map_or((None, None), |(id, sh)| (Some(id), sh));
        let shape_antes = shape.clone();
        let mut p = id.and_then(|id| scene.path_mut(id));
        for &(_, kk, v) in &escritas[i..fim] {
            let antes = ler_campo(sim, e, p.as_deref(), shape.as_ref(), kk, &mut t);
            escrever_campo(sim, e, p.as_deref_mut(), shape.as_mut(), kk, v, &mut t);
            let ficou = ler_campo(sim, e, p.as_deref(), shape.as_ref(), kk, &mut t);
            if let (Some(a), Some(f)) = (antes, ficou) {
                if a != f {
                    conducao.conduziu(e, kk, a);
                    feito.mudou = true;
                }
                ecos.push((e, kk, v, f));
            }
        }
        if let (Some(id), Some(sh)) = (id, shape)
            && Some(&sh) != shape_antes.as_ref()
        {
            if let (VecShape::Text(_), Some(ent)) = (&sh, Entity::try_from_bits(e)) {
                feito.textos.push((id, ent));
            }
            devolver_forma(sim, scene, e, id, sh);
        }
        i = fim;
    }
    devolver_textos(sim, t);
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (e, kk, v, f) in ecos {
        lidos.eco(e, kk, v, f);
    }
    feito
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

    fn valores(&self, sim: &mut SimWorld, scene: &VecScene) -> Vec<(u64, FieldKey, f32)> {
        let mut t = tirar_textos(sim);
        let out = self
            .autorado
            .keys()
            .filter_map(|&(e, kk)| {
                let (id, shape) = caminho(sim, e).map_or((None, None), |(id, sh)| (Some(id), sh));
                let p = id.and_then(|id| scene.path(id));
                Some((e, kk, ler_campo(sim, e, p, shape.as_ref(), kk, &mut t)?))
            })
            .collect();
        devolver_textos(sim, t);
        out
    }

    fn escrever_todos(
        &self,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        valores: impl Iterator<Item = (u64, FieldKey, f32)>,
    ) {
        let mut v: Vec<_> = valores.collect();
        v.sort_by_key(|&(e, kk, _)| (e, ordem(kk), kk));
        let mut t = tirar_textos(sim);
        for (e, kk, x) in v {
            let (id, mut shape) = caminho(sim, e).map_or((None, None), |(id, sh)| (Some(id), sh));
            let antes = shape.clone();
            let p = id.and_then(|id| scene.path_mut(id));
            escrever_campo(sim, e, p, shape.as_mut(), kk, x, &mut t);
            if let (Some(id), Some(sh)) = (id, shape)
                && Some(&sh) != antes.as_ref()
            {
                devolver_forma(sim, scene, e, id, sh);
            }
        }
        devolver_textos(sim, t);
    }
}

#[cfg(test)]
#[path = "canais_tests.rs"]
mod tests;
