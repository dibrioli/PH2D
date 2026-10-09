//! **As gotas do papel, do painel para dentro** (W1c, auditoria de 08/10): a dica diz o anel que a
//! LEI dá, a fábrica explica porque não há gotas sem abrir o Paper, e os campos inteiros arrastam
//! inteiros.

use ph2d_a11y::NodeId;
use ph2d_editor_core::Tool;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_i18n::tr;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::ids::*;
use ph2d_tool_painter::{Dependente, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 4000.0)
}

fn molhado() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_wetpaint_armed(true);
    t
}

/// O primeiro anel do dial em que a forma da gota AGE — perguntado à ferramenta pela porta do painel
/// (a lei dela é a do motor: `a_lei_de_inercia_das_gotas_e_a_do_motor`).
fn primeiro_anel_da_lei() -> u8 {
    (0u8..=8)
        .find(|&r| {
            let mut t = molhado();
            t.handle_panel_event(PanelEvent::SetValue(
                PAINTER_WETPAINT_TILT_RING,
                f64::from(r),
            ));
            !t.brush_settings().inercias.inerte(Dependente::FormaDaGota)
        })
        .expect("algum anel do dial deixa as gotas deslizar")
}

fn pinta(tool: &PainterTool, abre: bool) -> (MockPanelHost, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let _ = host.paint::<PainterLayersPanel>(&mut st, viewport());
    if abre {
        host.open_all_sections();
    }
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, rects)
}

fn pintado(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// ⭐ **A dica das gotas diz o anel que a LEI dá** — com a tela direita, a forma da gota (Easy, Size,
/// Length) e o Create Drips esmaecem, e a dica de cada um nomeia o «Tilt» e o primeiro anel em que uma
/// gota desliza. Recalibrar a física muda o número no texto sem tocar nele.
#[test]
fn a_dica_das_gotas_diz_o_anel_da_lei() {
    let anel = primeiro_anel_da_lei().to_string();
    let (host, _) = pinta(&molhado(), true);
    for id in [
        PAINTER_WETPAINT_DRIPS_CREATE,
        PAINTER_WETPAINT_DRIPS_EASY,
        PAINTER_WETPAINT_DRIP_SIZE,
        PAINTER_WETPAINT_DRIP_LENGTH,
    ] {
        let dica = host
            .store()
            .tooltip_for(id)
            .unwrap_or_else(|| panic!("{id:?} não esmaeceu com a tela direita"));
        assert!(
            dica.contains(&anel),
            "a dica de {id:?} não diz o anel {anel} da lei: «{dica}»"
        );
        assert!(
            dica.contains(tr("panel.painter_layers.wetpaint.tilt")),
            "a dica de {id:?} não nomeia o Tilt: «{dica}»"
        );
    }
}

/// ⭐ **De fábrica, «porque não há gotas» está à vista** — o Paper nasce recolhido (as rows das gotas
/// nem se pintam), então quem o diz é o Tilt, que está na dobra: a dica dele nomeia o Create Drips, o
/// Paper e o anel mínimo da lei.
#[test]
fn de_fabrica_o_tilt_diz_porque_nao_ha_gotas() {
    let anel = primeiro_anel_da_lei().to_string();
    let (host, rects) = pinta(&molhado(), false);
    assert!(
        pintado(&rects, PAINTER_WETPAINT_DRIPS_CREATE).is_none(),
        "fixture: o Paper já não nasce recolhido — este gate mede outra coisa"
    );
    for id in [PAINTER_WETPAINT_TILT_TOGGLE, PAINTER_WETPAINT_TILT_PAD] {
        assert!(pintado(&rects, id).is_some(), "{id:?} não está na dobra");
        let dica = host.store().tooltip_for(id).unwrap_or("");
        for parte in [
            anel.as_str(),
            tr("panel.painter_layers.paper.create_drips"),
            tr("panel.painter_layers.paper.paper"),
        ] {
            assert!(
                dica.contains(parte),
                "a dica de {id:?} não diz «{parte}»: «{dica}»"
            );
        }
    }
}

/// ⭐ **Drip Size e Drip Length arrastam INTEIROS** — o arrasto real (Down, Moves, Up) sobre a caixa:
/// todo valor que o painel ENCAMINHA é inteiro, e a caixa focada mostra um inteiro enquanto se arrasta.
#[test]
fn os_campos_das_gotas_arrastam_inteiros() {
    for id in [PAINTER_WETPAINT_DRIP_SIZE, PAINTER_WETPAINT_DRIP_LENGTH] {
        let mut tool = molhado();
        let (mut host, rects) = pinta(&tool, true);
        let mut st = PainterLayersPanelState;
        let r = pintado(&rects, id).expect("a caixa foi pintada");
        let (x0, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        let mut relogio = 0u128;
        let mut ponteiro = |host: &mut MockPanelHost, kind, x: f32| {
            relogio += 16_000_000;
            host.dispatch_pointer_event(PointerEvent {
                x,
                y,
                pressure: 1.0,
                kind,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: relogio,
            })
        };
        let mut encaminhados = Vec::new();
        let mut caixas = Vec::new();
        let _ = ponteiro(&mut host, PointerKind::Down, x0);
        for k in 1..=24 {
            let eventos = ponteiro(&mut host, PointerKind::Move, x0 + 7.0 * k as f32);
            for ev in eventos {
                let _ = host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
            }
            for a in host.drained_actions() {
                if let EditorAction::ToolPanelEvent(pe) = a {
                    if let PanelEvent::SetValue(i, v) = pe
                        && i == id
                    {
                        encaminhados.push(v);
                    }
                    tool.handle_panel_event(pe);
                }
            }
            set_current_brush(Some(tool.brush_settings()));
            let _ = host.paint::<PainterLayersPanel>(&mut st, viewport());
            let (_, _, buf, _, _) =
                ph2d_editor_core::widget::showcase::read_number_input(host.store(), id);
            caixas.push(buf.to_string());
        }
        let _ = ponteiro(&mut host, PointerKind::Up, x0 + 7.0 * 24.0);
        assert!(
            encaminhados.iter().any(|v| *v != 5.0),
            "{id:?}: o arrasto não encaminhou valor nenhum (controlo): {encaminhados:?}"
        );
        assert!(
            encaminhados.iter().all(|v| v.fract() == 0.0),
            "{id:?}: o painel encaminhou valores com casas: {encaminhados:?}"
        );
        assert!(
            caixas.iter().all(|b| !b.contains('.')),
            "{id:?}: a caixa mostra casas enquanto se arrasta: {caixas:?}"
        );
    }
}
