//! ⭐ D17 — uma referência a um Kanban que o projecto NÃO tem é INERTE (não se pinta, não se apanha,
//! não conta, não se exporta) e revive igual se o Kanban volta. Os caminhos que a trazem: restaurar
//! uma versão, importar um quadro, um par ao vivo, um projecto editado fora.

use super::*;
use ph2d_board_model::{BoardId, BoardOp};

/// O desenho e o clique deste fio leem os Kanbans do projecto (a app instala-os a cada quadro).
fn arm(t: &T) {
    ph2d_board_model::install_kanbans(t.hero.documents.kanbans_shared());
}

fn selection(t: &T) -> Vec<ElementId> {
    t.hero
        .documents
        .live
        .editor
        .as_ref()
        .map(|e| e.selection().iter().copied().collect())
        .unwrap_or_default()
}

/// A referência a `k` no quadro activo: viva no documento, mas sem desenho, sem clique, sem
/// contagem.
fn assert_inert(t: &mut T, k: KanbanId, at: (f32, f32)) {
    let b = t.hero.documents.active().unwrap();
    let el = t
        .hero
        .documents
        .boards()
        .get(b)
        .unwrap()
        .doc
        .live()
        .find(|e| e.kanban().is_some_and(|r| r.kanban == k))
        .cloned()
        .expect("a referência fica no documento (com a caixa, para reviver)");
    t.paint();
    let frames = &t.hero.documents.live.render_cache.kanbans;
    assert!(!frames.on_screen.contains_key(&el.id), "não se pinta");
    click(t, at);
    assert!(!selection(t).contains(&el.id), "um clique não a apanha");
    assert!(kb::selected(&mut t.hero).is_none(), "nada a abrir");
    assert!(t.key(BoardKey::Char('a'), CTRL, None));
    assert!(!selection(t).contains(&el.id), "Ctrl+A salta-a");
    assert_eq!(kb::link::boards_showing(&t.hero.documents, k), 0);
    assert!(el.kanban_missing());
}

/// O centro de ecrã da caixa de `el` enquanto ela se pinta.
fn centre(t: &mut T, k: KanbanId) -> (f32, f32) {
    let el = kanban_el(t).unwrap();
    assert_eq!(el.kanban().unwrap().kanban, k);
    middle(t, el.id)
}

/// ⭐ REAL — o Kanban posto, uma versão guardada, o Kanban apagado pelos Assets, a versão
/// restaurada: o quadro novo traz a referência a um Kanban que já não existe.
#[test]
fn a_version_restored_after_its_kanban_was_deleted_holds_an_inert_reference() {
    let mut t = T::new();
    arm(&t);
    let b = t.hero.documents.active().unwrap();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    let at = centre(&mut t, k);
    t.hero
        .documents
        .active_board_mut()
        .unwrap()
        .save_version("v".into(), 0);
    let ver = t.hero.documents.boards().get(b).unwrap().team.versions[0].id;
    delete_from_assets(&mut t, k);
    let r = t.hero.documents.restore_version(b, ver).expect("restaura");
    assert_eq!(t.hero.documents.active(), Some(r));
    assert_inert(&mut t, k, at);
}

/// ⭐ REAL — um quadro importado (ou um projecto editado fora) com uma referência a um Kanban que o
/// projecto não tem: inerte. O `Ctrl+Z` tira primeiro o quadro importado (a pilha é LIFO), depois
/// devolve o Kanban — sem pânico em nenhum passo.
#[test]
fn an_imported_reference_to_a_deleted_kanban_is_inert() {
    let mut t = T::new();
    arm(&t);
    let a = t.hero.documents.active().unwrap();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    let at = centre(&mut t, k);
    let doc = t.hero.documents.boards().get(a).unwrap().doc.clone();
    delete_from_assets(&mut t, k);
    let r = t
        .hero
        .documents
        .import_board("X".into(), doc, Default::default(), Default::default());
    t.hero.documents.activate(Some(r));
    t.paint();
    assert_inert(&mut t, k, at);
    assert!(t.hero.documents.undo_tab(false).is_some());
    assert!(
        t.hero.documents.boards().get(r).is_none(),
        "o importado sai"
    );
    assert!(t.hero.documents.undo_tab(false).is_some());
    assert!(
        t.hero.documents.kanbans().get(k).is_some(),
        "o Kanban volta"
    );
    t.paint();
}

/// ⭐ REAL — num quadro ao vivo, depois de apagarmos K, um par (que ainda o tinha) põe uma referência
/// a ele: inerte. O nosso `Ctrl+Z` do apagar devolve K e ELA revive — a mesma chave e o mesmo sítio,
/// pintada e clicável.
#[test]
fn a_peer_reference_to_a_deleted_kanban_is_inert_and_revives_on_undo() {
    let mut t = T::new();
    arm(&t);
    t.click_bar(Item::Kanban);
    let el = kanban_el(&t).unwrap();
    let k = el.kanban().unwrap().kanban;
    let at = centre(&mut t, k);
    let mut theirs = el.clone();
    theirs.id = t.hero.documents.active_board_mut().unwrap().doc.mint_id();
    delete_from_assets(&mut t, k);
    let doc = &mut t.hero.documents.active_board_mut().unwrap().doc;
    let _ = BoardOp::Put(theirs.clone()).apply(doc);
    assert_inert(&mut t, k, at);
    undo_key(&mut t, false);
    assert!(
        t.hero.documents.kanbans().get(k).is_some(),
        "o documento volta"
    );
    t.paint();
    let b = t.hero.documents.active_board().unwrap();
    let back = b.doc.get(theirs.id).expect("a do par ficou e revive");
    assert_eq!((back.x, back.y), (theirs.x, theirs.y));
    assert!(!back.kanban_missing());
    let frames = &t.hero.documents.live.render_cache.kanbans;
    assert!(
        frames.on_screen.contains_key(&theirs.id),
        "pinta-se outra vez"
    );
    assert_eq!(kb::link::boards_showing(&t.hero.documents, k), 1);
}

/// ⭐ REAL — um `.ph2dproj` mexido fora (o Kanban tirado do conjunto, a referência no quadro): nada
/// se pinta nem se apanha; o documento chegar fá-la pintar e apanhar.
#[test]
fn a_reference_to_a_kanban_the_project_lacks_is_not_hit_and_wakes_when_it_arrives() {
    let mut t = T::new();
    arm(&t);
    t.click_bar(Item::Kanban);
    let el = kanban_el(&t).unwrap();
    let k = el.kanban().unwrap().kanban;
    let at = centre(&mut t, k);
    let shared = t.hero.documents.kanbans_shared();
    let doc = shared.borrow_mut().remove(k).unwrap();
    assert_inert(&mut t, k, at);
    shared.borrow_mut().insert(doc);
    t.paint();
    assert!(
        t.hero
            .documents
            .live
            .render_cache
            .kanbans
            .on_screen
            .contains_key(&el.id)
    );
    click(&mut t, at);
    assert_eq!(selection(&t), vec![el.id], "o clique apanha-a outra vez");
}

/// Nenhuma referência VIVA a um Kanban que falta, em quadro nenhum.
fn no_orphan(t: &T) -> bool {
    t.hero.documents.boards().boards().iter().all(|b| {
        b.doc
            .live()
            .filter_map(|e| e.kanban())
            .all(|r| t.hero.documents.kanbans().get(r.kanban).is_some())
    })
}

/// Pela pilha das abas: apagar o quadro B que mostra K, apagar K, e todo `Ctrl+Z`/`Ctrl+Y` — a pilha é
/// LIFO, o K volta sempre antes de B, e nunca há referência órfã (não era defeito).
#[test]
fn deleting_a_board_then_its_kanban_never_brings_the_board_back_beside_an_orphan() {
    let mut t = T::new();
    let bb = t.hero.documents.active().unwrap();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    let a = t.hero.documents.create("A".into());
    crate::screens::hero::document_tabs::register_board(&mut t.hero.store, a);
    t.hero.documents.activate(Some(a));
    assert!(t.hero.documents.remove(bb));
    assert!(t.hero.documents.remove_kanban(k));
    for redo in [false, false, true, true, false, false, true, false, true] {
        let _ = t.hero.documents.undo_tab(redo);
        assert!(no_orphan(&t), "redo={redo}");
    }
}

/// O guarda do [`TabStep::Restore`]: um quadro que volta com um refazer que PÕE uma referência a
/// um Kanban apagado enquanto ele estava fora esquece esse refazer (a ordem forçada, que a pilha
/// não deixa chegar).
#[test]
fn a_restored_board_forgets_the_redo_that_would_put_a_deleted_kanban() {
    let mut t = T::new();
    let bb = t.hero.documents.active().unwrap();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    undo_key(&mut t, false);
    assert!(ref_at(&t, bb, k).is_none(), "o refazer espera");
    let a = t.hero.documents.create("A".into());
    crate::screens::hero::document_tabs::register_board(&mut t.hero.store, a);
    t.hero.documents.activate(Some(a));
    assert!(t.hero.documents.remove(bb));
    assert!(t.hero.documents.remove_kanban(k));
    t.hero.documents.swap_tab_undo_top();
    assert!(t.hero.documents.undo_tab(false).is_some(), "B volta, K não");
    assert!(t.hero.documents.kanbans().get(k).is_none());
    let h = &t.hero.documents.live.histories[&bb];
    assert!(
        !h.can_redo(),
        "o refazer que punha a referência órfã esqueceu-se"
    );
    let _: BoardId = bb;
}
