//! ⭐⭐⭐ **AS RESTRIÇÕES DE ORIENTAÇÃO** — uma aresta de feição FIXA o `θ` dos dois
//! triângulos que a tocam, e um `θ` fixo **sai do sistema**.
//!
//! ⛔⛔ **A LEI, e ela é a mesma da obra A** (`SPEC_restricoes_por_eliminacao.md` §1):
//! *uma restrição linear entra ELIMINANDO uma variável, nunca como termo de energia.*
//! A costura pagou a lição com números — penalizada, ela deixava `1,00` de rasgo e a
//! casca não fechava; eliminada, o resíduo é **zero**. ⇒ aqui o `θ_f` de uma face
//! restringida **deixa de ser incógnita**: o valor dele passa para o lado constante de
//! cada aresta dual que a toca, exactamente como o `κ` e o salto de período já passam.
//!
//! ⚠️ **Isto NÃO é o [`crate::ALIGN_WEIGHT`], e a diferença é o assunto todo.** O
//! alinhamento ao relevo é um termo **suave** com peso `0,03`, que puxa toda face
//! confiante para a direcção principal de curvatura — ele negoceia com a suavidade e
//! perde quase sempre, que é o que se quer num relevo macio. Uma **feição** não
//! negoceia: ou a cruz fica paralela ao vinco, ou o vinco não sobrevive à extracção.
//!
//! # ⚠️ O representante 4-RoSy, e por que ele é um GAUGE aqui
//!
//! Uma cruz tem quatro braços: `α` e `α + k·π/2` descrevem a mesma orientação. Ao
//! eliminar `θ_f` é preciso escrever **um** valor, e a escolha de `k` desloca o
//! constante de cada aresta incidente por múltiplos de `π/2` — que é exactamente o que
//! o salto de período `p_e` daquela aresta absorve. ⭐ *O campo resultante é o mesmo;
//! o que muda é a semente do arredondamento guloso.* ⇒ escolhe-se o `k` mais próximo
//! do `θ` corrente, que é a mesma regra que o termo de alinhamento já usa, e pelo
//! mesmo motivo: nenhuma face deve dar meia volta onde a forma não vira.
//!
//! # ⛔ A CERCA: duas feições que se cruzam em diagonal não têm resposta
//!
//! Duas linhas de vinco **perpendiculares** concordam — 90° é a identidade da cruz, e
//! a face fica bem definida. Duas a 45° não: nenhuma orientação de cruz é paralela às
//! duas. A espec manda ser **conservador** (*«marcar feição a mais é pior que a
//! menos»*), então a face em conflito **perde as duas restrições** e volta a ser
//! incógnita. ⚠️ A contagem sai no [`ConstrainReport::conflicts`] — *uma cerca sem
//! instrumento é indistinguível de um caso que nunca acontece.*

use std::collections::BTreeMap;

use ph2d_mesh::{FeatureEdge, Mesh};

use crate::{Dual, QUARTER, cross, dot};

/// ⭐ **Quanto duas leituras da mesma face podem discordar** — `5°`, medidos no
/// círculo de `π/2` da cruz.
///
/// ⚠️ **Ele é uma CERCA, não um coeficiente a afinar:** subir torna a restrição uma
/// média de duas direcções (que não é paralela a nenhuma), descer joga fora cantos
/// legítimos onde a discretização deixou as duas leituras a 1°–2° uma da outra.
pub const CONSTRAINT_AGREEMENT: f32 = core::f32::consts::PI / 36.0;

/// O que a restrição fez, contado.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConstrainReport {
    /// Arestas de feição que se pediram.
    pub edges: usize,
    /// ⭐ Faces cujo `θ` ficou FIXO — as variáveis eliminadas.
    pub faces: usize,
    /// ⛔ Faces largadas por **conflito** entre duas leituras. Ver a cerca no doc do
    /// módulo.
    pub conflicts: usize,
    /// Arestas cuja direcção não tinha projecção utilizável na moldura de uma face.
    pub degenerate: usize,
}

impl Dual {
    /// **O `θ` FIXO da face `f`**, na moldura dela e reduzido a `[0, π/2)`, ou `None`
    /// se ela continua a ser incógnita.
    #[must_use]
    pub fn constrained(&self, f: usize) -> Option<f32> {
        self.constrained.get(f).copied().flatten()
    }

    /// Quantas faces têm o `θ` fixo.
    #[must_use]
    pub fn constrained_count(&self) -> usize {
        self.constrained.iter().filter(|c| c.is_some()).count()
    }

    /// ⭐⭐⭐ **FIXA o `θ` dos triângulos que tocam cada aresta de feição.**
    ///
    /// ⚠️ **Ponto de extensão append-only** (`CLAUDE.md` §0.2): um [`Dual`] em que
    /// isto nunca é chamado é byte-idêntico ao de sempre — não há campo novo no
    /// caminho de quem não restringe, só um vector de `None`.
    ///
    /// ⚠️ **A malha entra outra vez** porque o grafo dual não guarda quem são os
    /// vértices de cada face: ele foi construído para o solver, que só precisa de
    /// faces e de `κ`. *Passá-la é mais barato que a guardar para um consumidor que
    /// quase nunca existe.*
    pub fn constrain(&mut self, mesh: &Mesh, edges: &[FeatureEdge]) -> ConstrainReport {
        let mut rep = ConstrainReport {
            edges: edges.len(),
            ..ConstrainReport::default()
        };
        if edges.is_empty() {
            return rep;
        }
        // ⚠️ **A direcção viaja com a aresta**, e não se re-deriva aqui: quem a decidiu foi
        // a [`ph2d_mesh::feature_edges`], por uma razão MEDIDA que o doc dela nomeia (a
        // direcção da ARESTA levava o campo de 25 para 242 singularidades).
        let want: BTreeMap<(u32, u32), [f32; 3]> = edges
            .iter()
            .map(|e| {
                (
                    (e.verts[0].min(e.verts[1]), e.verts[0].max(e.verts[1])),
                    e.dir,
                )
            })
            .collect();

        // Aresta pedida -> as faces que a usam. ⚠️ `BTreeMap` pela mesma razão do
        // [`Dual::build`]: a ordem entra na ordem em que as faces são escritas.
        let mut owner: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
        for (fi, f) in mesh.faces().iter().enumerate() {
            let v = f.verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                let key = (a.min(b), a.max(b));
                if want.contains_key(&key) {
                    owner.entry(key).or_default().push(fi);
                }
            }
        }

        // `None` = nunca tocada · `Some(None)` = em CONFLITO · `Some(Some(a))` = fixa.
        let mut claim: BTreeMap<usize, Option<f32>> = BTreeMap::new();
        for (key, who) in &owner {
            let d = want[key];
            for &fi in who {
                let Some(alpha) = self.angle_in_frame(fi, d) else {
                    rep.degenerate += 1;
                    continue;
                };
                match claim.get(&fi) {
                    None => {
                        claim.insert(fi, Some(alpha));
                    }
                    Some(None) => {}
                    Some(Some(prev)) => {
                        if quarter_gap(*prev, alpha) > CONSTRAINT_AGREEMENT {
                            claim.insert(fi, None);
                        }
                    }
                }
            }
        }

        for (fi, a) in claim {
            match a {
                Some(alpha) => {
                    self.constrained[fi] = Some(alpha);
                    rep.faces += 1;
                }
                None => rep.conflicts += 1,
            }
        }
        rep
    }

    /// O ângulo de `d` na moldura da face `f`, reduzido ao quarto de volta.
    fn angle_in_frame(&self, f: usize, d: [f32; 3]) -> Option<f32> {
        let fr = *self.frames().get(f)?;
        let k = dot(d, fr.n);
        let t = [
            k.mul_add(-fr.n[0], d[0]),
            k.mul_add(-fr.n[1], d[1]),
            k.mul_add(-fr.n[2], d[2]),
        ];
        if dot(t, t) <= 1.0e-24 {
            return None;
        }
        let b = cross(fr.n, fr.e);
        Some(dot(t, b).atan2(dot(t, fr.e)).rem_euclid(QUARTER))
    }
}

/// A distância entre dois ângulos **no círculo de `π/2`** — `0` quando as duas cruzes
/// são a mesma, `π/4` quando estão o mais longe possível uma da outra.
fn quarter_gap(a: f32, b: f32) -> f32 {
    let d = (b - a).rem_euclid(QUARTER);
    d.min(QUARTER - d)
}

#[cfg(test)]
#[path = "constrain_tests.rs"]
mod tests;
