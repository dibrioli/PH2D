//! Demo ready-to-smoke do Flip (ADR-0114): um objeto animado para VER o render do
//! traço, a **composição por-camada** (W1.T1.7) e agora a **tira de frames + os
//! Ghost Frames** (W3) na hora. Flag-gated por `PH2D_FLIP_DEMO=1` (default = cena
//! vazia — o app normal não mostra nada do Flip até a tool ser ativada).
//!
//! Coordenadas em mundo (a `Camera2d` default enquadra ~10 unidades de altura em
//! torno da origem). Ligue `PH2D_FLIP_DEMO=1`, ative a tool **Flip** e:
//! - a **tira** (faixa inferior) mostra as 3 células da camada ativa com a
//!   exposição de cada uma (8, 8, 1 quadros);
//! - pare em cima de uma chave e os **Ghost Frames** aparecem — o desenho anterior
//!   em verde, o seguinte em azul (eles somem no play);
//! - `↑`/`↓` pulam por DESENHO; `Tween` gera os inbetweens entre duas chaves;
//! - **play** (Space ou o botão) roda a animação. A camada FG usa blend
//!   **Multiply**: onde o quadrado que se move cruza o retângulo amarelo do BG, a
//!   sobreposição **escurece** — a prova visual de T1.7.

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDoc, FlipStroke, Hold, KeyKind, Point, Rgba};
use ph2d_painter_effects::BlendMode;

/// A cena Flip de boot: vazia por padrão; o objeto-demo sob `PH2D_FLIP_DEMO`.
pub(crate) fn demo_scene() -> FlipDoc {
    let mut doc = FlipDoc::new();
    if std::env::var_os("PH2D_FLIP_DEMO").is_none() {
        return doc;
    }
    eprintln!(
        "[ph2d-flip] PH2D_FLIP_DEMO ativo — 'Demo Flip': BG amarelo + FG Multiply, 3 chaves @12fps. Ative a tool Flip: tira embaixo, Ghost Frames ligados (verde/azul), setas pulam por desenho."
    );
    let oid = doc.push_object("Demo Flip");
    let obj = doc.object_mut(oid).expect("objeto recém-criado existe");
    obj.fps = 12.0;

    // BG: um retângulo amarelo PREENCHIDO (o campo que a FG vai multiplicar) + uma
    // moldura, segurando a animação inteira (1 desenho, hold implícito). Normal.
    let bg = obj.add_layer("BG");
    if let Some(d) = obj.insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe) {
        let dr = obj.drawing_mut(d).expect("desenho recém-criado");
        dr.strokes.push(filled_rect(
            Vec2::new(-2.5, -1.2),
            Vec2::new(2.5, 1.2),
            Rgba::new(0.95, 0.8, 0.2, 1.0), // amarelo
        ));
        dr.strokes.push(frame_rect());
    }

    // FG: blend MULTIPLY. 3 quadros-chave (0/8/16) com um quadrado magenta que
    // varre o amarelo → a interseção escurece (Multiply). Prova T1.7 no smoke.
    let fg = obj.add_layer("FG");
    obj.layer_mut(fg).expect("camada FG").blend = BlendMode::Multiply;
    let keys = [
        (0_i32, Vec2::new(-2.0, 0.0)),
        (8, Vec2::new(0.0, 0.0)),
        (16, Vec2::new(2.0, 0.0)),
    ];
    for (frame, cx) in keys {
        if let Some(d) = obj.insert_frame(fg, frame, Hold::Implicit, KeyKind::Keyframe) {
            obj.drawing_mut(d)
                .expect("desenho recém-criado")
                .strokes
                .push(filled_rect(
                    Vec2::new(cx.x - 0.9, cx.y - 0.9),
                    Vec2::new(cx.x + 0.9, cx.y + 0.9),
                    Rgba::new(0.85, 0.2, 0.7, 1.0), // magenta
                ));
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
            width: 2.0,
            opacity: 1.0,
            color: gray,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s
}

/// Um retângulo fechado PREENCHIDO (fill + contorno na mesma cor), de `min` a
/// `max` em mundo.
fn filled_rect(min: Vec2, max: Vec2, color: Rgba) -> FlipStroke {
    let mut s = FlipStroke::new();
    for corner in [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)] {
        s.push_point(Point {
            pos: corner,
            width: 2.0,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill {
        color,
        opacity: 1.0,
    });
    s
}
