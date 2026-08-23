//! ⭐⭐⭐ **G2 — PENTEAR CADA PATCH, e medir o SALTO DE PERÍODO de cada costura.**
//!
//! # O que esta fase entrega, e por que são duas coisas
//!
//! Uma cruz tem quatro braços; «a direcção do campo nesta face» é uma escolha entre
//! quatro. **Dentro** de um patch escolhe-se uma vez e propaga-se — é o pentear, e o
//! [`ph2d_crossfield::comb`] já o faz. ⭐ **Mas dois patches vizinhos penteiam-se
//! independentemente**, e as molduras deles ficam desencontradas por um múltiplo de
//! `90°`: é o **salto de período** da costura.
//!
//! ⚠️ **Sem o salto, a fase seguinte não consegue acoplar as costuras** — ela pediria
//! que `(u, v)` fosse igual dos dois lados, quando o que é igual é `(u, v)` de um lado
//! e **o do outro rodado** de `k` quartos de volta. *Pedir a igualdade crua torce a
//! grade em cada fronteira, e o defeito lê-se como geometria.*
//!
//! # ⛔⛔ A armadilha do alinhamento, e ela é silenciosa
//!
//! O [`ph2d_crossfield::comb`] **deixa de fora** faces degeneradas e devolve a lista já
//! sem elas. ⇒ *a saída dele nem sempre está alinhada com a entrada*, e usar o índice
//! `i` das duas como se fosse o mesmo daria a direcção do triângulo errado — **sem erro
//! nenhum a acusar**, só números pouco piores.
//!
//! ⭐ A rede é a coluna que ele já traz: `Holonomy::skipped`. Se ela não for `0`, este
//! módulo **recusa o patch** e conta a recusa ([`CombReport::misaligned`]). *Recusar é
//! uma resposta; alinhar por acaso não é.*

use ph2d_crossfield::Holonomy;
use ph2d_mesh::Mesh;
use ph2d_trace::PatchLayout;

use crate::cut::CutMesh;

/// Uma cruz tem quatro braços a `90°`.
const QUARTER: f32 = std::f32::consts::FRAC_PI_2;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn unit(a: [f32; 3]) -> Option<[f32; 3]> {
    let l = dot(a, a).sqrt();
    (l > 1.0e-12).then(|| [a[0] / l, a[1] / l, a[2] / l])
}

/// A componente de `v` no plano de `n`, normalizada.
fn tangent(v: [f32; 3], n: [f32; 3]) -> Option<[f32; 3]> {
    let d = dot(v, n);
    unit([
        d.mul_add(-n[0], v[0]),
        d.mul_add(-n[1], v[1]),
        d.mul_add(-n[2], v[2]),
    ])
}

fn normal_of(mesh: &Mesh, f: u32) -> Option<[f32; 3]> {
    let v = mesh.faces().get(f as usize)?.verts();
    if v.len() < 3 {
        return None;
    }
    let p = mesh.positions();
    unit(cross(
        sub(p[v[1] as usize], p[v[0] as usize]),
        sub(p[v[2] as usize], p[v[0] as usize]),
    ))
}

/// O que o pentear entregou.
#[derive(Debug, Clone, Default)]
pub struct Combed {
    /// Por patch, por **triângulo na ordem de [`CutMesh::tris`]**, a direcção penteada
    /// em `ℝ³`. ⚠️ Vazio = aquele patch foi recusado.
    pub dir: Vec<Vec<[f32; 3]>>,
    /// Por patch, a holonomia. ⭐ `defects > 0` = há singularidade **dentro**, e a
    /// moldura daquele patch não é consistente.
    pub holonomy: Vec<Option<Holonomy>>,
    /// ⭐⭐⭐ Por costura (na ordem de [`CutMesh::seams`]), quantos **quartos de volta**
    /// separam a moldura do lado `0` da do lado `1`.
    ///
    /// ⚠️ `None` quando não se conseguiu ler — e é uma resposta, não um `0`. *Um zero
    /// de «não sei» acoplaria a costura como se as molduras coincidissem.*
    pub jump: Vec<Option<i32>>,
}

/// O que esta fase mediu de si própria.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CombReport {
    /// Quantos patches entraram.
    pub patches: usize,
    /// Quantos foram penteados.
    pub combed: usize,
    /// ⛔ Quantos foram recusados por o `comb` ter deixado faces de fora — ver o doc
    /// deste módulo.
    pub misaligned: usize,
    /// ⛔ Quantos o `comb` recusou por completo.
    pub refused: usize,
    /// ⭐ Quantos patches trazem **singularidade dentro** (`defects > 0`).
    pub dirty: usize,
    /// Quantas costuras entraram.
    pub seams: usize,
    /// Quantas ficaram com salto lido.
    pub jumps: usize,
    /// **Em quantas costuras as arestas da cadeia DISCORDARAM** sobre o salto.
    ///
    /// ⚠️ **Discordar NÃO é necessariamente um defeito desta fase.** Um patch com
    /// singularidade dentro não é penteável — a moldura dele depende do caminho — e a
    /// costura dele **tem** de discordar. *Medido 2026-08-23: no toro as `4` costuras
    /// inconsistentes tocam todas um dos `3` patches sujos; na esfera lisa, com `0`
    /// sujos, são `0` de `42`.*
    ///
    /// ⇒ a régua com barra é a irmã abaixo.
    pub inconsistent: usize,
    /// ⭐⭐⭐ **Em quantas costuras cujos DOIS lados estão limpos as arestas
    /// discordaram** — e esta tem de ser `0`.
    ///
    /// ⛔⛔ **Uma barra sobre [`Self::inconsistent`] seria frouxa nos dois sentidos:**
    /// reprovaria sobre uma dívida do F3 que esta fase não pode curar, e — se alguém a
    /// baixasse para o número do toro — deixaria passar um defeito real na esfera.
    /// *A barra certa não é mais alta nem mais baixa: é sobre a população certa.*
    pub inconsistent_clean: usize,
}

/// ⭐⭐⭐ **PENTEIA CADA PATCH E LÊ O SALTO DE CADA COSTURA.**
#[must_use]
pub fn comb_patches(mesh: &Mesh, layout: &PatchLayout, cut: &CutMesh) -> (Combed, CombReport) {
    let mut rep = CombReport {
        patches: cut.tris.len(),
        ..CombReport::default()
    };
    let mut out = Combed {
        dir: vec![Vec::new(); cut.tris.len()],
        holonomy: vec![None; cut.tris.len()],
        jump: vec![None; cut.seams.len()],
    };

    for (p, faces) in cut.tri_face.iter().enumerate() {
        let Some((dirs, h)) = ph2d_crossfield::comb::comb(mesh, faces, &layout.face_dir) else {
            rep.refused += 1;
            continue;
        };
        // ⛔ **A rede do alinhamento** — ver o doc deste módulo.
        if h.skipped != 0 || dirs.len() != faces.len() {
            rep.misaligned += 1;
            out.holonomy[p] = Some(h);
            continue;
        }
        if h.defects > 0 {
            rep.dirty += 1;
        }
        rep.combed += 1;
        out.holonomy[p] = Some(h);
        out.dir[p] = dirs;
    }

    let (jump, inconsistent) = read_jumps(mesh, cut, &out);
    out.jump = jump;
    rep.seams = cut.seams.len();
    rep.jumps = out.jump.iter().filter(|j| j.is_some()).count();
    rep.inconsistent = inconsistent[0];
    rep.inconsistent_clean = inconsistent[1];
    (out, rep)
}

/// ⭐ **SÓ OS SALTOS**, a partir de um penteado já feito.
///
/// ⚠️ Existe separada para o **controlo positivo** poder rodar uma moldura à mão e
/// voltar a perguntar. *Sem ela, o gate teria de confiar que o número que nunca muda
/// está certo.*
#[must_use]
pub fn jumps_only(mesh: &Mesh, cut: &CutMesh, combed: &Combed) -> Vec<Option<i32>> {
    read_jumps(mesh, cut, combed).0
}

fn read_jumps(mesh: &Mesh, cut: &CutMesh, out: &Combed) -> (Vec<Option<i32>>, [usize; 2]) {
    let mut jump: Vec<Option<i32>> = vec![None; cut.seams.len()];
    // `[todas, so' as de lados limpos]` -- ver `CombReport::inconsistent_clean`.
    let mut inconsistent = [0usize; 2];
    // Por patch, `face da malha -> índice do triângulo`, para ir buscar a direcção.
    let slot: Vec<std::collections::BTreeMap<u32, usize>> = cut
        .tri_face
        .iter()
        .map(|fs| fs.iter().enumerate().map(|(i, &f)| (f, i)).collect())
        .collect();
    // Que faces tocam cada aresta da malha.
    let mut touch: std::collections::BTreeMap<(u32, u32), Vec<u32>> =
        std::collections::BTreeMap::new();
    for (f, face) in mesh.faces().iter().enumerate() {
        let v = face.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if let Ok(f) = u32::try_from(f) {
                touch.entry((a.min(b), a.max(b))).or_default().push(f);
            }
        }
    }

    for (s, seam) in cut.seams.iter().enumerate() {
        let (pa, pb) = (seam.side[0].patch as usize, seam.side[1].patch as usize);
        if out.dir[pa].is_empty() || out.dir[pb].is_empty() {
            continue;
        }
        // ⭐ **Lê-se em TODAS as arestas da cadeia, não numa.** O salto é da costura;
        // se as arestas discordarem, é porque uma moldura não é constante ao longo dela
        // — e isso tem de aparecer, não de ser mediado.
        let mut votes: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
        for (pos, w) in seam.chain.windows(2).enumerate() {
            let e = (w[0].min(w[1]), w[0].max(w[1]));
            let Some(fs) = touch.get(&e) else {
                continue;
            };
            // ⛔⛔ **QUAL FACE É DE QUE LADO — e não é a ordem de armazenamento.**
            //
            // O salto é medido do lado `0` para o lado `1`, logo **o sinal depende de
            // qual face é qual**. Tomar `fs[0]` e `fs[1]` dá um sinal arbitrário por
            // aresta, e as arestas de uma mesma costura discordariam entre si — *um
            // desacordo que a votação esconderia em vez de acusar*.
            //
            // ⭐ A atribuição certa lê-se do próprio corte: a cópia local de `w[0]`
            // naquela face tem de ser a que a costura registou para aquele lado.
            let local_in = |f: u32, p: usize, g: u32| -> Option<u32> {
                let i = *slot[p].get(&f)?;
                let v = mesh.faces().get(f as usize)?.verts();
                let k = v.iter().position(|&x| x == g)?;
                cut.tris.get(p)?.get(i)?.get(k).copied()
            };
            let want = [
                seam.side[0].local.get(pos).copied().flatten(),
                seam.side[1].local.get(pos).copied().flatten(),
            ];
            let (Some(l0), Some(l1)) = (want[0], want[1]) else {
                continue;
            };
            let mut fa = None;
            let mut fb = None;
            for &f in fs {
                if fa.is_none() && local_in(f, pa, w[0]) == Some(l0) {
                    fa = Some(f);
                } else if fb.is_none() && local_in(f, pb, w[0]) == Some(l1) {
                    fb = Some(f);
                }
            }
            let (Some(fa), Some(fb)) = (fa, fb) else {
                continue;
            };
            let (Some(ia), Some(ib)) = (slot[pa].get(&fa), slot[pb].get(&fb)) else {
                continue;
            };
            let Some(nb) = normal_of(mesh, fb) else {
                continue;
            };
            let (da, db) = (out.dir[pa][*ia], out.dir[pb][*ib]);
            // Transporta a moldura do lado 0 para o plano do lado 1 e mede o desvio.
            let (Some(r), Some(d)) = (tangent(da, nb), tangent(db, nb)) else {
                continue;
            };
            let (c, sn) = (dot(d, r), dot(cross(nb, d), r));
            #[allow(clippy::cast_possible_truncation)]
            let k = (sn.atan2(c) / QUARTER).round() as i32;
            *votes.entry(k.rem_euclid(4)).or_default() += 1;
        }
        if votes.is_empty() {
            continue;
        }
        if votes.len() > 1 {
            inconsistent[0] += 1;
            // ⭐ Um patch com singularidade dentro nao e' penteavel; a costura dele TEM de
            // discordar. So' conta como defeito desta fase se os dois lados estao limpos.
            let clean = |p: usize| out.holonomy[p].is_some_and(|h| h.defects == 0);
            if clean(pa) && clean(pb) {
                inconsistent[1] += 1;
            }
        }
        let best = votes
            .iter()
            .max_by_key(|&(_, &n)| n)
            .map(|(&k, _)| k)
            .unwrap_or(0);
        jump[s] = Some(best);
    }

    (jump, inconsistent)
}

#[cfg(test)]
#[path = "comb_tests.rs"]
mod tests;
