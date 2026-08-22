//! **O PREENCHIMENTO DE UM PATCH** — o domínio, os quatro bordos e a grade.
//!
//! Irmão da [`crate::stitch`], e o corte foi forçado pelo teto de LOC (817 contra
//! 700). ⭐ **Mas ele é de ASSUNTO:** lá mora *a montagem inteira* — a amostragem
//! partilhada dos arcos, a orientação, o alisamento e o relatório; aqui, *como um
//! patch vira quads*.
//!
//! ⚠️ **A lei que governa este ficheiro:** a grade é construída **no domínio** do
//! patch achatado ([`crate::param`]) e volta pela triangulação. A interpolação em
//! `ℝ³` continua aqui, e é só o caminho de recurso para um patch que não achate.

use ph2d_mesh::{Face, Mesh};
use ph2d_quantize::Quantization;
use ph2d_trace::PatchLayout;

use crate::fan::coons;
use crate::param::{PatchParam, corners_for};
use crate::report::{Points, Provenance};

/// **PREENCHE UM PATCH DE QUATRO LADOS COM UMA GRADE PLANA.**
///
/// ⭐⭐ **O leque é a construção errada para um retângulo, e o preço estava
/// medido antes de eu o ver.** Num patch de `n` lados o leque põe um vértice
/// **central** e `n` sub-grades em volta dele; para `n = 4` isso é: um vértice a
/// mais que não precisa de existir, quatro raios que são **cordas retas** a
/// atravessar a forma, e três costuras interiores onde não há fronteira nenhuma.
/// *E é entre os gomos do leque que a face dobra.*
///
/// Aqui a lei do F4 é lida ao contrário: num patch de quatro lados ela obriga
/// `L₀ = e₃ + e₁ = L₂` e `L₁ = e₀ + e₂ = L₃` — ⭐ **os lados opostos têm o mesmo
/// número de segmentos, por construção**. Logo o patch É uma grade `L₀ × L₁`, e
/// ela sai de **uma** interpolação de Coons sobre os quatro lados verdadeiros.
///
/// ⚠️ **Zero vértices novos de fronteira**: os quatro lados são os arcos que os
/// vizinhos também usam, então a costura continua a ser a mesma. O que desaparece
/// é só o interior inventado.
pub(crate) fn fill_rectangle(
    pts: &mut Points,
    faces: &mut Vec<Face>,
    surface: &Mesh,
    seed: f32,
    side_pts: &[Vec<u32>],
    dom: &Domain<'_>,
) {
    // O contorno: `bottom` e `top` opostos, `left` e `right` opostos. O lado 2
    // corre ao contrário do 0 (a fronteira roda), e o 3 ao contrário do 1.
    let bottom = side_pts[0].clone();
    let right = side_pts[1].clone();
    let top: Vec<u32> = side_pts[2].iter().rev().copied().collect();
    let left: Vec<u32> = side_pts[3].iter().rev().copied().collect();
    let (s, t) = (bottom.len() - 1, right.len() - 1);
    debug_assert_eq!(
        top.len() - 1,
        s,
        "a lei do F4 obriga os lados opostos a bater"
    );
    debug_assert_eq!(left.len() - 1, t, "idem para o outro par");
    let uv_bottom = dom.side(0);
    let uv_right = dom.side(1);
    let uv_top = dom.rev(2);
    let uv_left = dom.rev(3);
    let grid = build_grid(
        pts,
        surface,
        seed,
        Chains {
            bottom: &bottom,
            top: &top,
            left: &left,
            right: &right,
        },
        Chains2 {
            bottom: &uv_bottom,
            top: &uv_top,
            left: &uv_left,
            right: &uv_right,
        },
        (s, t),
        dom,
    );
    for k in 0..s {
        for l in 0..t {
            faces.push(Face::quad(
                grid[k][l],
                grid[k + 1][l],
                grid[k + 1][l + 1],
                grid[k][l + 1],
            ));
        }
    }
}

/// **O DOMÍNIO DE UM PATCH** — o achatamento dele, e o `uv` de cada ponto de saída
/// que já está na fronteira.
///
/// ⭐ **Ele existe para que a grade seja construída no DOMÍNIO e não no espaço.**
/// Ver [`crate::param`]: um ponto interpolado em `ℝ³` não está na peça, e agarrá-lo
/// à face mais próxima é o que dobra a malha sobre um patch grande e curvo.
///
/// ⚠️ **`param: None` é uma resposta legítima** — um patch que não achata volta à
/// construção antiga, que dobra mas devolve malha. O [`FillReport::flattened`] conta
/// quantos achataram, e é ele que diz se esta cura chegou ao patch que dói.
pub(crate) struct Domain<'a> {
    pub(crate) param: Option<&'a PatchParam>,
    /// Por lado, o `uv` de cada ponto de saída daquele lado, na ordem do lado.
    pub(crate) side_uv: Vec<Vec<[f32; 2]>>,
    /// Quantos pontos o domínio colocou, e quantos caíram fora dele.
    pub(crate) tally: std::cell::Cell<(usize, usize)>,
}

impl Domain<'_> {
    pub(crate) fn side(&self, i: usize) -> Vec<[f32; 2]> {
        self.side_uv[i].clone()
    }

    pub(crate) fn rev(&self, i: usize) -> Vec<[f32; 2]> {
        self.side_uv[i].iter().rev().copied().collect()
    }

    /// **O ponto 3D de um `uv` do interior** — pelo achatamento, ou pelo caminho
    /// antigo quando ele não existe.
    ///
    /// ⚠️ **A reprojeção sobre `surface` FICA nos dois caminhos**, e não é
    /// redundante: o achatamento aterra o ponto na malha **remalhada** (é ela que o
    /// layout indexa), e a forma que o artista esculpiu vive na original. O que
    /// desapareceu foi a projeção como forma de *corrigir a construção*.
    pub(crate) fn place(
        &self,
        pts: &mut Points,
        surface: &Mesh,
        seed: f32,
        uv: [f32; 2],
        fallback: [f32; 3],
        prov: Provenance,
    ) -> u32 {
        let hit = self.param.and_then(|q| q.sample(uv));
        let (ok, miss) = self.tally.get();
        self.tally.set(if hit.is_some() {
            (ok + 1, miss)
        } else {
            (ok, miss + 1)
        });
        pts.push_on(surface, hit.unwrap_or(fallback), seed, prov)
    }
}

/// **O `uv` DE CADA PONTO DE SAÍDA DE UM LADO**, na aresta do polígono.
///
/// ⭐⭐ **Por COMPRIMENTO DE ARCO, e é isso que faz a costura fechar.** O F5
/// amostra cada arco em segmentos iguais em comprimento ([`crate::fan::resample`])
/// e o achatamento põe os vértices de malha da fronteira na aresta do polígono pela
/// **mesma** fração de comprimento ([`crate::param`]) — então o `uv` que esta função
/// calcula para um ponto de saída cai exactamente entre os `uv` dos vértices de
/// malha vizinhos. *Duas leis diferentes aqui dariam um ponto de fronteira cujo
/// domínio discorda do da triangulação, e a grade nasceria torcida junto ao bordo.*
///
/// ⚠️ **O ponto de junção entre dois arcos aparece uma vez só** — é a mesma regra
/// da concatenação dos `side_pts`, e sem ela o lado teria um ponto a mais que a lei
/// do F4 conta.
pub(crate) fn side_uv(
    layout: &PatchLayout,
    quant: &Quantization,
    sides: &[Vec<(u32, bool)>],
    i: usize,
) -> Vec<[f32; 2]> {
    let n = sides.len();
    // ⭐ **A MESMA fonte de cantos que o achatamento** — ver [`corners_for`].
    let poly = corners_for(n);
    let (a, b) = (poly[i], poly[(i + 1) % n]);
    let side = &sides[i];
    // ⚠️ **Pelo `τ` e não pelo comprimento**, e não é detalhe: o `uv` de um ponto
    // de saída tem de cair entre os `uv` dos vértices de malha à volta dele, e o
    // achatamento distribui a fronteira por... comprimento. Ver o aviso abaixo.
    let tau_of = |x: u32| {
        layout
            .arc_tau
            .get(x as usize)
            .and_then(|t| t.last().copied())
            .unwrap_or(0.0)
    };
    let total: f32 = side.iter().map(|&(x, _)| tau_of(x)).sum();
    let mut out: Vec<[f32; 2]> = Vec::new();
    let mut run = 0.0f32;
    for &(x, _) in side {
        let len = tau_of(x);
        let steps = quant.arc[x as usize].max(1) as usize;
        for k in 0..=steps {
            if k == 0 && !out.is_empty() {
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let along = run + len * (k as f32) / (steps as f32);
            let f = if total > 0.0 { along / total } else { 0.0 };
            out.push([f.mul_add(b[0] - a[0], a[0]), f.mul_add(b[1] - a[1], a[1])]);
        }
        run += len;
    }
    out
}

/// Os quatro bordos de uma grade, em índices de ponto de saída.
pub(crate) struct Chains<'a> {
    pub(crate) bottom: &'a [u32],
    pub(crate) top: &'a [u32],
    pub(crate) left: &'a [u32],
    pub(crate) right: &'a [u32],
}

/// Os mesmos quatro bordos, no domínio.
pub(crate) struct Chains2<'a> {
    pub(crate) bottom: &'a [[f32; 2]],
    pub(crate) top: &'a [[f32; 2]],
    pub(crate) left: &'a [[f32; 2]],
    pub(crate) right: &'a [[f32; 2]],
}

/// Os índices da grade: bordos vindos das curvas, interior por Coons.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_grid(
    pts: &mut Points,
    surface: &Mesh,
    seed: f32,
    ids: Chains<'_>,
    uv: Chains2<'_>,
    (s, t): (usize, usize),
    dom: &Domain<'_>,
) -> Vec<Vec<u32>> {
    let at = |c: &[u32], k: usize, pos: &[[f32; 3]]| pos[c[k] as usize];
    let b: Vec<[f32; 3]> = (0..=s).map(|k| at(ids.bottom, k, &pts.pos)).collect();
    let tp: Vec<[f32; 3]> = (0..=s).map(|k| at(ids.top, k, &pts.pos)).collect();
    let l: Vec<[f32; 3]> = (0..=t).map(|k| at(ids.left, k, &pts.pos)).collect();
    let r: Vec<[f32; 3]> = (0..=t).map(|k| at(ids.right, k, &pts.pos)).collect();
    // ⭐⭐ **DUAS interpolações da MESMA lei, e é de propósito.** A do domínio diz
    // *onde na superfície*; a do espaço é o que sobra quando o patch não achata. Um
    // ponto que o achatamento aceita nunca usa a segunda.
    let inner = coons(&b, &tp, &l, &r);
    let inner_uv = coons(uv.bottom, uv.top, uv.left, uv.right);
    let mut grid = vec![vec![0u32; t + 1]; s + 1];
    for (k, row) in grid.iter_mut().enumerate() {
        for (l_i, cell) in row.iter_mut().enumerate() {
            *cell = if l_i == 0 {
                ids.bottom[k]
            } else if l_i == t {
                ids.top[k]
            } else if k == 0 {
                ids.left[l_i]
            } else if k == s {
                ids.right[l_i]
            } else {
                dom.place(
                    pts,
                    surface,
                    seed,
                    inner_uv[k][l_i],
                    inner[k][l_i],
                    Provenance::Grid,
                )
            };
        }
    }
    grid
}
