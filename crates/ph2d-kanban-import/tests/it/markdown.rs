//! GATE: o markdown é determinístico e igual ao instantâneo gravado. Mudar o formato de propósito:
//! `PH2D_BLESS=1` reescreve o instantâneo (e o diff dele entra no commit).

use ph2d_kanban_import::export_markdown;

use crate::imported;

const SNAPSHOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/trello_board_synthetic.expected.md"
);

#[test]
fn markdown_matches_the_snapshot() {
    let md = export_markdown(&imported().kanban);
    assert_eq!(md, export_markdown(&imported().kanban), "determinístico");
    if std::env::var_os("PH2D_BLESS").is_some() {
        std::fs::write(SNAPSHOT, &md).expect("bless");
    }
    let want = std::fs::read_to_string(SNAPSHOT).expect("o instantâneo");
    assert_eq!(md, want);
}
