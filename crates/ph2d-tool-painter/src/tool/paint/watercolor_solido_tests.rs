//! Os gates do **`Style: Solid` na aguada** ([`super::watercolor_solido`], doc 46 §2-7).

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

/// Um gesto à mão livre pelos `cantos`, `por_aresta` eventos por aresta, numa aguada azul de raio 4.
fn gesto(solid: bool, cantos: &[[f32; 2]], por_aresta: usize) -> PainterTool {
    let mut t = tool(128, PaintMedia::Watercolor, 4.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.style_solid = solid;
    t.on_canvas_pointer(cp(cantos[0], PointerPhase::Down));
    for w in cantos.windows(2) {
        for k in 1..=por_aresta {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / por_aresta as f32;
            t.on_canvas_pointer(cp(
                [
                    w[0][0] + (w[1][0] - w[0][0]) * f,
                    w[0][1] + (w[1][1] - w[0][1]) * f,
                ],
                PointerPhase::Move,
            ));
        }
    }
    t.on_canvas_pointer(cp(cantos[cantos.len() - 1], PointerPhase::Up));
    t
}

/// Os texels de um quadrado de `lado` px com canto em `(x0, y0)` que a aguada tingiu.
fn tingidos_em(t: &PainterTool, x0: usize, y0: usize, lado: usize) -> usize {
    (y0..y0 + lado)
        .flat_map(|y| (x0..x0 + lado).map(move |x| (y * 128 + x) * 4))
        .filter(|&i| t.canvas_rgba[i] < 240)
        .count()
}

/// **O SOLID ENCHE A AGUADA** — um laço fino à mão livre tinge a região que cerca (o meio do
/// quadrado fica a 30 px de todo o rastro).
#[test]
fn o_solid_enche_a_regiao_cercada_na_aguada() {
    let laco = [
        [24.0, 24.0],
        [104.0, 24.0],
        [104.0, 104.0],
        [24.0, 104.0],
        [24.0, 28.0],
    ];
    let linha = gesto(false, &laco, 10);
    let mancha = gesto(true, &laco, 10);
    assert_eq!(
        tingidos_em(&linha, 54, 54, 20),
        0,
        "controlo: sem Solid o miolo fica papel"
    );
    assert_eq!(
        tingidos_em(&mancha, 54, 54, 20),
        400,
        "o Solid não encheu o miolo da aguada"
    );
}

/// **A MANCHA DE UM QUADRO NÃO FICA NO SEGUINTE** — o caminho dá a volta ao quadrado e regressa
/// pelo meio, para a forma FINAL ter um entalhe (o triângulo entre o canto de partida e a perna que
/// volta) que as formas INTERMÉDIAS cobriam. Se a mancha provisória ficasse nos acumuladores da
/// aguada, o entalhe saía tingido.
#[test]
fn a_mancha_provisoria_nao_deixa_resto_na_aguada() {
    let volta = [
        [20.0, 20.0],
        [108.0, 20.0],
        [108.0, 108.0],
        [20.0, 108.0],
        [20.0, 60.0],
        [90.0, 60.0],
    ];
    let t = gesto(true, &volta, 12);
    // O entalhe: dentro do triângulo (20,20)–(20,60)–(90,60), longe do rastro (y 20 e 60) e da corda.
    assert!(
        tingidos_em(&t, 64, 80, 16) > 200,
        "controlo: a forma final tem de estar cheia"
    );
    assert_eq!(
        tingidos_em(&t, 27, 41, 8),
        0,
        "a mancha dos quadros intermédios ficou no entalhe da forma final"
    );
}

/// **A MANCHA NÃO PISCA QUANDO O TIQUE CARIMBA** (smoke do dono, 2026-10-04: *«esporadicamente a
/// área de preenchimento pisca»*). O lote do tique descasca a mancha, e o composite do quadro tem de
/// correr DEPOIS de ela voltar. Com a caneta parada o tique carimba sozinho — o `settle` do Space (o
/// pincel de fábrica) e o Airbrush, que carimba em todo tique, mesmo no quadro com movimento.
#[test]
fn a_mancha_nao_some_no_quadro_em_que_o_tique_carimba() {
    let laco = [
        [24.0, 24.0],
        [104.0, 24.0],
        [104.0, 104.0],
        [24.0, 104.0],
        [24.0, 28.0],
    ];
    // A rota congelada de um composite por evento (`wash.per_event`) compõe no próprio Move.
    for (metodo, por_evento) in [
        (StrokeMethod::Space, false),
        (StrokeMethod::Airbrush, false),
        (StrokeMethod::Space, true),
    ] {
        let mut t = tool(128, PaintMedia::Watercolor, 4.0);
        t.set_brush_color_srgb8([30, 60, 220]);
        t.paint.brush.stroke_method = metodo;
        t.wash.per_event = por_evento;
        t.paint.brush.style_solid = true;
        t.on_canvas_pointer(cp(laco[0], PointerPhase::Down));
        for w in laco.windows(2) {
            for k in 1..=10 {
                #[allow(clippy::cast_precision_loss)]
                let f = k as f32 / 10.0;
                t.on_canvas_pointer(cp(
                    [
                        w[0][0] + (w[1][0] - w[0][0]) * f,
                        w[0][1] + (w[1][1] - w[0][1]) * f,
                    ],
                    PointerPhase::Move,
                ));
                // Um quadro por evento, como no app: o Airbrush só deixa tinta no tique.
                t.paint_tick(0.1);
            }
        }
        assert_eq!(
            tingidos_em(&t, 54, 54, 20),
            400,
            "{metodo:?} (por evento: {por_evento}): a mancha sumiu no quadro do movimento"
        );
        for quadro in 0..4 {
            t.paint_tick(0.25);
            assert_eq!(
                tingidos_em(&t, 54, 54, 20),
                400,
                "{metodo:?} (por evento: {por_evento}): a mancha sumiu no quadro parado {quadro}"
            );
        }
    }
}

/// **UMA ELIPSE EM SOLID É UM DISCO NA AGUADA** — o editor de forma reconstrói a aguada inteira a
/// cada quadro, e a região entra nela.
#[test]
fn uma_elipse_em_solid_e_um_disco_na_aguada() {
    let elipse = |solid: bool| {
        let mut t = tool(128, PaintMedia::Watercolor, 3.0);
        t.paint.brush.style_solid = solid;
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([104.0, 64.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([104.0, 64.0], PointerPhase::Up));
        t
    };
    assert_eq!(
        tingidos_em(&elipse(false), 54, 54, 20),
        0,
        "controlo: o anel tem o miolo vazio"
    );
    assert_eq!(
        tingidos_em(&elipse(true), 54, 54, 20),
        400,
        "a elipse em Solid não encheu o miolo na aguada"
    );
}
