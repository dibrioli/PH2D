//! A costura das FORMAS do Vector no painel PIXEL (P6b, smoke do dono 26/10: *«os polígonos
//! deveriam ser os mesmos do módulo vector, com opções específicas no painel à direita como
//! fillet»*): cada campo do catálogo do Vector pinta-se para a forma na mão, carregado pelo rato (ou
//! escrito, com Enter) chega às opções; nenhum campo de forma se pinta sem forma na mão.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_panel::{
    PixelOptions, PixelPanelInfo, PixelRequest, PixelShape, PixelShapeValues, PixelTool,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_tools::{
    PixelToolsPanel, PixelToolsPanelState, SHAPE_FILL, set_current_pixel, shape_desc,
    shape_field_id, shape_option_id,
};
use ph2d_tool_vector::shapes::FieldUnit;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 6000.0,
};

fn info(tool: PixelTool) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        tool,
        palette: vec![[0, 0, 0, 255]],
        size: (64, 32),
        ..Default::default()
    }
}

fn pinta(
    i: PixelPanelInfo,
) -> (
    MockPanelHost,
    PixelToolsPanelState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
    let mut st = PixelToolsPanelState::default();
    set_current_pixel(i);
    let rects = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
    (host, st, rects)
}

fn aplica(
    host: &mut MockPanelHost,
    st: &mut PixelToolsPanelState,
    ev: Vec<WidgetEvent>,
) -> Vec<EditorAction> {
    for e in ev {
        let _ = host.apply_panel_event::<PixelToolsPanel>(st, e);
    }
    host.drained_actions()
}

fn com(s: PixelShape, i: usize, v: f64) -> PixelRequest {
    let mut o = PixelOptions::default();
    o.shapes.set(s, o.shapes.get(s).with(i, v));
    PixelRequest::Options(o)
}

/// O raio do Vector é de MUNDO: no Pixel todo campo em píxeis nasce a zero (o canto vivo do
/// Aseprite) — e os outros nascem os de omissão do Vector.
#[test]
fn every_vector_pixel_field_is_born_at_zero_cells_and_the_rest_at_the_vectors_default() {
    for s in PixelShape::ALL {
        let v = PixelShapeValues::defaults(s);
        let vec = s.kind().defaults();
        for (i, f) in shape_desc(s).fields.iter().enumerate() {
            let quer = if f.unit == FieldUnit::Px { 0.0 } else { vec[i] };
            assert_eq!(v.get(i), quer, "{s:?} campo {i} «{}»", f.label);
        }
    }
}

#[test]
fn each_shape_shows_the_vector_fields_and_each_reaches_the_options() {
    for (tool, s) in [
        (PixelTool::Rectangle, PixelShape::Rectangle),
        (PixelTool::FilledEllipse, PixelShape::Ellipse),
        (PixelTool::RegularPolygon, PixelShape::Polygon),
        (PixelTool::Star, PixelShape::Star),
    ] {
        for (i, f) in shape_desc(s).fields.iter().enumerate() {
            let (mut host, mut st, rects) = pinta(info(tool));
            let achado = |id| rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r);
            match f.unit {
                FieldUnit::Choice(nomes) => {
                    let j = nomes.len() - 1;
                    let id = shape_option_id(s, i, j);
                    let r =
                        achado(id).unwrap_or_else(|| panic!("{s:?}: «{}» não se pintou", f.label));
                    let ev = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
                    #[expect(clippy::cast_precision_loss, reason = "índice de escolha")]
                    let quer = com(s, i, j as f64);
                    assert_eq!(aplica(&mut host, &mut st, ev), [EditorAction::Pixel(quer)]);
                }
                _ => {
                    let id = shape_field_id(s, i);
                    assert!(
                        achado(id).is_some(),
                        "{s:?}: o campo «{}» não se pintou",
                        f.label
                    );
                    // Um quarto da faixa: nunca o valor de omissão (o meio de ±500 é o 0 que já lá está).
                    let v = f.min + (f.max - f.min) / 4.0;
                    let v = if f.unit == FieldUnit::Count {
                        v.round()
                    } else {
                        v
                    };
                    let ev = host.type_into_number(id, &v.to_string());
                    let a = aplica(&mut host, &mut st, ev);
                    let Some(EditorAction::Pixel(PixelRequest::Options(o))) = a.first() else {
                        panic!("{s:?} «{}»: {a:?}", f.label);
                    };
                    let got = o.shapes.get(s).get(i);
                    assert!((got - v).abs() < 1e-9, "{s:?} «{}»: {got} ≠ {v}", f.label);
                }
            }
        }
    }
}

#[test]
fn the_polygon_and_the_star_have_fill_and_no_shape_field_paints_without_a_shape() {
    for tool in [PixelTool::RegularPolygon, PixelTool::Star] {
        let (mut host, mut st, rects) = pinta(info(tool));
        let r = rects
            .iter()
            .find(|(n, _)| *n == SHAPE_FILL)
            .map(|(_, r)| *r)
            .expect("Fill");
        let ev = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        let quer = PixelOptions {
            shape_fill: true,
            ..PixelOptions::default()
        };
        assert_eq!(
            aplica(&mut host, &mut st, ev),
            [EditorAction::Pixel(PixelRequest::Options(quer))]
        );
    }
    let (_, _, rects) = pinta(info(PixelTool::Pencil));
    for s in PixelShape::ALL {
        for i in 0..shape_desc(s).fields.len() {
            assert!(rects.iter().all(|(n, _)| *n != shape_field_id(s, i)));
        }
    }
    assert!(rects.iter().all(|(n, _)| *n != SHAPE_FILL));
}
