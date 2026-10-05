//! **Os QUADROS no arquivo de projeto** (MiroClone, v184) — filho de `project_tests`, como o da
//! arte dos padrões: aqui só o que os quadros fazem ao *load*.

use super::*;

/// ⭐⭐ **Quadros ilegíveis RECUSAM o arquivo inteiro** e não tocam na sessão aberta — a lei da
/// animação e da arte dos padrões. Abrir sem eles faria o próximo Ctrl+S gravar o projecto sem os
/// quadros por cima do ficheiro.
#[test]
fn unreadable_boards_refuse_the_whole_file_and_leave_the_session_alone() {
    let mut app = headless_app();
    app.undo
        .push_undo(empty_state(), crate::undo::SelectionMark::default());
    let path = tmp_path("load_bad_boards");
    write_project_boards(&path, PROJECT_SCHEMA, Vec::new(), Vec::new(), vec![0xff; 4]);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert!(
        app.undo.can_undo(),
        "o load mutou a sessão antes de recusar os quadros ilegíveis"
    );
}

/// ⭐ **CONTROLO: o MESMO arquivo, com quadros bem-formados, ABRE** — senão a recusa podia ser
/// «recusa sempre» e passar.
#[test]
fn well_formed_boards_still_open_the_file() {
    let mut app = headless_app();
    app.undo
        .push_undo(empty_state(), crate::undo::SelectionMark::default());
    let mut set = ph2d_editor_core::documents::BoardSet::default();
    set.create("Retro".into());
    set.create("Ideas".into());
    let path = tmp_path("load_good_boards");
    let bytes = set.to_bytes().expect("serializa");
    write_project_boards(&path, PROJECT_SCHEMA, Vec::new(), Vec::new(), bytes);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert!(
        !app.undo.can_undo(),
        "o arquivo foi RECUSADO - a lei da recusa come o caso bom"
    );
}
