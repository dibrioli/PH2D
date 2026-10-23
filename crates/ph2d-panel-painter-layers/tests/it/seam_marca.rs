//! **As linhas da MARCA pelo ponteiro** (L4, `docs/Painter/Pinceis/03_MOTORES.md` §4): o cartão Jitter pinta uma
//! linha por `LINHAS_DA_MARCA` nos quatro meios, e digitar numa delas muda o `dab_jitter` do pincel com que o
//! tool pinta (`BrushSettings::marca`).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{CartaoDaMarca, LINHAS_DA_MARCA, PaintMedia, PainterTool, ids};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 4000.0)
}

/// O painel pintado com a secção Randomize Color e a sub-secção Jitter ABERTAS (nascem recolhidas: o clique no
/// cabeçalho abre cada uma, como o artista faz).
fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let mut rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    for (cab, dentro) in [
        (
            ids::PAINTER_BRUSH_RANDOMIZE_SECTION,
            ids::PAINTER_MARCA_SECONDARY,
        ),
        (ids::PAINTER_BRUSH_JITTER_SECTION, ids::PAINTER_BRUSH_JITTER),
    ] {
        if rect_of(&rects, dentro).is_some() {
            continue;
        }
        let r = rect_of(&rects, cab).expect("o cabeçalho da secção");
        for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        let _ = host.drained_actions();
        host.settle_section_folds();
        set_current_brush(Some(tool.brush_settings()));
        rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    }
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// ⭐ As linhas da marca estão nos seus cartões, nos quatro meios: as do jitter (L4) no cartão Jitter por baixo do
/// Unit (a ordem entre elas e as do pincel é a do dono, 23/10 — `o_cartao_jitter_tem_o_unit_no_topo_e_so_barras`),
/// as da cor pela mão (L11) na Randomize Color por baixo da amostra da secundária, pela ordem da tabela.
#[test]
fn o_cartao_jitter_pinta_as_linhas_da_marca() {
    for m in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let mut t = PainterTool::default();
        t.set_paint_media(m);
        let (_h, _s, rects) = painted(&t);
        let unit = rect_of(&rects, ids::PAINTER_BRUSH_JITTER_UNIT).expect("o Unit do cartão");
        let sec = rect_of(&rects, ids::PAINTER_MARCA_SECONDARY).expect("a amostra da secundária");
        let mut yc = sec.y;
        for (l, id) in LINHAS_DA_MARCA.into_iter().zip(ids::PAINTER_MARCA_FIELDS) {
            let r = rect_of(&rects, id)
                .unwrap_or_else(|| panic!("{m:?}: a linha {} não pintada", l.chave()));
            if l.cartao() == CartaoDaMarca::Jitter {
                assert!(r.y > unit.y, "{m:?}: {} acima do Unit", l.chave());
                continue;
            }
            assert!(r.y > yc, "{m:?}: {} fora de ordem", l.chave());
            yc = r.y;
        }
    }
}

/// ⭐ Digitar numa linha muda o pincel do tool — cada uma, pela ordem da tabela. As do cartão Jitter são barras
/// (dono 23/10): digita-se no CHIP ligado, que aceita o valor verdadeiro.
#[test]
fn digitar_uma_linha_da_marca_muda_o_pincel() {
    let mut t = PainterTool::default();
    for (i, id) in ids::PAINTER_MARCA_FIELDS.into_iter().enumerate() {
        let (mut host, mut st, rects) = painted(&t);
        assert!(rect_of(&rects, id).is_some());
        let alvo = match host.store().get(id) {
            Some(InteractiveState::Slider { .. }) => {
                host.store().linked_number(id).expect("o chip")
            }
            _ => id,
        };
        for ev in host.type_into_number(alvo, "0.5") {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        for action in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = action {
                t.handle_panel_event(pe);
            }
        }
        assert_eq!(
            t.brush_settings().marca[i],
            0.5,
            "{}",
            LINHAS_DA_MARCA[i].chave()
        );
    }
}

/// ⭐ O clique na amostra da secundária abre o seletor apontado para ela, e a escolha volta ao pincel do tool.
#[test]
fn a_amostra_da_secundaria_abre_o_seletor_e_volta_ao_pincel() {
    let mut t = PainterTool::default();
    let (mut host, mut st, rects) = painted(&t);
    let r = rect_of(&rects, ids::PAINTER_MARCA_SECONDARY).unwrap();
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    assert_eq!(
        host.store().picker_target(),
        Some(ids::PAINTER_MARCA_SECONDARY)
    );
    host.store_mut()
        .set_widget_color(ids::PAINTER_MARCA_SECONDARY, [255, 0, 51, 255]);
    let _ = host.paint::<PainterLayersPanel>(&mut st, viewport());
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            t.handle_panel_event(pe);
        }
    }
    assert_eq!(t.brush_settings().marca_secundaria, [1.0, 0.0, 0.2]);
}

/// O id de cada fileira da marca pela linha (nunca por índice escrito à mão).
fn marca(l: ph2d_tool_painter::LinhaDaMarca) -> NodeId {
    let i = LINHAS_DA_MARCA.iter().position(|x| *x == l).unwrap();
    ids::PAINTER_MARCA_FIELDS[i]
}

/// ⭐⭐ **Ordem do dono (23/10): «Jitter Unit deveria ir para o topo da seção e os parâmetros numéricos deveriam
/// ser todos sliders»** — o Unit abre o cartão e cada número é uma barra com o chip ligado, agrupados pelo que
/// sorteiam: quantas · onde · quando · tamanho e forma · ângulo · espelho · aparência.
///
/// Pelo painel REAL: a ordem em `y` das 14 fileiras, o registo (Slider + chip ligado, nenhuma caixa nua), e o gesto
/// — arrastar a barra do Scatter até ao fim e digitar no chip dela — chega ao pincel na faixa VERDADEIRA dela
/// (`0..ACROSS_MAX`, a única que não é `0..1`: uma pista mapeada como `0..1` pararia a meio).
#[test]
fn o_cartao_jitter_tem_o_unit_no_topo_e_so_barras() {
    use ph2d_tool_painter::LinhaDaMarca as L;
    let ordem: [(NodeId, &str); 14] = [
        (ids::PAINTER_BRUSH_JITTER_UNIT, "Unit"),
        (ids::PAINTER_BRUSH_SPRAY_COUNT, "Count"),
        (marca(L::Count), "Count jitter"),
        (marca(L::Particle), "Particle size"),
        (ids::PAINTER_BRUSH_JITTER, "Position"),
        (marca(L::Across), "Scatter"),
        (ids::PAINTER_BRUSH_JITTER_SPACING, "Spacing"),
        (ids::PAINTER_BRUSH_JITTER_SCALE, "Scale"),
        (marca(L::Roundness), "Roundness"),
        (ids::PAINTER_BRUSH_JITTER_ROTATE, "Rotation"),
        (marca(L::AnglePerStroke), "Stroke angle"),
        (marca(L::FlipX), "Flip X"),
        (marca(L::FlipY), "Flip Y"),
        (marca(L::Opacity), "Opacity jitter"),
    ];
    let mut t = PainterTool::default();
    let (mut host, mut st, rects) = painted(&t);
    let mut y_antes = f32::NEG_INFINITY;
    for (id, nome) in ordem {
        let r = rect_of(&rects, id).unwrap_or_else(|| panic!("{nome}: não pintada"));
        assert!(
            r.y > y_antes,
            "{nome} fora da ordem do dono (y={} ≤ {y_antes})",
            r.y
        );
        y_antes = r.y;
        if id == ids::PAINTER_BRUSH_JITTER_UNIT {
            continue;
        }
        assert!(
            matches!(host.store().get(id), Some(InteractiveState::Slider { .. })),
            "{nome}: tem de ser uma BARRA, não uma caixa de número"
        );
        let chip = host
            .store()
            .linked_number(id)
            .unwrap_or_else(|| panic!("{nome}: a barra sem chip ligado"));
        assert!(
            matches!(
                host.store().get(chip),
                Some(InteractiveState::NumberInput { .. })
            ),
            "{nome}: o chip ligado não é um número"
        );
        assert!(
            rect_of(&rects, chip).is_some(),
            "{nome}: o chip não foi pintado"
        );
    }
    // O gesto: arrastar a barra do Scatter até ao fim pousa no TECTO verdadeiro dela.
    let (_, hi, _) = L::Across.faixa();
    let i = LINHAS_DA_MARCA
        .iter()
        .position(|x| *x == L::Across)
        .unwrap();
    let r = rect_of(&rects, marca(L::Across)).unwrap();
    let y = r.y + r.h * 0.5;
    let evs = host.drag_at(r.x + 1.0, y, r.x + r.w + 40.0, y);
    for ev in evs {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            t.handle_panel_event(pe);
        }
    }
    assert_eq!(
        t.brush_settings().marca[i],
        hi,
        "a barra do Scatter até ao fim"
    );
    // E digitar no chip dela escreve o número EXACTO, fora de `0..1`.
    let chip = host.store().linked_number(marca(L::Across)).unwrap();
    for ev in host.type_into_number(chip, "1.37") {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            t.handle_panel_event(pe);
        }
    }
    assert_eq!(t.brush_settings().marca[i], 1.37, "o chip do Scatter");
}
