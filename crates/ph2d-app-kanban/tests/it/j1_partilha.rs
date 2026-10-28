//! J1 — «Share» pela costura do hero: DOIS heróis neste processo, ligados por 127.0.0.1 (a entrada da
//! internet sem STUN nem DHT, como o `loopback` da `ph2d-kanban-sync`). A Ana partilha o Kanban pelo
//! popover; a Bia cola o link e entra; mover um cartão num aparece no outro; os avatares mostram o
//! colega, e o cartão que a Bia abre ganha o anel no ecrã da Ana.

use crate::w6::toca;
use crate::{Mesa, pega};
use ph2d_kanban_model::{KanbanId, ListId};
use std::time::{Duration, Instant};

const PRAZO: Duration = Duration::from_secs(30);

pub(crate) fn com_nome(nome: &str) -> Mesa {
    let mut m = Mesa::new();
    m.hero.documents.set_author(nome.into());
    m.docs().share_local_for_tests();
    m
}

/// Quadros nos dois até `feito` (com um pouco de espera: a rede corre noutras threads).
pub(crate) fn ate(
    a: &mut Mesa,
    b: &mut Mesa,
    o_que: &str,
    mut feito: impl FnMut(&mut Mesa, &mut Mesa) -> bool,
) {
    let t = Instant::now();
    while !feito(a, b) {
        assert!(t.elapsed() < PRAZO, "não aconteceu: {o_que}");
        a.quadro();
        b.quadro();
        std::thread::sleep(Duration::from_millis(3));
    }
}

fn lista_de(m: &Mesa, k: KanbanId, titulo_cartao: &str) -> Option<String> {
    let kb = m.kanban(k);
    let c = kb.cards.values().find(|c| c.title == titulo_cartao)?;
    kb.lists.get(&c.list).map(|l| l.title.clone())
}

/// A Ana partilha a cena 1 pelo popover e a Bia entra pelo link: `(ana, bia, kanban da Ana, o
/// mesmo na Bia)`, os dois já a verem-se.
pub(crate) fn ligados() -> (Mesa, Mesa, KanbanId, KanbanId) {
    let mut a = com_nome("Ana");
    let id = ph2d_app_kanban::smoke::stage(&mut a.hero, 1).expect("cena 1");
    a.quadros(2);
    let mut b = com_nome("Bia");
    b.quadro();
    let vazio = b.docs().create();
    b.quadros(2);
    toca(&mut a, id, "ShareButton");
    toca(&mut a, id, "Share(Start)");
    ate(&mut a, &mut b, "o link da Ana", |a, _| {
        a.docs().probe_share(id.0).1.is_some()
    });
    let link = a.docs().probe_share(id.0).1.expect("link");
    toca(&mut b, vazio, "ShareButton");
    toca(&mut b, vazio, "Share(LinkBox)");
    b.escreve(&link);
    b.enter();
    let mut deles = None;
    ate(&mut a, &mut b, "o Kanban da Ana na Bia", |_, b| {
        deles = b.docs().open_kanbans().into_iter().find(|k| *k != vazio);
        deles.is_some_and(|k| {
            b.hero
                .documents
                .kanbans()
                .get(k)
                .is_some_and(|x| x.cards.len() > 3)
        })
    });
    let deles = deles.expect("aba nova");
    ate(&mut a, &mut b, "os avatares", |a, b| {
        a.docs().probe_share(id.0).2.contains(&"Bia".to_owned())
            && b.docs().probe_share(deles.0).2.contains(&"Ana".to_owned())
    });
    let esc = ph2d_editor_core::screens::hero::board_keys::BoardKey::Escape;
    a.tecla(esc, crate::verso::sem(), None);
    b.tecla(esc, crate::verso::sem(), None);
    a.quadro();
    b.quadro();
    (a, b, id, deles)
}

#[test]
fn j1_dois_herois_partilham_um_kanban_por_link_o_cartao_movido_chega_e_os_avatares_mostram_o_colega()
 {
    let mut a = com_nome("Ana");
    let id = ph2d_app_kanban::smoke::stage(&mut a.hero, 1).expect("cena 1");
    a.quadros(2);
    let mut b = com_nome("Bia");
    b.quadro();
    let vazio = b.docs().create();
    b.quadros(2);
    // A Ana: Share → «Share this Kanban»; o link chega (e vai para a área de transferência).
    toca(&mut a, id, "ShareButton");
    toca(&mut a, id, "Share(Start)");
    ate(&mut a, &mut b, "o link da Ana", |a, _| {
        a.docs().probe_share(id.0).1.is_some()
    });
    let link = a.docs().probe_share(id.0).1.expect("link");
    assert!(link.starts_with("ph2d-board://"), "{link}");
    // A Bia: Share → cola o link → Enter.
    toca(&mut b, vazio, "ShareButton");
    toca(&mut b, vazio, "Share(LinkBox)");
    b.escreve(&link);
    b.enter();
    let mut deles = None;
    ate(&mut a, &mut b, "o Kanban da Ana na Bia", |_, b| {
        deles = b.docs().open_kanbans().into_iter().find(|k| *k != vazio);
        deles.is_some_and(|k| {
            b.hero
                .documents
                .kanbans()
                .get(k)
                .is_some_and(|x| x.cards.len() > 3)
        })
    });
    let deles = deles.expect("aba nova");
    assert_eq!(b.kanban(deles).header.name, a.kanban(id).header.name);
    assert_eq!(
        b.activa(),
        Some(deles.0),
        "o Kanban recebido abre-se numa aba"
    );
    // Os avatares: cada um vê o outro.
    ate(&mut a, &mut b, "os avatares", |a, b| {
        a.docs().probe_share(id.0).2.contains(&"Bia".to_owned())
            && b.docs().probe_share(deles.0).2.contains(&"Ana".to_owned())
    });
    // A Ana arrasta o 1.º cartão do Backlog para a Doing.
    let esc = ph2d_editor_core::screens::hero::board_keys::BoardKey::Escape;
    a.tecla(esc, crate::verso::sem(), None);
    a.quadro();
    assert_eq!(a.docs().probe_w6(id.0).0, None, "o Esc fecha o popover");
    let backlog: ListId = a.lista(id, "Backlog");
    let doing = a.lista(id, "Doing");
    let c = a.cartoes(id, backlog)[0];
    let titulo = a.kanban(id).cards[&c].title.clone();
    let r = a.rect_cartao(id, c);
    let d = a.rect_lista(id, doing);
    let p = pega(r);
    let fim = (
        d.x + d.w / 2.0 + (p.0 - r.x) - r.w / 2.0,
        d.y + 42.0 + (p.1 - r.y) - r.h / 2.0,
    );
    a.arrasta(p, fim, 8);
    assert!(a.larga(fim.0, fim.1));
    a.quadro();
    assert_eq!(lista_de(&a, id, &titulo).as_deref(), Some("Doing"));
    ate(&mut a, &mut b, "o cartão movido na Bia", |_, b| {
        lista_de(b, deles, &titulo).as_deref() == Some("Doing")
    });
    // A Bia abre um cartão: o ecrã da Ana mostra-o aberto por ela.
    b.assenta(deles);
    let cb = b
        .kanban(deles)
        .cards
        .values()
        .find(|x| x.title == titulo)
        .expect("c")
        .id;
    let rb = b.rect_cartao(deles, cb);
    b.clica(pega(rb));
    assert_eq!(b.docs().probe_back(deles.0), Some(cb));
    ate(&mut a, &mut b, "o anel do cartão aberto", |a, _| {
        a.docs()
            .probe_seen(id.0)
            .0
            .iter()
            .any(|(k, w)| *k == cb && w.contains(&"Bia".to_owned()))
    });
    // A Ana pára: a Bia sai da sala e o Kanban fica com ela.
    toca(&mut a, id, "ShareButton");
    toca(&mut a, id, "Share(Stop)");
    ate(&mut a, &mut b, "a Bia vê o fim", |_, b| {
        b.docs().probe_share(deles.0).0 == "Off"
    });
    assert!(b.hero.documents.kanbans().get(deles).is_some());
}
