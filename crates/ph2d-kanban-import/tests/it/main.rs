//! Os gates da importação/exportação do Trello (W10).

mod csv;
mod fixture;
mod fuzz;
mod markdown;
mod round_trip;

use ph2d_kanban_import::{Imported, import_trello};
use ph2d_kanban_model::KanbanId;

pub const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/trello_board_synthetic.json"
);

pub fn fixture_bytes() -> Vec<u8> {
    std::fs::read(FIXTURE).expect("fixture")
}

pub fn imported() -> Imported {
    import_trello(&fixture_bytes(), KanbanId(7), None).expect("a fixture é um quadro")
}
