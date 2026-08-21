//! **O LEQUE DE UM PATCH** — onde ficam o centro, os raios e as grades.
//!
//! # A grade de um quad do leque
//!
//! O quad entre os lados `i` e `i+1` tem quatro cantos: o corte do lado `i`, o
//! canto do patch entre os dois lados, o corte do lado `i+1`, e o **centro**. Os
//! seus quatro lados medem `e_{i+1}`, `e_i`, `e_{i+1}` e `e_i` — opostos iguais,
//! que é o que faz dele uma grade e não um remendo.
//!
//! ⚠️ **O interior sai por COONS, não por média dos cantos.** A média ignora a
//! forma dos quatro bordos: numa quina do patch ela puxa a grade para dentro e as
//! linhas cruzam-se. Coons interpola as **curvas**, e por isso respeita um bordo
//! que curva — que é o caso normal, porque os bordos são cadeias sobre a
//! superfície.

/// **A GRADE de um quad do leque, por interpolação transfinita (Coons).**
///
/// Recebe os quatro bordos já amostrados e devolve `(s+1) × (t+1)` posições, com
/// `grid[k][l]`, `k` ao longo de `bottom`/`top` e `l` ao longo de `left`/`right`.
///
/// - `bottom[k]`, `k = 0..=s` — de `(0,0)` a `(s,0)`
/// - `top[k]`, `k = 0..=s` — de `(0,t)` a `(s,t)`
/// - `left[l]`, `l = 0..=t` — de `(0,0)` a `(0,t)`
/// - `right[l]`, `l = 0..=t` — de `(s,0)` a `(s,t)`
///
/// ⚠️ **Os cantos têm de bater**: `bottom[0] == left[0]`, e por aí. Quem os passa
/// trocados recebe uma grade que parece plausível e tem os bordos torcidos.
#[must_use]
pub fn coons(
    bottom: &[[f32; 3]],
    top: &[[f32; 3]],
    left: &[[f32; 3]],
    right: &[[f32; 3]],
) -> Vec<Vec<[f32; 3]>> {
    let s = bottom.len().saturating_sub(1);
    let t = left.len().saturating_sub(1);
    let mut grid = vec![vec![[0.0f32; 3]; t + 1]; s + 1];
    for (k, row) in grid.iter_mut().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let u = if s == 0 { 0.0 } else { k as f32 / s as f32 };
        for (l, cell) in row.iter_mut().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let w = if t == 0 { 0.0 } else { l as f32 / t as f32 };
            for c in 0..3 {
                // As duas réguas lineares, menos a bilinear dos cantos — é isso
                // que faz os quatro bordos serem respeitados **exatamente**.
                let along = (1.0 - w).mul_add(bottom[k][c], w * top[k][c]);
                let across = (1.0 - u).mul_add(left[l][c], u * right[l][c]);
                let corners = (1.0 - u) * (1.0 - w) * bottom[0][c]
                    + u * (1.0 - w) * bottom[s][c]
                    + (1.0 - u) * w * top[0][c]
                    + u * w * top[s][c];
                cell[c] = along + across - corners;
            }
        }
    }
    grid
}

/// **AMOSTRA uma polilinha em `n` segmentos IGUAIS em comprimento de arco.**
///
/// ⚠️ **Igual em comprimento, não em número de vértices.** Uma cadeia de malha
/// tem arestas de tamanhos diferentes; dividir por contagem põe os pontos onde a
/// malha por acaso é densa, e a grade herda a densidade da triangulação em vez de
/// a do alvo. *O que se quer amostrar é a CURVA, não a lista.*
#[must_use]
pub fn resample(chain: &[[f32; 3]], n: usize) -> Vec<[f32; 3]> {
    if chain.is_empty() {
        return Vec::new();
    }
    if n == 0 || chain.len() == 1 {
        return vec![chain[0]];
    }
    let mut cum = Vec::with_capacity(chain.len());
    let mut total = 0.0f32;
    cum.push(0.0f32);
    for w in chain.windows(2) {
        total += dist(w[0], w[1]);
        cum.push(total);
    }
    if total <= 0.0 {
        return vec![chain[0]; n + 1];
    }
    let mut out = Vec::with_capacity(n + 1);
    let mut seg = 0usize;
    for k in 0..=n {
        #[allow(clippy::cast_precision_loss)]
        let want = total * (k as f32) / (n as f32);
        while seg + 2 < chain.len() && cum[seg + 1] < want {
            seg += 1;
        }
        let (a, b) = (cum[seg], cum[seg + 1]);
        let f = if b > a { (want - a) / (b - a) } else { 0.0 };
        let (p, q) = (chain[seg], chain[seg + 1]);
        out.push([
            f.mul_add(q[0] - p[0], p[0]),
            f.mul_add(q[1] - p[1], p[1]),
            f.mul_add(q[2] - p[2], p[2]),
        ]);
    }
    out
}

/// Interpola `n` segmentos entre dois pontos.
#[must_use]
pub fn segment(a: [f32; 3], b: [f32; 3], n: usize) -> Vec<[f32; 3]> {
    (0..=n)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let f = if n == 0 { 0.0 } else { k as f32 / n as f32 };
            [
                f.mul_add(b[0] - a[0], a[0]),
                f.mul_add(b[1] - a[1], a[1]),
                f.mul_add(b[2] - a[2], a[2]),
            ]
        })
        .collect()
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}
