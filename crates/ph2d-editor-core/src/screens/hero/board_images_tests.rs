//! W14 — as imagens pelo caminho REAL do ecrã: a porta que põe imagens no quadro (a que o soltar e o
//! colar chamam), a barra que aparece para uma imagem e o clique que a muda; copiar e colar noutro
//! quadro leva os bytes.

use super::*;
use crate::screens::hero::board_images::insert_images;
use ph2d_board_model::ImageFile;

fn file(n: u8, w: u32, h: u32) -> ImageFile {
    ImageFile::new(vec![n; 4], [w, h])
}

impl T {
    fn images(&self) -> Vec<Element> {
        self.elements()
            .into_iter()
            .filter(|e| e.image().is_some())
            .collect()
    }
}

/// ⭐ A imagem nasce CENTRADA no ponto do ecrã onde se largou, no tamanho natural (cabe), sozinha
/// seleccionada; a barra mostra os cantos e a opacidade (e nada de preenchimento); o clique na
/// opacidade muda a imagem — UM passo de desfazer.
#[test]
fn a_dropped_image_lands_on_the_point_and_its_bar_changes_it() {
    let mut t = T::new();
    let p = t.at(0.3, 0.4);
    assert_eq!(insert_images(&mut t.hero, vec![file(1, 120, 80)], Some([p.0.into(), p.1.into()])), 1);
    t.paint();
    let img = t.images().remove(0);
    let b = t.hero.documents.active_board().unwrap();
    let area = board_view::area_of(board_view::area(t.hero.last_layout.as_ref().unwrap()));
    let c = b.camera.to_screen(area, img.center());
    assert!((c[0] - f64::from(p.0)).abs() < 1e-3 && (c[1] - f64::from(p.1)).abs() < 1e-3);
    assert_eq!([img.w, img.h], [120.0, 80.0]);
    assert!(t.find(node(Item::Round(true))).is_some(), "os cantos");
    assert!(t.find(node(Item::Fill(None))).is_none(), "preenchimento numa imagem");
    let half = OPACITIES.iter().position(|o| *o == 50).expect("há 50 %");
    t.click_bar(Item::Opacity(half));
    assert_eq!(t.images()[0].image().unwrap().style.opacity, 50);
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(t.images()[0].image().unwrap().style.opacity, 100);
}

/// Uma imagem enorme nasce a caber em metade da área à vista (no zoom actual).
#[test]
fn a_huge_image_is_born_at_half_the_visible_area() {
    let mut t = T::new();
    insert_images(&mut t.hero, vec![file(2, 40_000, 20_000)], None);
    let img = t.images().remove(0);
    let b = t.hero.documents.active_board().unwrap();
    let area = board_view::area_of(board_view::area(t.hero.last_layout.as_ref().unwrap()));
    let (fw, fh) = (area[2] / b.camera.zoom / 2.0, area[3] / b.camera.zoom / 2.0);
    assert!(img.w <= fw + 1e-9 && img.h <= fh + 1e-9, "{} × {} > {fw} × {fh}", img.w, img.h);
    assert!((img.w / img.h - 2.0).abs() < 1e-9);
    assert!((img.w - fw).abs() < 1e-9 || (img.h - fh).abs() < 1e-9, "encolheu de mais");
}

/// ⭐ Copiar uma imagem e colá-la noutro quadro leva os bytes dela.
#[test]
fn copy_paste_into_another_board_carries_the_bytes() {
    let mut t = T::new();
    let f = file(3, 50, 50);
    let key = f.id();
    insert_images(&mut t.hero, vec![f], None);
    t.paint();
    assert!(t.key(BoardKey::Char('c'), CTRL, None));
    t.hero.apply_event(WidgetEvent::Click(ids::DOC_TAB_NEW));
    t.paint();
    assert!(t.images().is_empty(), "o quadro novo nasce vazio");
    assert!(t.key(BoardKey::Char('v'), CTRL, None));
    assert_eq!(t.images().len(), 1);
    let b = t.hero.documents.active_board().unwrap();
    assert!(b.file(key).is_some(), "colou sem os bytes");
}

/// ⭐ Auditoria W14 — uma folha de cálculo copiada traz as células E uma imagem delas: o texto ganha
/// (uma nota por célula); só sem texto (uma captura, a imagem de um navegador) a imagem entra.
#[test]
fn text_on_the_clipboard_wins_over_an_image() {
    use crate::screens::hero::board_images::takes_image_paste;
    let t = T::new();
    assert!(!takes_image_paste(&t.hero, Some("a\tb\nc\td")), "a folha de cálculo");
    assert!(takes_image_paste(&t.hero, None), "a captura de ecrã");
    assert!(takes_image_paste(&t.hero, Some("  ")));
}
