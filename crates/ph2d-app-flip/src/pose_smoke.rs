//! **A cena pronta para o smoke do gizmo de POSE** (`PH2D_FLIP_POSE_SMOKE=1`, W7.5).
//!
//! O Enio não monta cena (feedback_ready_to_smoke_example): o app abre com 1 objeto
//! Flip cuja chave 0 desenha um quadrado, a chave 12 é uma **INSTÂNCIA** dele já
//! movida (pose ≠ identidade), o playhead parado NA instância e a tool Flip em modo
//! **Edit** — o gizmo da pose já visível enquadrando a arte posada.
//!
//! Roteiro: arrastar uma **quina** gira (anel de hover) / escala; uma **borda**
//! escala num eixo; arrastar a ARTE move (o gesto de sempre). Conferir que a chave 0
//! e o objeto **não se mexem**, que Ctrl+Z desfaz o gesto inteiro, e que voltar o
//! playhead à chave 0 (arte exclusiva) **não** mostra gizmo de pose.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

/// O frame corrente do roteiro (mesmo padrão do `build_smoke`).
pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_POSE_SMOKE").is_some())
}

/// Um quadrado de traço grosso (px de tela) entre `a` e `b`, fechado.
pub fn square(a: Vec2, b: Vec2, color: Rgba) -> FlipStroke {
    let mut s = FlipStroke::new();
    for p in [
        Vec2::new(a.x, a.y),
        Vec2::new(b.x, a.y),
        Vec2::new(b.x, b.y),
        Vec2::new(a.x, b.y),
    ] {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s
}
