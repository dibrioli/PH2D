//! ⭐ W4 V2 — **o Kanban no canvas do Quadro recebe o que a aba recebe**, pelas portas do hero: a roda
//! (a lista sob o rato; `Shift` e os dois dedos na horizontal; `Ctrl`+roda e a roda fora do Kanban
//! activo = o zoom do Quadro), a composição (IME), o ponteiro (mão / I) e a rolagem de cada lista
//! guardada na vista do ELEMENTO (que viaja no ficheiro do Quadro).

use crate::duas_janelas::Duas;
use ph2d_board_model::{BoardSet, ElementId, KanbanBoardView};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::board_video::kanban as kb;
use ph2d_editor_core::screens::hero::board_view::{self, BoardScroll};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerKind};
use ph2d_kanban_model::{CardId, KanbanId, ListId};

fn vista(d: &Duas, el: ElementId) -> KanbanBoardView {
    let b = d.hero.documents.active_board().unwrap();
    b.doc.get(el).unwrap().kanban().unwrap().view.clone()
}

fn zoom(d: &Duas) -> f64 {
    d.hero.documents.active_board().unwrap().camera.zoom
}

/// Um ponto de ecrã no fundo do Kanban (no vão entre as duas primeiras listas): não agarra nada.
fn fundo(d: &mut Duas, el: ElementId) -> (f32, f32) {
    let probe = d.docs().probe_canvas(el.0).unwrap();
    let mut rs: Vec<Rect> = probe.lists.iter().map(|l| l.1).collect();
    rs.sort_by(|a, b| a.x.total_cmp(&b.x));
    let (a, b) = (rs[0], rs[1]);
    let x = a.x + a.w;
    d.ecra(el, Rect::new(x, a.y + 20.0, b.x - x, 8.0))
}

/// O Kanban posto, seleccionado e ACTIVO (o 2.º clique, no fundo), depois com a vista ampliada para
/// a Backlog transbordar (a roda tem o que rolar).
fn activo() -> (Duas, KanbanId, ElementId) {
    let (mut d, id, el) = Duas::new();
    board_view::select(&mut d.hero, [el]);
    d.quadros(2);
    let p = fundo(&mut d, el);
    d.clica(p);
    assert_eq!(
        kb::active(&d.hero),
        Some(el),
        "o 2.º clique entra no Kanban"
    );
    let b = d.hero.documents.active_board_mut().unwrap();
    b.doc.kanban_view_mut(el).unwrap().zoom = 1.6;
    d.quadros(2);
    (d, id, el)
}

fn cartao(d: &mut Duas, el: ElementId, c: CardId) -> Rect {
    let probe = d.docs().probe_canvas(el.0).unwrap();
    probe.cards.iter().find(|x| x.0 == c).unwrap().1
}

/// A faixa de cima do cartão (dentro do corpo da lista mesmo com o fim dele cortado).
fn topo_de(r: Rect) -> Rect {
    Rect::new(r.x, r.y + 4.0, r.w, 8.0)
}

fn lista_r(d: &mut Duas, el: ElementId, l: ListId) -> Rect {
    let probe = d.docs().probe_canvas(el.0).unwrap();
    probe.lists.iter().find(|x| x.0 == l).unwrap().1
}

/// Um ponto do Quadro à esquerda do Kanban, à altura do meio dele.
fn fora_de(d: &Duas, el: ElementId) -> (f32, f32) {
    let pl = kb::placement(&d.hero, el).unwrap();
    let p = pl.to_screen * ph2d_vector::Point::new(-40.0, pl.size[1] / 2.0);
    (p.x as f32, p.y as f32)
}

fn roda(d: &mut Duas, p: (f32, f32), s: BoardScroll) -> bool {
    let taken = board_view::scroll(&mut d.hero, p.0, p.1, s);
    d.quadro();
    taken
}

/// ⭐ A roda no Kanban activo rola a lista sob o rato (e não dá zoom ao Quadro); `Shift` e os dois
/// dedos de lado rolam o quadro dele; `Ctrl`+roda dá zoom ao Quadro também por cima dele; fora do
/// Kanban, ou com ele só seleccionado, a roda é do Quadro.
#[test]
fn a_roda_no_kanban_activo_rola_as_listas_e_o_ctrl_roda_e_do_quadro() {
    let (mut d, id, el) = activo();
    let backlog = d.lista(id, "Backlog");
    let k = d.hero.documents.kanbans().get(id).unwrap().clone();
    let primeiro = k.cards_in(backlog)[0].id;
    let z0 = zoom(&d);
    let antes = cartao(&mut d, el, primeiro);
    let sobre = d.ecra(el, topo_de(antes));
    // Para baixo (para o utilizador): uma linha = 16 px da janela.
    assert!(
        roda(&mut d, sobre, BoardScroll::Wheel(-16.0)),
        "a roda é do Kanban"
    );
    assert_eq!(zoom(&d), z0, "sem zoom no Quadro");
    let y = vista(&d, el).scroll_y.get(&backlog).copied().unwrap_or(0.0);
    assert!(y > 0.0, "a Backlog rolou e ficou na vista do elemento: {y}");
    d.quadros(30);
    let depois = cartao(&mut d, el, primeiro);
    assert!(depois.y < antes.y, "o cartão subiu: {antes:?} → {depois:?}");
    // Para cima volta ao topo (nunca abaixo de zero).
    for _ in 0..8 {
        roda(&mut d, sobre, BoardScroll::Wheel(16.0));
    }
    assert_eq!(vista(&d, el).scroll_y.get(&backlog), None, "no topo");

    // Shift + roda: o quadro do Kanban na horizontal.
    d.hero.store.set_shift_held(true);
    assert!(roda(&mut d, sobre, BoardScroll::Wheel(-16.0)));
    d.hero.store.set_shift_held(false);
    let x1 = vista(&d, el).scroll_x;
    assert!(x1 > 0.0, "Shift + roda rola na horizontal: {x1}");
    // Os dois dedos para a esquerda: o conteúdo vai com eles, vê-se mais à direita.
    assert!(roda(&mut d, sobre, BoardScroll::Pan([-30.0, 0.0])));
    assert!(
        vista(&d, el).scroll_x > x1,
        "os dois dedos de lado rolam o quadro do Kanban"
    );
    assert_eq!(zoom(&d), z0);

    // Ctrl + roda: o zoom do Quadro, mesmo por cima do Kanban activo; a pinça também.
    d.hero.store.set_cmd_held(true);
    assert!(roda(&mut d, sobre, BoardScroll::Wheel(16.0)));
    d.hero.store.set_cmd_held(false);
    assert!(zoom(&d) > z0, "Ctrl + roda dá zoom ao Quadro");
    let z1 = zoom(&d);
    let r = cartao(&mut d, el, primeiro);
    let sobre = d.ecra(el, topo_de(r));
    assert!(roda(&mut d, sobre, BoardScroll::Pinch(1.2)));
    assert!(zoom(&d) > z1, "a pinça é do Quadro");

    // Fora do Kanban activo a roda é do Quadro.
    let z2 = zoom(&d);
    let fora = fora_de(&d, el);
    roda(&mut d, fora, BoardScroll::Wheel(-16.0));
    assert!(zoom(&d) < z2, "fora do Kanban: zoom no Quadro");
    // Só seleccionado (Esc sai): a roda por cima dele é do Quadro.
    board_keys::key(
        &mut d.hero,
        &mut d.ts,
        BoardKey::Escape,
        Modifiers::default(),
        true,
        None,
        None,
    );
    assert_eq!(kb::active(&d.hero), None);
    let (z3, x3) = (zoom(&d), vista(&d, el).scroll_x);
    let r = cartao(&mut d, el, primeiro);
    let sobre = d.ecra(el, topo_de(r));
    roda(&mut d, sobre, BoardScroll::Wheel(-16.0));
    assert!(zoom(&d) < z3, "seleccionado não é activo: zoom no Quadro");
    assert_eq!(vista(&d, el).scroll_x, x3);
}

/// A rolagem de cada lista é da VISTA do elemento: grava-se no ficheiro do Quadro e volta; mudada por
/// fora (desfazer, um colega), a vista viva adopta-a no quadro seguinte.
#[test]
fn a_rolagem_de_cada_lista_vive_no_elemento_e_no_ficheiro() {
    let (mut d, id, el) = activo();
    let backlog = d.lista(id, "Backlog");
    let primeiro = d
        .hero
        .documents
        .kanbans()
        .get(id)
        .unwrap()
        .cards_in(backlog)[0]
        .id;
    let r = cartao(&mut d, el, primeiro);
    let sobre = d.ecra(el, topo_de(r));
    roda(&mut d, sobre, BoardScroll::Wheel(-16.0));
    let v = vista(&d, el);
    assert!(v.scroll_y.contains_key(&backlog));
    let bytes = d.hero.documents.boards().to_bytes().unwrap();
    let lido = BoardSet::from_bytes(&bytes).unwrap();
    let k = lido
        .boards()
        .iter()
        .find_map(|b| b.doc.get(el)?.kanban().cloned())
        .unwrap();
    assert_eq!(k.view, v, "a vista volta inteira do ficheiro");

    // Por fora: a Backlog de volta ao topo.
    d.quadros(30);
    let rolado = cartao(&mut d, el, primeiro);
    let b = d.hero.documents.active_board_mut().unwrap();
    b.doc.kanban_view_mut(el).unwrap().scroll_y.clear();
    d.quadros(60);
    let topo = cartao(&mut d, el, primeiro);
    assert!(
        topo.y > rolado.y,
        "a vista viva adoptou o topo: {rolado:?} → {topo:?}"
    );
    let l = lista_r(&mut d, el, backlog);
    assert!(topo.y >= l.y, "o 1.º cartão dentro da lista");
}

/// ⭐ A composição (o IME, o carácter confirmado) entra no campo aberto do Kanban no canvas, como na
/// aba; e o ponteiro sobre ele é o I, sobre um cartão a mão, no fundo a seta, fora do Kanban nenhum.
#[test]
fn o_ime_escreve_no_campo_do_canvas_e_o_ponteiro_segue_o_que_esta_por_baixo() {
    let (mut d, id, el) = activo();
    let backlog = d.lista(id, "Backlog");
    let primeiro = d
        .hero
        .documents
        .kanbans()
        .get(id)
        .unwrap()
        .cards_in(backlog)[0]
        .id;

    // O ponteiro: a mão no cartão, a seta no fundo, nada fora; só seleccionado, nada.
    let r = cartao(&mut d, el, primeiro);
    let sobre = d.ecra(el, topo_de(r));
    d.move_(sobre);
    let look = |d: &Duas, p: (f32, f32)| board_view::pointer_look(&d.hero, p.0, p.1);
    assert_eq!(look(&d, sobre), Some(PointerLook::Hand), "a mão num cartão");
    let f = fundo(&mut d, el);
    d.move_(f);
    assert_eq!(look(&d, f), Some(PointerLook::Arrow), "a seta no fundo");
    assert_eq!(
        look(&d, fora_de(&d, el)),
        None,
        "fora do Kanban: o do Quadro"
    );

    // O título da Backlog no lugar; o IME escreve nele (substitui a selecção, como a aba).
    let r = lista_r(&mut d, el, backlog);
    let t = d.ecra(el, Rect::new(r.x + 10.0, r.y + 8.0, 40.0, 24.0));
    assert!(d.rato(PointerKind::Down, t) && d.rato(PointerKind::Up, t));
    d.quadro();
    let campo = |d: &mut Duas| d.docs().probe_canvas(el.0).unwrap().field;
    assert_eq!(
        campo(&mut d).as_deref(),
        Some("Backlog"),
        "o título abriu no lugar"
    );
    d.move_(t);
    assert_eq!(look(&d, t), Some(PointerLook::Text), "o I sobre o texto");
    d.hero.handle_preedit("\u{b4}", None);
    let arena = bumpalo::Bump::new();
    for c in "Idéias".chars() {
        let _ = d.hero.handle_text_input(c, &arena);
    }
    d.hero.handle_preedit("", None);
    assert_eq!(
        campo(&mut d).as_deref(),
        Some("Idéias"),
        "o IME escreveu no campo"
    );
    board_keys::key(
        &mut d.hero,
        &mut d.ts,
        BoardKey::Enter,
        Modifiers::default(),
        true,
        Some("\r"),
        None,
    );
    d.quadro();
    let k = d.hero.documents.kanbans().get(id).unwrap().clone();
    assert_eq!(
        k.lists[&backlog].title, "Idéias",
        "o documento tem o nome composto"
    );

    board_keys::key(
        &mut d.hero,
        &mut d.ts,
        BoardKey::Escape,
        Modifiers::default(),
        true,
        None,
        None,
    );
    board_keys::key(
        &mut d.hero,
        &mut d.ts,
        BoardKey::Escape,
        Modifiers::default(),
        true,
        None,
        None,
    );
    assert_eq!(kb::active(&d.hero), None);
    assert_eq!(
        look(&d, sobre),
        None,
        "só seleccionado: o ponteiro é do Quadro"
    );
}
