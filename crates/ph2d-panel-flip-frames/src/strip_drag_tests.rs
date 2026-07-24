//! Os gates do arrasto da tira — módulo-irmão do `strip_drag.rs` pelo cap de LOC (HR-18).
//!
//! O `#[path]` mantém `tests` como FILHO do módulo (`use super::*` alcança os privados),
//! então nada mudou de visibilidade — só de arquivo.

use super::*;
use crate::state::FlipCell;
use ph2d_editor_core::interaction::GestureMods;
use ph2d_host::PointerButton;

fn cell(key: i32, exposure: u32) -> FlipCell {
    FlipCell {
        key,
        exposure,
        breakdown: false,
        instanced: false,
        selected: false,
        pinned: false,
        weight: 1.0,
    }
}

/// Chaves em 0/4/8, cada uma expondo 4 — e uma faixa estreita o bastante para o teto de
/// px-por-quadro não mascarar a escala.
fn fixture() -> (StripRuler, FlipStripSnapshot) {
    let snap = FlipStripSnapshot {
        has_layer: true,
        cells: vec![cell(0, 4), cell(4, 4), cell(8, 4)],
        ..Default::default()
    };
    let ruler = StripRuler::resolve(Rect::new(0.0, 0.0, 120.0, 100.0), &snap).expect("régua");
    (ruler, snap)
}

fn gesture(kind: FlipStripHitKind, phase: GesturePhase, x: f32) -> FlipStripGesture {
    FlipStripGesture {
        surface: ph2d_a11y::NodeId(1),
        kind,
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

/// Arrasta a célula `i` do meio dela até o quadro `to_frame`, e devolve o que ficou na
/// fila de pedidos.
fn drag_cell(i: u16, to_frame: i32) -> Vec<FlipStripIntent> {
    let (r, s) = fixture();
    let _ = crate::state::drain_flip_strip_intents(); // a fila é thread_local
    let mut d = None;
    let body = r.cell_rect(i as usize, &s).unwrap();
    let start = body.x + body.w * 0.5;
    let kind = FlipStripHitKind::Cell { index: i };
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, start));
    // O ponteiro anda o mesmo tanto que o alvo tem de andar.
    let delta = (to_frame - s.cells[i as usize].key) as f32 * r.ppf;
    apply(
        &mut d,
        &r,
        &s,
        gesture(kind, GesturePhase::Update, start + delta),
    );
    apply(
        &mut d,
        &r,
        &s,
        gesture(kind, GesturePhase::End, start + delta),
    );
    crate::state::drain_flip_strip_intents()
}

/// 🔴 O gesto central: pegar a chave do meio e largá-la um quadro adiante pede
/// exatamente `move_frame(4 → 5)`. Mutação que sangra: aplicar no Update em vez do End
/// (a fila teria um pedido por passo do percurso — e o MOVER reordenaria a lista sob o
/// próprio índice, a razão de ele NÃO ser vivo).
#[test]
fn dragging_a_cell_asks_to_move_its_key() {
    assert_eq!(
        drag_cell(1, 5),
        vec![FlipStripIntent::MoveKey { from: 4, to: 5 }]
    );
}

/// 🔴 **O arrasto é relativo ao ponto de pega** — pegar a célula pelo MEIO e não mover o
/// ponteiro não move a chave. Com alvo absoluto (`target = frame sob o cursor`) a chave
/// saltaria para debaixo do dedo no primeiro Update: aqui, dois quadros adiante.
#[test]
fn grabbing_a_wide_cell_off_centre_does_not_teleport_it() {
    let (r, s) = fixture();
    let _ = crate::state::drain_flip_strip_intents();
    let mut d = None;
    let body = r.cell_rect(1, &s).unwrap();
    let grab = body.x + body.w * 0.75; // bem à direita do começo da célula
    let kind = FlipStripHitKind::Cell { index: 1 };
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, grab));
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, grab));
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::End, grab));
    assert!(
        crate::state::drain_flip_strip_intents().is_empty(),
        "sem percurso não há pedido — a chave não pode saltar para o dedo"
    );
}

/// 🔴 **A chave encosta na vizinha, não passa por cima dela.** `move_frame` RECUSA um
/// destino ocupado, e um gesto que às vezes não faz nada ensina intermitência.
#[test]
fn a_key_stops_against_its_neighbour_instead_of_being_refused() {
    // A do meio (4) puxada para 99: a vizinha começa em 8 ⇒ para em 7.
    assert_eq!(
        drag_cell(1, 99),
        vec![FlipStripIntent::MoveKey { from: 4, to: 7 }]
    );
    // E para trás: a anterior começa em 0 ⇒ para em 1.
    assert_eq!(
        drag_cell(1, -99),
        vec![FlipStripIntent::MoveKey { from: 4, to: 1 }]
    );
}

/// A PRIMEIRA chave para em 0 (o tempo começa ali) e a ÚLTIMA não tem teto — arrastá-la
/// para a direita é como uma cena se estende.
#[test]
fn the_first_key_stops_at_zero_and_the_last_one_has_no_ceiling() {
    assert!(
        drag_cell(0, -5).is_empty(),
        "a primeira já está em 0: puxar para trás não pede nada"
    );
    assert_eq!(
        drag_cell(2, 20),
        vec![FlipStripIntent::MoveKey { from: 8, to: 20 }]
    );
}

/// 🔴 **O hold é VIVO** (Enio, smoke 2026-07-24): o pedido sai no UPDATE — a célula
/// estica na tela enquanto o dedo anda — e o End não duplica nada quando o ponteiro
/// não andou mais. Dois Updates no mesmo alvo pedem UMA vez (emite-quando-muda).
/// Mutação que sangra: voltar a aplicar só no End (o 1º drain sai vazio).
#[test]
fn dragging_the_hold_edge_stretches_the_exposure_live() {
    let (r, s) = fixture();
    let _ = crate::state::drain_flip_strip_intents();
    let mut d = None;
    let kind = FlipStripHitKind::HoldEdge { index: 0 };
    let edge = r
        .hold_edge_rect(0, &s)
        .expect("a célula de 4 quadros tem grip");
    let start = edge.x + edge.w * 0.5;
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, start));
    // Leva a borda até o meio do quadro 6 ⇒ exposição 7 (quadros 0..=6) — JÁ no Update.
    let x6 = r.x_of_frame(6) + r.ppf * 0.5;
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, x6));
    assert_eq!(
        crate::state::drain_flip_strip_intents(),
        vec![FlipStripIntent::SetHold { key: 0, frames: 7 }],
        "o hold aplica enquanto o ponteiro anda, não no soltar"
    );
    // Mais um Update no MESMO lugar: alvo igual, nada novo na fila.
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, x6));
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::End, x6));
    assert!(
        crate::state::drain_flip_strip_intents().is_empty(),
        "alvo que não mudou não pede de novo — nem no End"
    );
    assert!(d.is_none(), "o End encerra a sessão");
}

/// A exposição nunca chega a zero: arrastar a borda para trás da própria chave a deixa
/// em 1 quadro (um desenho que fica zero quadro na tela não é um desenho, é um delete —
/// e delete é outro botão).
#[test]
fn the_hold_never_shrinks_below_one_frame() {
    let (r, s) = fixture();
    let _ = crate::state::drain_flip_strip_intents();
    let mut d = None;
    let kind = FlipStripHitKind::HoldEdge { index: 1 };
    let edge = r.hold_edge_rect(1, &s).unwrap();
    apply(
        &mut d,
        &r,
        &s,
        gesture(kind, GesturePhase::Begin, edge.x + edge.w * 0.5),
    );
    let far_left = r.x_of_frame(-20);
    apply(
        &mut d,
        &r,
        &s,
        gesture(kind, GesturePhase::Update, far_left),
    );
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::End, far_left));
    assert_eq!(
        crate::state::drain_flip_strip_intents(),
        vec![FlipStripIntent::SetHold { key: 4, frames: 1 }]
    );
}

/// 🔴 **O mapeamento do gesto é o do Begin.** O hold vivo muda o total de quadros, a
/// tira re-escala, e uma régua VIVA leria o MESMO x como um quadro maior — realimentação
/// positiva: a exposição dispararia sozinha. A sessão congela a régua da pegada, então
/// o mesmo x continua pedindo o mesmo alvo mesmo com a tira já esticada.
/// Mutação que sangra: `hold_step` ler a régua do frame em vez de `d.ruler`.
#[test]
fn the_holds_mapping_is_frozen_at_the_grab() {
    let (r, s) = fixture();
    let _ = crate::state::drain_flip_strip_intents();
    let mut d = None;
    let kind = FlipStripHitKind::HoldEdge { index: 2 };
    let edge = r.hold_edge_rect(2, &s).expect("grip da última");
    apply(
        &mut d,
        &r,
        &s,
        gesture(kind, GesturePhase::Begin, edge.x + edge.w * 0.5),
    );
    // Estica a última em +2 quadros (4 → 6).
    let x = r.x_of_frame(13) + r.ppf * 0.5;
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, x));
    assert_eq!(
        crate::state::drain_flip_strip_intents(),
        vec![FlipStripIntent::SetHold { key: 8, frames: 6 }]
    );
    // O documento acompanhou (é o hold vivo): o snapshot do frame seguinte tem a
    // exposição nova, e a régua RESOLVIDA dele lê o mesmo x como OUTRO quadro — a
    // fixture tem de conter o fenômeno, senão a mutação não sangra.
    let s2 = FlipStripSnapshot {
        has_layer: true,
        cells: vec![cell(0, 4), cell(4, 4), cell(8, 6)],
        ..Default::default()
    };
    let r2 = StripRuler::resolve(Rect::new(0.0, 0.0, 120.0, 100.0), &s2).expect("régua");
    assert_ne!(
        r.frame_at_x(x),
        r2.frame_at_x(x),
        "pré-condição: a tira re-escalou e a régua viva discorda da congelada"
    );
    // O mesmo x, sob o snapshot novo e a régua nova: o alvo NÃO anda sozinho.
    apply(&mut d, &r2, &s2, gesture(kind, GesturePhase::Update, x));
    apply(&mut d, &r2, &s2, gesture(kind, GesturePhase::End, x));
    assert!(
        crate::state::drain_flip_strip_intents().is_empty(),
        "com a régua congelada, o mesmo x pede o mesmo alvo — sem realimentação"
    );
}

/// 🔴 **Um toque não pede nada e devolve a célula** — é o `PanelEvent::Click` de sempre
/// (selecionar a chave, com o modificador que o shell lê). Se o toque começasse a pedir
/// um `MoveKey{from: k, to: k}`, todo clique viraria uma edição no-op na fila de undo.
#[test]
fn a_tap_asks_for_nothing_and_reports_the_cell() {
    let (r, s) = fixture();
    let _ = crate::state::drain_flip_strip_intents();
    let mut d = None;
    let kind = FlipStripHitKind::Cell { index: 2 };
    let x = r.cell_rect(2, &s).unwrap().x + 1.0;
    apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, x));
    let tapped = apply(&mut d, &r, &s, gesture(kind, GesturePhase::Click, x));
    assert_eq!(tapped, Some(2));
    assert!(crate::state::drain_flip_strip_intents().is_empty());
    assert!(d.is_none(), "o toque encerra a sessão");
}

/// Fixture com SELEÇÃO: chaves nos `keys` dados, com as de índice em `sel` marcadas.
fn sel_fixture(keys: &[(i32, u32)], sel: &[usize]) -> (StripRuler, FlipStripSnapshot) {
    let cells = keys
        .iter()
        .enumerate()
        .map(|(i, &(k, e))| FlipCell {
            selected: sel.contains(&i),
            ..cell(k, e)
        })
        .collect();
    let snap = FlipStripSnapshot {
        has_layer: true,
        cells,
        ..Default::default()
    };
    let ruler = StripRuler::resolve(Rect::new(0.0, 0.0, 120.0, 100.0), &snap).expect("régua");
    (ruler, snap)
}

/// Arrasta a célula `i` de `snap` pelo delta de quadros dado e devolve a fila.
fn drag_cell_of(r: &StripRuler, s: &FlipStripSnapshot, i: u16, delta: i32) -> Vec<FlipStripIntent> {
    let _ = crate::state::drain_flip_strip_intents();
    let mut d = None;
    let body = r.cell_rect(i as usize, s).unwrap();
    let start = body.x + body.w * 0.5;
    let kind = FlipStripHitKind::Cell { index: i };
    apply(&mut d, r, s, gesture(kind, GesturePhase::Begin, start));
    let dx = delta as f32 * r.ppf;
    apply(
        &mut d,
        r,
        s,
        gesture(kind, GesturePhase::Update, start + dx),
    );
    apply(&mut d, r, s, gesture(kind, GesturePhase::End, start + dx));
    crate::state::drain_flip_strip_intents()
}

/// 🔴 **Pegar uma célula MARCADA move a seleção inteira** — o idioma do dope-sheet
/// (W7 marcou N quadros; o gesto age nos N). Cada marcada pede o MESMO delta.
/// Mutação que sangra: emitir só a célula pega (o resto da seleção fica para trás).
#[test]
fn dragging_a_marked_cell_moves_the_whole_selection() {
    let (r, s) = sel_fixture(&[(0, 4), (4, 4), (8, 4)], &[0, 2]);
    assert_eq!(
        drag_cell_of(&r, &s, 0, 1),
        vec![
            FlipStripIntent::MoveKey { from: 8, to: 9 },
            FlipStripIntent::MoveKey { from: 0, to: 1 },
        ],
        "as DUAS marcadas movem +1 (e a não marcada de 4 fica)"
    );
}

/// 🔴 **O limite do grupo é o vizinho NÃO marcado (e o piso 0)** — a chave mais
/// apertada trava o grupo inteiro, que encosta e para (nunca atravessa quem ficou).
#[test]
fn the_selection_stops_against_its_unselected_neighbours() {
    // Para a DIREITA: a marcada de 0 para em 3 (a não marcada vive em 4) ⇒ delta 3,
    // mesmo a de 8 tendo espaço infinito.
    let (r, s) = sel_fixture(&[(0, 4), (4, 4), (8, 4)], &[0, 2]);
    assert_eq!(
        drag_cell_of(&r, &s, 2, 99),
        vec![
            FlipStripIntent::MoveKey { from: 8, to: 11 },
            FlipStripIntent::MoveKey { from: 0, to: 3 },
        ]
    );
    // Para a ESQUERDA: a de 2 bate no piso 0 ⇒ delta −2 (a de 8 podia −3).
    let (r, s) = sel_fixture(&[(2, 2), (4, 4), (8, 4)], &[0, 2]);
    assert_eq!(
        drag_cell_of(&r, &s, 0, -99),
        vec![
            FlipStripIntent::MoveKey { from: 2, to: 0 },
            FlipStripIntent::MoveKey { from: 8, to: 6 },
        ]
    );
}

/// 🔴 **Marcadas ADJACENTES movem na ordem que pousa** — `+1` em {4,5}: a da esquerda
/// primeiro pediria `4→5` com o 5 ainda ocupado pela irmã, e o `move_frame` RECUSA.
/// Para a direita emite-se da direita para a esquerda (e o espelho para a esquerda).
/// Mutação que sangra: emitir sempre na ordem da lista.
#[test]
fn adjacent_marked_keys_move_in_the_order_that_lands() {
    let (r, s) = sel_fixture(&[(0, 4), (4, 1), (5, 4)], &[1, 2]);
    assert_eq!(
        drag_cell_of(&r, &s, 1, 1),
        vec![
            FlipStripIntent::MoveKey { from: 5, to: 6 },
            FlipStripIntent::MoveKey { from: 4, to: 5 },
        ],
        "para a direita, a mais à direita anda primeiro"
    );
    assert_eq!(
        drag_cell_of(&r, &s, 2, -1),
        vec![
            FlipStripIntent::MoveKey { from: 4, to: 3 },
            FlipStripIntent::MoveKey { from: 5, to: 4 },
        ],
        "para a esquerda, a mais à esquerda anda primeiro"
    );
}

/// **Pegar uma célula NÃO marcada move só ela** — a seleção não é do gesto: ela marca
/// alvos de multiframe, e um arrasto fora dela é o gesto de sempre.
#[test]
fn an_unmarked_grab_moves_only_itself() {
    let (r, s) = sel_fixture(&[(0, 4), (4, 4), (8, 4)], &[0, 2]);
    assert_eq!(
        drag_cell_of(&r, &s, 1, 1),
        vec![FlipStripIntent::MoveKey { from: 4, to: 5 }]
    );
}

/// 🔴 **O preview do grupo mostra CADA marcada no seu destino** — o gesto mostra tudo
/// o que vai mudar, cada contorno com a largura da própria exposição. Mutação que
/// sangra: preview só da célula pega.
#[test]
fn the_group_preview_outlines_every_marked_cell_at_its_target() {
    let (r, s) = sel_fixture(&[(0, 2), (4, 4), (8, 4)], &[0, 2]);
    let _ = crate::state::drain_flip_strip_intents();
    let mut state = crate::state::FlipStripState::default();
    let body = r.cell_rect(0, &s).unwrap();
    let start = body.x + body.w * 0.5;
    let kind = FlipStripHitKind::Cell { index: 0 };
    apply(
        &mut state.drag,
        &r,
        &s,
        gesture(kind, GesturePhase::Begin, start),
    );
    apply(
        &mut state.drag,
        &r,
        &s,
        gesture(kind, GesturePhase::Update, start + r.ppf),
    );
    let ghosts = preview_rects(&state, &r, &s);
    assert_eq!(ghosts.len(), 2, "um contorno por célula marcada");
    assert_eq!(ghosts[0].x, r.x_of_frame(1), "a de 0 prevista em 1");
    assert_eq!(ghosts[1].x, r.x_of_frame(9), "a de 8 prevista em 9");
    assert!(
        ghosts[1].w > ghosts[0].w,
        "cada contorno com a largura da PRÓPRIA exposição (4 > 2)"
    );
    let _ = crate::state::drain_flip_strip_intents();
}

/// A tira mudou embaixo do gesto (outra via editou o documento): a sessão é largada em
/// vez de agir sobre a chave que passou a ocupar aquele índice — nas DUAS formas de
/// ficar obsoleto: o índice não existe mais, ou existe com OUTRA chave dentro.
#[test]
fn a_stale_index_drops_the_session_instead_of_moving_the_wrong_key() {
    let (r, s) = fixture();
    let _ = crate::state::drain_flip_strip_intents();
    let session = StripDrag {
        kind: DragKind::MoveKey,
        key: 4,
        grab_frame: 4,
        target: 6,
        applied: 4,
        ruler: r,
        group: false,
    };
    // Índice fora da lista.
    let mut d = Some(session);
    let gone = FlipStripHitKind::Cell { index: 9 };
    assert_eq!(
        apply(&mut d, &r, &s, gesture(gone, GesturePhase::Update, 10.0)),
        None
    );
    assert!(d.is_none());
    // Índice válido, chave trocada (a sessão diz 4; o índice 0 guarda a chave 0).
    let mut d = Some(session);
    let swapped = FlipStripHitKind::Cell { index: 0 };
    assert_eq!(
        apply(&mut d, &r, &s, gesture(swapped, GesturePhase::Update, 10.0)),
        None
    );
    assert!(d.is_none());
    // E a 3ª forma: sessão de GRUPO cuja célula pega perdeu a MARCA (outra via mexeu
    // na seleção) — a sessão descreve um conjunto que não existe mais.
    let mut d = Some(StripDrag {
        group: true,
        ..session
    });
    let same = FlipStripHitKind::Cell { index: 1 }; // chave 4, não marcada na fixture
    assert_eq!(
        apply(&mut d, &r, &s, gesture(same, GesturePhase::Update, 10.0)),
        None
    );
    assert!(d.is_none());
    assert!(crate::state::drain_flip_strip_intents().is_empty());
}
