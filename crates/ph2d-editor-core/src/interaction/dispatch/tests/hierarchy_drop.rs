//! ⭐ **Largar um arrasto da Hierarquia** — reparenta só DENTRO do painel onde começou
//! (`WidgetStore::hierarchy_drop_in_own_panel`, item 8a do CodeEditor: fora dele é a queda entre
//! painéis, e largar sobre o script não pode mudar o objecto de pai).

use super::*;

const PAINEL: NodeId = NodeId(99_000);
const PAI: NodeId = NodeId(100_000);
const ARRASTADO: NodeId = NodeId(100_001);

/// Duas linhas "vivas" da ponte (fora da faixa do fixture antigo 400..=411): o pai em y=0..20, o
/// arrastado em y=30..50, dentro de um painel de 200×60.
fn duas_linhas() -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(8);
    store.register(PAI, InteractiveState::Plain);
    store.register(ARRASTADO, InteractiveState::Plain);
    store.set_hierarchy_row_ids([PAI, ARRASTADO].into_iter().collect());
    store.set_hierarchy_order(vec![PAI, ARRASTADO]);
    store.set_panel_rect(PAINEL, Rect::new(0.0, 0.0, 200.0, 60.0));
    let mut hits = HitIndex::new();
    hits.register(PAI, Rect::new(0.0, 0.0, 200.0, 20.0));
    hits.register(ARRASTADO, Rect::new(0.0, 30.0, 200.0, 20.0));
    (store, hits)
}

/// `Down` no arrastado, um `Move` que passa o limiar e o `Up` em `(x, 10)` — a altura do meio do pai.
fn arrasta_e_larga_em(x: f32) -> Vec<WidgetEvent> {
    let (mut store, hits) = duas_linhas();
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 100.0, 40.0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, x, 10.0),
        &arena,
    );
    dispatch_pointer(&mut store, &hits, pointer(PointerKind::Up, x, 10.0), &arena).to_vec()
}

fn reparenta_no_pai(evts: &[WidgetEvent]) -> bool {
    evts.iter().any(|e| {
        matches!(
            e,
            WidgetEvent::HierReparent {
                dragged,
                new_parent: Some(np),
                before: None,
                after: _,
            } if *dragged == ARRASTADO && *np == PAI
        )
    })
}

/// Pre-M14.6B: as linhas vivas (NodeIds 100_000+) nunca viravam candidatas ao arrasto. O conjunto
/// publicado por `set_hierarchy_row_ids` é a fonte única; largar no meio do pai = `Inside`.
#[test]
fn hierarchy_drag_in_live_mode_emits_reparent_intent() {
    let evts = arrasta_e_larga_em(100.0);
    assert!(
        reparenta_no_pai(&evts),
        "expected HierReparent Inside({PAI:?}); got {evts:?}"
    );
}

/// ⛔ À altura do pai mas FORA do painel (x = 500): até 21/10 reparentava (só olhava a altura).
#[test]
fn largar_fora_do_painel_da_hierarquia_nao_reparenta() {
    let evts = arrasta_e_larga_em(500.0);
    assert!(
        !evts
            .iter()
            .any(|e| matches!(e, WidgetEvent::HierReparent { .. })),
        "fora do painel não é reparentar: {evts:?}"
    );
}
