//! **A cena pronta para o smoke do COLORIZE** (`PH2D_FLIP_COLORIZE_SMOKE=1`).
//!
//! A arte é a que EXPÕE o valor do LazyBrush — o que só ele faz: uma moldura dividida por
//! uma linha **fora do centro** com um **VÃO no meio**, e DOIS rabiscos (um de cada lado).
//! Clicar **Apply** tem de:
//!
//! 1. **A cor segue a TINTA, não o meio.** O divisor está em x≈+1 (não no centro), então a
//!    cor da ESQUERDA é dona de mais área — a fronteira entre as cores cola na linha, não no
//!    ponto médio dos rabiscos.
//! 2. **O vão NÃO vaza.** O buraco no divisor está aberto, mas a cor não escorre por ele: o
//!    corte paga a largura do vão em branco em vez de contornar. É o *"um vão não precisa
//!    fechar"* — o balde comum precisaria fechar o vão com o Gap Closure.
//!
//! Dois rabiscos vêm pré-semeados **e VISÍVEIS** (o overlay ao vivo), então dá para conferir
//! o Apply de imediato — e depois rabiscar os seus, vendo a marca sair sob o cursor.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_COLORIZE_SMOKE").is_some())
}

pub const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);

/// Um traço de mão (tremor determinístico) — numa reta perfeita o contorno colapsa e as
/// rotas empatam; a arte trêmula é o que a cena existe para exercer.
pub fn hand(pts: &[Vec2], seed: usize) -> FlipStroke {
    let mut s = FlipStroke::new();
    for (i, p) in pts.iter().enumerate() {
        let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
        s.push_point(Point {
            pos: Vec2::new(p.x + h(i + seed) * 0.05, p.y + h(i + seed + 91) * 0.05),
            width: 0.26,
            opacity: 1.0,
            color: INK,
        });
    }
    s.hardness = 0.35;
    s
}

/// Amostra um segmento reto em `n` pontos.
pub fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        })
        .collect()
}
