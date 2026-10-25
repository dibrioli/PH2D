//! ⭐⭐⭐ **A família VETOR nos canais** (plano 35 §3.0) — o lado do quadro do
//! [`crate::canais_campos`]: [`publicar`] o que o documento vale em `CanaisLidos`, [`aplicar`] o que
//! a curva mandou em `CanaisEscritos` (com o eco da quantização), e a [`ConducaoVetor`]: a lei do
//! `ph2d_preview_drive` para o documento vetorial — **o que a curva escreve a tocar é
//! pré-visualização**, o undo e o save fotografam o AUTORADO; parada a timeline, a curva vira
//! documento. A família `CAD` (a medida animada, W4.5) drena-se aqui também: o documento é o mesmo
//! ([`ph2d_vec_entities::medida`]).

use ph2d_ecs::canal::familia::{CAD, VETOR};
use ph2d_ecs::canal_conducao::ConducaoDosCanais;
use ph2d_ecs::{
    CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld, TextosDosCanais, VecPathRef, VecShape,
};
use ph2d_vec_entities::medida::{self, CadVivo, ConducaoCad};
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

/// ⭐ **Publica o que o documento de cada entidade vale AGORA** (através do eco). Um path que a medida animada está
/// a mover publica o desenho AUTORADO: o que o solver mostra não é o artista a mexer (o Auto Key não o grava).
pub fn publicar(sim: &mut SimWorld, scene: &VecScene, conducao: &ConducaoVetor, entidades: &[u64]) {
    let mut t = tirar_textos(sim);
    let mut lotes: Vec<(u64, Vec<(FieldKey, f32)>)> = Vec::with_capacity(entidades.len());
    for &e in entidades {
        let Some(ent) = Entity::try_from_bits(e) else {
            continue;
        };
        let mut valores: Vec<(FieldKey, f32)> = Vec::new();
        let mut alvo = Alvo::default();
        if let Some((id, shape)) = caminho(sim, e)
            && let Some(p) = conducao.cad.path(id).or_else(|| scene.path(id))
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
            valores.extend(medida::campos(scene, id));
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
        lidos.esquecer(e, CAD);
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

/// ⭐ **Aplica ao documento o que a curva mandou neste quadro** e regista o eco: os campos do vetor e, depois
/// deles, as medidas animadas (a folha resolve-se a partir do autorado, [`medida::aplicar`]).
pub fn aplicar(sim: &mut SimWorld, scene: &mut VecScene, conducao: &mut ConducaoVetor) -> Aplicado {
    let cad = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(CAD))
        .unwrap_or_default();
    let mut feito = aplicar_vetor(sim, scene, conducao);
    feito.mudou |= medida::aplicar(sim, scene, cad, &mut conducao.cad);
    feito
}

fn aplicar_vetor(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    conducao: &mut ConducaoVetor,
) -> Aplicado {
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
                feito.mudou |= conducao.escreveu(e, kk, a, f);
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
/// O livro e a lei de esquecer (a tocar × parado) são os de [`ConducaoDosCanais`], partilhados
/// com o Flip e a física.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoVetor {
    campos: ConducaoDosCanais,
    /// O desenho que a medida animada está a mover.
    cad: ConducaoCad,
}

/// O vivo que [`ConducaoVetor::substituir`] tirou, para [`ConducaoVetor::repor`].
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Vivo {
    campos: Vec<(u64, FieldKey, f32)>,
    cad: CadVivo,
}

impl std::ops::Deref for ConducaoVetor {
    type Target = ConducaoDosCanais;
    fn deref(&self) -> &ConducaoDosCanais {
        &self.campos
    }
}

impl std::ops::DerefMut for ConducaoVetor {
    fn deref_mut(&mut self) -> &mut ConducaoDosCanais {
        &mut self.campos
    }
}

impl ConducaoVetor {
    /// Esquece quem não foi conduzido neste quadro (chamado onde o `PreviewDrive::settle` é): os campos e
    /// o desenho da medida animada.
    pub fn settle(&mut self) {
        self.campos.settle();
        self.cad.settle();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.campos.is_empty() && self.cad.is_empty()
    }

    /// Repõe o AUTORADO no documento para a fotografia (o desenho da medida animada primeiro, os campos por cima);
    /// devolve o vivo para [`Self::repor`].
    pub fn substituir(&self, sim: &mut SimWorld, scene: &mut VecScene) -> Vivo {
        let cad = self.cad.substituir(scene);
        let campos = self.valores(sim, scene);
        self.escrever_todos(sim, scene, self.autorado());
        Vivo { campos, cad }
    }

    /// Devolve o vivo que [`Self::substituir`] tirou (na ordem inversa).
    pub fn repor(&self, sim: &mut SimWorld, scene: &mut VecScene, vivo: Vivo) {
        self.escrever_todos(sim, scene, vivo.campos.into_iter());
        ConducaoCad::repor(scene, vivo.cad);
    }

    fn valores(&self, sim: &mut SimWorld, scene: &VecScene) -> Vec<(u64, FieldKey, f32)> {
        let mut t = tirar_textos(sim);
        let out = self
            .campos()
            .filter_map(|(e, kk)| {
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
