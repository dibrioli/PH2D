//! ⭐⭐⭐ **O PORTÃO DA WAVE: as restrições «este arco é uma isolinha» são CONSISTENTES?**
//!
//! # A restrição, escrita
//!
//! O `ACHADO_ordem_das_fases` §23.14/§23.15 fixou a morada: o G3 minimiza o
//! desalinhamento do **gradiente** contra o campo e mais nada, e por isso praticamente
//! nenhum arco do layout sai uma linha de grade. A cura é uma equação por arco, imposta
//! por **eliminação**.
//!
//! Um arco vai do canto `A` ao canto `B`, na carta de um patch. Com `e` o eixo
//! **atravessado** (o transversal à direcção do arco), a exigência é
//!
//! ```text
//!     e · z_B  −  e · z_A  =  0
//! ```
//!
//! e cada cópia escreve-se `z = R^rot · y_classe + off`. ⭐⭐ Como `e·(R^rot·y)` é
//! `turn2(e, −rot) · y`, e uma rotação de quarto de volta leva um eixo a **outro eixo
//! com sinal**, a equação colapsa em
//!
//! ```text
//!     s_B · y_B[j_B]  −  s_A · y_A[j_A]  =  c
//! ```
//!
//! ⭐⭐⭐ **Dois ESCALARES, coeficientes `±1`.** É a mesma forma que a costura já elimina
//! — só que ali a variável eliminada era um 2-vector inteiro e aqui é **metade de um**.
//!
//! # ⛔ Por que este módulo mede antes de o solver existir
//!
//! Um conjunto de diferenças com sinal é consistente **se e só se** toda volta fechar. Se
//! um ciclo de arcos exigir `+1` e `−1` para o mesmo par, a eliminação é **impossível como
//! está** — e descobri-lo depois de refazer o relaxador custaria a wave inteira.
//!
//! ⚠️ **A escolha do eixo atravessado sai do mapa vivo** (o menor componente do
//! deslocamento). *Ela é uma leitura, não um dado* — e é por isso que um conflito de sinal
//! aqui pode ser um arco cuja direcção o mapa actual lê ao contrário, e não uma
//! impossibilidade. A coluna [`ArcLineSystem::ambiguous`] conta os que estão perto de
//! `45°`, onde a leitura não decide.

use crate::cut::CutMesh;
use crate::solve::{GridMap, turn2};
use crate::weld::Weld;

/// Abaixo desta razão entre o menor e o maior componente, a direcção do arco é uma
/// leitura e não um facto.
///
/// ⚠️ **`0,8` e não `1,0`:** a `45°` exactos os dois eixos são igualmente transversais, e
/// escolher um é escolher o sinal de uma equação à sorte. *Um conflito nascido daí é meu,
/// não da peça.*
pub const AMBIGUOUS_RATIO: f32 = 0.8;

/// Uma união com **sinal e deslocamento**: `y_filho = σ · y_pai + δ`.
struct Signed {
    parent: Vec<u32>,
    sign: Vec<f32>,
    off: Vec<f32>,
}

impl Signed {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..u32::try_from(n).unwrap_or(0)).collect(),
            sign: vec![1.0; n],
            off: vec![0.0; n],
        }
    }

    /// A raiz de `x`, com `(σ, δ)` acumulados: `y_x = σ · y_raiz + δ`.
    fn find(&mut self, x: u32) -> (u32, f32, f32) {
        if self.parent[x as usize] == x {
            return (x, 1.0, 0.0);
        }
        let p = self.parent[x as usize];
        let (root, sp, dp) = self.find(p);
        // `y_x = s_x·y_p + d_x` e `y_p = sp·y_raiz + dp`.
        let s = self.sign[x as usize] * sp;
        let d = self.sign[x as usize].mul_add(dp, self.off[x as usize]);
        self.parent[x as usize] = root;
        self.sign[x as usize] = s;
        self.off[x as usize] = d;
        (root, s, d)
    }
}

/// ⭐⭐⭐ **O que o portão mediu.**
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ArcLineSystem {
    /// Escalares ao todo — `2 ×` as classes da soldadura.
    pub scalars: usize,
    /// Arcos que produziram equação.
    pub arcs: usize,
    /// ⛔ Arcos saltados: sem arco, sem extremos, ou fora do mapa.
    pub skipped: usize,
    /// ⚠️ Arcos perto de `45°`, onde o eixo atravessado é uma leitura — ver
    /// [`AMBIGUOUS_RATIO`].
    pub ambiguous: usize,
    /// ⭐ Equações que ELIMINARAM um escalar.
    pub eliminated: usize,
    /// Equações que fecharam ciclo — não eliminam nada, e são as que têm de bater.
    pub cycles: usize,
    /// ⛔⛔⛔ **CONFLITOS DE SINAL**: uma volta exige `+1` e `−1` para o mesmo par.
    ///
    /// *Se isto não for zero, a eliminação é impossível como está escrita.*
    pub sign_conflicts: usize,
    /// O desacordo NUMÉRICO das voltas que fecham em sinal, em células: a mediana.
    pub offset_p50: f32,
    /// O pior.
    pub offset_max: f32,
}

/// ⭐⭐⭐ **MONTA AS EQUAÇÕES E PERGUNTA SE ELAS BATEM.**
///
/// ⚠️ Ela **não resolve nada** e não toca no mapa. *O portão de uma wave mede a premissa
/// dela; construir o solver antes é construir sobre uma premissa que ninguém leu.*
#[must_use]
pub fn measure_arc_lines(cut: &CutMesh, w: &Weld, map: &GridMap) -> ArcLineSystem {
    let n = w.classes();
    let mut out = ArcLineSystem {
        scalars: 2 * n,
        ..ArcLineSystem::default()
    };
    let mut uf = Signed::new(2 * n);
    let mut offs: Vec<f32> = Vec::new();

    for seam in &cut.seams {
        if seam.arc.is_none() {
            out.skipped += 1;
            continue;
        }
        let side = &seam.side[0];
        let p = side.patch as usize;
        let (Some(&la), Some(&lb)) = (
            side.local.iter().flatten().next(),
            side.local.iter().flatten().next_back(),
        ) else {
            out.skipped += 1;
            continue;
        };
        let (Some((ca, ra)), Some((cb, rb))) = (w.of(p, la as usize), w.of(p, lb as usize)) else {
            out.skipped += 1;
            continue;
        };
        let Some(row) = map.uv.get(p) else {
            out.skipped += 1;
            continue;
        };
        let (Some(za), Some(zb)) = (row.get(la as usize), row.get(lb as usize)) else {
            out.skipped += 1;
            continue;
        };
        // ── O eixo ATRAVESSADO: o do menor componente do deslocamento.
        let d = [zb[0] - za[0], zb[1] - za[1]];
        let (big, small) = (d[0].abs().max(d[1].abs()), d[0].abs().min(d[1].abs()));
        if big > 0.0 && small / big > AMBIGUOUS_RATIO {
            out.ambiguous += 1;
        }
        let e = if d[0].abs() >= d[1].abs() {
            [0.0, 1.0]
        } else {
            [1.0, 0.0]
        };
        // ⭐ `e·(R^rot·y) = turn2(e, −rot)·y`, e o resultado é um eixo COM SINAL.
        let axis = |rot: i32| -> (usize, f32) {
            let v = turn2(e, -rot);
            if v[0].abs() > v[1].abs() {
                (0, v[0])
            } else {
                (1, v[1])
            }
        };
        let (ja, sa) = axis(ra);
        let (jb, sb) = axis(rb);
        // `off = z − R^rot·y` — lido do mapa vivo, que é o que esta sonda tem.
        let ya = w.value_pub(map, ca);
        let yb = w.value_pub(map, cb);
        let oa = {
            let r = turn2(ya, ra);
            [za[0] - r[0], za[1] - r[1]]
        };
        let ob = {
            let r = turn2(yb, rb);
            [zb[0] - r[0], zb[1] - r[1]]
        };
        let c = (e[0] * oa[0] + e[1] * oa[1]) - (e[0] * ob[0] + e[1] * ob[1]);

        // `Y_B = σ·Y_A + δ`, com `σ = s_A·s_B` e `δ = c·s_B`.
        let sigma = sa * sb;
        let delta = c * sb;
        #[allow(clippy::cast_possible_truncation)]
        let (ia, ib) = ((2 * ca + ja) as u32, (2 * cb + jb) as u32);
        out.arcs += 1;

        let (root_a, s_a, d_a) = uf.find(ia);
        let (root_b, s_b, d_b) = uf.find(ib);
        if root_a == root_b {
            out.cycles += 1;
            // `Y_A = s_a·Y_r + d_a`, `Y_B = s_b·Y_r + d_b`; a equação exige
            // `Y_B = σ·Y_A + δ`.
            if (s_b - sigma * s_a).abs() > 1.0e-3 {
                out.sign_conflicts += 1;
            } else {
                offs.push((d_b - sigma.mul_add(d_a, delta)).abs());
            }
            continue;
        }
        out.eliminated += 1;
        // Pendurar `root_b` em `root_a`: de `Y_B = σ·Y_A + δ` sai
        // `Y_r_b = (σ·s_a/s_b)·Y_r_a + (σ·d_a + δ − d_b)/s_b`.
        uf.parent[root_b as usize] = root_a;
        uf.sign[root_b as usize] = sigma * s_a / s_b;
        uf.off[root_b as usize] = (sigma.mul_add(d_a, delta) - d_b) / s_b;
    }

    offs.sort_by(f32::total_cmp);
    out.offset_p50 = offs.get(offs.len() / 2).copied().unwrap_or(0.0);
    out.offset_max = offs.last().copied().unwrap_or(0.0);
    out
}

#[cfg(test)]
#[path = "arcline_tests.rs"]
mod tests;
