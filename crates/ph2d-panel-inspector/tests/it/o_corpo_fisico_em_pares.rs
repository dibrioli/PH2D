//! ⭐ **Physics Body na lei da compactação** (R5, foto `docs/CartoesUI/maquete/r4_physics_body.png`):
//! Mass + Density e Trigger + One-Way são irmãos curtos, e cada opção continua a pedir o que pedia.

use crate::seam_physics::with_body;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids::{
    INSP_PHYS_DENSITY, INSP_PHYS_MASSMODE, INSP_PHYS_ONEWAY, INSP_PHYS_SENSOR, INSP_PHYS_WALLMAT,
};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_physics,
};

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 2400.0,
};

fn palco() -> (
    ph2d_ui_testkit::MockPanelHost,
    InspectorState,
    Vec<(NodeId, Rect)>,
) {
    set_cards_on(true);
    let mut info = with_body();
    info.mass_is_read = true;
    set_current_inspector_physics(Some(info));
    let mut host = crate::host_em_metros();
    let mut st = InspectorState::default();
    crate::o_inspector_em_cartoes_por_tipo::abre_tudo(&mut host);
    let _ = crate::o_inspector_em_cartoes_por_tipo::pinta(&mut host, &mut st);
    crate::o_inspector_em_cartoes_por_tipo::abre_tudo(&mut host);
    let rects = crate::o_inspector_em_cartoes_por_tipo::pinta(&mut host, &mut st);
    (host, st, rects)
}

fn r(rects: &[(NodeId, Rect)], id: NodeId) -> Rect {
    rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
}

fn lado_a_lado(a: Rect, b: Rect, nomes: &str) {
    assert!(
        (a.y + a.h * 0.5 - (b.y + b.h * 0.5)).abs() < 0.5 && b.x >= a.x + a.w,
        "{nomes}: {a:?} × {b:?}"
    );
}

/// Foto `r5_physics_body`. *Mutação: `corpo::choices` sem o pedido ao fluxo ⇒ vermelho.*
#[test]
fn mass_density_and_trigger_one_way_share_the_line() {
    let (_, _, rects) = palco();
    lado_a_lado(
        r(&rects, INSP_PHYS_MASSMODE[1]),
        r(&rects, INSP_PHYS_DENSITY),
        "Mass + Density",
    );
    lado_a_lado(
        r(&rects, INSP_PHYS_SENSOR[1]),
        r(&rects, INSP_PHYS_ONEWAY[0]),
        "Trigger + One-Way",
    );
    set_current_inspector_physics(None);
    set_cards_on(false);
}

/// As opções que passaram a meia largura continuam a pedir o mesmo: clique pelo despacho real.
/// *Mutação: o `Curto::Escolha` trocar os ids das opções ⇒ vermelho.*
#[test]
fn each_short_choice_still_asks_what_it_asked() {
    for id in [
        INSP_PHYS_MASSMODE[1],
        INSP_PHYS_SENSOR[1],
        INSP_PHYS_ONEWAY[1],
        INSP_PHYS_WALLMAT[1],
    ] {
        let (mut host, mut st, rects) = palco();
        let caixa = r(&rects, id);
        let eventos = host.click_at(caixa.x + caixa.w * 0.5, caixa.y + caixa.h * 0.5);
        assert!(
            eventos
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "{id:?} morto sob o rato: {eventos:?}"
        );
        for e in eventos {
            let _ = host.apply_panel_event::<InspectorPanel>(&mut st, e);
        }
        let acoes = host.drained_actions();
        assert!(
            acoes
                .iter()
                .any(|a| matches!(a, EditorAction::InspectorPhysicsEdit { .. })),
            "{id:?} não pediu nada: {acoes:?}"
        );
    }
    set_current_inspector_physics(None);
    set_cards_on(false);
}
