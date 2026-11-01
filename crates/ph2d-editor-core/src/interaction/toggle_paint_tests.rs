//! A lei do arrasto sobre interruptores, pelo despacho real (`dispatch_pointer`).

use super::*;
use crate::interaction::{WidgetEvent, WidgetStore, dispatch_pointer};
use bumpalo::Bump;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

const G: &str = "teste";

fn ev(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn id(n: u64) -> NodeId {
    NodeId(0x7A61_0000 + n)
}

/// Uma coluna de `n` células de 20 px, `ligado(i)` o que cada uma mostra.
fn coluna(n: u64, ligado: impl Fn(u64) -> bool) -> HitIndex {
    limpa_grupo(G);
    soltar();
    let mut hit = HitIndex::new();
    for i in 0..n {
        let r = Rect::new(0.0, i as f32 * 20.0, 20.0, 20.0);
        hit.register(id(i), r);
        celula(id(i), G, ligado(i), r);
    }
    hit
}

fn cliques(store: &mut WidgetStore, hit: &HitIndex, e: PointerEvent) -> Vec<NodeId> {
    let arena = Bump::new();
    dispatch_pointer(store, hit, e, &arena)
        .iter()
        .filter_map(|w| match w {
            WidgetEvent::Click(i) => Some(*i),
            _ => None,
        })
        .collect()
}

#[test]
fn premir_vira_e_o_estado_novo_e_a_tinta() {
    let g = Gesto::premir(id(0), G, true, 0.0, 0.0);
    assert!(!g.tinta(), "premir uma célula ligada pinta DESLIGADO");
    let g = Gesto::premir(id(0), G, false, 0.0, 0.0);
    assert!(g.tinta(), "premir uma célula desligada pinta LIGADO");
}

#[test]
fn arrastar_poe_a_tinta_e_nao_vira_quem_ja_a_tem() {
    // 0 ligada (premida → tinta desligado), 1 ligada, 2 desligada, 3 ligada.
    let hit = coluna(4, |i| i != 2);
    let mut store = WidgetStore::default();
    assert_eq!(
        cliques(&mut store, &hit, ev(PointerKind::Down, 10.0, 10.0)),
        vec![id(0)]
    );
    let mut viradas = Vec::new();
    for y in [30.0, 50.0, 70.0] {
        viradas.extend(cliques(&mut store, &hit, ev(PointerKind::Move, 10.0, y)));
    }
    assert_eq!(
        viradas,
        vec![id(1), id(3)],
        "a 2 já mostrava a tinta: não vira"
    );
    assert!(em_curso());
    cliques(&mut store, &hit, ev(PointerKind::Up, 10.0, 70.0));
    assert!(!em_curso(), "soltar acaba o gesto");
}

#[test]
fn reentrar_numa_celula_nao_faz_nada() {
    let hit = coluna(2, |_| true);
    let mut store = WidgetStore::default();
    cliques(&mut store, &hit, ev(PointerKind::Down, 10.0, 10.0));
    let mut viradas = Vec::new();
    for y in [30.0, 10.0, 30.0, 10.0] {
        viradas.extend(cliques(&mut store, &hit, ev(PointerKind::Move, 10.0, y)));
    }
    assert_eq!(
        viradas,
        vec![id(1)],
        "cada célula vira no máximo uma vez por gesto"
    );
}

#[test]
fn um_rato_rapido_nao_salta_celulas() {
    let hit = coluna(6, |_| true);
    let mut store = WidgetStore::default();
    cliques(&mut store, &hit, ev(PointerKind::Down, 10.0, 10.0));
    // Um só `Move` de 0 até 5: o segmento atravessa 1..=5.
    let viradas = cliques(&mut store, &hit, ev(PointerKind::Move, 10.0, 110.0));
    assert_eq!(viradas, vec![id(1), id(2), id(3), id(4), id(5)]);
}

#[test]
fn sem_gesto_o_move_nao_pinta() {
    let hit = coluna(2, |_| true);
    let mut store = WidgetStore::default();
    assert!(cliques(&mut store, &hit, ev(PointerKind::Move, 10.0, 30.0)).is_empty());
}

#[test]
fn outro_grupo_nao_e_pintado() {
    let mut hit = coluna(1, |_| true);
    let r = Rect::new(0.0, 20.0, 20.0, 20.0);
    hit.register(id(9), r);
    celula(id(9), "outro", true, r);
    let mut store = WidgetStore::default();
    cliques(&mut store, &hit, ev(PointerKind::Down, 10.0, 10.0));
    assert!(cliques(&mut store, &hit, ev(PointerKind::Move, 10.0, 30.0)).is_empty());
    limpa_grupo("outro");
}
