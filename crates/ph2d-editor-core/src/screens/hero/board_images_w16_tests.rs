//! W16 — espelhar e recortar uma imagem pelo clique REAL na barra dela e pelo teclado.

use super::*;
use crate::screens::hero::board_images::insert_images;
use ph2d_board_model::ImageFile;

const SHIFT: Modifiers = Modifiers {
    shift: true,
    ctrl: false,
    alt: false,
    meta: false,
};

fn put(t: &mut T) {
    let f = ImageFile::new(vec![7; 4], [120, 80]);
    assert_eq!(insert_images(&mut t.hero, vec![f], None), 1);
    t.paint();
}

fn image(t: &T) -> Element {
    t.elements()
        .into_iter()
        .find(|e| e.image().is_some())
        .expect("uma imagem")
}

/// ⭐ Os dois botões de espelhar estão na barra da imagem; o clique vira a imagem nesse eixo, o
/// segundo clique devolve-a; Shift+H e Shift+V fazem o mesmo; o desfazer volta um passo.
#[test]
fn the_flip_buttons_and_keys_mirror_the_picture() {
    let mut t = T::new();
    put(&mut t);
    for it in [Item::Flip(0), Item::Flip(1)] {
        assert!(t.find(node(it)).is_some(), "{it:?} na barra da imagem");
    }
    let flip = |t: &T| image(t).image().unwrap().flip;
    t.click_bar(Item::Flip(0));
    assert_eq!(flip(&t), [true, false]);
    t.click_bar(Item::Flip(1));
    assert_eq!(flip(&t), [true, true]);
    t.click_bar(Item::Flip(0));
    assert_eq!(flip(&t), [false, true]);
    assert!(t.key(BoardKey::Char('h'), SHIFT, None));
    assert_eq!(flip(&t), [true, true]);
    assert!(t.key(BoardKey::Char('V'), SHIFT, None));
    assert_eq!(flip(&t), [true, false]);
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(flip(&t), [true, true], "um passo de desfazer por espelho");
}

/// Os controlos novos vão no FIM: nenhum id que já existia se move.
#[test]
fn the_w16_items_come_last_and_move_no_id() {
    let v = items();
    let g = v
        .iter()
        .position(|i| *i == Item::Gray)
        .expect("o preto e branco");
    assert_eq!(v[g + 1..g + 4], [Item::Flip(0), Item::Flip(1), Item::Crop]);
    // W18 — o vídeo logo depois deles; W23 — a grelha das pontas depois dele e, no fim de tudo,
    // Rascunho/Final da barra da forma.
    assert_eq!(v[g + 4], Item::Video);
    assert_eq!(v[g + 5], Item::MoreHeads(0));
    let n = v.len();
    assert_eq!(v[n - 2..], [Item::Look(false), Item::Look(true)]);
    assert!(
        v[g + 5..n - 2]
            .iter()
            .all(|i| matches!(i, Item::MoreHeads(_) | Item::Head(..)))
    );
}

/// ⭐ O recorte entra pelo botão da barra e por `Enter` (com a imagem seleccionada), e sai por `Enter`
/// e `Escape` — sem `Enter` abrir um texto na imagem; o desenho recebe a imagem em recorte.
#[test]
fn the_crop_button_and_enter_open_the_crop_and_enter_or_escape_close_it() {
    let mut t = T::new();
    put(&mut t);
    let cropping = |t: &T| {
        t.hero
            .documents
            .live
            .editor
            .as_ref()
            .and_then(ph2d_board_edit::Editor::cropping)
    };
    assert!(
        t.find(node(Item::Crop)).is_some(),
        "o botão na barra da imagem"
    );
    t.click_bar(Item::Crop);
    let id = image(&t).id;
    assert_eq!(cropping(&t), Some(id));
    t.paint();
    assert_eq!(t.hero.documents.live.render_cache.cropping, Some(id));
    assert!(t.key(BoardKey::Escape, Modifiers::default(), None));
    assert_eq!(cropping(&t), None);
    assert!(t.key(BoardKey::Enter, Modifiers::default(), None));
    assert_eq!(cropping(&t), Some(id), "Enter numa imagem recorta");
    let editing = |t: &T| {
        t.hero
            .documents
            .live
            .editor
            .as_ref()
            .is_some_and(ph2d_board_edit::Editor::is_editing_text)
    };
    assert!(!editing(&t), "e não abre um texto nela");
    assert!(t.key(BoardKey::Enter, Modifiers::default(), None));
    assert_eq!(cropping(&t), None);
    t.paint();
    assert_eq!(t.hero.documents.live.render_cache.cropping, None);
}

/// ⭐ Auditoria W16 — o botão de recortar só aparece quando o clique recorta (UMA imagem): com duas
/// seleccionadas não se pinta (era um clique morto); e trocar de ferramenta larga o recorte.
#[test]
fn the_crop_button_shows_only_when_it_crops_and_a_tool_leaves_the_crop() {
    let mut t = T::new();
    let f = |n: u8| ImageFile::new(vec![n; 4], [120, 80]);
    assert_eq!(insert_images(&mut t.hero, vec![f(1), f(2)], None), 2);
    t.paint();
    assert!(t.find(node(Item::Flip(0))).is_some(), "a barra das imagens");
    assert!(
        t.find(node(Item::Crop)).is_none(),
        "duas imagens: sem recortar"
    );
    let cropping = |t: &T| {
        t.hero
            .documents
            .live
            .editor
            .as_ref()
            .and_then(ph2d_board_edit::Editor::cropping)
    };
    let mut t = T::new();
    put(&mut t);
    t.click_bar(Item::Crop);
    assert!(cropping(&t).is_some());
    t.click_bar(Item::Tool(ph2d_board_edit::Tool::Shape(
        ph2d_board_model::ShapeType::Rectangle,
    )));
    t.paint();
    assert_eq!(cropping(&t), None, "outra ferramenta larga o recorte");
}

/// Auditoria W16 — o editor é um só para todos os quadros: trocar de aba fecha o recorte (no outro
/// quadro o mesmo id é outro elemento).
#[test]
fn switching_boards_closes_the_crop() {
    let mut t = T::new();
    put(&mut t);
    t.click_bar(Item::Crop);
    let editor = |t: &T| {
        t.hero
            .documents
            .live
            .editor
            .as_ref()
            .and_then(ph2d_board_edit::Editor::cropping)
    };
    assert!(editor(&t).is_some());
    let other = t.hero.documents.create("B".into());
    t.hero.documents.activate(Some(other));
    t.paint();
    assert_eq!(editor(&t), None);
    assert_eq!(t.hero.documents.live.render_cache.cropping, None);
}
