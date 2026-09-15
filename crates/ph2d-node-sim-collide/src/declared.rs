//! ⭐⭐ **A PEÇA QUE DECLAROU UMA FORMA** (doc 109 §5 — report do dono, 2026-09-13: *«o collider
//! não é gerado conforme a forma da Shape»*), **e como ela RODA** (§6: *«precisa destravar a rot»*).
//!
//! Irmão do `lib.rs` pelo tecto de LOC e por ASSUNTO: lá o obstáculo cresce pelo RAIO da peça (a
//! inflação de Minkowski de um disco); aqui a peça é a forma inteira que ela declarou — uma caixa
//! orientada, ou um disco com o centro fora de `P` —, e a pergunta *«quão fundo, para que lado e
//! ONDE?»* é a da folha `ph2d-contact`, a mesma que o `sim.step` faz entre peças.
//!
//! ⚠️ **Só o modo `Auto` chega aqui** (`Fixed` e `Sprite Size` substituem a declaração por um raio,
//! por definição), e um disco CENTRADO que não roda fica no caminho de sempre, **ao bit**.
//!
//! ⚠️ **A resposta continua a ser a do `lib.rs`**: esta porta devolve a normal, o quanto EMPURRAR e o
//! quanto RODAR, e o `respond` único move a peça e reflecte a velocidade. *Uma resposta por forma é
//! como um colisor ganha um defeito por forma.*

use super::{SHAPE_BOWL, SHAPE_BOX, SHAPE_DISC};
use ph2d_contact::{Colisor, Contacto, Forma};

/// **O que o contacto de uma peça declarada pede.**
#[derive(Default)]
pub(super) struct Toque {
    /// `(normal, empurrão)`, ou `None` quando ela não toca.
    pub empurrao: Option<([f32; 2], f32)>,
    /// ⭐ **A alavanca da NORMAL no ponto** (`r × n`) — num disco centrado é zero, e numa caixa
    /// pousada de chapa também. É por ela que o impulso normal faz a peça TOMBAR.
    pub braco_n: f32,
    /// ⭐ **A alavanca da TANGENTE no ponto** (`r · n`, doc 109 §7) — num disco o raio inteiro,
    /// exactamente onde a da normal é zero. É por ela que o atrito faz a peça ROLAR.
    pub braco_t: f32,
}

/// ⭐⭐⭐ **A peça sai INTEIRA da parede, e o binário vai para o `spin`** (doc 109 §6 + §8).
///
/// ⚠️⚠️ **Esta função repartia a penetração entre empurrar e RODAR** (`k = 1 + invI·braço²`,
/// `λ = pen/k`, e o resto virava um empurrão de ângulo somado ao `rot`). Medido no report do dono de
/// 2026-09-15, isso produzia um **ciclo de 2 tiques**: o ângulo de uma peça pousada na taça
/// alternava `+3,98° / −2,25°` a cada tique, 17 trocas de sinal em 18 passos. A causa é a MOEDA —
/// um empurrão de ângulo não tem memória, logo nada o amortece, e o [`super::resposta`] declara no
/// cabeçalho dele que a moeda deste nó é o **`spin`**. ⇒ a penetração é devolvida INTEIRA (a lei
/// linear de sempre) e o binário sai pelo impulso normal, em `spin`, onde o `angular_damping` e o
/// termo auto-corrector do atrito já lhe pegam. Tabela no doc 109 §8.
#[allow(clippy::too_many_arguments)]
pub(super) fn toque(
    shape: i32,
    p: [f32; 2],
    height: f32,
    c: [f32; 2],
    radius: f32,
    plane_n: [f32; 2],
    half: [f32; 2],
    col: &Colisor,
) -> Toque {
    let Some(ct) = contact_declared(shape, p, height, c, radius, plane_n, half, col) else {
        return Toque::default();
    };
    let centro = col.centro(p);
    Toque {
        empurrao: Some((ct.normal, ct.penetracao)),
        braco_n: ct.braco(centro),
        braco_t: ct.braco_tangente(centro),
    }
}

/// O contacto da peça em `p` que declarou `col`: a normal PARA ONDE ela tem de sair, a profundidade
/// e o ponto, ou `None`.
#[allow(clippy::too_many_arguments)]
fn contact_declared(
    shape: i32,
    p: [f32; 2],
    height: f32,
    c: [f32; 2],
    radius: f32,
    plane_n: [f32; 2],
    half: [f32; 2],
    col: &Colisor,
) -> Option<Contacto> {
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
        // O plano: o que o toca é a FACE da peça ao longo da normal — o suporte da forma. ⭐ E o
        // PONTO é o do suporte, que de chapa é o meio da face (binário zero: uma peça pousada não
        // tomba) e inclinada é a quina (binário que a deita).
        _ => {
            let cp = col.centro(p);
            let face = cp[0] * plane_n[0] + cp[1] * plane_n[1] - col.suporte(plane_n);
            (face < height).then(|| Contacto {
                normal: plane_n,
                penetracao: height - face,
                // ⚠️ O ponto do SUPORTE, e não `centro − n · suporte`: aquele cai sempre debaixo do
                // centro, com braço zero, e a caixa inclinada nunca se endireitava.
                ponto: col.ponto_de_suporte(p, plane_n),
            })
        }
    }
}

/// **A taça** — a peça tem de caber INTEIRA dentro do círculo.
///
/// Um disco encolhe a taça pelo raio dele (a lei do `lib.rs`, agora com o centro deslocado). Uma
/// caixa sai pelo CANTO mais longe do centro: empurrar a peça ao longo da direcção desse canto traz
/// exactamente esse canto à parede, e as varreduras da zona trazem os outros. O ponto do contacto é
/// esse mesmo ponto extremo — é ali que a parede a toca.
///
/// ⚠️ **Uma caixa maior que a taça não cabe em sítio nenhum** — a resposta honesta é o centro, a
/// mesma que o disco dá com `inner = 0`.
fn bowl(p: [f32; 2], col: &Colisor, c: [f32; 2], radius: f32) -> Option<Contacto> {
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
            (dist > inner).then(|| Contacto {
                normal: [-n[0], -n[1]],
                penetracao: dist - inner,
                ponto: [cp[0] + n[0] * r, cp[1] + n[1] * r],
            })
        }
        Forma::Caixa { meia, .. } => {
            if meia[0].hypot(meia[1]) >= radius {
                return (dist > f32::EPSILON).then(|| Contacto {
                    normal: [-dx / dist, -dy / dist],
                    penetracao: dist,
                    ponto: cp,
                });
            }
            let mut longe: Option<([f32; 2], f32)> = None;
            for k in col.cantos(p)? {
                let d = (k[0] - c[0]).hypot(k[1] - c[1]);
                if longe.is_none_or(|(_, m)| d > m) {
                    longe = Some((k, d));
                }
            }
            let (k, d) = longe?;
            (d > radius).then(|| Contacto {
                normal: [(c[0] - k[0]) / d, (c[1] - k[1]) / d],
                penetracao: d - radius,
                ponto: k,
            })
        }
    }
}
