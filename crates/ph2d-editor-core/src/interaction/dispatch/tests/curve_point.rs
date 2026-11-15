//! O botão direito num ponto da curva Custom do gráfico da timeline (dono 15/11): chega ao painel
//! como um Click secundário (que o apaga) e não abre menu nenhum nem captura o rato. O mesmo botão
//! numa ALÇA do ponto, ou na linha da curva, não é um pedido de apagar.

use super::*;
use crate::interaction::{GesturePhase, TimelineHitKind};

const SURFACE: NodeId = NodeId(700);
const ALVO: NodeId = NodeId(702);
const HIT: Rect = Rect::new(50.0, 50.0, 14.0, 14.0);

fn setup(kind: TimelineHitKind) -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        ALVO,
        InteractiveState::TimelineSurface {
            parent: SURFACE,
            kind,
            canvas: Rect::new(0.0, 0.0, 400.0, 200.0),
        },
    );
    let mut hits = HitIndex::new();
    hits.register(ALVO, HIT);
    (store, hits)
}

fn direito(store: &mut WidgetStore, hits: &HitIndex) {
    let arena = Bump::new();
    let ev = PointerEvent {
        button: ph2d_host::PointerButton::Secondary,
        ..pointer(PointerKind::Down, 57.0, 57.0)
    };
    let _ = dispatch_pointer(store, hits, ev, &arena);
}

#[test]
fn o_botao_direito_num_ponto_da_curva_vai_ao_painel_e_nao_abre_menu() {
    let kind = TimelineHitKind::CurvePoint {
        target: 9,
        key: 1,
        node: 1,
        part: 0,
    };
    let (mut store, hits) = setup(kind);
    direito(&mut store, &hits);
    let gestos: Vec<_> = store.drain_timeline_gestures().collect();
    assert_eq!(gestos.len(), 1, "{gestos:?}");
    assert_eq!(gestos[0].kind, kind);
    assert_eq!(gestos[0].phase, GesturePhase::Click);
    assert_eq!(gestos[0].button, ph2d_host::PointerButton::Secondary);
    assert_eq!(store.context_menu(), None, "nenhum menu");
    assert_eq!(store.active_id(), None, "nem captura");
}

#[test]
fn o_botao_direito_numa_alca_ou_na_linha_nao_e_apagar() {
    for kind in [
        TimelineHitKind::CurvePoint {
            target: 9,
            key: 1,
            node: 1,
            part: 2,
        },
        TimelineHitKind::CurveLine { target: 9, key: 1 },
    ] {
        let (mut store, hits) = setup(kind);
        direito(&mut store, &hits);
        assert_eq!(store.drain_timeline_gestures().count(), 0, "{kind:?}");
    }
}
