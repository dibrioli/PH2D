//! ⭐ **A medida animada** (W4.5, `docs/BlenderConstraints/04_W4_cad.md` §3): a curva da família
//! [`ph2d_ecs::canal::familia::CAD`] dá o valor de uma regra de distância, ângulo ou raio, e a pose resolve-se
//! SEMPRE a partir do desenho AUTORADO — nunca do quadro anterior ⇒ tocar e arrastar a timeline dão o mesmo bit
//! (um sketch sub-restringido tem infinitas soluções: partir de outro sítio dava outra).
//!
//! O que a curva mostra é pré-visualização (a lei do `ph2d_preview_drive`): a [`ConducaoCad`] guarda o desenho e
//! as medidas autorados; o undo e o save fotografam-nos, e quem publica o documento lê-o daqui ([`ConducaoCad::path`]).
//! Uma edição do artista (a cena deixou de ser o que o solver escreveu) passa a ser o autorado.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::canal_cad::{MEDIDA, chave, regra_de};
use ph2d_ecs::{CanaisLidos, Entity, FieldKey, SimWorld, VecPathRef};
use ph2d_vec_constraints::{Folha, resolve};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraId};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, xform_of};

/// O valor que o canal guarda (graus num ângulo; metros — a unidade do mundo — no resto).
#[must_use]
pub fn no_canal(lei: &Lei) -> Option<f32> {
    let m = lei.medida()?;
    let v = if matches!(lei, Lei::Angulo(..)) {
        m.to_degrees()
    } else {
        m
    };
    #[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
    Some(v as f32)
}

fn do_canal(lei: &Lei, v: f32) -> f64 {
    let v = f64::from(v);
    if matches!(lei, Lei::Angulo(..)) {
        v.to_radians()
    } else {
        v
    }
}

/// As medidas cujo DONO é o path `path` ([`Lei::dono`]), como o canal as lê — a publicação do documento.
#[must_use]
pub fn campos(scene: &VecScene, path: VecPathId) -> Vec<(FieldKey, f32)> {
    scene
        .regras()
        .iter()
        .filter(|r| r.lei.dono() == Some(path))
        .filter_map(|r| Some((chave(MEDIDA, r.id.0), no_canal(&r.lei)?)))
        .collect()
}

/// O afim local → folha de cada path citado, pela entidade que o leva.
fn xforms(sim: &mut SimWorld, ids: &BTreeSet<VecPathId>) -> VecXforms {
    let mut q = sim.world_mut().query::<(Entity, &VecPathRef)>();
    let donos: Vec<(VecPathId, Entity)> = q
        .iter(sim.world())
        .filter(|(_, r)| ids.contains(&r.0))
        .map(|(e, r)| (r.0, e))
        .collect();
    donos
        .into_iter()
        .map(|(id, e)| {
            let t = crate::transform::world_transform(sim, e);
            (id, crate::transform::xform_of_transform(t))
        })
        .collect()
}

/// O que a curva moveu: o autorado e o que se escreveu por cima dele.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoCad {
    autorado: BTreeMap<VecPathId, VecPath>,
    medidas: BTreeMap<RegraId, f64>,
    escrito: BTreeMap<VecPathId, VecPath>,
    valores: BTreeMap<RegraId, f64>,
    neste_quadro: bool,
}

/// O vivo que [`ConducaoCad::substituir`] tirou da cena, para [`ConducaoCad::repor`].
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CadVivo {
    paths: Vec<VecPath>,
    medidas: Vec<(RegraId, f64)>,
}

impl ConducaoCad {
    /// O path `id` tal como o artista o fez, se a curva o está a mover — o documento que se publica.
    #[must_use]
    pub fn path(&self, id: VecPathId) -> Option<&VecPath> {
        self.autorado.get(&id)
    }

    /// Um quadro sem escrita da curva: a medida deixou de ser conduzida e o que se vê passa a ser documento.
    pub fn settle(&mut self) {
        if !std::mem::take(&mut self.neste_quadro) {
            *self = Self::default();
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.autorado.is_empty() && self.medidas.is_empty()
    }

    /// Repõe o AUTORADO na cena para a fotografia; devolve o vivo.
    pub fn substituir(&self, scene: &mut VecScene) -> CadVivo {
        let mut vivo = CadVivo::default();
        for (&id, p) in &self.autorado {
            if let Some(q) = scene.path_mut(id) {
                vivo.paths.push(std::mem::replace(q, p.clone()));
            }
        }
        for (&id, &m) in &self.medidas {
            if let Some(d) = scene.regra_mut(id).and_then(|r| r.lei.medida_mut()) {
                vivo.medidas.push((id, std::mem::replace(d, m)));
            }
        }
        vivo
    }

    /// Devolve o vivo que [`Self::substituir`] tirou.
    pub fn repor(scene: &mut VecScene, vivo: CadVivo) {
        for p in vivo.paths {
            if let Some(q) = scene.path_mut(p.id) {
                *q = p;
            }
        }
        for (id, m) in vivo.medidas {
            if let Some(d) = scene.regra_mut(id).and_then(|r| r.lei.medida_mut()) {
                *d = m;
            }
        }
    }
}

/// ⭐ **Aplica as escritas da família `CAD`** (as de `CanaisEscritos::drenar(CAD)`): a medida de cada regra vale o
/// da curva e a folha resolve-se a partir do desenho autorado. Só re-resolve quando uma medida ou o autorado mudou
/// (parado num quadro não custa nada). Regista o eco. Devolve se a cena mudou.
pub fn aplicar(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    escritas: Vec<(u64, FieldKey, f32)>,
    cond: &mut ConducaoCad,
) -> bool {
    let mut pedidas: BTreeMap<RegraId, f64> = BTreeMap::new();
    let mut ecos = Vec::with_capacity(escritas.len());
    for (e, k, v) in escritas {
        let id = RegraId(regra_de(k));
        if let Some(r) = scene.regra(id)
            && k.campo == MEDIDA
            && r.lei.medida().is_some()
        {
            pedidas.insert(id, do_canal(&r.lei, v));
            ecos.push((e, k, v));
        }
    }
    if pedidas.is_empty() {
        return false;
    }
    cond.neste_quadro = true;
    let ids: BTreeSet<VecPathId> = scene
        .regras()
        .iter()
        .flat_map(|r| r.lei.pontos())
        .map(PontoCad::path)
        .collect();
    cond.autorado.retain(|id, _| ids.contains(id));
    cond.escrito.retain(|id, _| ids.contains(id));
    let mut refaz = false;
    for &id in &ids {
        if let Some(p) = scene.path(id)
            && cond.escrito.get(&id) != Some(p)
        {
            cond.autorado.insert(id, p.clone());
            refaz = true;
        }
    }
    for (&id, &v) in &pedidas {
        let doc = scene.regra(id).and_then(|r| r.lei.medida());
        let escrito = cond.valores.get(&id).copied();
        if doc != escrito
            && let Some(d) = doc
        {
            cond.medidas.insert(id, d);
        }
        refaz |= doc != escrito || escrito != Some(v);
    }
    if refaz {
        for (&id, &v) in &pedidas {
            if let Some(d) = scene.regra_mut(id).and_then(|r| r.lei.medida_mut()) {
                *d = v;
            }
            cond.valores.insert(id, v);
        }
        let xfs = xforms(sim, &ids);
        let res = {
            let folha = Folha::new(
                ids.iter()
                    .filter_map(|&id| cond.autorado.get(&id).map(|p| (p, xform_of(&xfs, id)))),
            );
            resolve(&folha, scene.regras(), None, None)
        };
        for &id in &ids {
            let Some(mut p) = cond.autorado.get(&id).cloned() else {
                continue;
            };
            res.solucao.aplica(&mut p, xform_of(&xfs, id));
            if let Some(q) = scene.path_mut(id) {
                q.clone_from(&p);
            }
            cond.escrito.insert(id, p);
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (e, k, v) in ecos {
        if let Some(f) = scene
            .regra(RegraId(regra_de(k)))
            .and_then(|r| no_canal(&r.lei))
        {
            lidos.eco(e, k, v, f);
        }
    }
    refaz
}

#[cfg(test)]
#[path = "medida_tests.rs"]
mod tests;
