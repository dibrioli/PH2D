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

/// **A MÃO, COMO RESTRIÇÃO SUAVE** — para onde cada vértice deve ir, e quanto do
/// caminho ele faz.
///
/// # ⛔⛔ Por que ela não é uma FORÇA EXTERNA
///
/// Uma força externa é `F = m·a`, e a massa de um vértice cai com a área dele:
/// refinar a malha divide a massa e **não** divide a força elástica, que é um
/// campo. Medido no pincel de tecido, com o gesto entrando como aceleração:
///
/// | vértices | aresta | resposta |
/// |---|---|---|
/// | `362` | `0,196` | `151 %` do raio |
/// | `3 010` | `0,065` | `17 %` |
/// | `12 162` | `0,033` | **`4 %`** |
///
/// ⇒ **o mesmo material comportava-se de forma diferente conforme a densidade**,
/// e numa escultura de verdade (a cena do dono tem `50 000` faces) o pincel era
/// mudo. *Um material de contínuo não pode depender de como se o malha.*
///
/// ⭐⭐ A restrição resolve isto por construção: a rigidez dela é `w` vezes a
/// **escala local do próprio sistema** (inércia + elástico), então a razão entre
/// «o que a mão pede» e «o que o material resiste» é a mesma em qualquer
/// densidade.
///
/// ⛔ **E ela é um MÚLTIPLO da escala, não uma fração do CAMINHO.** A 1.ª forma
/// era `k = w/(1−w)·escala`, o que dá *«vá exatamente para onde a mão está»* em
/// `w → 1` — cinemática pura. Medido: com o gesto a arrastar o mesmo vértice
/// por vinte dabs enquanto os vizinhos ficam, a malha esticava **`9,63×`**. Com
/// `k = w·escala` a mão empata com o material em `w = 1`, e é o solver que
/// reparte — que é a definição de a mão PEDIR em vez de MANDAR.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClothDrive<'a> {
    /// Para onde cada vértice deve ir. Vazio = ninguém é conduzido.
    pub goal: &'a [V3],
    /// Quanto cada vértice sente a mão, em `[0, 1]` — a curva do pincel.
    pub weight: &'a [f64],
    /// **A RIGIDEZ DA MÃO**, na mesma unidade do módulo do material.
    ///
    /// ⛔⛔ **Ela é ABSOLUTA de propósito, e as duas alternativas foram medidas
    /// e falham cada uma de um lado:**
    ///
    /// | âncora | material decide? | independe da densidade? |
    /// |---|---|---|
    /// | o bloco inteiro (inércia + elástico) | ⛔ **não** (`1000×` mais duro dá o mesmo esticão) | ✅ sim |
    /// | só a inércia | ✅ sim (`1,67× → 1,01×`) | ⛔ **não** (`52 % → 4 %` ao refinar) |
    /// | **absoluta** | ✅ | ✅ |
    ///
    /// A razão é a escala: a Hessiana elástica por vértice é `O(μ)` e não muda
    /// com a densidade; a inércia é `O(área)` e desaparece ao refinar. Uma mão
    /// ancorada na inércia some junto; uma ancorada no elástico escala com ele e
    /// nunca perde. Uma rigidez ABSOLUTA vive na mesma escala do elástico e é
    /// independente dele — que é exatamente o que *«a mão puxa, o material
    /// resiste»* quer dizer.
    ///
    /// ⚠️ **E ela NÃO é um ganho inventado:** é um módulo, com unidade, e a
    /// calibração dela tem critério nomeado (ver o `stroke_cloth` do pincel).
    pub stiffness: f64,
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
    drive: &ClothDrive<'_>,
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

    let accel = |_i: usize| -> V3 { cfg.gravity };

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
                    // ⚠️ **A MÃO ENTRA AQUI, e a rigidez sai da ESCALA LOCAL** —
                    // ver [`ClothDrive`]. Com `w` da fração do caminho, a rigidez
                    // `k = w/(1−w)·escala` põe o mínimo do bloco exatamente a `w`
                    // do caminho entre a solução livre e a meta, em qualquer
                    // densidade de malha.
                    let aw = drive.weight.get(i).copied().unwrap_or(0.0);
                    if aw > 0.0
                        && let Some(g) = drive.goal.get(i)
                    {
                        let k = aw * drive.stiffness;
                        for r in 0..3 {
                            f[r] += k * (g[r] - state.x[i][r]);
                            hh[r][r] += k;
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
