//! GATE: o quadro REAL do dono (exportado do Trello em 30/10, anonimizado — `fixtures/…real_anon.md`).
//! Conta o que entrou e cada perda pelo nome: um número que muda aqui é uma decisão, não um acaso.

use ph2d_kanban_import::{LossKind, export_trello_json, import_trello};
use ph2d_kanban_model::KanbanId;

const REAL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/trello_board_real_anon.json"
);

fn real() -> ph2d_kanban_import::Imported {
    let bytes = std::fs::read(REAL).expect("fixture real");
    import_trello(&bytes, KanbanId(3), None).expect("o quadro real é um quadro")
}

#[test]
fn the_owners_real_board_imports_every_list_card_and_checklist() {
    let imp = real();
    let c = imp.report.counts;
    assert_eq!((c.lists, c.cards, c.labels, c.members), (8, 51, 6, 6));
    assert_eq!(
        (c.checklists, c.items, c.attachments, c.fields),
        (9, 37, 1, 0)
    );
    assert_eq!((c.comments, c.activities, c.files), (4, 143, 0));
    let k = &imp.kanban;
    let shown: Vec<usize> = k
        .lists_in_order()
        .iter()
        .map(|l| k.cards_in(l.id).len())
        .collect();
    assert_eq!(
        shown,
        [5, 10, 14, 8, 1, 1, 8],
        "as listas à vista, pela ordem do Trello"
    );
    let archived: Vec<usize> = k
        .lists
        .values()
        .filter(|l| l.archived)
        .map(|l| k.cards_in(l.id).len())
        .collect();
    assert_eq!(
        archived,
        [3],
        "«Em Andamento» arquivada, com os seus 3 cartões"
    );
}

#[test]
fn the_owners_real_board_names_every_loss() {
    let r = real().report;
    let unsupported: Vec<&str> = r
        .losses
        .iter()
        .filter(|l| l.kind == LossKind::UnsupportedAction)
        .map(|l| l.detail.as_str())
        .collect();
    assert_eq!(
        unsupported,
        [
            "addMemberToBoard ×7",
            "addToOrganizationBoard ×1",
            "unconfirmedBoardInvitation ×6",
            "updateBoard.prefs ×8",
            "updateCard.dueReminder ×2",
            "updateCard.idLabels ×7",
            "updateCard.pos ×20",
            "updateList.pos ×3",
        ]
    );
    // Quem saiu do quadro (o Trello apaga-lhe o nome: `member.name` vazio) é membro desconhecido.
    assert_eq!(r.count(LossKind::UnknownMember), 4);
    assert_eq!(r.count(LossKind::Subscription), 1);
    assert_eq!(r.losses.len(), 8 + 4 + 1);
}

#[test]
fn the_owners_real_board_round_trips() {
    let first = real().kanban;
    let json = export_trello_json(&first);
    let second = import_trello(json.as_bytes(), KanbanId(3), None)
        .expect("reimport")
        .kanban;
    assert_eq!(first, second);
}
