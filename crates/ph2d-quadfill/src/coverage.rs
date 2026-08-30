//! ⭐⭐⭐ **A COBERTURA — quanto da ESCULTURA a retopologia deixou para trás.**
//!
//! ⛔⛔⛔ **É a régua que NINGUÉM tem, e a ausência dela está medida dos dois lados.** As réguas
//! desta linha — e as do padrão-ouro contra o qual ela se mede — olham todas a **saída**:
//! topologia (`χ`, bordo, não-manifold, componentes) e forma por face (aspecto, enviesamento,
//! planaridade, torção). ⛔ **Nenhuma mede distância à ENTRADA.** *Uma ponta comida sai fechada,
//! com quads bonitos, e passa em tudo.*
//!
//! # ⚠️ A DIREÇÃO é a lei inteira
//!
//! Há duas distâncias entre duas malhas, e **só uma delas vê uma amputação**:
//!
//! | direcção | o que responde | numa ponta amputada |
//! |---|---|---|
//! | **saída → entrada** | *«a malha nova está pousada na escultura?»* | ⛔ **`0` — passa** |
//! | ⭐ **entrada → saída** | *«a escultura toda foi coberta?»* | ⭐ **grande — acusa** |
//!
//! ⚠️ **Eu medi a direcção errada primeiro, em 2026-08-30**, sobre a peça que o artista
//! fotografou: a saída estava a `≤ 3,4 ×` a aresta de entrada da escultura em toda a parte, e a
//! leitura foi *«não há faces fora do lugar»* — verdade, e irrelevante. *O que se perdeu não
//! estava na saída; estava na entrada, sem ninguém do outro lado.*
//!
//! # ⭐ E a CASCA é o que a torna acionável
//!
//! O número global não move: numa peça com doze espinhos, a `p50` da peça inteira mal reage a
//! duas pontas comidas. É a **casca exterior** ([`COVERAGE_SHELL`]) que responde — e a
//! progressão por casca, medida na peça do artista, nomeia o defeito sozinha:
//!
//! | `r / Rmax` | `Detail 0,50` (fábrica) | `Detail 0,85` + `Follow Curvature 1` |
//! |---|---|---|
//! | `[0,00 · 0,50)` | `0,44 %` | `0,17 %` |
//! | `[0,50 · 0,75)` | `0,50 %` | `0,19 %` |
//! | `[0,75 · 0,90)` | `2,72 %` | `0,21 %` |
//! | ⭐ `[0,90 · 1,00]` | ⛔ **`6,02 %`** (pior `9,46 %`) | ⭐ **`0,28 %`** (pior `3,12 %`) |
//!
//! ⇒ *o defeito é monótono no raio e vale `21×` entre as duas configurações* — e a régua chega lá
//! **sem saber o que é uma ponta**, ao contrário da [`super::tip_survival`], que tem de as achar
//! primeiro.
//!
//! # ⚠️ A distância é ao TRIÂNGULO, não ao vértice mais próximo
//!
//! Amostrar a saída por vértices (ou por vértices + pontos médios) é mais curto de escrever e
//! **sobre-estima** a falta — numa grade grossa, por até meia aresta de quad, que na peça do
//! artista é da ordem do próprio defeito que se quer medir. ⇒ ponto-a-triângulo exacto, com uma
//! grelha uniforme por caixa envolvente a pagar a busca.

use ph2d_mesh::Mesh;
use std::collections::BTreeMap;

/// **A CASCA EXTERIOR** — a fracção do raio máximo acima da qual um vértice conta como «ponta».
///
/// ⚠️ **É o mesmo `0,90` que a progressão medida na peça do artista mostra ser o degrau**
/// (`2,72 % → 6,02 %`), e ⛔ **não** o `0,55` da [`super::tip_survival`]: aquele é o piso de um
/// **ápice** (um máximo local do raio, que se procura), este é o de uma **casca** (todo vértice
/// distante, que não se procura). *Duas perguntas diferentes, dois números diferentes.*
pub const COVERAGE_SHELL: f32 = 0.90;

/// **A BARRA** — falta de cobertura, em fracção da diagonal da peça, acima da qual há defeito.
///
/// ⚠️ **Derivada da separação medida, não escolhida por conforto:** a configuração limpa dá
/// `0,28 %` na casca e a amputada dá `6,02 %`; `2 %` fica **`7×` acima** de uma e **`3×` abaixo**
/// da outra. *Uma barra que não separa as duas medições que a motivaram não é uma barra.*
pub const COVERAGE_DEFECT: f32 = 0.02;

/// O que a escultura perdeu, em fracções da **diagonal da caixa da entrada**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coverage {
    /// Vértices da entrada medidos. ⛔⛔ **`0` significa NÃO MEDIDO**, nunca «perfeito».
    ///
    /// ⚠️ *Um zero de «não medido» e um de «perfeito» são o mesmo byte* — esta linha pagou-o
    /// três vezes. Quem imprime esta estrutura confere **esta** contagem antes dos valores.
    pub samples: usize,
    /// Mediana da falta, sobre a peça inteira.
    pub p50: f32,
    /// Percentil 95.
    pub p95: f32,
    /// A pior falta da peça.
    pub worst: f32,
    /// Vértices da entrada na casca exterior ([`COVERAGE_SHELL`]). `0` = NÃO MEDIDO.
    pub shell_samples: usize,
    /// Mediana da falta **na casca** — a coluna que reage a uma ponta comida.
    pub shell_p50: f32,
    /// A pior falta na casca.
    pub shell_worst: f32,
}

impl Coverage {
    /// **A MEDIÇÃO ACONTECEU?** — ver [`Self::samples`].
    #[must_use]
    pub const fn measured(&self) -> bool {
        self.samples > 0
    }

    /// **A CASCA está fora da barra?** ⛔ `false` quando não houve medição — *não medido não é
    /// aprovado, mas também não é acusação.*
    #[must_use]
    pub fn shell_is_defective(&self) -> bool {
        self.shell_samples > 0 && self.shell_p50 > COVERAGE_DEFECT
    }
}

/// ⭐⭐⭐ **Para cada vértice da ENTRADA, a distância à superfície da SAÍDA.**
///
/// Devolve fracções da diagonal da caixa da entrada. ⛔ Entrada vazia, saída vazia ou peça
/// degenerada devolvem [`Coverage::samples`] `== 0` — ver o campo.
#[must_use]
pub fn coverage(input: &Mesh, output: &Mesh) -> Coverage {
    let zero = Coverage {
        samples: 0,
        p50: 0.0,
        p95: 0.0,
        worst: 0.0,
        shell_samples: 0,
        shell_p50: 0.0,
        shell_worst: 0.0,
    };
    let pin = input.positions();
    let pout = output.positions();
    if pin.is_empty() || pout.is_empty() || output.faces().is_empty() {
        return zero;
    }
    let bounds = input.bounds();
    let span = {
        let d = [
            bounds.max[0] - bounds.min[0],
            bounds.max[1] - bounds.min[1],
            bounds.max[2] - bounds.min[2],
        ];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    // ⚠️ `is_finite` primeiro, e não `!(span > 0.0)`: as duas apanham o `NaN`, mas só esta diz
    // **quais** os dois estados recusados — e o `clippy::neg_cmp_op_on_partial_ord` tem razão
    // em pedi-lo, porque num tipo parcialmente ordenado a negação de `>` não é `<=`.
    if !span.is_finite() || span <= 0.0 {
        return zero;
    }

    // ── Os triângulos da saída (leque por face — a distância a um leque de um quad plano é a
    //    distância ao quad, e num quad torcido é a do sólido que ele de facto desenha).
    let mut tris: Vec<[[f32; 3]; 3]> = Vec::new();
    for f in output.faces() {
        let v = f.verts();
        for k in 1..v.len().saturating_sub(1) {
            tris.push([
                pout[v[0] as usize],
                pout[v[k] as usize],
                pout[v[k + 1] as usize],
            ]);
        }
    }
    if tris.is_empty() {
        return zero;
    }

    // ── A grelha: cada triângulo entra em toda célula que a caixa dele toca.
    let cell = span / 64.0;
    let key = |p: [f32; 3]| -> [i32; 3] {
        [
            (p[0] / cell).floor() as i32,
            (p[1] / cell).floor() as i32,
            (p[2] / cell).floor() as i32,
        ]
    };
    let mut grid: BTreeMap<[i32; 3], Vec<u32>> = BTreeMap::new();
    for (ti, t) in tris.iter().enumerate() {
        let lo = key([
            t[0][0].min(t[1][0]).min(t[2][0]),
            t[0][1].min(t[1][1]).min(t[2][1]),
            t[0][2].min(t[1][2]).min(t[2][2]),
        ]);
        let hi = key([
            t[0][0].max(t[1][0]).max(t[2][0]),
            t[0][1].max(t[1][1]).max(t[2][1]),
            t[0][2].max(t[1][2]).max(t[2][2]),
        ]);
        // ⚠️ Um triângulo enorme numa grelha fina encheria a memória; o tecto trata-o como
        // «grande» e ele passa a ser visitado por toda a busca. Medido: não acontece numa saída
        // de quads, e é a alternativa honesta a saltá-lo.
        let cells = (i64::from(hi[0] - lo[0]) + 1)
            * (i64::from(hi[1] - lo[1]) + 1)
            * (i64::from(hi[2] - lo[2]) + 1);
        let ti = u32::try_from(ti).unwrap_or(u32::MAX);
        if cells > 4096 {
            grid.entry([i32::MIN, i32::MIN, i32::MIN])
                .or_default()
                .push(ti);
            continue;
        }
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    grid.entry([x, y, z]).or_default().push(ti);
                }
            }
        }
    }
    let oversized: &[u32] = grid
        .get(&[i32::MIN, i32::MIN, i32::MIN])
        .map_or(&[], Vec::as_slice);

    // ── O centro e o raio máximo da entrada, para a casca.
    let n = pin.len() as f32;
    let mut centre = [0.0f32; 3];
    for q in pin {
        for k in 0..3 {
            centre[k] += q[k] / n;
        }
    }
    let radius = |q: &[f32; 3]| -> f32 {
        let d = [q[0] - centre[0], q[1] - centre[1], q[2] - centre[2]];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    let rmax = pin.iter().fold(0.0f32, |acc, q| acc.max(radius(q)));

    let mut all: Vec<f32> = Vec::with_capacity(pin.len());
    let mut shell: Vec<f32> = Vec::new();
    for q in pin {
        let mut best = f32::INFINITY;
        for &ti in oversized {
            best = best.min(point_tri_sq(*q, &tris[ti as usize]));
        }
        let k = key(*q);
        let mut r = 0i32;
        loop {
            for x in (k[0] - r)..=(k[0] + r) {
                for y in (k[1] - r)..=(k[1] + r) {
                    for z in (k[2] - r)..=(k[2] + r) {
                        if r > 0
                            && (x - k[0]).abs() != r
                            && (y - k[1]).abs() != r
                            && (z - k[2]).abs() != r
                        {
                            continue;
                        }
                        for &ti in grid.get(&[x, y, z]).map_or(&[][..], Vec::as_slice) {
                            best = best.min(point_tri_sq(*q, &tris[ti as usize]));
                        }
                    }
                }
            }
            // ⚠️ A garantia: tudo o que está no anel seguinte dista pelo menos `r · cell`.
            let reach = cell * r as f32;
            if (best.sqrt() <= reach && best.is_finite()) || r > 256 {
                break;
            }
            r += 1;
        }
        let d = if best.is_finite() {
            best.sqrt() / span
        } else {
            1.0
        };
        all.push(d);
        if rmax > 0.0 && radius(q) >= COVERAGE_SHELL * rmax {
            shell.push(d);
        }
    }
    all.sort_by(f32::total_cmp);
    shell.sort_by(f32::total_cmp);
    let pick = |v: &[f32], p: f32| -> f32 {
        if v.is_empty() {
            return 0.0;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "indice de percentil sobre um vector nao vazio, fechado no comprimento"
        )]
        let i = ((p * v.len() as f32) as usize).min(v.len() - 1);
        v[i]
    };
    Coverage {
        samples: all.len(),
        p50: pick(&all, 0.5),
        p95: pick(&all, 0.95),
        worst: all.last().copied().unwrap_or(0.0),
        shell_samples: shell.len(),
        shell_p50: pick(&shell, 0.5),
        shell_worst: shell.last().copied().unwrap_or(0.0),
    }
}

/// Distância ao QUADRADO de um ponto ao triângulo — o caso fechado, sem iteração.
fn point_tri_sq(p: [f32; 3], t: &[[f32; 3]; 3]) -> f32 {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]));
    let (a, b, c) = (t[0], t[1], t[2]);
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return dot(ap, ap);
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return dot(bp, bp);
    }
    let along = |base: [f32; 3], dir: [f32; 3], s: f32| {
        let q = [
            dir[0].mul_add(s, base[0]),
            dir[1].mul_add(s, base[1]),
            dir[2].mul_add(s, base[2]),
        ];
        let d = sub(p, q);
        dot(d, d)
    };
    let vc = d1.mul_add(d4, -(d3 * d2));
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return along(a, ab, d1 / (d1 - d3));
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return dot(cp, cp);
    }
    let vb = d5.mul_add(d2, -(d1 * d6));
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return along(a, ac, d2 / (d2 - d6));
    }
    let va = d3.mul_add(d6, -(d5 * d4));
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return along(b, sub(c, b), (d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    let (v, w) = (vb * denom, vc * denom);
    let q = [
        ac[0].mul_add(w, ab[0].mul_add(v, a[0])),
        ac[1].mul_add(w, ab[1].mul_add(v, a[1])),
        ac[2].mul_add(w, ab[2].mul_add(v, a[2])),
    ];
    let d = sub(p, q);
    dot(d, d)
}

#[cfg(test)]
#[path = "coverage_tests.rs"]
mod tests;
