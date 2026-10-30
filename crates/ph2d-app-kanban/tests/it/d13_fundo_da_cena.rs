//! D13 — «Use as Kanban background…» da Hierarquia (dono 30/10): a imagem que a shell leu do objecto
//! passa a ser o fundo do Kanban escolhido (UM passo de desfazer, a aba abre-se), ou de um Kanban
//! NOVO; o que não é imagem e o objecto sem píxeis dizem porquê, sem tocar no fundo.

use crate::Mesa;
use crate::d11_fundo::foto;
use ph2d_kanban_model::{Background, FileId};

#[test]
fn d13_the_picture_of_a_scene_object_becomes_the_chosen_kanban_background() {
    let (mut m, id) = Mesa::cena1();
    let before = m.kanban(id).header.background;
    let png = foto(48, 30);
    let t =
        ph2d_app_kanban::scene_background(&mut m.hero, Some(id.0), "Hero", Ok(Some(png.clone())));
    assert!(t.is_none(), "{t:?}");
    m.quadro();
    assert_eq!(
        m.kanban(id).header.background,
        Background::Image(FileId::of(&png))
    );
    assert_eq!(
        m.hero
            .documents
            .kanbans()
            .file(FileId::of(&png))
            .map(|f| f.size),
        Some(Some([48, 30]))
    );
    assert!(
        m.docs().open_kanbans().contains(&id),
        "a aba do Kanban abriu"
    );
    m.desfaz();
    assert_eq!(
        m.kanban(id).header.background,
        before,
        "UM passo de desfazer"
    );

    for file in [
        Err("board.send.unreadable"),
        Ok(None),
        Ok(Some(b"nada".to_vec())),
    ] {
        let t = ph2d_app_kanban::scene_background(&mut m.hero, Some(id.0), "Hero", file);
        assert!(t.is_some(), "o porquê vai para o ecrã");
        assert_eq!(m.kanban(id).header.background, before, "e o fundo fica");
    }
    let t =
        ph2d_app_kanban::scene_background(&mut m.hero, Some(9_999), "Hero", Ok(Some(png.clone())));
    assert!(t.is_some(), "um Kanban apagado entre o pedido e a escolha");
}

#[test]
fn d13_new_kanban_is_created_with_the_picture_as_background() {
    let (mut m, _) = Mesa::cena1();
    let n = m.hero.documents.kanbans().iter().count();
    let png = foto(40, 20);
    assert!(
        ph2d_app_kanban::scene_background(&mut m.hero, None, "Hero", Ok(Some(png.clone())))
            .is_none()
    );
    let set = m.hero.documents.kanbans();
    assert_eq!(set.iter().count(), n + 1);
    let novo = set.iter().last().expect("o Kanban novo");
    assert_eq!(novo.header.background, Background::Image(FileId::of(&png)));
}

/// 📷 O modal «Use as Kanban background» sobre o editor (`#[ignore]`: a placa), em `smoke/D13/`.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --profile smoke --test it d13_foto -- --ignored`
#[test]
#[ignore = "a placa: foto para docs/TrelloClone/smoke/D13/"]
fn d13_foto_o_modal() {
    let (mut m, _) = Mesa::cena1();
    m.hero
        .documents
        .kanbans_shared()
        .borrow_mut()
        .create("Game jam");
    m.hero.documents.activate(None);
    ph2d_editor_core::screens::hero::send_to_board::open_kanban_background(
        &mut m.hero.store,
        ph2d_editor_core::NodeId(1),
    );
    let mut ts = ph2d_text::TextSystem::new();
    let vp = ph2d_editor_core::zones::Rect::new(0.0, 0.0, crate::AREA.w, crate::AREA.h);
    let mut s = ph2d_vector::VectorScene::new();
    for _ in 0..2 {
        s = ph2d_vector::VectorScene::new();
        ph2d_editor_core::screens::hero::paint_hero_screen(&mut m.hero, vp, &mut s, &mut ts);
    }
    crate::j2_fotos::grava_em("D13", "modal_escolhe_o_kanban", &s);
}
