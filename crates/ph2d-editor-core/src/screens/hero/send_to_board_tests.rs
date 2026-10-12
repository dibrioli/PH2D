//! «Send to board…» (W15) pelo caminho REAL do ecrã: o pintor do menu da Hierarquia esconde a linha
//! para os objectos que um quadro não recebe (e o que não se pinta não se clica), o modal lista os
//! quadros pela ordem das abas, filtra pela busca e o clique de verdade levanta a escolha.

use super::*;
use crate::interaction::{KEY_ENTER, KEY_ESCAPE};
use crate::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use bumpalo::Bump;
use ph2d_board_model::ImageFile;
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};

const SENT_ROW: NodeId = NodeId(100_900);

fn painted() -> (HeroScreen, TextSystem) {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut text = TextSystem::without_system_fonts();
    repaint(&mut hero, &mut text);
    (hero, text)
}

fn repaint(hero: &mut HeroScreen, text: &mut TextSystem) {
    let mut scene = VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(hero, vp, &mut scene, text);
}

/// O centro do alvo `id` onde quer que o pintor o tenha registado (passo de 2 px).
fn locate(hero: &HeroScreen, id: NodeId) -> Option<(f32, f32)> {
    let vp = hero.last_viewport;
    let mut hits = Vec::new();
    let mut y = vp.y;
    while y < vp.y + vp.h {
        let mut x = vp.x;
        while x < vp.x + vp.w {
            if hero.hit_index.hit(x, y) == Some(id) {
                hits.push((x, y));
            }
            x += 2.0;
        }
        y += 2.0;
    }
    let n = hits.len() as f32;
    (!hits.is_empty()).then(|| {
        let sx: f32 = hits.iter().map(|p| p.0).sum();
        let sy: f32 = hits.iter().map(|p| p.1).sum();
        (sx / n, sy / n)
    })
}

fn ptr(kind: PointerKind, (x, y): (f32, f32)) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

/// Down + Up sobre `id` pelo despacho do hero, e cada evento pelo `apply_event` — o caminho da shell.
fn click(hero: &mut HeroScreen, id: NodeId) {
    let at = locate(hero, id).unwrap_or_else(|| panic!("{id:?} não está pintado"));
    let arena = Bump::new();
    let mut events = hero
        .handle_pointer(ptr(PointerKind::Down, at), &arena)
        .to_vec();
    events.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, at), &arena));
    for e in events {
        hero.apply_event(e);
    }
}

fn key(hero: &mut HeroScreen, keycode: u32) {
    let arena = Bump::new();
    let e = KeyEvent {
        keycode,
        modifiers: Modifiers::default(),
        kind: KeyKind::Down,
        timestamp_ns: 0,
    };
    for e in hero.handle_key(e, &arena).to_vec() {
        hero.apply_event(e);
    }
}

fn type_text(hero: &mut HeroScreen, s: &str) {
    for ch in s.chars() {
        let arena = Bump::new();
        for e in hero.handle_text_input(ch, &arena).to_vec() {
            hero.apply_event(e);
        }
    }
}

/// Os pedidos que o modal levantou.
fn chosen(hero: &mut HeroScreen) -> Vec<(NodeId, Option<u64>)> {
    hero.bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::Hierarchy(HierRequest::SendToBoardChosen { row, board }) => {
                Some((row, board))
            }
            _ => None,
        })
        .collect()
}

/// Três quadros e a aba da cena activa, como quem está na Hierarquia.
fn with_boards(names: &[&str]) -> (HeroScreen, TextSystem, Vec<BoardId>) {
    let (mut hero, mut text) = painted();
    let ids = names
        .iter()
        .map(|n| hero.documents.create((*n).to_owned()))
        .collect();
    hero.documents.activate(None);
    repaint(&mut hero, &mut text);
    (hero, text, ids)
}

/// ⭐ A linha só EXISTE para um objecto que um quadro recebe: sem a linha na lista publicada ela não
/// se pinta (e por isso não se clica); com ela, pinta-se. O vizinho «Export Image…» é o controlo: ele
/// pinta-se nas duas. (Mutação: `menu_row_is_hidden` devolver sempre `false` ⇒ a 1.ª asserção — RED.)
#[test]
fn the_send_row_exists_only_for_a_row_that_goes_to_a_board() {
    let (mut hero, mut text) = painted();
    let menu = ContextMenuRequest {
        x: 200.0,
        y: 120.0,
        kind: ContextMenuKind::HierarchyRow { row: SENT_ROW },
    };
    hero.store.open_context_menu(menu);
    repaint(&mut hero, &mut text);
    assert!(
        locate(&hero, ids::CTX_MENU_HIER_EXPORT_IMAGE).is_some(),
        "o menu abriu"
    );
    assert!(
        locate(&hero, ids::CTX_MENU_HIER_SEND_TO_BOARD).is_none(),
        "«Send to board…» pintado para um objecto que um quadro não recebe"
    );
    hero.store
        .set_hierarchy_board_rows(std::iter::once(SENT_ROW).collect());
    repaint(&mut hero, &mut text);
    assert!(
        locate(&hero, ids::CTX_MENU_HIER_SEND_TO_BOARD).is_some(),
        "«Send to board…» não se pinta para uma imagem"
    );
    assert!(goes_to_board(ObjectKind::Image) && goes_to_board(ObjectKind::Vector));
    assert!(goes_to_board(ObjectKind::Flip) && !goes_to_board(ObjectKind::Skeleton));
    assert!(!goes_to_board(ObjectKind::Empty) && !goes_to_board(ObjectKind::Painted));
}

/// ⭐ O modal lista os quadros pela ordem das abas; o clique de verdade numa linha levanta a escolha
/// DESSE quadro e fecha; «New board» levanta `None`; a busca filtra e o `Enter` escolhe o 1.º.
#[test]
fn the_modal_lists_the_boards_and_a_real_click_raises_the_choice() {
    let (mut hero, mut text, b) = with_boards(&["Moodboard", "Ideas", "Sprint ideas"]);
    open(&mut hero.store, SENT_ROW);
    repaint(&mut hero, &mut text);
    for slot in 0..3 {
        assert!(
            locate(&hero, ids::CTX_SEND_BOARD_ROWS[slot]).is_some(),
            "linha {slot}"
        );
    }
    assert!(
        locate(&hero, ids::CTX_SEND_BOARD_ROWS[3]).is_none(),
        "uma 4.ª linha sem quadro"
    );
    click(&mut hero, ids::CTX_SEND_BOARD_ROWS[1]);
    assert_eq!(chosen(&mut hero), vec![(SENT_ROW, Some(b[1].0))]);
    assert!(hero.store.context_menu().is_none(), "o modal ficou aberto");

    open(&mut hero.store, SENT_ROW);
    repaint(&mut hero, &mut text);
    type_text(&mut hero, "SPRINT");
    repaint(&mut hero, &mut text);
    assert!(
        locate(&hero, ids::CTX_SEND_BOARD_ROWS[1]).is_none(),
        "a busca não filtrou"
    );
    key(&mut hero, KEY_ENTER);
    assert_eq!(chosen(&mut hero), vec![(SENT_ROW, Some(b[2].0))]);

    open(&mut hero.store, SENT_ROW);
    repaint(&mut hero, &mut text);
    click(&mut hero, ids::CTX_SEND_BOARD_NEW);
    assert_eq!(chosen(&mut hero), vec![(SENT_ROW, None)]);
    assert_eq!(
        hero.documents.boards().boards().len(),
        3,
        "escolher não cria nada"
    );
}

/// O `Esc` desiste: o modal fecha e nada segue para a shell.
#[test]
fn escape_gives_up() {
    let (mut hero, mut text, _) = with_boards(&["Moodboard"]);
    open(&mut hero.store, SENT_ROW);
    repaint(&mut hero, &mut text);
    key(&mut hero, KEY_ESCAPE);
    assert!(hero.store.context_menu().is_none());
    assert!(chosen(&mut hero).is_empty());
}

/// ⭐ Mais quadros do que linhas: as doze primeiras, e a busca alcança o resto.
#[test]
fn more_boards_than_rows_are_reached_by_the_search() {
    let names: Vec<String> = (1..=15).map(|n| format!("B{n:02}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let (hero, _, b) = with_boards(&refs);
    let (shown, more) = visible(&hero.documents, "");
    assert_eq!((shown.len(), more), (ids::CTX_SEND_BOARD_ROWS.len(), 3));
    assert_eq!(visible(&hero.documents, "b14").0, vec![b[13]]);
}

/// ⭐ Um quadro sem desenho ainda (criado e escrito no mesmo passo) recebe a imagem no centro da
/// CÂMARA, no tamanho natural — antes, `insert_images` devolvia `0` sem área.
#[test]
fn an_image_lands_in_a_board_that_was_never_drawn() {
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(hero.last_layout.is_none());
    hero.documents.create("Fresh".to_owned());
    let file = ImageFile::new(vec![7; 4], [300, 120]);
    assert_eq!(
        super::super::board_images::insert_images(&mut hero, vec![file], None),
        1
    );
    let b = hero.documents.active_board().expect("activo");
    let img = b.doc.live_in_z_order()[0].clone();
    assert_eq!([img.w, img.h], [300.0, 120.0]);
    let c = img.center();
    assert!((c[0] - b.camera.center_x).abs() < 1e-9 && (c[1] - b.camera.center_y).abs() < 1e-9);
}
