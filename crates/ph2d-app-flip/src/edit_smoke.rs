//! **A cena pronta para o smoke do domínio POINT** (`PH2D_FLIP_EDIT_SMOKE=1`, W8).
//!
//! O app abre com 1 objeto Flip (uma senoide de 24 âncoras + um quadrado preenchido),
//! a tool Flip em modo **Edit** e o domínio já em **Point** — as âncoras na tela
//! (dim; selecionadas em acento).
//!
//! Roteiro: clicar numa âncora seleciona SÓ ela · Shift+clique alterna · marquee pega
//! as de dentro · arrastar uma selecionada move a seleção (as outras ficam) · Delete
//! dissolve · **All/None** do painel agem por ponto · trocar pro **Sculpt** com meia
//! senoide selecionada: o Smooth alisa SÓ a metade (máscara fina) · voltar ao domínio
//! **Stroke**: um clique volta a pegar o traço inteiro (a seleção promove por `any`).

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_EDIT_SMOKE").is_some())
}

/// Uma senoide com `n` âncoras — pontos de sobra para o pick/marquee/dissolve.
pub fn wave(n: usize) -> FlipStroke {
    let mut s = FlipStroke::new();
    for k in 0..n {
        let t = k as f32 / (n - 1) as f32;
        s.push_point(Point {
            pos: Vec2::new(-3.0 + 6.0 * t, libm::sinf(t * 12.0)),
            width: 5.0,
            opacity: 1.0,
            color: Rgba::new(0.9, 0.9, 0.95, 1.0),
        });
    }
    s
}
