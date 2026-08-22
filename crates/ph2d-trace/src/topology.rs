//! **A TOPOLOGIA DA DECOMPOSIÇÃO** — as três contas de `V − E + F` que dizem se o
//! traçado decompôs a superfície ou outra coisa.
//!
//! ⭐⭐ **O corte contra o [`super::patches`] é de ASSUNTO, e foi forçado pela HR-18**
//! (755 contra 700): lá mora **como** a malha se recorta nas paredes (o flood, as
//! fronteiras, os cantos, os arcos); aqui **se o resultado ainda é a mesma peça**.
//!
//! ⛔ **Este módulo nasce de um remesh que fechava o buraco de um toro em silêncio**
//! (2026-08-22). A malha saía com `χ = 2` onde a topologia exige `0` e passava em
//! **todas** as cercas que existiam: 100 % de quads, zero arestas de bordo, zero
//! não-manifold, cada arco usado exactamente duas vezes, toda fronteira um laço só.
//! *Uma peça pode passar em toda asserção e ter deixado de ser um toro.*
//!
//! | conta | o que responde | onde entra |
//! |---|---|---|
//! | [`mesh_euler`] | o `χ` da peça que chegou | a referência da comparação |
//! | [`patch_chi`] | o `χ` da região de cada patch — um disco dá `1` | **diagnóstico**: diz QUAL patch engoliu a asa |
//! | [`super::PatchLayout::complex_euler`] | o `χ` do complexo *cantos · arcos · patches* | ⭐ **a cerca**, e a única que prevê o `χ` da malha final |

use std::collections::BTreeSet;

use super::PatchLayout;

/// **A CARACTERÍSTICA DE EULER de cada patch**, sobre a região de faces dele.
///
/// ⭐ **Um disco dá `1`**; um anel dá `0`; uma asa com uma fronteira dá `−1`. Ver
/// [`PatchLayout::chi`] para o porquê de a contagem de fronteiras não bastar.
///
/// ⚠️ **Um passe único sobre as faces**, com os conjuntos por patch — a alternativa
/// (um passe por patch) seria `n_patches` varreduras da malha inteira dentro de um
/// laço que já corre até 32 vezes.
pub(crate) fn patch_chi(
    faces: &[ph2d_mesh::Face],
    face_patch: &[u32],
    n_patches: usize,
) -> Vec<i64> {
    let mut verts: Vec<BTreeSet<u32>> = vec![BTreeSet::new(); n_patches];
    let mut edges: Vec<BTreeSet<(u32, u32)>> = vec![BTreeSet::new(); n_patches];
    let mut count: Vec<i64> = vec![0; n_patches];
    for (fi, f) in faces.iter().enumerate() {
        let Some(&p) = face_patch.get(fi) else {
            continue;
        };
        let p = p as usize;
        if p >= n_patches {
            continue;
        }
        count[p] += 1;
        let v = f.verts();
        for k in 0..v.len() {
            verts[p].insert(v[k]);
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            edges[p].insert((a.min(b), a.max(b)));
        }
    }
    (0..n_patches)
        .map(|p| {
            i64::try_from(verts[p].len()).unwrap_or(0) - i64::try_from(edges[p].len()).unwrap_or(0)
                + count[p]
        })
        .collect()
}

/// **`V − E + F` da malha inteira** — a característica de Euler da superfície.
///
/// ⚠️ **Contada das FACES**, e não do `vert_count()` da malha: um vértice órfão
/// (sem face nenhuma) não pertence à superfície e contá-lo daria `χ` a mais.
pub(crate) fn mesh_euler(faces: &[ph2d_mesh::Face]) -> i64 {
    let mut verts: BTreeSet<u32> = BTreeSet::new();
    let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
    for f in faces {
        let v = f.verts();
        for k in 0..v.len() {
            verts.insert(v[k]);
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            edges.insert((a.min(b), a.max(b)));
        }
    }
    i64::try_from(verts.len()).unwrap_or(0) - i64::try_from(edges.len()).unwrap_or(0)
        + i64::try_from(faces.len()).unwrap_or(0)
}

impl PatchLayout {
    /// ⭐⭐ **`V − E + F` do COMPLEXO de patches** — *cantos · arcos · patches*.
    ///
    /// ⭐ **É a única grandeza desta fase que prevê o `χ` da MALHA final.** Uma
    /// decomposição honesta da superfície devolve o `χ` dela **seja qual for o
    /// número de patches**: a estrutura CW mínima de um toro é *um* patch cuja
    /// fronteira percorre duas arestas duas vezes cada, e ela dá `1 − 2 + 1 = 0`.
    ///
    /// ⛔ **Quando o número não bate, há um patch a ser contado como disco sem o
    /// ser.** Uma região com género entra na conta como `+1` quando vale `−1`, e a
    /// diferença é exactamente `2` — foi assim que um toro saiu com o buraco
    /// fechado em 2026-08-22, com 100 % de quads e zero arestas de bordo.
    ///
    /// ⚠️ **Os cantos contam-se por VÉRTICE DE MALHA distinto**, sobre todos os
    /// patches: um vértice que seja canto de quatro patches é **um** 0-célula.
    #[must_use]
    pub fn complex_euler(&self) -> i64 {
        let corners: BTreeSet<u32> = self.corners.iter().flatten().copied().collect();
        i64::try_from(corners.len()).unwrap_or(0) - i64::try_from(self.arc_chain.len()).unwrap_or(0)
            + i64::try_from(self.side_arcs.len()).unwrap_or(0)
    }
}
