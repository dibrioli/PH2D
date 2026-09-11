//! **A cena pronta para o smoke do BALDE** (`PH2D_FLIP_FILL_SMOKE=1`).
//!
//! Cobre de uma vez os três fechamentos de 2026-07-18, e a arte é montada para que cada
//! um tenha um lugar onde é **contraditável** — não onde é conveniente:
//!
//! 1. **BUGS #22 — a cor termina no eixo.** A moldura é desenhada com pincel **MACIO**
//!    (`hardness = 0.35`), que é onde a franja vivia: com pincel duro o defeito é
//!    identicamente zero. Preencher e olhar a borda: a cor não pode passar da linha.
//! 2. **O DONUT.** A célula da esquerda tem uma **ilha** solta no meio. Preencher em
//!    volta dela: a ilha **não** pode ser pintada por cima.
//! 3. **A identidade (R3 revogada).** A forma da direita é fechada e sozinha — o balde
//!    põe a cor no PRÓPRIO traço dela. Preencher, ir pro **Sculpt**, selecionar SÓ a
//!    linha e esculpir: a cor tem de ir junto. (A seleção é o que separa as duas rotas;
//!    sem ela as duas empatam.)
//! 4. **O Gap Closure AO VIVO (doc 06 §8).** A caixa de baixo tem um vão DELIBERADO na
//!    parede de cima (0,2 unidade, fora do alcance da solda — ver o comentário da
//!    cena). Ctrl+roda sobe o Gap com o slider acompanhando; o helper VERDE aparece
//!    tapando a boca quando o alcance a atinge, e o clique preenche.
//!
//! Roteiro impresso no terminal quando a cena monta.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_FILL_SMOKE").is_some())
}

pub const INK: Rgba = Rgba::new(0.92, 0.92, 0.95, 1.0);

/// Um traço de mão: os pontos com um tremor determinístico, como o artista desenha.
///
/// ⚠️ **O tremor não é enfeite.** Numa reta perfeita o contorno vetorizado já colapsa nos
/// cantos exatos e as duas rotas do balde dão o mesmo resultado — a arte lisa esconde
/// justamente o que esta cena existe para mostrar.
pub fn hand(pts: &[Vec2], seed: usize, closed: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for (i, p) in pts.iter().enumerate() {
        let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
        s.push_point(Point {
            pos: Vec2::new(p.x + h(i + seed) * 0.06, p.y + h(i + seed + 91) * 0.06),
            width: 0.28,
            opacity: 1.0,
            color: INK,
        });
    }
    s.closed = closed;
    // **MACIO de propósito** — ver o item 1 do cabeçalho.
    s.hardness = 0.35;
    s
}

/// Amostra um segmento reto em `n` pontos (a mão não desenha com 2 vértices).
pub fn seg(a: Vec2, b: Vec2, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        })
        .collect()
}

pub fn ring(cx: f32, cy: f32, r: f32, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            Vec2::new(cx + r * libm::cosf(t), cy + r * libm::sinf(t))
        })
        .collect()
}
