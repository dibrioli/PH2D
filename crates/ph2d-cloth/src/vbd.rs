//! **O PASSO** — Vertex Block Descent: um Newton `3×3` por vértice, Gauss-Seidel
//! por cor.
//!
//! ⚠️ **A ordem das três coisas é a lei, e cada uma existe por um motivo medido:**
//! a **inicialização adaptativa** (o vértice parado FICA parado), a varredura por
//! **cor** (ninguém lê o que o vizinho está a escrever), e o **salto** quando o
//! bloco é singular (um vértice degenerado não pode virar `NaN` e alastrar).

use crate::{
    ClothMaterial, ClothRest, ClothTopology, V3, add, bending, dot, membrane, norm, scale, sub,
};

/// **O ESTADO VIVO do tecido** — o que sobrevive de um evento de ponteiro para o
/// seguinte, dentro do mesmo traço.
///
/// ⚠️⚠️ **É a primeira coisa neste módulo que sobrevive ao evento**, e é isso que
/// separa o Cloth dos 23 verbos anteriores: eles respondem `alvo = f(pre, dab)` e
/// são função pura do gesto. Aqui o resultado do evento *N* é a entrada do *N+1* —
/// e a diferença entre isso ser a feature e ser o defeito que a W9a curou é o
/// **relógio**: ali não havia nenhum, aqui há sub-passos determinísticos.
#[derive(Clone, Debug, Default)]
pub struct ClothState {
    /// As posições.
    pub x: Vec<V3>,
    /// As velocidades.
    pub v: Vec<V3>,
    /// A aceleração realizada no passo anterior — só a inicialização adaptativa a lê.
    a_prev: Vec<V3>,
}

impl ClothState {
    /// Nasce em repouso, na pose em que o traço começou.
    #[must_use]
    pub fn at_rest(x: &[V3]) -> Self {
        Self {
            x: x.to_vec(),
            v: vec![[0.0; 3]; x.len()],
            a_prev: vec![[0.0; 3]; x.len()],
        }
    }
}

/// **O ORÇAMENTO** — e ele é do relógio, não do material.
///
/// ⚠️ **`substeps` e `iterations` são as duas metades do mesmo teto, e não são
/// intercambiáveis.** *Small Steps* (Macklin et al. 2019) mede que `n` sub-passos
/// de uma iteração batem um passo de `n` iterações; o VBD mantém a estabilidade
/// nos dois. ⇒ o default gasta o orçamento em **sub-passos**, e as iterações
/// existem para o caso rígido.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepConfig {
    /// O passo de tempo do evento inteiro.
    pub dt: f64,
    /// Em quantos sub-passos ele é partido.
    pub substeps: u32,
    /// Iterações de VBD por sub-passo.
    pub iterations: u32,
    /// Aceleração de campo (gravidade), igual para todo vértice.
    pub gravity: V3,
}

impl Default for StepConfig {
    fn default() -> Self {
        Self {
            dt: 1.0 / 60.0,
            substeps: 4,
            iterations: 1,
            gravity: [0.0; 3],
        }
    }
}

/// Resolve `H·Δ = f` para um bloco `3×3`, ou devolve `None` se ele for singular.
///
/// ⚠️ **`None` significa «este vértice não se move nesta iteração», e não «erro».**
/// É o tratamento do próprio VBD: a Hessiana da inércia é sempre de posto cheio,
/// então o caso é raro — e saltar é a única resposta finita. *Um `normalize` de
/// vetor nulo poria `NaN` no alvo, e a recomputação de normais o alastraria à
/// malha inteira: um vértice degenerado apagaria a peça.*
fn solve3(h: &[[f64; 3]; 3], f: V3) -> Option<V3> {
    let c0 = h[1][1] * h[2][2] - h[1][2] * h[2][1];
    let c1 = h[1][2] * h[2][0] - h[1][0] * h[2][2];
    let c2 = h[1][0] * h[2][1] - h[1][1] * h[2][0];
    let det = h[0][0] * c0 + h[0][1] * c1 + h[0][2] * c2;
    // A escala do bloco: sem ela o limiar seria absoluto, e um material rígido
    // (Hessiana grande) passaria enquanto um mole (Hessiana pequena) seria
    // rejeitado por ser pequeno, não por ser singular.
    let s = h.iter().flatten().fold(0.0f64, |m, v| m.max(v.abs()));
    if !det.is_finite() || det.abs() <= 1e-12 * s.max(1e-30).powi(3) {
        return None;
    }
    let inv = 1.0 / det;
    let m = [
        [
            c0 * inv,
            (h[0][2] * h[2][1] - h[0][1] * h[2][2]) * inv,
            (h[0][1] * h[1][2] - h[0][2] * h[1][1]) * inv,
        ],
        [
            c1 * inv,
            (h[0][0] * h[2][2] - h[0][2] * h[2][0]) * inv,
            (h[0][2] * h[1][0] - h[0][0] * h[1][2]) * inv,
        ],
        [
            c2 * inv,
            (h[0][1] * h[2][0] - h[0][0] * h[2][1]) * inv,
            (h[0][0] * h[1][1] - h[0][1] * h[1][0]) * inv,
        ],
    ];
    let d = [
        m[0][0] * f[0] + m[0][1] * f[1] + m[0][2] * f[2],
        m[1][0] * f[0] + m[1][1] * f[1] + m[1][2] * f[2],
        m[2][0] * f[0] + m[2][1] * f[1] + m[2][2] * f[2],
    ];
    d.iter().all(|c| c.is_finite()).then_some(d)
}

/// A força e a Hessiana ELÁSTICAS num vértice — a soma sobre os elementos
/// incidentes, que é a linha do meio da fórmula do VBD.
fn elastic(
    topo: &ClothTopology,
    rest: &ClothRest,
    mat: &ClothMaterial,
    x: &[V3],
    i: usize,
    lame: (f64, f64),
) -> (V3, [[f64; 3]; 3]) {
    let (mut f, mut h) = ([0.0f64; 3], [[0.0f64; 3]; 3]);
    let mut take = |g: V3, hh: [[f64; 3]; 3]| {
        for r in 0..3 {
            f[r] -= g[r];
            for c in 0..3 {
                h[r][c] += hh[r][c];
            }
        }
    };
    let iu = u32::try_from(i).unwrap_or(u32::MAX);
    for t in topo.tri_of.of(i) {
        let tri = topo.tris[*t as usize];
        let Some(slot) = tri.iter().position(|v| *v == iu) else {
            continue;
        };
        let (g, hh) = membrane::accumulate(x, tri, &rest.tri[*t as usize], lame.0, lame.1, slot);
        take(g, hh);
    }
    for p in topo.hinge_of.of(i) {
        let (hi, slot) = ((p / 4) as usize, (p % 4) as usize);
        let (g, hh) = bending::accumulate(x, topo.hinges[hi], &rest.hinge[hi], mat.bending, slot);
        take(g, hh);
    }
    (f, h)
}

/// **UM PASSO** do tecido, do evento inteiro.
///
/// `ext` é aceleração externa **por vértice** (a força do pincel dividida pela
/// massa); vazio quer dizer nenhuma. `pinned` é o anel de falloff — e pregado aqui
/// não é uma mola forte, é o vértice **não ser atualizado**: massa infinita de
/// verdade, sem termo de penalidade e sem constante para afinar.
pub fn step(
    topo: &ClothTopology,
    rest: &ClothRest,
    mat: &ClothMaterial,
    pinned: &[bool],
    ext: &[V3],
    cfg: &StepConfig,
    state: &mut ClothState,
) {
    let n = topo.verts;
    if state.x.len() != n || cfg.substeps == 0 || !cfg.dt.is_finite() || cfg.dt <= 0.0 {
        return;
    }
    let lame = mat.lame();
    let h = cfg.dt / f64::from(cfg.substeps);
    let inv_h2 = 1.0 / (h * h);
    let kd = mat.damping.max(0.0) / h;

    let accel = |i: usize| -> V3 {
        let e = ext.get(i).copied().unwrap_or([0.0; 3]);
        add(cfg.gravity, e)
    };

    for _ in 0..cfg.substeps {
        let x_t = state.x.clone();
        let v_t = state.v.clone();

        // ── inicialização adaptativa ────────────────────────────────────────
        // ⚠️ **A fração `ã` é presa em `[0,1]` e sai da aceleração REALIZADA no
        // passo anterior, projetada na direção da externa.** Ela inclui a
        // gravidade na previsão quando o movimento se parece com queda livre, e
        // **mantém a posição quando o corpo está parado** — que é o que evita
        // esticar e penetrar numa solução parcialmente convergida.
        let mut y = vec![[0.0f64; 3]; n];
        for i in 0..n {
            let a = accel(i);
            y[i] = add(add(x_t[i], scale(v_t[i], h)), scale(a, h * h));
            if pinned.get(i).copied().unwrap_or(false) {
                continue;
            }
            let la = norm(a);
            let frac = if la > 1e-12 {
                (dot(state.a_prev[i], a) / (la * la)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            state.x[i] = add(add(x_t[i], scale(v_t[i], h)), scale(a, frac * h * h));
        }

        // ── a varredura ─────────────────────────────────────────────────────
        for _ in 0..cfg.iterations {
            for bin in &topo.bins {
                for vi in bin {
                    let i = *vi as usize;
                    if pinned.get(i).copied().unwrap_or(false) {
                        continue;
                    }
                    let m = rest.mass[i];
                    if m <= 0.0 || !m.is_finite() {
                        continue;
                    }
                    let (fe, he) = elastic(topo, rest, mat, &state.x, i, lame);
                    let mi = m * inv_h2;
                    let d = sub(state.x[i], y[i]);
                    let dt_x = sub(state.x[i], x_t[i]);
                    let mut f = [0.0f64; 3];
                    let mut hh = [[0.0f64; 3]; 3];
                    for r in 0..3 {
                        // inércia + elástico + a metade de FORÇA do Rayleigh
                        f[r] = -mi * d[r] + fe[r]
                            - kd * (he[r][0] * dt_x[0] + he[r][1] * dt_x[1] + he[r][2] * dt_x[2]);
                        for c in 0..3 {
                            hh[r][c] = he[r][c] * (1.0 + kd) + if r == c { mi } else { 0.0 };
                        }
                    }
                    if let Some(dx) = solve3(&hh, f) {
                        state.x[i] = add(state.x[i], dx);
                    }
                }
            }
        }

        // ── as velocidades, e a aceleração que a próxima inicialização lê ───
        let inv = 1.0 / h;
        for i in 0..n {
            if pinned.get(i).copied().unwrap_or(false) {
                state.x[i] = x_t[i];
                state.v[i] = [0.0; 3];
                state.a_prev[i] = [0.0; 3];
                continue;
            }
            let nv = scale(sub(state.x[i], x_t[i]), inv);
            state.a_prev[i] = scale(sub(nv, v_t[i]), inv);
            state.v[i] = nv;
        }
    }
}
