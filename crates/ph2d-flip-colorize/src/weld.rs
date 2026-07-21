//! **A solda das juntas** — o buraco de quina, fechado onde ele nasce.
//!
//! A parede do raster é o **EIXO** do traço (raio 0), e essa escolha é deliberada e testada
//! (BUGS #14/#15: a cor tem de ir até o eixo para se enfiar por baixo da metade externa da
//! linha, senão a borda fica ~1 px aquém e o zoom a denuncia). Mas ela tem um preço que
//! ninguém tinha nomeado: **dois traços cujos CORPOS pintados se sobrepõem folgado na tela
//! podem ter EIXOS que não se tocam**. Numa quina desenhada à mão — 4 paredes, 4 traços — as
//! pontas ficam a 0,02-0,04 doc uma da outra enquanto o corpo tem 0,26 de largura: o artista
//! vê uma quina fechada, e o raster tem um buraco de 3,7-6,5 px por onde a cor escorre para
//! fora da moldura (auditoria 2026-07-21, item 5; report do Enio *"nada tira a extrapolação"*).
//!
//! **A regra é DERIVADA, não um knob:** se a tinta que o artista pintou cobre o vão — isto é,
//! se a distância entre as duas partes é menor que a soma das meias-larguras delas —, então a
//! parede é contínua ali, porque na tela ela é. Não há número mágico, não há slider novo, e
//! um vão DELIBERADO (o do smoke tem 1,2 doc contra 0,26 de tinta, 4,6×) não é tocado.
//!
//! **Por que aqui e não uma erosão global** (`09 §3` do plano, a pesquisa em fontes
//! primárias): fechar vão por *abertura morfológica* (a trapped-ball com raio grande) come as
//! quinas e as câmaras estreitas — o implementador do MyPaint construiu exatamente isso e a
//! descartou por escrito, citando *"loss of information for sharp corners"*. A solda é
//! **local** (só entre as partes que se tocam), **monotônica** (soldar não pode abrir nada) e
//! tem **efeito zero** onde não há junta.

use ph2d_core::Vec2;

/// Uma junta a fechar: um segmento de parede virtual entre `a` e `b`.
pub(crate) type Weld = (Vec2, Vec2);

/// O envelope de um traço: bbox dos EIXOS + a maior meia-largura dele.
struct Envelope {
    lo: Vec2,
    hi: Vec2,
    max_r: f32,
    /// Comprimento acumulado ao longo da polilinha (`arc[i]` = do ponto 0 até o `i`).
    arc: Vec<f32>,
}

fn envelope(pts: &[Vec2], w: &[f32]) -> Envelope {
    let mut lo = Vec2::splat(f32::INFINITY);
    let mut hi = Vec2::splat(f32::NEG_INFINITY);
    let mut max_r = 0.0f32;
    let mut arc = Vec::with_capacity(pts.len());
    let mut acc = 0.0f32;
    for (i, p) in pts.iter().enumerate() {
        lo = lo.min(*p);
        hi = hi.max(*p);
        max_r = max_r.max(w.get(i).copied().unwrap_or(0.0).max(0.0));
        if i > 0 {
            acc += (*p - pts[i - 1]).length();
        }
        arc.push(acc);
    }
    Envelope { lo, hi, max_r, arc }
}

/// O ponto mais próximo de `p` no segmento `a..b`, como `(t, ponto, distância²)`.
fn closest_on_segment(p: Vec2, a: Vec2, b: Vec2) -> (f32, Vec2, f32) {
    let ab = b - a;
    let l2 = ab.x * ab.x + ab.y * ab.y;
    let t = if l2 <= 0.0 {
        0.0
    } else {
        (((p - a).x * ab.x + (p - a).y * ab.y) / l2).clamp(0.0, 1.0)
    };
    let q = a + ab * t;
    let d = p - q;
    (t, q, d.x * d.x + d.y * d.y)
}

/// As soldas que a arte pede: para cada **ponta de traço aberto**, o ponto mais próximo em
/// cada outro traço (e no PRÓPRIO traço, longe da ponta — o círculo que quase fecha) cujo
/// corpo pintado alcança a ponta.
///
/// ⚠️ **Uma ponta pode soldar em mais de um traço** (uma junta em T tripla), e isso é o certo:
/// se o corpo dela cobre os dois, a tela mostra os três unidos. O que ela **não** faz é soldar
/// duas vezes no mesmo traço — por traço fica a junta mais próxima, que é a que o artista
/// desenhou.
pub(crate) fn welds(strokes: &[(Vec<Vec2>, Vec<f32>, bool)]) -> Vec<Weld> {
    let env: Vec<Envelope> = strokes.iter().map(|(pts, w, _)| envelope(pts, w)).collect();
    let mut out: Vec<Weld> = Vec::new();

    for (si, (pts, w, closed)) in strokes.iter().enumerate() {
        if *closed || pts.len() < 2 {
            continue; // um traço fechado não tem ponta solta
        }
        for &ei in &[0usize, pts.len() - 1] {
            let e = pts[ei];
            // Sem corpo pintado (`r_e = 0`) nada cobre vão nenhum, e o teste de alcance
            // abaixo já diz isso — um atalho aqui seria uma 2ª porta para a mesma regra,
            // provada inerte por mutação (o `r_e <= 0` sobrevivia a todo gate).
            let r_e = w.get(ei).copied().unwrap_or(0.0).max(0.0);
            for (ti, (t_pts, t_w, t_closed)) in strokes.iter().enumerate() {
                if t_pts.len() < 2 {
                    continue;
                }
                // Broadphase: o alcance máximo desta ponta contra ESTE traço.
                let reach = r_e + env[ti].max_r;
                if e.x < env[ti].lo.x - reach
                    || e.x > env[ti].hi.x + reach
                    || e.y < env[ti].lo.y - reach
                    || e.y > env[ti].hi.y + reach
                {
                    continue;
                }
                // A vizinhança da PRÓPRIA ponta não é junta: ela é o traço. Medida em
                // comprimento de ARCO (o índice não é distância), com a mesma régua do
                // alcance — assim um traço que dá a volta e quase se fecha AINDA solda.
                let self_skip = if ti == si { 2.0 * reach } else { -1.0 };
                let a_e = env[si].arc[ei];

                let n = t_pts.len();
                let last = if *t_closed { n } else { n - 1 };
                let mut best: Option<(f32, Vec2, f32)> = None; // (d², q, r_q)
                for i in 0..last {
                    let (a, b) = (t_pts[i], t_pts[(i + 1) % n]);
                    let (t, q, d2) = closest_on_segment(e, a, b);
                    if ti == si {
                        let arc_q =
                            env[ti].arc[i] + (env[ti].arc[(i + 1).min(n - 1)] - env[ti].arc[i]) * t;
                        if (arc_q - a_e).abs() <= self_skip {
                            continue;
                        }
                    }
                    let r_a = t_w.get(i).copied().unwrap_or(0.0).max(0.0);
                    let r_b = t_w.get((i + 1) % n).copied().unwrap_or(0.0).max(0.0);
                    let r_q = r_a + (r_b - r_a) * t;
                    if d2 > (r_e + r_q) * (r_e + r_q) {
                        continue;
                    }
                    if best.is_none_or(|(bd2, ..)| d2 < bd2) {
                        best = Some((d2, q, r_q));
                    }
                }
                // `d2 > 0`: as pontas já coincidentes não precisam de solda (e um segmento
                // de comprimento zero não é parede).
                if let Some((_, q, _)) = best.filter(|(d2, ..)| *d2 > 0.0) {
                    out.push((e, q));
                }
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "weld_tests.rs"]
mod tests;
