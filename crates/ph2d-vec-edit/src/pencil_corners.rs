//! **As QUINAS do lápis** — onde a mão virou de propósito, o traço não arredonda.
//!
//! O ajuste de Hobby ([`crate::hobby`]) é suave em todo nó interior por construção: é isso que
//! faz as curvas bonitas, e é isso que arredondava cada quina desenhada (o smoke do dono, 09/10).
//! A cura é a do próprio MetaPost: uma quina é a CONCATENAÇÃO de dois caminhos (`p & q`) — cada
//! troço entre quinas é um ajuste de Hobby aberto, e os dois encontram-se num nó de quina com os
//! handles independentes.
//!
//! **O que é uma quina** mede-se no traço, não no nó: a direcção que CHEGA a uma amostra (a corda
//! dos `window_px` antes dela) e a que SAI (a dos `window_px` depois) viram pelo menos `min_turn_deg`,
//! e essa viragem está CONCENTRADA — vista numa janela três vezes maior, ela não cresce mais que
//! `spread` vezes. Uma quina vira tudo num ponto (a janela larga vê a mesma viragem); uma curva vira
//! aos poucos (a janela larga vê o triplo). Os números saem da sonda `measure_pencil_corners`.

use ph2d_vec_scene::VecVertex;

/// Os números do detector — um valor só, para a sonda os varrer no mesmo processo.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Corners {
    /// A janela de cada lado da amostra, px de tela.
    pub(crate) window_px: f64,
    /// A viragem mínima de uma quina (graus).
    pub(crate) min_turn_deg: f64,
    /// Viragem na janela tripla ÷ viragem na janela: até isto é quina (concentrada).
    pub(crate) spread: f64,
}

impl Corners {
    /// Os valores medidos (sonda `measure_pencil_corners`, 20 mãos × tremor ±0,5–2 px × o
    /// estabilizador por omissão): o CENTRO de um planalto em que as quinas de 45–150° com lados de
    /// 25–200 px são achadas a 100 % e círculos de 10–400 px e S não ganham nenhuma — janela 12–16,
    /// viragem 25–35°, concentração 1,2–1,4. Com janela 8 a quina arredondada pelo estabilizador
    /// já escapa (60° a 69–96 %); com concentração 2 o círculo pequeno ganha quinas.
    pub(crate) const MEASURED: Self = Self {
        window_px: 14.0,
        min_turn_deg: 30.0,
        spread: 1.3,
    };
}

/// O ajuste do traço inteiro: troços de Hobby entre as quinas, unidos por nós de quina.
pub(crate) fn fit(samples: &[[f64; 2]], tol: f64, px_to_world: f64) -> Vec<VecVertex> {
    fit_with(&Corners::MEASURED, samples, tol, px_to_world)
}

pub(crate) fn fit_with(
    c: &Corners,
    samples: &[[f64; 2]],
    tol: f64,
    px_to_world: f64,
) -> Vec<VecVertex> {
    let cuts = corners(c, samples, px_to_world);
    let mut out: Vec<VecVertex> = Vec::new();
    let mut from = 0;
    for &to in cuts.iter().chain(std::iter::once(&(samples.len() - 1))) {
        let knots = crate::pencil::decimate(&samples[from..=to], tol);
        let piece = crate::hobby::fit_hobby_open(&knots);
        match (out.last_mut(), piece.first()) {
            // A junção: a chegada é a do troço anterior, a partida a do seguinte.
            (Some(join), Some(first)) => {
                *join = VecVertex {
                    in_handle: join.in_handle,
                    out_handle: first.out_handle,
                    ..VecVertex::corner(join.anchor)
                };
                out.extend_from_slice(&piece[1..]);
            }
            _ => out.extend(piece),
        }
        from = to;
    }
    out
}

/// Os índices de `samples` onde a mão fez quina (crescentes, nunca as pontas).
pub(crate) fn corners(c: &Corners, samples: &[[f64; 2]], px_to_world: f64) -> Vec<usize> {
    let n = samples.len();
    if n < 3 {
        return Vec::new();
    }
    let mut cum = Vec::with_capacity(n);
    cum.push(0.0);
    for w in samples.windows(2) {
        cum.push(cum[cum.len() - 1] + dist(w[0], w[1]));
    }
    let total = cum[n - 1];
    let w = c.window_px * px_to_world;
    // O ponto do traço à distância `s` (interpolado), e a viragem entre as cordas de `±h`.
    let at = |s: f64| -> [f64; 2] {
        let s = s.clamp(0.0, total);
        let j = cum.partition_point(|&x| x < s).clamp(1, n - 1);
        let seg = cum[j] - cum[j - 1];
        let u = if seg > 0.0 {
            (s - cum[j - 1]) / seg
        } else {
            0.0
        };
        let (a, b) = (samples[j - 1], samples[j]);
        [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u]
    };
    let turn = |i: usize, h: f64| -> f64 {
        let p = samples[i];
        let (a, b) = (at(cum[i] - h), at(cum[i] + h));
        let (u, v) = ([p[0] - a[0], p[1] - a[1]], [b[0] - p[0], b[1] - p[1]]);
        let cross = u[0] * v[1] - u[1] * v[0];
        let dot = u[0] * v[0] + u[1] * v[1];
        cross.atan2(dot).abs()
    };
    let min_turn = c.min_turn_deg.to_radians();
    let mut cuts: Vec<(usize, f64)> = Vec::new();
    for i in 1..n - 1 {
        let t = turn(i, w);
        if t < min_turn || turn(i, 3.0 * w) > c.spread * t {
            continue;
        }
        // Uma quina só por vez: dentro da janela, fica a que vira mais.
        match cuts.last_mut() {
            Some((j, tj)) if cum[i] - cum[*j] < w => {
                if t > *tj {
                    *j = i;
                    *tj = t;
                }
            }
            _ => cuts.push((i, t)),
        }
    }
    // A janela tem de caber dos dois lados: um gancho na ponta não é quina. ⚠️ Descarta-se DEPOIS
    // do máximo local — antes, o pico do gancho saía da corrida e o vizinho, que via metade do
    // gancho na janela da frente, ganhava o lugar dele.
    cuts.into_iter()
        .filter(|&(i, _)| cum[i] >= w && total - cum[i] >= w)
        .map(|(i, _)| i)
        .collect()
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

#[cfg(test)]
#[path = "pencil_corners_tests.rs"]
mod tests;
