//! Testes do **domínio POINT** do Edit Mode (W8) — módulo-irmão de
//! `flip_select_tests` pelo cap de LOC do HR-18. `super` é `flip_select`.

use super::*;
use crate::flip_select::apply_style_delta;
use ph2d_flip::{FlipStroke, Point, Rgba};

fn line(pts: &[(f32, f32)], width: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    s
}

fn drawing(strokes: Vec<FlipStroke>) -> FlipDrawing {
    let mut d = FlipDrawing::new();
    d.strokes = strokes;
    d
}

/// 🔴 **O pick de ponto pega a âncora MAIS PRÓXIMA dentro do raio** — e o raio é px de
/// TELA (acompanha o zoom pela mesma conversão do pick de traço). Mutação que sangra:
/// devolver o primeiro hit em vez do mais próximo, ou ignorar o `px_to_world`.
#[test]
fn the_point_pick_takes_the_nearest_anchor_within_reach() {
    let d = drawing(vec![
        line(&[(0.0, 0.0), (10.0, 0.0)], 4.0),
        line(&[(0.4, 0.0), (20.0, 0.0)], 4.0),
    ]);
    // Clique em (0.3, 0): o ponto (0.4) do traço 1 é mais perto que o (0.0) do traço 0.
    assert_eq!(
        point_at(&d, Vec2::new(0.3, 0.0), 1.0, &Xform::IDENTITY),
        Some((1, 0)),
        "tem de ser a ancora mais proxima, nao a primeira dentro do raio"
    );
    // Longe de tudo (raio é 8 px de tela): nada.
    assert_eq!(
        point_at(&d, Vec2::new(0.0, 50.0), 1.0, &Xform::IDENTITY),
        None
    );
    // Zoom OUT (1 px de tela = 4 unidades): o mesmo clique a 20 unidades pega.
    assert_eq!(
        point_at(&d, Vec2::new(10.0, 20.0), 4.0, &Xform::IDENTITY),
        Some((0, 1)),
        "o raio de pick tem de acompanhar o zoom"
    );
}

/// 🔴 **O plano do down por ponto tem o colapso ADIADO** (a regra do W6.1, ponto a
/// ponto): clicar num ponto JÁ selecionado não colapsa — arrasta o grupo; soltar sem
/// arrastar colapsa. Mutação que sangra: colapsar no down (o arrasto de grupo morre).
#[test]
fn the_point_down_defers_the_collapse_like_the_stroke_down() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)], 4.0)]);
    // Clique num ponto NÃO selecionado (sem shift): vira a seleção e abre Move.
    let plan = plan_down_points(&mut d, Some((0, 1)), false, false);
    assert_eq!(plan, DownPoints::Move { collapse_to: None });
    assert!(d.strokes[0].point_selected(1));
    assert!(!d.strokes[0].point_selected(0));

    // Acende mais um; clique no já-selecionado: NÃO mexe na seleção, adia o colapso.
    d.strokes[0].set_point_selected(2, true);
    let plan = plan_down_points(&mut d, Some((0, 1)), false, false);
    assert_eq!(
        plan,
        DownPoints::Move {
            collapse_to: Some((0, 1))
        }
    );
    assert!(
        d.strokes[0].point_selected(2),
        "clicar num ponto selecionado nao pode destruir a multissele\u{e7}\u{e3}o no toque"
    );

    // Shift+clique alterna e resolve no down.
    let plan = plan_down_points(&mut d, Some((0, 2)), true, false);
    assert_eq!(plan, DownPoints::Click);
    assert!(!d.strokes[0].point_selected(2));

    // Vazio sem shift: limpa e abre marquee.
    let plan = plan_down_points(&mut d, None, false, false);
    assert_eq!(plan, DownPoints::Marquee { additive: false });
    assert!(!d.strokes[0].point_selected(1));
}

/// **O marquee por ponto acende só o que está DENTRO da caixa** — ponto é ponto
/// (dentro-ou-fora); quem quer a linha inteira usa o domínio Stroke.
#[test]
fn the_point_marquee_selects_only_the_anchors_inside_the_box() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)], 4.0)]);
    assert!(apply_marquee_points(
        &mut d,
        Vec2::new(5.0, -1.0),
        Vec2::new(15.0, 1.0),
        false,
    ));
    assert!(!d.strokes[0].point_selected(0));
    assert!(d.strokes[0].point_selected(1));
    assert!(!d.strokes[0].point_selected(2));
    // Aditivo (shift): soma sem apagar.
    assert!(apply_marquee_points(
        &mut d,
        Vec2::new(15.0, -1.0),
        Vec2::new(25.0, 1.0),
        true,
    ));
    assert!(d.strokes[0].point_selected(1) && d.strokes[0].point_selected(2));
}

/// 🔴 **Com seleção de ponto PARCIAL, o estilo por-ponto mira SÓ os pontos
/// selecionados** (cor/opacidade/largura) — e os por-CURVA (dureza) seguem no traço
/// inteiro (meio-traço não tem meia-dureza). Mutação que sangra: tirar o ramo
/// `partial` do `apply_style_delta` (a cor pinta o traço todo).
#[test]
fn a_partial_point_selection_narrows_the_per_point_style_writes() {
    let mut d = drawing(vec![line(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)], 4.0)]);
    d.strokes[0].set_point_selected(1, true);
    let prev = ph2d_tool_flip::FlipStyleSnapshot::default();
    let now = ph2d_tool_flip::FlipStyleSnapshot {
        stroke: [255, 0, 0, 255],
        hardness: 0.5,
        ..prev
    };
    assert!(apply_style_delta(&mut d, &prev, &now));
    let red = crate::flip_draw::srgb8_to_linear([255, 0, 0, 255]);
    let colors = d.strokes[0].colors();
    assert_eq!(colors[1], red, "o ponto selecionado tem de ganhar a cor");
    assert_ne!(
        colors[0], red,
        "o ponto NAO-selecionado nao pode ser pintado"
    );
    assert_ne!(colors[2], red);
    // A dureza é por-CURVA: muda o traço inteiro.
    assert!((d.strokes[0].hardness - 0.5).abs() < f32::EPSILON);
}
