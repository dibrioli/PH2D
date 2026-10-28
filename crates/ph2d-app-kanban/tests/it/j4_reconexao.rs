//! J4 — a sala de um Kanban partilhado sobrevive a uma queda: o fio da Bia cai (como o Wi-Fi que
//! soluça), o ecrã dela diz «Reconnecting…», ela volta SOZINHA (retrocesso exponencial) e o que
//! cada um fez enquanto ela estava fora chega ao outro.

use crate::Mesa;
use crate::j1_partilha::{ate, ligados};
use ph2d_kanban_model::{KanbanId, KanbanOp, Record};

fn mover(m: &mut Mesa, k: KanbanId, titulo: &str, para: &str) {
    let shared = m.hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let kb = set.doc_mut(k).expect("o Kanban");
    let lista = kb
        .lists
        .values()
        .find(|l| l.title == para)
        .expect("lista")
        .id;
    let mut c = kb
        .cards
        .values()
        .find(|c| c.title == titulo)
        .expect("cartão")
        .clone();
    c.list = lista;
    let _ = KanbanOp::put(Record::card(c)).apply(kb);
}

fn lista_de(m: &Mesa, k: KanbanId, titulo: &str) -> Option<String> {
    let kb = m.kanban(k);
    let c = kb.cards.values().find(|c| c.title == titulo)?;
    kb.lists.get(&c.list).map(|l| l.title.clone())
}

#[test]
fn j4_o_convidado_cai_volta_sozinho_e_o_que_cada_um_fez_fora_funde() {
    let (mut a, mut b, id, deles) = ligados();
    let kb = a.kanban(id);
    let titulos: Vec<String> = kb
        .cards
        .values()
        .filter(|c| kb.lists.get(&c.list).is_some_and(|l| l.title != "Done"))
        .map(|c| c.title.clone())
        .collect();
    let (t1, t2) = (titulos[0].clone(), titulos[1].clone());
    b.docs().share_drop_for_tests();
    let mut viu_a_voltar = false;
    // Fora: a Bia move um cartão, a Ana outro.
    mover(&mut b, deles, &t1, "Done");
    mover(&mut a, id, &t2, "Done");
    ate(
        &mut a,
        &mut b,
        "a Bia de volta e as duas mudanças nos dois",
        |a, b| {
            viu_a_voltar |= b.docs().probe_reconnecting(deles.0);
            lista_de(a, id, &t1).as_deref() == Some("Done")
                && lista_de(b, deles, &t2).as_deref() == Some("Done")
                && !b.docs().probe_reconnecting(deles.0)
        },
    );
    assert!(
        viu_a_voltar,
        "o ecrã da Bia disse «Reconnecting…» enquanto estava fora"
    );
    assert_eq!(b.docs().probe_share(deles.0).0, "Guest", "continua na sala");
    ate(&mut a, &mut b, "os avatares de novo", |a, b| {
        a.docs().probe_share(id.0).2.contains(&"Bia".to_owned())
            && b.docs().probe_share(deles.0).2.contains(&"Ana".to_owned())
    });
}
