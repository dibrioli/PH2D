//! **O CAD na caneta** (W4.4, `docs/BlenderConstraints/04_W4_cad.md` §3): o arrasto de âncora resolve as regras
//! DEPOIS do snap (o snap dá o alvo, o solver o resto), e a guia que está acesa vira regra. Bloco `impl PenTool`
//! inerente; as portas com encaixe embrulham as de sempre e a shell troca uma pela outra.

use std::collections::BTreeSet;

use ph2d_vec_constraints::{Arrasto, Folha, Solucao, a_mais, resolve};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms, xform_of};

use super::{Part, PenClick, PenTool};
use crate::snap::{SnapResult, SnapSource};

/// O estado de UM gesto: o palpite do passo anterior, os paths que o solver escreveu e as regras que a guia acesa
/// juntou (as últimas da cena, trocadas a cada passo enquanto a guia muda).
#[derive(Clone, Debug, Default)]
pub(crate) struct CadGesto {
    palpite: Option<Solucao>,
    paths: Vec<VecPathId>,
    inferidas: Vec<RegraCad>,
    leis: Vec<Lei>,
}

impl PenTool {
    /// [`Self::on_press`] com o snap inteiro: o vértice que a caneta POSICIONA ganha a regra da guia acesa.
    pub fn on_press_encaixe(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        px_to_world: f64,
        alt: bool,
        snap: &mut dyn FnMut([f64; 2]) -> SnapResult,
    ) -> PenClick {
        let mut ultimo = SnapResult::default();
        let click = self.on_press(scene, p, px_to_world, alt, &mut |q| {
            ultimo = snap(q);
            ultimo.apply(q)
        });
        if matches!(click, PenClick::Added | PenClick::Started)
            && let Some(id) = self.active
            && let Some(path) = scene.path_mut(id)
        {
            path.normaliza_ids();
            if let Some(v) = path.verts.last() {
                let m = PontoCad::Vert(id, v.id);
                for lei in leis_da_guia(m, &ultimo) {
                    junta_regra(scene, &self.xforms, lei);
                }
            }
        }
        click
    }

    /// [`Self::on_drag`] com o snap inteiro: no arrasto de âncora, o solver leva o resto da folha e a guia acesa
    /// vira regra (a do último passo fica ao soltar).
    pub fn on_drag_encaixe(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        snap: &mut dyn FnMut([f64; 2]) -> SnapResult,
    ) -> bool {
        let mut ultimo = None;
        let consumed = self.on_drag(scene, p, &mut |q| {
            let r = snap(q);
            ultimo = Some(r);
            r.apply(q)
        });
        if let (true, Some(g), Some(r)) = (consumed, self.grab, ultimo)
            && g.part == Part::Anchor
        {
            self.cad_passo(scene, g.path, g.vert, r.apply(p), &r);
        }
        consumed
    }

    /// Os paths que o CAD está a escrever neste gesto: a shell não lhes assenta a origem a meio (o `Xform` publicado
    /// ficaria velho para o passo seguinte).
    #[must_use]
    pub fn cad_em_gesto(&self) -> &[VecPathId] {
        &self.cad.paths
    }

    pub(crate) fn cad_solta(&mut self) {
        self.cad = CadGesto::default();
    }

    fn cad_passo(
        &mut self,
        scene: &mut VecScene,
        path: VecPathId,
        i: usize,
        alvo: [f64; 2],
        r: &SnapResult,
    ) {
        let Some(p) = scene.path_mut(path) else {
            return;
        };
        p.normaliza_ids();
        let Some(m) = p.vert(i).map(|v| PontoCad::Vert(path, v.id)) else {
            return;
        };
        // As regras que a guia juntou no passo anterior saem: neste passo a guia pode ser outra.
        let n = scene
            .regras()
            .len()
            .saturating_sub(self.cad.inferidas.len());
        if scene.regras().ends_with(&self.cad.inferidas) {
            scene.regras_mut().truncate(n);
        }
        self.cad_resolve(scene, m, alvo);
        let leis = leis_da_guia(m, r);
        if leis != self.cad.leis {
            self.cad.inferidas = leis
                .iter()
                .filter_map(|&l| junta_regra(scene, &self.xforms, l))
                .collect();
            self.cad.leis = leis;
        } else {
            scene
                .regras_mut()
                .extend(self.cad.inferidas.iter().copied());
        }
    }

    fn cad_resolve(&mut self, scene: &mut VecScene, m: PontoCad, alvo: [f64; 2]) {
        let regras = scene.regras();
        if !regras
            .iter()
            .any(|r| !r.referencia && r.lei.pontos().contains(&m))
        {
            return;
        }
        let ids: BTreeSet<VecPathId> = regras
            .iter()
            .flat_map(|r| r.lei.pontos())
            .map(PontoCad::path)
            .collect();
        let res = {
            let folha = Folha::new(
                ids.iter()
                    .filter_map(|&id| scene.path(id).map(|p| (p, self.xf(id)))),
            );
            resolve(
                &folha,
                regras,
                Some(Arrasto { ponto: m, alvo }),
                self.cad.palpite.as_ref(),
            )
        };
        for &id in &ids {
            let xf = self.xf(id);
            if let Some(p) = scene.path_mut(id) {
                res.solucao.aplica(p, xf);
            }
        }
        self.cad.paths = ids.into_iter().collect();
        self.cad.palpite = Some(res.solucao);
    }
}

/// As leis que um encaixe em âncoras diz sobre o ponto `m`: os dois eixos no MESMO vértice = coincidente; o eixo X
/// partilhado = vertical; o Y = horizontal. Grade, guia da régua e curva não citam vértice nenhum.
fn leis_da_guia(m: PontoCad, r: &SnapResult) -> Vec<Lei> {
    let dono = |a: Option<crate::snap::SnapAxis>| {
        a.filter(|a| a.from == SnapSource::Shape)
            .and_then(|a| a.dono)
            .filter(|&q| q != m)
    };
    match (dono(r.x), dono(r.y)) {
        (Some(qx), Some(qy)) if qx == qy => vec![Lei::Coincidente(qx, m)],
        (qx, qy) => qx
            .map(|q| Lei::Vertical(SegCad(q, m)))
            .into_iter()
            .chain(qy.map(|q| Lei::Horizontal(SegCad(q, m))))
            .collect(),
    }
}

/// A mesma lei, lida como o artista a lê: os dois extremos de uma H/V ou de uma coincidência trocados são a mesma.
fn mesma(a: &Lei, b: &Lei) -> bool {
    let troca = |s: SegCad| SegCad(s.1, s.0);
    match (*a, *b) {
        (Lei::Horizontal(s), Lei::Horizontal(t)) | (Lei::Vertical(s), Lei::Vertical(t)) => {
            s == t || s == troca(t)
        }
        (Lei::Coincidente(p, q), Lei::Coincidente(u, v)) => (p, q) == (u, v) || (p, q) == (v, u),
        _ => a == b,
    }
}

/// ⭐ **A porta única de juntar uma regra ao documento.** Repetida ⇒ nada. A mais (não sobe o posto na pose de agora,
/// [`a_mais`]): uma MEDIDA nasce REF (só mostra), uma lei geométrica não entra — o desenho já a cumpre. Devolve a regra
/// juntada.
pub fn junta_regra(scene: &mut VecScene, xforms: &VecXforms, lei: Lei) -> Option<RegraCad> {
    if scene.regras().iter().any(|r| mesma(&r.lei, &lei)) {
        return None;
    }
    let medida = matches!(lei, Lei::Distancia(..) | Lei::Angulo(..) | Lei::Raio(..));
    let ids: BTreeSet<VecPathId> = scene
        .regras()
        .iter()
        .flat_map(|r| r.lei.pontos())
        .chain(lei.pontos())
        .map(PontoCad::path)
        .collect();
    let nova = RegraCad::nova(lei);
    let sobra = {
        let folha = Folha::new(
            ids.iter()
                .filter_map(|&id| scene.path(id).map(|p| (p, xform_of(xforms, id)))),
        );
        a_mais(&folha, scene.regras(), nova)
    };
    let regra = match (sobra, medida) {
        (false, _) => nova,
        (true, true) => RegraCad {
            referencia: true,
            ..nova
        },
        (true, false) => return None,
    };
    scene.regras_mut().push(regra);
    Some(regra)
}

#[cfg(test)]
#[path = "cad_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "cad_medida_tests.rs"]
mod medida;
