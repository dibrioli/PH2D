//! J6 — **o verso pela porta única do gesto**, pela costura do hero: marcar a entrega como
//! concluída no verso é um gesto da pessoa, e a regra «quando concluída → arquivar» reage a ele
//! num passo de desfazer próprio (o efeito da regra sai primeiro, depois o gesto).

use crate::Mesa;
use crate::verso::{abre, toca};
use ph2d_kanban_auto::{Action, By, Kind, Spec, Trigger};
use ph2d_kanban_model::{CardId, Due, KanbanId, KanbanOp, Member, Moment, Record};

/// A regra «quando a entrega de um cartão é marcada concluída, arquivá-lo».
fn regra_concluida_arquiva(m: &Mesa, id: KanbanId) {
    let spec = Spec {
        name: String::new(),
        enabled: true,
        owner: Member("Enio".into()),
        kind: Kind::Rule {
            trigger: Trigger::DueDone { done: true },
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

/// Dá ao cartão `c` uma entrega por concluir (o verso mostra então a caixa «DueDone»).
fn com_entrega(m: &Mesa, id: KanbanId, c: CardId) {
    let shared = m.hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let mut card = set.get(id).unwrap().cards[&c].clone();
    card.due = Some(Due {
        at: Moment(1_800_000_000_000),
        done: false,
        remind_before_min: None,
    });
    set.apply(id, vec![KanbanOp::put(Record::Card(card.into()))]);
}

#[test]
fn j6_a_regra_reage_a_caixa_da_entrega_no_verso_e_o_desfazer_vem_em_passos() {
    let (mut m, id, c) = abre("Write the logline");
    com_entrega(&m, id, c);
    regra_concluida_arquiva(&m, id);
    m.quadro();
    toca(&mut m, id, "DueDone");
    let k = m.kanban(id);
    assert!(k.cards[&c].due.expect("entrega").done, "o verso marcou-a");
    assert!(k.cards[&c].archived, "a regra reagiu ao gesto do verso");
    assert!(
        k.activities.values().any(|a| a.by == Member::butler()),
        "a atividade do arquivar diz Butler"
    );
    assert_eq!(m.desfaz(), Some(true));
    let k = m.kanban(id);
    assert!(!k.cards[&c].archived, "o 1.º Ctrl+Z tira o que a regra fez");
    assert!(k.cards[&c].due.expect("entrega").done, "e deixa o gesto");
    assert_eq!(m.desfaz(), Some(true));
    assert!(
        !m.kanban(id).cards[&c].due.expect("entrega").done,
        "o 2.º Ctrl+Z tira o gesto"
    );
}
