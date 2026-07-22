//! **Auto-flip** — B foi desenhado no sentido CONTRÁRIO de A?
//!
//! Módulo irmão do [`crate::tween`]: é uma responsabilidade à parte, e erra de outro
//! jeito. O tween erra em *interpolação* (o inbetween sai torto); isto erra em
//! *geometria de decisão* (o traço dá um NÓ no meio do caminho, porque o ponto 0 de A
//! foi interpolado contra o ponto final de B).
//!
//! O teste é puramente geométrico — cordas que se cruzam, ou direções opostas — com
//! desempate por distância quando as cordas são quase paralelas (< 15°). Comparar
//! SENOS, nunca chamar `acos` (HR-5).
//!
//! ⚠️ **Traço FECHADO responde por outra pergunta** ([`opposite_winding`]) — ver lá.

use crate::stroke::FlipStroke;
use ph2d_core::Vec2;

/// `sin(15°)` — o limiar de "cordas quase paralelas", em forma polinomial.
const SIN_15: f32 = 0.258_819_04;

pub(crate) fn should_flip(a: &FlipStroke, b: &FlipStroke) -> bool {
    // **Num traço fechado as "pontas" são VIZINHAS** (a costura), então a corda entre elas
    // não descreve o percurso — o teste abaixo vira ruído e inverte anéis à toa.
    //
    // ⚠️ Era um bug REAL, e a espiral o expôs: num "O" que gira 90° o flip espúrio
    // transformava a rotação limpa numa reflexão, o ajuste via `σ ≈ 0` e caía na
    // translação — o anel cortava caminho pela CORDA em vez de percorrer o arco.
    //
    // Para um anel a pergunta certa é o SENTIDO do percurso: dois windings opostos
    // percorrem o contorno em direções contrárias, e interpolar entre eles vira a forma do
    // avesso (o `ph2d-vec-blend` mediu isso — a forma COLAPSA no meio).
    if a.closed || b.closed {
        return opposite_winding(a, b);
    }
    let (Some(a0), Some(a1)) = (a.point(0), a.point(a.len().saturating_sub(1))) else {
        return false;
    };
    let (Some(b0), Some(b1)) = (b.point(0), b.point(b.len().saturating_sub(1))) else {
        return false;
    };
    let (a0, a1, b0, b1) = (a0.pos, a1.pos, b0.pos, b1.pos);
    let (c1, c2) = (b0 - a0, b1 - a1); // as duas cordas, como vetores

    if segments_cross(a0, b0, a1, b1) {
        // Quase paralelas: o cruzamento é ruído numérico — decide a distância.
        if nearly_parallel(c1, c2) {
            return dist2(a0, b1) + dist2(a1, b0) < dist2(a0, b0) + dist2(a1, b1);
        }
        return true;
    }
    let (da, db) = (a1 - a0, b1 - b0);
    da.x * db.x + da.y * db.y < 0.0
}

/// **Os dois anéis são percorridos em sentidos contrários?** (área com sinal, shoelace).
///
/// É o teste que substitui as cordas no caso fechado. Área ZERO (um anel degenerado, ou
/// um traço marcado `closed` com dois pontos) não decide nada — e aí não inverter é a
/// resposta segura: uma inversão espúria é o defeito que este módulo existe para evitar.
///
/// ⚠️ **O que isto NÃO resolve, de propósito:** dois anéis de mesmo sentido cujo ponto 0
/// está em lugares diferentes do contorno (a FASE da costura). O tween os interpola com a
/// costura desalinhada, e a forma do meio fica torcida. A resposta é o alinhamento de fase
/// por correlação circular que o `ph2d-vec-blend` já construiu para o Blend do vetor —
/// wave própria, nomeada no handoff, e não um `if` a mais aqui.
fn opposite_winding(a: &FlipStroke, b: &FlipStroke) -> bool {
    let area = |s: &FlipStroke| -> f32 {
        let p = s.positions();
        let n = p.len();
        if n < 3 {
            return 0.0;
        }
        // Shoelace: o dobro da área com sinal (o fator 2 não muda o SINAL, que é o que
        // importa aqui, então não se paga por ele).
        (0..n)
            .map(|i| {
                let (u, v) = (p[i], p[(i + 1) % n]);
                u.x * v.y - v.x * u.y
            })
            .sum()
    };
    let (wa, wb) = (area(a), area(b));
    wa * wb < 0.0
}

/// Cordas com ângulo < 15° entre si (mesmo sentido): `|sin| < sin15` e `cos > 0`.
fn nearly_parallel(u: Vec2, v: Vec2) -> bool {
    let cross = (u.x * v.y - u.y * v.x).abs();
    let dot = u.x * v.x + u.y * v.y;
    let mag = (u.x * u.x + u.y * u.y).sqrt() * (v.x * v.x + v.y * v.y).sqrt();
    dot > 0.0 && cross < SIN_15 * mag
}

fn dist2(p: Vec2, q: Vec2) -> f32 {
    let d = q - p;
    d.x * d.x + d.y * d.y
}

/// Os segmentos `p→q` e `r→s` se cruzam? (teste de orientação 2D, sem divisão.)
fn segments_cross(p: Vec2, q: Vec2, r: Vec2, s: Vec2) -> bool {
    fn orient(a: Vec2, b: Vec2, c: Vec2) -> f32 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }
    let (d1, d2) = (orient(p, q, r), orient(p, q, s));
    let (d3, d4) = (orient(r, s, p), orient(r, s, q));
    (d1 * d2 < 0.0) && (d3 * d4 < 0.0)
}
