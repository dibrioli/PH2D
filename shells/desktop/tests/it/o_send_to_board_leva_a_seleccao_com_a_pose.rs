//! **O «Send to board…» atravessa a shell inteira** (MiroClone W16): o pedido da linha
//! (`HierRequest::SendToBoard`) e a escolha do modal (`HierRequest::SendToBoardChosen`) chegam à
//! `fase_send_to_board`, que leva a SELECÇÃO (`gather` sobre `iter_selected`), põe a POSE da cena na
//! sprite (`posed_sprite`) e entrega tudo numa leva (`deliver`).
//!
//! ⚠️ A fase precisa de `gfx` (a placa, a sprite lida pela textura, o Flip pela placa) e o `App::new()`
//! nasce headless de propósito — a mesma cerca de `the_colorize_scribble_crosses_the_selected_frames`.
//! Por isso este gate lê o FONTE e pina só a COSTURA; o comportamento de cada peça tem teste próprio
//! na família (`ph2d_app_board::send_to_board::tests`: o modal → a escolha → a entrega, o ciclo da escolha
//! sobre um mundo ECS real com um apagado e um sem nome (`gather`, A11), a selecção
//! inteira com a pose de cada um, a lei da pose) e na Hierarquia (`send_to_board_from_the_row`).

const BUS: &str = include_str!("../../src/render_loop/fase_bus_hierarchy.rs");
const COMMITS: &str = include_str!("../../src/render_loop/fase_hero_commits.rs");
const FASE: &str = include_str!("../../src/render_loop/fase_send_to_board.rs");

/// O corpo da fase (a lei, não o cabeçalho).
fn fase_body() -> &'static str {
    let start = FASE
        .find("pub(super) fn fase_send_to_board(")
        .expect("a fase do «Send to board…» sumiu — re-mire este gate");
    let body = &FASE[start..];
    assert!(
        body.len() > 2000,
        "o recorte da fase deu {} bytes — perdeu o sujeito",
        body.len()
    );
    body
}

#[test]
fn the_two_requests_reach_the_phase() {
    for (req, slot) in [
        (
            "HierRequest::SendToBoard {",
            "pd.send_to_board_ask.get_or_insert(row)",
        ),
        (
            "HierRequest::SendToBoardChosen { row, board }",
            "pd.send_to_board.get_or_insert((row, board))",
        ),
    ] {
        let at = BUS
            .find(req)
            .unwrap_or_else(|| panic!("o dreno não casa {req}"));
        let arm = &BUS[at..at + 300.min(BUS.len() - at)];
        assert!(
            arm.contains(slot),
            "{req} não guarda o pedido em {slot}: {arm}"
        );
    }
    assert!(
        COMMITS.contains(
            "self.fase_send_to_board(take(&mut pd.send_to_board_ask), take(&mut pd.send_to_board));"
        ),
        "os dois pedidos não chegam à fase"
    );
}

#[test]
fn the_phase_sends_the_selection_posed_in_one_delivery() {
    let body = fase_body();
    let order = [
        "hero.gizmo.iter_selected()",
        "gather(world, clicked.to_bits(), &selected, kind,",
        "posed_sprite(",
        "deliver(hero, board, items)",
    ];
    let mut at = 0;
    for needle in order {
        let found = body[at..]
            .find(needle)
            .unwrap_or_else(|| panic!("a fase não chama `{needle}` (ou fora de ordem)"));
        at += found + needle.len();
    }
    assert_eq!(
        body.matches("deliver(").count(),
        1,
        "uma entrega por escolha — a selecção vai numa leva só (um passo de desfazer)"
    );
}

/// A11 — quem decide o apagado («Deleted object») e o nome de quem não o tem é a FAMÍLIA
/// (`ph2d_app_board::send_to_board::gather`, com gate de comportamento sobre um mundo ECS real:
/// `the_choice_reads_the_scene_counts_the_deleted_and_names_the_nameless`). Aqui só a costura: a fase dá-lhe
/// o mundo da CENA e o tipo de cada objecto, e não repete a decisão.
#[test]
fn the_phase_lets_the_family_read_the_scene() {
    let body = fase_body();
    assert!(body.contains("let world = gfx.sim.world();"));
    assert!(
        body.contains("let kind = |e| ph2d_app_components::component_attach::kind_of(world, e);")
    );
    assert_eq!(
        body.matches("gather(world, clicked.to_bits(), &selected, kind,")
            .count(),
        1
    );
    for twin in ["Sent::gone()", "Sent::name_of(", "targets("] {
        assert!(
            !body.contains(twin),
            "a fase voltou a decidir `{twin}` ao lado da família — uma lei, um sítio"
        );
    }
}
