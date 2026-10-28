//! W4 do Kanban — o Kanban no desenho: o nível baixo a partir do documento (a moldura com o fundo, o
//! cabeçalho, uma coluna da cor de cada lista), o gancho que o substitui, e onde ficou no ecrã.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ph2d_board_model::{Board, BoardKanban, BoardOp, BoardSet, Element, Rgba, Style};
use ph2d_board_route::RouteCache;
use ph2d_kanban_model::{
    Card, CardId, FracKey, Hue, KanbanId, KanbanOp, KanbanSet, List, ListId, Meta, Moment, Record,
};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Brush, Color, DrawOp, VectorScene};

use super::*;

fn list(id: u64, title: &str, color: Option<Hue>, pos: &FracKey) -> KanbanOp {
    KanbanOp::Put(Box::new(Record::List(List {
        id: ListId(id),
        title: title.into(),
        pos: pos.clone(),
        color,
        card_limit: None,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
    })))
}

fn card(id: u64, list: u64, pos: &FracKey) -> KanbanOp {
    KanbanOp::Put(Box::new(Record::Card(Card {
        id: CardId(id),
        list: ListId(list),
        pos: pos.clone(),
        title: format!("c{id}"),
        description: String::new(),
        labels: Vec::new(),
        members: Vec::new(),
        start: None,
        due: None,
        cover: None,
        fields: BTreeMap::new(),
        location: String::new(),
        votes: Vec::new(),
        created: Moment(0),
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
        origin: None,
        template: false,
    })))
}

/// Um Kanban com uma lista verde (2 cartões) e uma sem cor (1), e o quadro com o elemento dele.
fn fixture() -> (KanbanSource, Board, KanbanId) {
    let mut set = KanbanSet::default();
    let k = set.create("Sprint");
    let a = FracKey::between(None, None);
    let b = FracKey::between(Some(&a), None);
    let ops = vec![
        list(1, "Doing", Some(Hue::Green), &a),
        list(2, "Done", None, &b),
        card(3, 1, &a),
        card(4, 1, &b),
        card(5, 2, &a),
    ];
    set.apply(k, ops);
    let mut bs = BoardSet::default();
    let bid = bs.create("q".into());
    let doc = &mut bs.get_mut(bid).unwrap().doc;
    let bk = BoardKanban::new(k, Style::new(None, None, Rgba([0, 0, 0, 255])));
    let el = Element::new_kanban(doc.mint_id(), doc.z_on_top(), bk, [0.0, 0.0, 960.0, 540.0]);
    BoardOp::Put(el).apply(doc);
    let mut board = bs.boards()[0].clone();
    (board.camera.center_x, board.camera.center_y) = (480.0, 270.0);
    (Rc::new(RefCell::new(set)), board, k)
}

fn ops(b: &Board, c: &mut RenderCache) -> Vec<DrawOp> {
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut scene = VectorScene::new();
    scene.record();
    let mut ts = TextSystem::without_system_fonts();
    let area = [0.0, 0.0, 1280.0, 720.0];
    crate::paint(b, area, &mut scene, Theme::Forge, &mut ts, c, &routes, None);
    scene.take_record()
}

fn fills(ops: &[DrawOp], c: Color) -> usize {
    ops.iter()
        .filter(|o| matches!(o, DrawOp::Fill { brush: Brush::Solid(x), .. } if *x == c))
        .count()
}

/// ⭐ Sem gancho: a moldura com o fundo do Kanban, o cabeçalho, e UMA coluna por lista na cor dela.
#[test]
fn without_a_hook_the_low_level_paints_one_column_per_list_in_its_colour() {
    let (src, b, _) = fixture();
    install_kanbans(src);
    let mut c = RenderCache::default();
    let o = ops(&b, &mut c);
    let th = Theme::Forge;
    assert_eq!(
        fills(&o, crate::token(ColorToken::KanbanBoardBg, th)),
        1,
        "a moldura"
    );
    assert_eq!(
        fills(&o, crate::token(ColorToken::KanbanHeaderBg, th)),
        1,
        "o cabeçalho"
    );
    let (green, ..) = kanban_list_colors(th, Some(Hue::Green));
    let (plain, ..) = kanban_list_colors(th, None);
    assert_ne!(green, plain);
    assert_eq!(fills(&o, green), 1, "a lista verde");
    assert_eq!(fills(&o, plain), 1, "a lista sem cor");
    let pl = c
        .kanbans
        .on_screen
        .get(&b.doc.live().next().unwrap().id)
        .unwrap();
    assert_eq!(pl.lod, KanbanLod::High);
    assert_eq!(pl.size, [960.0, 540.0]);
}

/// A coluna da lista com mais cartões é mais alta (a contagem também se vê na silhueta).
#[test]
fn a_longer_list_is_a_taller_column() {
    let (src, b, _) = fixture();
    install_kanbans(src);
    let o = ops(&b, &mut RenderCache::default());
    let th = Theme::Forge;
    let height = |c: Color| {
        o.iter()
            .find_map(|op| match op {
                DrawOp::Fill {
                    brush: Brush::Solid(x),
                    path,
                    ..
                } if *x == c => Some(ph2d_vector::Shape::bounding_box(path).height()),
                _ => None,
            })
            .unwrap()
    };
    let (green, ..) = kanban_list_colors(th, Some(Hue::Green));
    let (plain, ..) = kanban_list_colors(th, None);
    assert!(height(green) > height(plain));
}

struct Spy(Vec<(KanbanId, KanbanLod)>);

impl KanbanPainter for Spy {
    fn paint(&mut self, p: KanbanPaint<'_>) -> bool {
        self.0.push((p.kanban, p.lod));
        true
    }
}

/// Com o gancho a pintar, o nível baixo não se pinta; o gancho recebe o Kanban e o nível.
#[test]
fn an_installed_hook_paints_instead_of_the_low_level() {
    let (src, b, k) = fixture();
    install_kanbans(src);
    let spy = Rc::new(RefCell::new(Spy(Vec::new())));
    install_kanban_painter(spy.clone());
    assert!(kanban_painter_installed());
    let o = ops(&b, &mut RenderCache::default());
    let (green, ..) = kanban_list_colors(Theme::Forge, Some(Hue::Green));
    assert_eq!(fills(&o, green), 0);
    assert_eq!(spy.borrow().0, vec![(k, KanbanLod::High)]);
}

#[test]
fn the_level_of_detail_follows_the_screen_scale() {
    assert_eq!(KanbanLod::at(0.1), KanbanLod::Low);
    assert_eq!(KanbanLod::at(0.5), KanbanLod::Mid);
    assert_eq!(KanbanLod::at(1.0), KanbanLod::High);
}
