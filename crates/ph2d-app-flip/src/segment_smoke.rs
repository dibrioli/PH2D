//! **A cena pronta para o smoke do domínio SEGMENT** (`PH2D_FLIP_SEGMENT_SMOKE=1`, §4.B).
//!
//! O Enio não monta cena ([[feedback_ready_to_smoke_example]]): o app abre com a tool Flip
//! no modo **Edit**, domínio **Segment** já armado, e quatro alvos — um para cada coisa que
//! o modo promete e que os gates não conseguem mostrar na tela.
//!
//! Roteiro (clicar, em cada um):
//!
//! 1. **O X (duas linhas que se cruzam)** — clicar num braço acende SÓ aquele braço, do
//!    cruzamento até a ponta. É a promessa central: o traço vizinho é a tesoura.
//! 2. **O triângulo intacto** — nada o cruza, então clicar em qualquer aresta acende a
//!    forma INTEIRA (o *fallback* do `§11`; é o caso comum do balde, que produz traço
//!    fechado). Sem ele, um clique aqui não acenderia nada.
//! 3. **O quadrado cortado por uma linha de OUTRA CAMADA** — duas coisas de uma vez:
//!    (a) o corte é do **QUADRO**, não do desenho ativo (a linha vermelha vive na camada
//!    "Cutter" e mesmo assim corta); (b) o pedaço da ESQUERDA **enrola na costura** — ele
//!    é um pedaço só, mas atravessa o ponto onde a polilinha fecha (os *"dois ranges"* do
//!    `§11`). Clicar na aresta esquerda tem de acender a quina de baixo E a de cima.
//! 4. **A curva cortada duas vezes** — o caso real (a caneta produz polilinha densa, não
//!    polígono): três pedaços, e clicar no do meio acende só o meio.
//!
//! Confira também: **arrastar** um pedaço o move (o gesto é o do domínio Point — o dado é o
//! mesmo `point_sel`); **Shift+clique** soma pedaços; a **caixa de seleção** acende o pedaço
//! INTEIRO que ela tocou (não recorta na borda dela); trocar para **Point** ou **Stroke** e
//! voltar — Point↔Segment preserva a seleção (mesmo dado), Stroke a promove/limpa.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_SEGMENT_SMOKE").is_some())
}

/// Uma polilinha pelos vértices dados, largura de tela grossa (fácil de acertar).
pub fn stroke(verts: &[Vec2], color: Rgba, closed: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &p in verts {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color,
        });
    }
    s.closed = closed;
    s
}

/// Uma cúbica amostrada em 17 pontos — a polilinha DENSA que a caneta produz (de Casteljau
/// à mão; sem transcendental, HR-5).
pub fn curve(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Vec<Vec2> {
    (0..17)
        .map(|i| {
            let t = i as f32 / 16.0;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            Vec2::new(
                a * p0.x + b * p1.x + c * p2.x + d * p3.x,
                a * p0.y + b * p1.y + c * p2.y + d * p3.y,
            )
        })
        .collect()
}

pub const INK: Rgba = Rgba::new(0.9, 0.9, 0.95, 1.0);
pub const CUT: Rgba = Rgba::new(0.9, 0.25, 0.2, 1.0);
