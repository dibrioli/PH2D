//! Demo ready-to-smoke do Flip (ADR-0113 W1): um objeto animado para VER o render
//! do traço na hora. Flag-gated por `PH2D_FLIP_DEMO=1` (default = cena vazia, como
//! a pipeline vetorial nova — o app normal não mostra nada do Flip até a tool do W2).
//!
//! Coordenadas em mundo (a `Camera2d` default enquadra ~10 unidades de altura em
//! torno da origem). Ligue `PH2D_FLIP_DEMO=1` e dê play (o transporte) para ver o
//! traço da camada FG saltar entre os 3 quadros-chave.

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDoc, FlipStroke, Hold, KeyKind, Point, Rgba};

/// A cena Flip de boot: vazia por padrão; o objeto-demo sob `PH2D_FLIP_DEMO`.
pub(crate) fn demo_scene() -> FlipDoc {
    let mut doc = FlipDoc::new();
    if std::env::var_os("PH2D_FLIP_DEMO").is_none() {
        return doc;
    }
    eprintln!("[ph2d-flip] PH2D_FLIP_DEMO ativo — objeto 'Demo Flip' (BG + FG 3 quadros @12fps)");
    let oid = doc.push_object("Demo Flip");
    let obj = doc.object_mut(oid).expect("objeto recém-criado existe");
    obj.fps = 12.0;

    // BG: uma moldura + um quadrado PREENCHIDO (prova o fill), segurando a
    // animação inteira (1 desenho, hold implícito).
    let bg = obj.add_layer("BG");
    if let Some(d) = obj.insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe) {
        let dr = obj.drawing_mut(d).expect("desenho recém-criado");
        dr.strokes.push(filled_square());
        dr.strokes.push(frame_rect());
    }

    // FG: 3 quadros-chave (0/8/16) com um traço que salta + varia a hardness.
    let fg = obj.add_layer("FG");
    let keys = [
        (0_i32, Vec2::new(-2.0, 0.0)),
        (8, Vec2::new(0.0, 1.2)),
        (16, Vec2::new(2.0, 0.0)),
    ];
    for (frame, pos) in keys {
        if let Some(d) = obj.insert_frame(fg, frame, Hold::Implicit, KeyKind::Keyframe) {
            obj.drawing_mut(d)
                .expect("desenho recém-criado")
                .strokes
                .push(moving_mark(pos));
        }
    }
    doc
}

/// Uma moldura retangular fechada (traço fino, dura, cinza) em torno da cena.
fn frame_rect() -> FlipStroke {
    let mut s = FlipStroke::new();
    let gray = Rgba::new(0.6, 0.6, 0.6, 1.0);
    for corner in [
        Vec2::new(-3.0, -2.0),
        Vec2::new(3.0, -2.0),
        Vec2::new(3.0, 2.0),
        Vec2::new(-3.0, 2.0),
    ] {
        s.push_point(Point {
            pos: corner,
            width: 0.05,
            opacity: 1.0,
            color: gray,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s
}

/// Um quadrado fechado PREENCHIDO (fill verde translúcido, contorno fino) no
/// canto direito — a prova visual do T1.6 (fill triangulado).
fn filled_square() -> FlipStroke {
    let mut s = FlipStroke::new();
    let outline = Rgba::new(0.2, 0.7, 0.3, 1.0);
    for corner in [
        Vec2::new(1.4, -1.4),
        Vec2::new(2.4, -1.4),
        Vec2::new(2.4, -0.4),
        Vec2::new(1.4, -0.4),
    ] {
        s.push_point(Point {
            pos: corner,
            width: 0.04,
            opacity: 1.0,
            color: outline,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill {
        color: Rgba::new(0.2, 0.7, 0.3, 1.0),
        opacity: 0.5,
    });
    s
}

/// Um traço vertical curto e grosso (vermelho, macio) na posição `pos` — a marca
/// que se move entre os quadros.
fn moving_mark(pos: Vec2) -> FlipStroke {
    let mut s = FlipStroke::new();
    let red = Rgba::new(0.9, 0.1, 0.1, 1.0);
    s.push_point(Point {
        pos: Vec2::new(pos.x, pos.y - 0.6),
        width: 0.3,
        opacity: 1.0,
        color: red,
    });
    s.push_point(Point {
        pos: Vec2::new(pos.x, pos.y + 0.6),
        width: 0.3,
        opacity: 1.0,
        color: red,
    });
    s.hardness = 0.6;
    s
}
