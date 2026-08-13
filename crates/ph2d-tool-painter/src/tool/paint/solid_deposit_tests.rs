//! Os gates do `Style: Solid` no PRODUTO ([`super::solid_deposit`]) — o motor tem os seus em
//! `ph2d_painter_brush::solid`; aqui pergunta-se o que o **gesto** faz.

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::PanelEvent;

/// Quantos texels do canvas deixaram de ser o branco de fundo.
fn inked(t: &crate::tool::PainterTool) -> usize {
    t.canvas_rgba.chunks_exact(4).filter(|p| p[0] < 250).count()
}

/// Desenha um laço quadrado de lado `s` centrado em `c`, e devolve os texels entintados.
fn loop_gesture(t: &mut crate::tool::PainterTool, c: f32, s: f32) -> usize {
    let pts = [
        [c - s, c - s],
        [c + s, c - s],
        [c + s, c + s],
        [c - s, c + s],
        [c - s, c - s],
    ];
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    // ⚠️ Vários passos por aresta, e por `windows(2)`: o primeiro ponto REPETE no fim (o laço fecha),
    // então procurar o índice de um ponto na lista devolve o do começo — a fixture nasceu com esse
    // defeito e o gate estourou nele antes de medir coisa nenhuma.
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        for k in 1..=8 {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / 8.0;
            t.on_canvas_pointer(cp(
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
                PointerPhase::Move,
            ));
        }
    }
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Up));
    inked(t)
}

/// **UM LAÇO FINO E SOLTO PINTA UMA MANCHA GORDA** — o `Style` do Alchemy, e o pedido do Enio
/// (*"para forma sólida a espessura da linha passa a não ser considerada"*) na sua forma medível.
///
/// ⚠️ O oráculo é a RAZÃO contra o mesmo gesto em Line, não um número absoluto: o que a feature
/// promete é que a tinta deixa de ser o rastro do pincel e passa a ser a área cercada.
#[test]
fn a_thin_loop_in_solid_paints_the_region_it_encircles() {
    let side = 256u32;
    let mut line = tool(side, PaintMedia::Digital, 4.0);
    let as_line = loop_gesture(&mut line, 128.0, 60.0);

    let mut solid = tool(side, PaintMedia::Digital, 4.0);
    solid.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let as_solid = loop_gesture(&mut solid, 128.0, 60.0);

    // A área cercada é 120×120 = 14 400; o rastro de um pincel de raio 4 sobre o perímetro é ~4 000.
    assert!(
        as_solid > as_line * 2,
        "Solid nao encheu a regiao: {as_solid} texels contra {as_line} do Line"
    );
    assert!(
        as_solid > 13_000,
        "a mancha deveria cobrir a area cercada (~14 400): {as_solid}"
    );
}

/// **A ESPESSURA NÃO ENTRA** — o mesmo gesto com dois raios de pincel tem de pintar o MESMO número de
/// texels em Solid, e números bem diferentes em Line.
///
/// ⚠️ A segunda metade é o CONTROLE: sem ela o gate passaria num produto que simplesmente ignora o
/// pincel em toda parte.
#[test]
fn in_solid_the_brush_width_does_not_enter_and_in_line_it_does() {
    let side = 256u32;
    let mut a = tool(side, PaintMedia::Digital, 3.0);
    a.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let thin = loop_gesture(&mut a, 128.0, 60.0);

    let mut b = tool(side, PaintMedia::Digital, 24.0);
    b.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let thick = loop_gesture(&mut b, 128.0, 60.0);
    assert_eq!(thin, thick, "a espessura mudou a forma solida");

    let mut c = tool(side, PaintMedia::Digital, 3.0);
    let line_thin = loop_gesture(&mut c, 128.0, 60.0);
    let mut d = tool(side, PaintMedia::Digital, 24.0);
    let line_thick = loop_gesture(&mut d, 128.0, 60.0);
    assert!(
        line_thick > line_thin + 1000,
        "controle: em Line a espessura TEM de entrar ({line_thin} contra {line_thick})"
    );
}

/// **DESMARCADO É BYTE-IDÊNTICO** — a rede que torna toda esta wave reversível.
#[test]
fn the_unticked_checkbox_leaves_the_canvas_byte_identical() {
    let side = 128u32;
    let mut a = tool(side, PaintMedia::Digital, 6.0);
    let _ = loop_gesture(&mut a, 64.0, 30.0);
    let before: Vec<u8> = a.canvas_rgba.to_vec();

    // Ligar e DESLIGAR devolve o estado — o toggle não pode deixar resíduo.
    let mut b = tool(side, PaintMedia::Digital, 6.0);
    b.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    b.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let _ = loop_gesture(&mut b, 64.0, 30.0);
    assert_eq!(before, *b.canvas_rgba, "o traço mudou com o checkbox desligado");
}

/// **O CHECKBOX ALCANÇA A FERRAMENTA** — o clique do painel flipa o estado publicado, nos dois
/// sentidos, e ele chega aos três slots de relevo (Deposit / Faca / Sculpt são UM assunto).
#[test]
fn the_panel_click_reaches_the_tool_and_every_relief_slot() {
    let mut t = tool(64, PaintMedia::Digital, 8.0);
    assert!(!t.brush_settings().style_solid, "o default tem de ser Line");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    assert!(t.brush_settings().style_solid, "o clique nao chegou ao tool");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    assert!(!t.brush_settings().style_solid, "o clique nao volta");
}
