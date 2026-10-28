//! GATE: Trello → Kanban → Trello → Kanban dá o MESMO Kanban (e a 2.ª exportação os mesmos bytes).

use ph2d_kanban_import::{
    DirFiles, export_trello_json, import_trello, import_trello_into, label_color_name,
    label_color_of,
};
use ph2d_kanban_model::{KanbanId, KanbanSet, LabelColor, Link};

use crate::{fixture_bytes, imported};

#[test]
fn round_trip_trello_json_gives_the_same_kanban() {
    let first = imported().kanban;
    let json = export_trello_json(&first);
    let second = import_trello(json.as_bytes(), KanbanId(7), None)
        .expect("reimport")
        .kanban;
    assert_eq!(first, second);
    assert_eq!(
        json,
        export_trello_json(&second),
        "a exportação é determinística"
    );
}

#[test]
fn round_trip_keeps_project_files_and_the_image_cover() {
    let dir = tempdir();
    std::fs::write(dir.join("storyboard.pdf"), b"%PDF-1.7 storyboard").expect("w");
    let first =
        import_trello(&fixture_bytes(), KanbanId(7), Some(&DirFiles(dir.clone()))).expect("import");
    let files = first
        .kanban
        .attachments
        .values()
        .filter(|a| matches!(a.link, Link::File(_)))
        .count();
    assert_eq!((files, first.files.len()), (1, 1));
    let json = export_trello_json(&first.kanban);
    let second = import_trello(json.as_bytes(), KanbanId(7), None).expect("reimport");
    assert_eq!(first.kanban, second.kanban);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn import_into_a_set_adds_one_kanban_and_its_files() {
    let mut set = KanbanSet::default();
    let other = set.create("antes");
    let (id, report) = import_trello_into(&mut set, &fixture_bytes(), None).expect("import");
    assert_ne!(id, other);
    assert_eq!(
        set.get(id).map(|k| k.header.name.as_str()),
        Some("Produção do curta «Lua»")
    );
    assert_eq!(
        report.counts.cards,
        set.get(id).map_or(0, |k| k.cards.len())
    );
    assert!(import_trello_into(&mut set, b"[]", None).is_err());
    assert_eq!(set.iter().count(), 2, "um erro não deixa Kanban vazio");
}

#[test]
fn the_thirty_trello_label_colours_map_both_ways() {
    for c in LabelColor::all() {
        assert_eq!(label_color_of(&label_color_name(c)), Some(c));
    }
    assert_eq!(label_color_of("teal"), None);
    assert_eq!(label_color_of("_dark"), None);
}

pub fn tempdir() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let d = std::env::temp_dir().join(format!(
        "ph2d-kanban-import-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&d).expect("tempdir");
    d
}
