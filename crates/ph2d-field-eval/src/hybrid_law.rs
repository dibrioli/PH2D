//! ⭐⭐⭐ **A LEI DA BOOLEANA EM NÚMEROS** — o gémeo `f32` do [`crate::ops_bool`].
//!
//! ⚠️ **É a única duplicação de lei deste módulo, e ela tem JUIZ:** o gate
//! `the_numeric_law_is_the_same_law_as_the_tree` avalia as duas formas no mesmo documento e
//! compara ponto a ponto. Sem ele, as duas divergiriam na primeira wave que mexesse numa e
//! esquecesse a outra — e o sintoma seria *um filete com outro raio*, não um erro. Já mordeu: a
//! W146 negou os campos na árvore e esqueceu-os aqui, e foi este gate que o apanhou.
//!
//! ⚠️ **Por que ela vive num ficheiro próprio** (2026-09-10): o [`crate::hybrid`] passou o tecto de
//! `700` LOC, e a lei é a fatia com responsabilidade própria — *as fitas e o avaliador* de um lado,
//! *a aritmética da booleana* do outro. ⛔ A cura de um tecto de LOC é **cortar por
//! responsabilidade**, nunca subir uma entrada de tolerância.

use ph2d_field::{Blend, Op};

/// ⭐ **A lei da booleana, em números** — a mesma de [`crate::ops`], escrita uma segunda vez porque
/// um `min` entre uma fita de JIT e uma grade de voxels não pode acontecer dentro de nenhuma das
/// duas.
///
/// ⚠️ **É a única duplicação de lei deste módulo, e ela tem juiz**: o gate
/// `the_numeric_law_is_the_same_law_as_the_tree` avalia as duas formas no mesmo documento e compara
/// ponto a ponto. Sem ele, as duas divergiriam na primeira wave que mexesse numa e esquecesse a
/// outra — e o sintoma seria um filete com outro raio, não um erro.
#[must_use]
pub fn apply(op: Op, a: f32, b: f32) -> f32 {
    match op {
        Op::Union(blend) => decoracao(a.min(b), a, b, blend).unwrap_or_else(|| union(a, b, blend)),
        // De Morgan, exatamente como a árvore faz — **excepto** para as três decorações, que
        // recebem a superfície e ficam orientadas (ver `ops_bool::decoracao`).
        // ⚠️⚠️ **Os campos vão NEGADOS, como na árvore** (`ops_bool::intersection`): a `decoracao`
        // lê-os na orientação da UNIÃO, e é isso que dá sentido à guarda do sulco. ⛔ Esquecer a
        // negação aqui compila, desenha e só o
        // `the_numeric_law_is_the_same_law_as_the_tree` o diz — *foi o que ele disse.*
        Op::Intersection(blend) => {
            decoracao(a.max(b), -a, -b, blend).unwrap_or_else(|| -union(-a, -b, blend))
        }
        Op::Difference(blend) => {
            let nb = -b;
            decoracao(a.max(nb), -a, b, blend).unwrap_or_else(|| -union(-a, b, blend))
        }
    }
}

/// ⭐⭐⭐ **A gémea numérica de [`crate::ops_bool::decoracao`]** — a superfície entra, e por isso um
/// sulco escava nas TRÊS operações.
///
/// ⚠️ **Sem esta função, o `the_numeric_law_is_the_same_law_as_the_tree` reprova numa intersecção**
/// — que é exactamente o serviço que ele presta.
fn decoracao(d: f32, a: f32, b: f32, blend: Blend) -> Option<f32> {
    let s = (a - b) * std::f32::consts::FRAC_1_SQRT_2;
    match blend {
        Blend::Bead { radius } if radius > 0.0 => Some(d.min(a.hypot(b) - radius)),
        Blend::Groove { radius } if radius > 0.0 => {
            Some(d.max((radius - a.hypot(b)).min(a.max(b))))
        }
        Blend::Ridge { radius, width } if radius > 0.0 => {
            Some(d.min((d - radius).max(s.abs() - width)))
        }
        _ => None,
    }
}

fn union(a: f32, b: f32, blend: Blend) -> f32 {
    match blend {
        Blend::Sharp => a.min(b),
        Blend::Exact { radius } if radius > 0.0 => {
            let ux = (radius - a).max(0.0);
            let uy = (radius - b).max(0.0);
            a.min(b).max(radius) - ux.hypot(uy)
        }
        // ⭐⭐⭐ **O CHANFRO** (W99) — a mesma linha da árvore (`ops::union_chamfer`), em números.
        Blend::Chamfer { radius } if radius > 0.0 => a
            .min(b)
            .min((a + b - radius) * std::f32::consts::FRAC_1_SQRT_2),
        Blend::Organic { radius } if radius > 0.0 => {
            // ⭐ A calibração, igual à da árvore — ver [`Blend::ORGANIC_REACH`].
            let k = radius * Blend::ORGANIC_REACH;
            let h = 0.5f32.mul_add((b - a) / k, 0.5).clamp(0.0, 1.0);
            let mixed = (a - b).mul_add(h, b);
            mixed - k * h * (1.0 - h)
        }
        // ─────────── W145: as cinco novas, na MESMA ordem da árvore ───────────
        //
        // ⚠️ **Cada linha aqui é a gémea de uma de [`crate::ops_bool`]**, e o juiz é o
        // `the_numeric_law_is_the_same_law_as_the_tree`: sem ele as duas divergem na primeira wave
        // que mexa numa e esqueça a outra, e o sintoma é uma junta com outro tamanho — nunca um
        // erro.
        Blend::Soft { radius } if radius > 0.0 => {
            let k = radius * Blend::SOFT_REACH;
            let h = (k - (a - b).abs()).max(0.0) / k;
            a.min(b) - h * h * h * k / 6.0
        }
        Blend::Bevel { radius, bias } if radius > 0.0 => {
            // ⭐ O mesmo desvio da árvore: simétrico devolve a fórmula do chanfro, ao bit.
            if (bias - 1.0).abs() < 1.0e-9 {
                a.min(b)
                    .min((a + b - radius) * std::f32::consts::FRAC_1_SQRT_2)
            } else {
                let (ca, cb) = (radius, radius * bias);
                let norma = (ca.powi(-2) + cb.powi(-2)).sqrt();
                a.min(b).min((a / ca + b / cb - 1.0) / norma)
            }
        }
        // ⚠️ Raio zero é união DURA, e não uma fórmula com um zero dentro — é o mesmo ramo que a
        // árvore toma, e a razão é a mesma: com `r = 0` as duas são algebricamente idênticas.
        Blend::Exact { .. }
        | Blend::Chamfer { .. }
        | Blend::Organic { .. }
        | Blend::Soft { .. }
        | Blend::Bead { .. }
        | Blend::Groove { .. }
        | Blend::Ridge { .. }
        | Blend::Bevel { .. } => a.min(b),
    }
}
