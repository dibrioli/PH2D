//! ⭐⭐ **OS GATES DO EDITOR GRÁFICO DO EASING** (ciclo 17, doc 126 §6) — abre?, a miniatura
//! escreve a família?, a alça arrastada CHEGA ao nó pela porta dos números?

use super::{EditorKind, Open, key_of, on_click};
use crate::snapshot::{CardParam, EaseView, GraphIntent, drain_intents, set_card_eases};
use crate::state::MotionGraphPanelState;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::{ParamUiHint, ParamWidget};
use ph2d_param_editors::EditorKey;

const NODE: u32 = 9;
const PARAM: &str = "ease_curve";

fn vista(curve: i32) -> EaseView {
    EaseView {
        curve,
        dir: 0,
        shape: ph2d_ease::Shape::DEFAULT,
        samples: [0.0; crate::snapshot::EASE_AMOSTRAS],
    }
}

fn ease_param() -> CardParam {
    CardParam::from_hint(
        ParamUiHint {
            param: PARAM,
            label: "Ease",
            min: 0.0,
            max: 14.0,
            step: 1.0,
            widget: ParamWidget::Ease,
        },
        0.0,
    )
}

fn aberto() -> MotionGraphPanelState {
    MotionGraphPanelState {
        editor: Some(Open {
            node: NODE,
            param: PARAM,
            title: "Ease",
            kind: EditorKind::Ease,
            screen: (100.0, 100.0),
        }),
        ..MotionGraphPanelState::default()
    }
}

/// O clique na row `Ease` abre a janela (com a curva publicada) e não escreve nada.
#[test]
fn a_click_on_the_ease_row_opens_the_graph_editor() {
    let mut st = MotionGraphPanelState::default();
    let _ = drain_intents();
    set_card_eases(Vec::new());
    super::arm(
        &mut st,
        NODE,
        &ease_param(),
        Rect::new(10.0, 10.0, 190.0, 22.0),
    );
    assert!(st.editor.is_none(), "sem curva publicada nao abre");
    set_card_eases(vec![(NODE, PARAM, vista(0))]);
    super::arm(
        &mut st,
        NODE,
        &ease_param(),
        Rect::new(10.0, 10.0, 190.0, 22.0),
    );
    assert_eq!(st.editor.as_ref().map(|o| o.kind), Some(EditorKind::Ease));
    assert!(drain_intents().is_empty(), "abrir nao e' editar");
    set_card_eases(Vec::new());
}

/// Uma miniatura escreve a FAMÍLIA pela porta dos números (`SetParam`), e um id alheio não é
/// consumido.
#[test]
fn a_tile_writes_the_family_as_a_number() {
    let st = aberto();
    set_card_eases(vec![(NODE, PARAM, vista(0))]);
    let (own, swatch) = key_of(NODE, PARAM);
    let key = EditorKey {
        own: &own,
        swatch: &swatch,
    };
    let _ = drain_intents();
    assert!(on_click(&st, key.sub("fam/12")));
    assert_eq!(
        drain_intents(),
        vec![GraphIntent::SetParam {
            node: NODE,
            param: "ease_curve",
            value: 12.0
        }]
    );
    assert!(!on_click(&st, ph2d_param_editors::fnv_id("um/id/qualquer")));
    set_card_eases(Vec::new());
}

/// ⭐⭐ **A alça do `Bezier` arrastada chega ao nó** — pelo `apply_event` REAL do painel (o mesmo
/// caminho do editor de curva), com as duas escritas da alça.
#[test]
fn dragging_a_bezier_handle_writes_both_numbers() {
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::PanelHostInternal;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    let mut st = aberto();
    set_card_eases(vec![(NODE, PARAM, vista(ph2d_ease::BEZIER))]);
    let _ = drain_intents();
    let (own, swatch) = key_of(NODE, PARAM);
    let key = EditorKey {
        own: &own,
        swatch: &swatch,
    };
    host.store_mut()
        .set_curve_point_drag(key.root(), 0, 0, 0.3, 0.75);
    let saida = host.apply_panel_event::<crate::MotionGraphPanel>(
        &mut st,
        WidgetEvent::ValueChanged(key.root()),
    );
    assert_eq!(saida, ph2d_editor_core::panel::EventOutcome::Consumed);
    let (lo, hi) = ph2d_param_editors::ease::Y_FAIXA;
    let y = lo + 0.75 * (hi - lo);
    let saiu = drain_intents();
    assert_eq!(
        saiu,
        vec![
            GraphIntent::SetParam {
                node: NODE,
                param: "bez_x1",
                value: 0.3
            },
            GraphIntent::SetParam {
                node: NODE,
                param: "bez_y1",
                value: y
            },
        ]
    );
    set_card_eases(Vec::new());
}

/// ⭐ **A JANELA PINTA o que promete, DENTRO do painel**: as quinze miniaturas, as três direções
/// (numa família que a lê) e a alça da forma ficam registadas para o dedo — pela pintura REAL do
/// painel, com o painel do grafo no retângulo MEDIDO no app (`1310 × 205,5` px, a timeline aberta).
/// ⛔ A 1.ª redacção empilhava tudo em `~400 px` e o gráfico ficava fora do painel.
#[test]
fn the_window_paints_every_tile_and_the_shape_handle_inside_the_panel() {
    use ph2d_editor_core::screens::layout::{CenterSplit, HeroLayout};
    let janela = Rect::new(0.0, 0.0, 1930.0, 1012.0);
    let mut layout = HeroLayout::for_viewport_split(
        janela,
        false,
        ph2d_editor_core::screens::layout::rail_w(),
        CenterSplit::Horizontal {
            t: CenterSplit::T_DEFAULT,
        },
    );
    let painel = Rect::new(312.0, 566.5, 1310.0, 205.5); // LITERAL-PX-OK: medido no app
    layout.motion_graph = painel;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    let mut st = aberto();
    if let Some(o) = st.editor.as_mut() {
        o.screen = (900.0, 700.0); // aberta a meio do painel, como uma row de cartão
    }
    set_card_eases(vec![(NODE, PARAM, vista(ph2d_ease::BACK))]);
    let hits = host.paint_with_layout::<crate::MotionGraphPanel>(&mut st, layout, janela);
    let (own, swatch) = key_of(NODE, PARAM);
    let key = EditorKey {
        own: &own,
        swatch: &swatch,
    };
    let rect_de = |id| hits.iter().find(|(h, _)| *h == id).map(|(_, r)| *r);
    let mut ids: Vec<ph2d_a11y::NodeId> = (0..=ph2d_ease::LAST)
        .map(|f| key.sub(&format!("fam/{f}")))
        .collect();
    ids.extend((0..3).map(|d| key.sub(&format!("dir/{d}"))));
    ids.push(key.sub("alca/0"));
    for id in ids {
        let r = rect_de(id).unwrap_or_else(|| panic!("{id:?} nao foi pintado"));
        let centro = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            centro.0 > painel.x
                && centro.0 < painel.x + painel.w
                && centro.1 > painel.y
                && centro.1 < painel.y + painel.h,
            "{id:?} em {r:?} saiu do painel {painel:?}"
        );
    }
    set_card_eases(Vec::new());
}
