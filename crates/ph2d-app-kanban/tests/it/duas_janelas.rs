//! ⭐ W4 — **o MESMO Kanban em duas janelas** (ADR-0195 §3.1): a aba e o elemento do canvas do Quadro
//! abertos ao mesmo tempo, pelo hero inteiro (`paint_hero_screen`, os ganchos instalados pelo
//! `ph2d_app_kanban::install` como na shell). Uma operação numa aparece na outra no quadro seguinte,
//! nos dois sentidos: arrastar um cartão no canvas → a aba mostra-o na lista nova; renomear uma lista
//! na aba → o canvas pinta o nome novo.

use ph2d_board_model::ElementId;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::board_video::kanban::{self as kb, link};
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroScreen, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_kanban_model::{KanbanId, ListId};
use ph2d_text::TextSystem;
use ph2d_vector::{Point, VectorScene};

struct Duas {
    hero: HeroScreen,
    ts: TextSystem,
    agora: u128,
}

impl Duas {
    /// A shell: a família instalada (os ganchos do canvas), a cena 1 na aba, e um Quadro com o MESMO
    /// Kanban posto por «Place on board».
    fn new() -> (Self, KanbanId, ElementId) {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut hero = HeroScreen::new(NodeId(1));
        ph2d_app_kanban::install(&mut hero);
        let mut d = Self {
            hero,
            ts: TextSystem::without_system_fonts(),
            agora: 1_000_000_000_000,
        };
        let id = ph2d_app_kanban::smoke::stage(&mut d.hero, 1).expect("cena 1");
        d.quadros(2);
        d.hero.documents.create("Board".into());
        link::request(link::BoardRequest::Place { kanban: id });
        d.quadros(3);
        let el = link::placements(&d.hero.documents, id)
            .first()
            .map(|p| p.element)
            .expect("o Kanban posto no quadro");
        (d, id, el)
    }

    fn quadro(&mut self) {
        self.agora += crate::QUADRO_NS;
        self.hero.store.note_clock(self.agora);
        ph2d_app_kanban::frame(&mut self.hero);
        ph2d_editor_core::screens::hero::document_tabs::follow_surfaces(&mut self.hero);
        ph2d_board_render::install_kanbans(self.hero.documents.kanbans_shared());
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.ts);
    }

    fn quadros(&mut self, n: usize) {
        for _ in 0..n {
            self.quadro();
        }
    }

    fn rato(&mut self, kind: PointerKind, (x, y): (f32, f32)) -> bool {
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: self.agora,
        };
        let p = PointerButton::Primary;
        board_view::pointer(&mut self.hero, input, kind, p, x, y, true)
    }

    fn move_(&mut self, (x, y): (f32, f32)) {
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: self.agora,
        };
        board_view::pointer_move(&mut self.hero, input, x, y);
    }

    fn docs(&mut self) -> &mut ph2d_app_kanban::KanbanDocs {
        let s = self.hero.documents.surfaces_mut();
        s.find_mut::<ph2d_app_kanban::KanbanDocs>().expect("família")
    }

    fn lista(&self, id: KanbanId, titulo: &str) -> ListId {
        let k = self.hero.documents.kanbans();
        let l = k.get(id).unwrap().lists.values().find(|l| l.title == titulo);
        l.expect("lista").id
    }

    /// O ponto de ecrã do centro de um rect em px da vista do elemento `el`.
    fn ecra(&self, el: ElementId, r: Rect) -> (f32, f32) {
        let pl = kb::placement(&self.hero, el).expect("o Kanban à vista");
        let k = self.hero.documents.active_board().unwrap().doc.get(el).unwrap();
        let z = k.kanban().unwrap().view.zoom;
        let c = (f64::from(r.x + r.w / 2.0) * z, f64::from(r.y + r.h / 2.0) * z);
        let p = pl.to_screen * Point::new(c.0, c.1);
        (p.x as f32, p.y as f32)
    }
}

#[test]
fn a_aba_e_o_canvas_mostram_o_mesmo_kanban_nos_dois_sentidos() {
    let (mut d, id, el) = Duas::new();
    let board = d.hero.documents.active().expect("o quadro activo");
    assert!(
        kb::placement(&d.hero, el).is_some_and(|p| p.lod == ph2d_board_render::KanbanLod::High),
        "a zoom 1 o Kanban do canvas está inteiro"
    );
    let backlog = d.lista(id, "Backlog");
    let todo = d.lista(id, "To Do");
    let card = d.hero.documents.kanbans().get(id).unwrap().cards_in(backlog)[0].id;

    // ── canvas → aba: entrar no Kanban (seleccionado, 2.º clique) e arrastar um cartão para a To Do.
    board_view::select(&mut d.hero, [el]);
    d.quadro();
    let probe = d.docs().probe_canvas(el.0).expect("a vista do elemento pintou");
    let r = probe.cards.iter().find(|c| c.0 == card).expect("o cartão no canvas").1;
    let alvo = probe.lists.iter().find(|l| l.0 == todo).unwrap().1;
    let de = d.ecra(el, r);
    let para = d.ecra(el, Rect::new(alvo.x, alvo.y + 60.0, alvo.w, 40.0));
    assert!(d.rato(PointerKind::Down, de), "o 2.º clique entra no Kanban");
    assert_eq!(kb::active(&d.hero), Some(el));
    for i in 1..=8 {
        let t = i as f32 / 8.0;
        d.move_((de.0 + (para.0 - de.0) * t, de.1 + (para.1 - de.1) * t));
        d.quadro();
    }
    assert!(d.rato(PointerKind::Up, para));
    d.quadro();
    assert_eq!(
        d.hero.documents.kanbans().get(id).unwrap().cards[&card].list,
        todo,
        "o documento tem o cartão na To Do"
    );
    // A aba, no quadro seguinte, mostra-o lá.
    d.docs().open(id);
    d.quadro();
    // (as molas da aba levam-no de onde ela o viu da última vez até ao lugar novo)
    for _ in 0..240 {
        if !d.docs().probe_moving(id.0) {
            break;
        }
        d.quadro();
    }
    let na_aba = d.docs().probe_card_rect(id.0, card).expect("o cartão na aba");
    let todo_aba = d.docs().probe_list_rect(id.0, todo).unwrap();
    assert!(
        na_aba.x >= todo_aba.x && na_aba.x + na_aba.w <= todo_aba.x + todo_aba.w,
        "a aba pinta o cartão dentro da To Do: {na_aba:?} em {todo_aba:?}"
    );

    // ── aba → canvas: renomear a Backlog na aba (clicar o título, escrever, Enter).
    let r = d.docs().probe_list_rect(id.0, backlog).unwrap();
    let t = (r.x + 30.0, r.y + 20.0);
    assert!(d.rato(PointerKind::Down, t) && d.rato(PointerKind::Up, t));
    d.quadro();
    assert_eq!(d.docs().probe_field(id.0).as_deref(), Some("Backlog"));
    // O campo abre com o texto todo seleccionado (o Trello): escrever substitui-o.
    for c in "Ideas".chars() {
        let k = BoardKey::Char(c.to_ascii_lowercase());
        let s = c.to_string();
        board_keys::key(&mut d.hero, &mut d.ts, k, Modifiers::default(), true, Some(&s), None);
    }
    board_keys::key(&mut d.hero, &mut d.ts, BoardKey::Enter, Modifiers::default(), true, Some("\r"), None);
    d.quadro();
    d.hero.documents.activate(Some(board));
    d.quadro();
    let titulos = d.docs().probe_canvas(el.0).unwrap().titles;
    assert!(
        titulos.iter().any(|t| t == "IDEAS"),
        "o canvas pinta o nome novo no quadro seguinte: {titulos:?}"
    );
}

fn linha(rows: &[(String, Rect)], nome: &str) -> Rect {
    rows.iter()
        .find(|(n, _)| n == nome)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{nome} não está em {rows:?}"))
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

impl Duas {
    fn clica(&mut self, p: (f32, f32)) {
        assert!(self.rato(PointerKind::Down, p), "o carregar em {p:?}");
        self.rato(PointerKind::Up, p);
        self.quadro();
    }
}

/// «Place on board» põe uma referência NOVA no quadro activo; com o Kanban em dois quadros, «Show on
/// board» pergunta qual (o popover com os nomes) e leva ao escolhido, com a câmara no elemento.
#[test]
fn show_on_board_escolhe_entre_dois_quadros_e_centra_a_camara() {
    let (mut d, id, _) = Duas::new();
    let segundo = d.hero.documents.create("Second".into());
    link::request(link::BoardRequest::Place { kanban: id });
    d.quadros(2);
    let onde = link::placements(&d.hero.documents, id);
    assert_eq!(onde.len(), 2, "o mesmo Kanban em dois quadros: {onde:?}");
    let el2 = onde.iter().find(|p| p.board == segundo).unwrap().element;
    // Na aba: «…» → «Show on board» → o popover dos quadros → «Second».
    d.docs().open(id);
    d.quadro();
    let menu = d.docs().probe_rect(id.0, "menu").unwrap();
    d.clica(centro(menu));
    let rows = d.docs().probe_menu_rows(id.0);
    d.clica(centro(linha(&rows, "ShowOnBoard")));
    let rows = d.docs().probe_menu_rows(id.0);
    assert_eq!(rows.len(), 2, "uma linha por quadro: {rows:?}");
    d.clica(centro(linha(&rows, "Board(1)")));
    d.quadro();
    assert_eq!(d.hero.documents.active(), Some(segundo), "o quadro escolhido");
    let b = d.hero.documents.active_board().unwrap();
    let e = b.doc.get(el2).unwrap();
    assert_eq!(
        [b.camera.center_x, b.camera.center_y],
        [e.x + e.w / 2.0, e.y + e.h / 2.0],
        "a câmara no meio do Kanban"
    );
}

/// No canvas, clicar num cartão abre o VERSO por cima do quadro (o modal: o rato é dele em todo o
/// quadro); clicar no fundo escuro fecha-o e o Kanban continua activo.
#[test]
fn o_verso_abre_no_canvas_por_cima_do_quadro_e_fecha_no_fundo() {
    let (mut d, id, el) = Duas::new();
    let backlog = d.lista(id, "Backlog");
    let card = d.hero.documents.kanbans().get(id).unwrap().cards_in(backlog)[0].id;
    board_view::select(&mut d.hero, [el]);
    d.quadro();
    let probe = d.docs().probe_canvas(el.0).unwrap();
    let r = probe.cards.iter().find(|c| c.0 == card).unwrap().1;
    let p = d.ecra(el, r);
    d.clica(p);
    assert_eq!(d.docs().probe_canvas(el.0).unwrap().back, Some(card), "o verso abriu");
    // Um canto do quadro, longe do modal centrado: o fundo escuro do verso, não o quadro.
    d.clica((40.0, HERO_VIEWPORT_H - 80.0));
    assert_eq!(d.docs().probe_canvas(el.0).unwrap().back, None, "o fundo fecha o verso");
    assert_eq!(kb::active(&d.hero), Some(el), "e o Kanban continua activo");
}

/// De mais longe (nível médio) o cartão mostra só o título e a capa: fica mais baixo que o inteiro.
#[test]
fn no_nivel_medio_o_cartao_perde_os_distintivos() {
    let (mut d, id, el) = Duas::new();
    let k = d.hero.documents.kanbans().get(id).unwrap().clone();
    let card = k.cards.values().find(|c| c.title == "Storyboard act 1").unwrap().id;
    let alto = |d: &mut Duas| {
        let p = d.docs().probe_canvas(el.0).unwrap();
        p.cards.iter().find(|c| c.0 == card).unwrap().1.h
    };
    let inteiro = alto(&mut d);
    d.hero.documents.active_board_mut().unwrap().camera.zoom = 0.6;
    d.quadros(2);
    let lod = kb::placement(&d.hero, el).unwrap().lod;
    assert_eq!(lod, ph2d_board_render::KanbanLod::Mid);
    for _ in 0..240 {
        d.quadro();
        if alto(&mut d) < inteiro - 1.0 {
            break;
        }
    }
    assert!(alto(&mut d) < inteiro, "sem rótulos nem distintivos o cartão encolhe");
}
