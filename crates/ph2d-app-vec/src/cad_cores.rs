//! **As cores do CAD fora do fio do quadro** (W4.4, `docs/BlenderConstraints/04_W4_cad.md` §1): a análise da ezpz
//! custa 9–12 ms a 200 pontos e ½ s a 800, então `estado()` corre numa thread e SÓ quando a ESTRUTURA muda (as regras,
//! ou que vértices os paths citados têm); o quadro pinta a última resposta que chegou.

use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

use ph2d_vec_constraints::{Estado, Folha, estado};
use ph2d_vec_scene::cad::{PontoCad, RegraCad};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, VertId, Xform, xform_of};

/// O que, mudando, muda as cores: as regras e os ids dos vértices de cada path citado. As POSIÇÕES ficam de fora — um
/// arrasto não pede análise.
type Chave = (Vec<RegraCad>, Vec<(VecPathId, Vec<VertId>)>);

#[derive(Default)]
struct Interno {
    pronto: Option<(Chave, Arc<Estado>)>,
    correndo: Option<(Chave, JoinHandle<Estado>)>,
}

/// A última análise e a que está a correr. Uma de cada vez: uma estrutura nova espera a anterior acabar.
#[derive(Default)]
pub struct CadCores {
    interno: Mutex<Interno>,
}

fn chave(scene: &VecScene) -> Chave {
    let mut ids: Vec<VecPathId> = scene
        .regras()
        .iter()
        .flat_map(|r| r.lei.pontos())
        .map(PontoCad::path)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let paths = ids
        .into_iter()
        .filter_map(|id| {
            let p = scene.path(id)?;
            let verts = p
                .verts
                .iter()
                .chain(p.subpaths.iter().flat_map(|c| &c.verts));
            Some((id, verts.map(|v| v.id).collect()))
        })
        .collect();
    (scene.regras().to_vec(), paths)
}

impl CadCores {
    /// As cores das regras da cena, se já há uma análise da estrutura de AGORA. Pede uma nova quando a estrutura
    /// mudou e nenhuma está a correr; nunca espera.
    pub fn ve(&self, scene: &VecScene, xforms: &VecXforms) -> Option<Arc<Estado>> {
        if scene.regras().is_empty() {
            return None;
        }
        let k = chave(scene);
        let mut i = self.interno.lock().unwrap_or_else(PoisonError::into_inner);
        if i.correndo.as_ref().is_some_and(|(_, h)| h.is_finished())
            && let Some((kc, h)) = i.correndo.take()
            && let Ok(e) = h.join()
        {
            i.pronto = Some((kc, Arc::new(e)));
        }
        if let Some((kp, e)) = &i.pronto
            && *kp == k
        {
            return Some(Arc::clone(e));
        }
        if i.correndo.is_none() {
            let paths: Vec<(VecPath, Xform)> =
                k.1.iter()
                    .filter_map(|(id, _)| Some((scene.path(*id)?.clone(), xform_of(xforms, *id))))
                    .collect();
            let regras = k.0.clone();
            let h = std::thread::spawn(move || {
                estado(&Folha::new(paths.iter().map(|(p, x)| (p, *x))), &regras)
            });
            i.correndo = Some((k, h));
        }
        None
    }

    /// Espera a análise que está a correr (os gates; o quadro nunca chama isto).
    #[cfg(test)]
    pub(crate) fn espera(&self) {
        let mut i = self.interno.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((k, h)) = i.correndo.take()
            && let Ok(e) = h.join()
        {
            i.pronto = Some((k, Arc::new(e)));
        }
    }
}
