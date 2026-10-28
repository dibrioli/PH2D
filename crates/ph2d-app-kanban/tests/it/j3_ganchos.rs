//! J3 — **a porta única do gesto** pela costura do hero: o Butler e os scripts `ph2d.kanban.on`
//! reagem ao arrasto da pessoa, cada um no seu passo de desfazer; o desfazer não reage. E a W6 no
//! elemento do Quadro (o filtro e a edição rápida, com «Convert to sticky note»).

use crate::duas_janelas::Duas;
use crate::{Mesa, pega};
use ph2d_board_model::ElementId;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::board_video::kanban as kb;
use ph2d_editor_core::screens::hero::board_view;
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_kanban_auto::{Action, Added, By, Kind, Spec, Trigger};
use ph2d_kanban_model::{CardId, KanbanId, ListId, Member, Moment};

/// A regra «quando um cartão entra em `list`, arquivá-lo», posta no Kanban.
fn regra_arquiva(m: &Mesa, id: KanbanId, list: ListId) {
    let spec = Spec {
        name: String::new(),
        enabled: true,
        owner: Member("Enio".into()),
        kind: Kind::Rule {
            trigger: Trigger::CardAdded {
                list: Some(list),
                how: Added::Any,
            },
            by: By::Anyone,
            conditions: Vec::new(),
        },
        actions: vec![Action::Archive],
    };
    let shared = m.hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let op = spec
        .create(set.get(id).unwrap(), Moment(0))
        .expect("automação válida");
    set.apply(id, vec![op]);
}

/// Arrasta o 1.º cartão do Backlog para o topo da lista `para` e larga.
fn leva(m: &mut Mesa, id: KanbanId, para: &str) -> CardId {
    let backlog = m.lista(id, "Backlog");
    let to = m.lista(id, para);
    let c = m.cartoes(id, backlog)[0];
    let r = m.rect_cartao(id, c);
    let d = m.rect_lista(id, to);
    let p = pega(r);
    let alvo = (d.x + d.w / 2.0, d.y + 42.0);
    let fim = (
        alvo.0 + (p.0 - r.x) - r.w / 2.0,
        alvo.1 + (p.1 - r.y) - r.h / 2.0,
    );
    m.arrasta(p, fim, 8);
    assert!(m.larga(fim.0, fim.1));
    m.quadro();
    assert_eq!(
        m.kanban(id).cards[&c].list,
        to,
        "o arrasto largou na {para}"
    );
    c
}

#[test]
fn j3_o_butler_reage_ao_arrasto_num_passo_proprio_e_o_desfazer_nao_reage() {
    let (mut m, id) = Mesa::cena1();
    let done = m.lista(id, "Done");
    let backlog = m.lista(id, "Backlog");
    regra_arquiva(&m, id, done);
    let c = leva(&mut m, id, "Done");
    assert!(m.kanban(id).cards[&c].archived, "a regra arquivou o cartão");
    assert_eq!(m.desfaz(), Some(true));
    let k = m.kanban(id);
    assert!(
        !k.cards[&c].archived,
        "o 1.º Ctrl+Z tira o que o Butler fez"
    );
    assert_eq!(k.cards[&c].list, done, "e deixa o gesto da pessoa");
    assert_eq!(m.desfaz(), Some(true));
    assert_eq!(m.kanban(id).cards[&c].list, backlog);
    // Refazer devolve o gesto SEM o Butler correr outra vez.
    let r = ph2d_editor_core::screens::hero::surface_input::undo(&mut m.hero, true);
    m.quadro();
    assert_eq!(r, Some(true));
    let k = m.kanban(id);
    assert_eq!(k.cards[&c].list, done);
    assert!(
        !k.cards[&c].archived,
        "refazer não é gesto: nenhuma regra corre"
    );
}

#[test]
fn j3_um_script_de_kanban_pedido_pelo_editor_corre_e_o_handler_reage_ao_gesto() {
    let (mut m, id) = Mesa::cena1();
    let _ = ph2d_kanban_hooks::take_lines();
    // O que o editor faz no Ctrl+S de um script que usa `ph2d.kanban`.
    ph2d_kanban_hooks::request_run(
        41,
        "watch.luau",
        "print(\"armed\")\nph2d.kanban.on(\"card_moved\", function(e) e.card:comment(\"seen \" .. e.list.name) end)",
    );
    m.quadro();
    let lines = ph2d_kanban_hooks::take_lines();
    assert!(
        lines
            .iter()
            .any(|l| l.key == 41 && l.text == "armed" && !l.error),
        "o print chega ao console do script: {lines:?}"
    );
    assert_eq!(
        ph2d_kanban_hooks::with_scripts(|s| s.loaded().len()),
        1,
        "o handler ficou activo"
    );
    let c = leva(&mut m, id, "Doing");
    let seen = |m: &Mesa| {
        m.kanban(id)
            .comments
            .values()
            .filter(|x| x.card == c && x.text == "seen Doing" && !x.meta.deleted)
            .count()
    };
    assert_eq!(seen(&m), 1, "o handler comentou o movimento");
    assert_eq!(m.desfaz(), Some(true));
    assert_eq!(seen(&m), 0, "o 1.º Ctrl+Z tira o comentário do script");
    assert_eq!(m.desfaz(), Some(true));
    assert_eq!(seen(&m), 0, "desfazer o movimento não acorda o handler");
    ph2d_kanban_hooks::unload(41);
}

#[test]
fn j3_a_aba_nao_mostra_convert_to_sticky_note() {
    let (mut m, id) = Mesa::cena1();
    let c = crate::verso::cartao(&m, id, "Design the fox");
    crate::w6::foca(&mut m, id, c);
    assert!(crate::w6::tecla(&mut m, 'e'));
    let hits = m.docs().probe_w6_hits(id.0);
    assert!(hits.iter().any(|(n, _)| n == "Quick(Archive)"));
    assert!(
        !hits.iter().any(|(n, _)| n == "Quick(ToNote)"),
        "na aba não há Quadro onde pôr a nota"
    );
}

/// Uma tecla pelo hero (premida e solta) e um quadro.
fn tecla(d: &mut Duas, k: BoardKey, text: Option<&str>) {
    let mods = Modifiers::default();
    if crate::compoe_primeiro(&mut d.hero, k, mods) {
        d.quadro();
        return;
    }
    board_keys::key(&mut d.hero, &mut d.ts, k, mods, true, text, None);
    board_keys::key(&mut d.hero, &mut d.ts, k, mods, false, None, None);
    d.quadro();
}

fn cartao_r(d: &mut Duas, el: ElementId, c: CardId) -> Rect {
    let probe = d.docs().probe_canvas(el.0).unwrap();
    probe.cards.iter().find(|x| x.0 == c).unwrap().1
}

/// O Kanban posto no Quadro, seleccionado e ACTIVO (o 2.º clique no fundo, entre as listas).
fn activo() -> (Duas, KanbanId, ElementId) {
    let (mut d, id, el) = Duas::new();
    board_view::select(&mut d.hero, [el]);
    d.quadros(2);
    let probe = d.docs().probe_canvas(el.0).unwrap();
    let mut rs: Vec<Rect> = probe.lists.iter().map(|l| l.1).collect();
    rs.sort_by(|a, b| a.x.total_cmp(&b.x));
    let x = rs[0].x + rs[0].w;
    let fundo = d.ecra(el, Rect::new(x, rs[0].y + 20.0, rs[1].x - x, 8.0));
    d.clica(fundo);
    assert_eq!(
        kb::active(&d.hero),
        Some(el),
        "o 2.º clique entra no Kanban"
    );
    (d, id, el)
}

/// ⭐ A W6 DENTRO do Quadro: `F` abre o filtro do elemento; com o foco num cartão, `E` abre a
/// edição rápida no lugar, com «Convert to sticky note» (que leva o cartão para uma nota).
#[test]
fn j3_no_quadro_o_f_abre_o_filtro_e_o_e_a_edicao_rapida_com_a_nota() {
    let (mut d, id, el) = activo();
    tecla(&mut d, BoardKey::Char('f'), Some("f"));
    let w6 = d.docs().probe_canvas_w6(el.0).expect("a vista do elemento");
    assert_eq!(
        w6.over.as_deref(),
        Some("Filter"),
        "F abre o filtro no elemento"
    );
    assert!(
        w6.hits.iter().any(|h| h.0 == "Inside"),
        "o painel pintou-se"
    );
    tecla(&mut d, BoardKey::Escape, None);
    assert_eq!(
        d.docs().probe_canvas_w6(el.0).unwrap().over,
        None,
        "Esc fecha-o"
    );
    assert_eq!(
        kb::active(&d.hero),
        Some(el),
        "o Esc do painel não sai do Kanban"
    );

    let k = d.hero.documents.kanbans().get(id).unwrap().clone();
    let c = k
        .cards
        .values()
        .find(|c| c.title == "Design the fox")
        .unwrap()
        .id;
    let r = cartao_r(&mut d, el, c);
    let p = d.ecra(el, r);
    d.clica(p);
    assert_eq!(
        d.docs().probe_canvas(el.0).unwrap().back,
        Some(c),
        "o clique abre o verso"
    );
    tecla(&mut d, BoardKey::Escape, None);
    tecla(&mut d, BoardKey::Char('e'), Some("e"));
    let w6 = d.docs().probe_canvas_w6(el.0).unwrap();
    assert_eq!(w6.quick, Some(c), "E abre a edição rápida no elemento");
    assert!(w6.hits.iter().any(|h| h.0 == "Quick(Archive)"));
    let to_note = w6.hits.iter().rev().find(|h| h.0 == "Quick(ToNote)");
    let to_note = to_note.expect("no Quadro há «Convert to sticky note»").1;
    let notes = |d: &Duas| d.hero.documents.active_board().unwrap().doc.live().count();
    let antes = notes(&d);
    let p = d.ecra(el, to_note);
    d.clica(p);
    d.quadros(2);
    let k = d.hero.documents.kanbans().get(id).unwrap().clone();
    assert!(
        k.cards.get(&c).is_none_or(|x| x.meta.deleted),
        "o cartão saiu do Kanban"
    );
    assert_eq!(notes(&d), antes + 1, "e virou uma nota no Quadro");
}
