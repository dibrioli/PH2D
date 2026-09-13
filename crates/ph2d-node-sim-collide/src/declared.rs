//! ⭐⭐ **A PEÇA QUE DECLAROU UMA FORMA** (doc 109 §5 — report do dono, 2026-09-13: *«o collider
//! não é gerado conforme a forma da Shape»*).
//!
//! Irmão do `lib.rs` pelo tecto de LOC e por ASSUNTO: lá o obstáculo cresce pelo RAIO da peça (a
//! inflação de Minkowski de um disco); aqui a peça é a forma inteira que ela declarou — uma caixa
//! orientada, ou um disco com o centro fora de `P` —, e a pergunta *«quão fundo e para que lado?»* é
//! a da folha `ph2d-contact`, a mesma que o `sim.step` faz entre peças.
//!
//! ⚠️ **Só o modo `Auto` chega aqui** (`Fixed` e `Sprite Size` substituem a declaração por um raio,
//! por definição), e um disco CENTRADO fica no caminho de sempre, **ao bit**.
//!
//! ⚠️ **A resposta continua a ser a do `lib.rs`**: esta porta devolve a normal e a profundidade, e
//! o `respond` único move a peça e reflecte a velocidade. *Uma resposta por forma é como um colisor
//! ganha um defeito por forma.*

use super::{SHAPE_BOWL, SHAPE_BOX, SHAPE_DISC};
use ph2d_contact::{Colisor, Forma};

/// O contacto da peça em `p` que declarou `col`: a normal PARA ONDE ela tem de sair e a
/// profundidade, ou `None`.
#[allow(clippy::too_many_arguments)]
pub(super) fn contact_declared(
    shape: i32,
    p: [f32; 2],
    height: f32,
    c: [f32; 2],
    radius: f32,
    plane_n: [f32; 2],
    half: [f32; 2],
    col: &Colisor,
) -> Option<([f32; 2], f32)> {
    match shape {
        // O obstáculo é o `a` do par, então a normal já aponta PARA a peça. `eixo_x = false` dá o
        // `[0, 1]` que o disco sempre usou num centro exacto.
        SHAPE_DISC => ph2d_contact::contato(&Colisor::disco(radius), c, col, p, false),
        // O eixo da caixa sai da normal do plano, como no `box_contact`: `(cos, sin) = (n.y, −n.x)`.
        SHAPE_BOX => ph2d_contact::contato(
            &Colisor::caixa(half, [plane_n[1], -plane_n[0]]),
            c,
            col,
            p,
            false,
        ),
        SHAPE_BOWL => bowl(p, col, c, radius),
        // O plano: o que o toca é a FACE da peça ao longo da normal — o suporte da forma.
        _ => {
            let cp = col.centro(p);
            let face = cp[0] * plane_n[0] + cp[1] * plane_n[1] - col.suporte(plane_n);
            (face < height).then_some((plane_n, height - face))
        }
    }
}

/// **A taça** — a peça tem de caber INTEIRA dentro do círculo.
///
/// Um disco encolhe a taça pelo raio dele (a lei do `lib.rs`, agora com o centro deslocado). Uma
/// caixa sai pelo CANTO mais longe do centro: empurrar a peça ao longo da direcção desse canto traz
/// exactamente esse canto à parede, e as varreduras da zona trazem os outros.
///
/// ⚠️ **Uma caixa maior que a taça não cabe em sítio nenhum** — a resposta honesta é o centro, a
/// mesma que o disco dá com `inner = 0`.
fn bowl(p: [f32; 2], col: &Colisor, c: [f32; 2], radius: f32) -> Option<([f32; 2], f32)> {
    let cp = col.centro(p);
    let (dx, dy) = (cp[0] - c[0], cp[1] - c[1]);
    let dist = dx.hypot(dy);
    match col.forma {
        Forma::Disco(r) => {
            let n = if dist > f32::EPSILON {
                [dx / dist, dy / dist]
            } else {
                [0.0, 1.0]
            };
            let inner = (radius - r).max(0.0);
            (dist > inner).then_some(([-n[0], -n[1]], dist - inner))
        }
        Forma::Caixa { meia, .. } => {
            if meia[0].hypot(meia[1]) >= radius {
                return (dist > f32::EPSILON).then(|| ([-dx / dist, -dy / dist], dist));
            }
            let mut longe: Option<([f32; 2], f32)> = None;
            for k in col.cantos(p)? {
                let d = (k[0] - c[0]).hypot(k[1] - c[1]);
                if longe.is_none_or(|(_, m)| d > m) {
                    longe = Some((k, d));
                }
            }
            let (k, d) = longe?;
            (d > radius).then(|| ([(c[0] - k[0]) / d, (c[1] - k[1]) / d], d - radius))
        }
    }
}
