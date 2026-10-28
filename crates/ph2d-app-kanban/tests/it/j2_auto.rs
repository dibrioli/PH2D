//! J2 — **o Butler no ecrã** pela costura do hero: o painel de automação abre pelo verso («Add
//! button»), o construtor EM FRASE monta uma regra chip a chip («when a card is added to Done, mark
//! the due date complete and archive the card»), grava-a (um passo de desfazer), liga/desliga, edita;
//! os botões de cartão e de quadro correm em nome do Butler; e a sugestão do que se repete cria a
//! regra (ou cala-se). O DISPARO da regra num gesto é do lote J3.

use crate::Mesa;
use crate::verso::{abre, eu, sem};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_auto::{Action, Added, Kind, Spec, Trigger, Word, specs};
use ph2d_kanban_model::{
    Actor, AutomationId, Due, KanbanId, KanbanOp, Member, Moment, Record, apply_logged,
};

/// Um clique no alvo do verso que começa por `nome`, rolando o verso até ele estar à vista.
pub(crate) fn toca_v(m: &mut Mesa, id: KanbanId, nome: &str) {
    for _ in 0..60 {
        let r = m
            .docs()
            .probe_back_hits(id.0)
            .into_iter()
            .rev()
            .find(|(n, _)| n.starts_with(nome))
            .unwrap_or_else(|| panic!("o alvo {nome} não foi pintado no verso"))
            .1;
        if r.y >= crate::AREA.y && r.y + r.h <= crate::AREA.y + crate::AREA.h {
            m.clica((r.x + r.w / 2.0, r.y + r.h / 2.0));
            return;
        }
        let dy = if r.y < crate::AREA.y { 16.0 } else { -16.0 };
        m.roda(crate::AREA.w / 2.0, crate::AREA.h / 2.0, dy);
    }
    panic!("o alvo {nome} nunca ficou à vista");
}

pub(crate) fn alvo_auto(m: &mut Mesa, id: KanbanId, nome: &str) -> Rect {
    m.docs()
        .probe_auto_hits(id.0)
        .into_iter()
        .rev()
        .find(|(n, _)| n.starts_with(nome))
        .unwrap_or_else(|| panic!("o alvo {nome} não foi pintado no painel"))
        .1
}

pub(crate) fn toca_auto(m: &mut Mesa, id: KanbanId, nome: &str) {
    let r = alvo_auto(m, id, nome);
    m.clica((r.x + r.w / 2.0, r.y + r.h / 2.0));
}

pub(crate) fn linhas(m: &mut Mesa, id: KanbanId) -> Vec<String> {
    m.docs().probe_auto(id.0).map(|p| p.1).unwrap_or_default()
}

/// Escolhe a linha `texto` do menu aberto de um chip.
pub(crate) fn escolhe(m: &mut Mesa, id: KanbanId, texto: &str) {
    let menu: Vec<String> = linhas(m, id)
        .into_iter()
        .filter_map(|l| l.strip_prefix("choice:").map(str::to_owned))
        .collect();
    let i = menu
        .iter()
        .position(|l| l == texto)
        .unwrap_or_else(|| panic!("«{texto}» não está no menu: {menu:?}"));
    toca_auto(m, id, &format!("Auto(Choice({i}))"));
}

fn w(word: Word) -> &'static str {
    tr(word.key())
}

/// O painel aberto pelo «Add button» do verso, no separador `tab`, sem construtor.
pub(crate) fn painel(titulo: &str, tab: &str) -> (Mesa, KanbanId) {
    let (mut m, id, _) = abre(titulo);
    toca_v(&mut m, id, "Side(AddButton)");
    assert_eq!(
        m.docs().probe_auto(id.0).map(|p| p.0).as_deref(),
        Some("CardButton"),
        "o «Add button» abre o painel nos botões de cartão"
    );
    toca_auto(&mut m, id, &format!("Auto(Tab({tab}))"));
    (m, id)
}

fn regras(m: &Mesa, id: KanbanId) -> Vec<(AutomationId, Spec)> {
    specs(&m.kanban(id))
}

#[test]
fn j2_o_construtor_em_frase_cria_edita_e_liga_uma_regra() {
    let (mut m, id) = painel("Write the logline", "Rule");
    let done = m.lista(id, "Done");
    toca_auto(&mut m, id, "Auto(Create)");
    let draft = linhas(&mut m, id);
    assert!(
        draft.iter().any(|l| l.starts_with(&format!(
            "draft:{} {}",
            w(Word::When),
            w(Word::CardAdded)
        ))),
        "a frase começa no gatilho: {draft:?}"
    );
    // O valor do gatilho: a lista.
    toca_auto(&mut m, id, "Auto(Chip(Arg(Trigger, 1)))");
    escolhe(&mut m, id, "Done");
    // Duas acções, cada uma pela cabeça do chip.
    for (i, word) in [(0, Word::MarkDone), (1, Word::Archive)] {
        toca_auto(&mut m, id, "Auto(AddAction)");
        toca_auto(&mut m, id, &format!("Auto(Chip(Head(Action({i}))))"));
        escolhe(&mut m, id, w(word));
    }
    toca_auto(&mut m, id, "Auto(Save)");
    let rs = regras(&m, id);
    assert_eq!(rs.len(), 1, "uma regra gravada");
    let (aid, s) = rs[0].clone();
    assert_eq!(
        s.kind,
        Kind::Rule {
            trigger: Trigger::CardAdded {
                list: Some(done),
                how: Added::Any
            },
            by: ph2d_kanban_auto::By::Anyone,
            conditions: Vec::new(),
        }
    );
    assert_eq!(s.actions, [Action::MarkDue { done: true }, Action::Archive]);
    assert!(s.enabled);
    let lista = linhas(&mut m, id);
    let frase = format!(
        "rules:on:{} {} Done {} {} {}",
        w(Word::When),
        w(Word::CardAdded),
        w(Word::MarkDone),
        w(Word::And),
        w(Word::Archive)
    );
    assert!(lista.contains(&frase), "a lista mostra a frase: {lista:?}");
    // A gravação: o ficheiro leva a regra.
    let set = ph2d_kanban_model::KanbanSet::from_bytes(&m.bytes()).expect("abre");
    assert_eq!(specs(set.get(id).unwrap()), rs, "a regra viaja no ficheiro");
    // Desligar; o desfazer volta a ligar.
    toca_auto(&mut m, id, "Auto(Toggle(");
    assert!(!regras(&m, id)[0].1.enabled, "desligada");
    assert!(m.desfaz().is_some());
    assert!(regras(&m, id)[0].1.enabled, "o desfazer liga-a");
    // Editar: a 2.ª acção passa a «remove all labels»; continua UMA regra (a mesma id).
    toca_auto(&mut m, id, "Auto(Edit(");
    toca_auto(&mut m, id, "Auto(Chip(Head(Action(1))))");
    escolhe(&mut m, id, w(Word::ClearLabels));
    toca_auto(&mut m, id, "Auto(Save)");
    let rs = regras(&m, id);
    assert_eq!(rs.len(), 1);
    assert_eq!(rs[0].0, aid, "editar troca o corpo, não cria outra");
    assert_eq!(rs[0].1.actions[1], Action::ClearLabels);
    // Apagar, e o desfazer devolve-a; três desfazer voltam ao documento sem regra.
    toca_auto(&mut m, id, "Auto(Delete(");
    assert!(regras(&m, id).is_empty());
    assert!(m.desfaz().is_some());
    assert_eq!(
        regras(&m, id)[0].1.actions[1],
        Action::ClearLabels,
        "o apagar desfeito"
    );
    assert!(m.desfaz().is_some());
    assert_eq!(
        regras(&m, id)[0].1.actions[1],
        Action::Archive,
        "a edição desfeita"
    );
    assert!(m.desfaz().is_some());
    assert!(regras(&m, id).is_empty(), "a criação desfeita");
}

#[test]
fn j2_o_esc_desce_um_degrau_e_a_regra_sem_accao_nao_grava() {
    let (mut m, id) = painel("Write the logline", "Rule");
    toca_auto(&mut m, id, "Auto(Create)");
    toca_auto(&mut m, id, "Auto(Save)");
    assert!(regras(&m, id).is_empty(), "sem acção não grava");
    toca_auto(&mut m, id, "Auto(Chip(Head(Trigger)))");
    assert!(linhas(&mut m, id).iter().any(|l| l.starts_with("choice:")));
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    assert!(
        !linhas(&mut m, id).iter().any(|l| l.starts_with("choice:")),
        "o Esc fecha o menu"
    );
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    assert!(
        !linhas(&mut m, id).iter().any(|l| l.starts_with("draft:")),
        "depois o construtor"
    );
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    assert_eq!(m.docs().probe_auto(id.0), None, "depois o painel");
    assert!(
        m.docs().probe_back(id.0).is_some(),
        "o verso continua por baixo"
    );
}

/// Cria pelo construtor um botão com o nome `nome` e a acção `accao` (o painel no separador certo).
fn botao(m: &mut Mesa, id: KanbanId, nome: &str, accao: Word) {
    toca_auto(m, id, "Auto(Chip(Arg(Name, 0)))");
    for c in nome.chars() {
        let t = c.to_string();
        m.tecla(BoardKey::Other, sem(), Some(&t));
    }
    m.enter();
    toca_auto(m, id, "Auto(AddAction)");
    toca_auto(m, id, "Auto(Chip(Head(Action(0))))");
    escolhe(m, id, w(accao));
    toca_auto(m, id, "Auto(Save)");
}

#[test]
fn j2_os_botoes_de_cartao_e_de_quadro_correm_em_nome_do_butler() {
    let (mut m, id, c) = abre("Write the logline");
    toca_v(&mut m, id, "Side(AddButton)");
    botao(&mut m, id, "Ship it", Word::Archive);
    let rs = regras(&m, id);
    assert_eq!(rs.len(), 1);
    assert_eq!(rs[0].1.name, "Ship it");
    assert_eq!(rs[0].1.kind, Kind::CardButton);
    toca_auto(&mut m, id, "Auto(Close)");
    assert_eq!(m.docs().probe_auto(id.0), None);
    // O verso mostra o botão na secção «Buttons»; premi-lo arquiva o cartão em nome do Butler.
    toca_v(&mut m, id, "CardButton(");
    let k = m.kanban(id);
    assert!(k.cards[&c].archived, "o botão de cartão arquivou-o");
    assert!(
        k.activities.values().any(|a| a.by == Member::butler()),
        "a atividade diz Butler"
    );
    assert!(m.desfaz().is_some());
    assert!(!m.kanban(id).cards[&c].archived, "um passo de desfazer");
    // O botão de quadro: «Sweep» arquiva cada cartão da Done; «Run» no painel prime-o.
    toca_v(&mut m, id, "Side(AddButton)");
    toca_auto(&mut m, id, "Auto(Tab(BoardButton))");
    toca_auto(&mut m, id, "Auto(Create)");
    toca_auto(&mut m, id, "Auto(Chip(Arg(Scope, 1)))");
    escolhe(&mut m, id, "Done");
    botao(&mut m, id, "Sweep", Word::Archive);
    let done = m.lista(id, "Done");
    assert!(!m.cartoes(id, done).is_empty(), "a Done tem cartões");
    toca_auto(&mut m, id, "Auto(Run(");
    assert!(
        m.cartoes(id, done).is_empty(),
        "o botão de quadro arquivou a Done"
    );
}

/// Três cartões movidos para a Done por «eu», com a entrega marcada logo a seguir.
fn tres_vezes(m: &mut Mesa, id: KanbanId) {
    let done = m.lista(id, "Done");
    let shared = m.hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let k = set.doc_mut(id).unwrap();
    let cards: Vec<_> = k
        .cards
        .values()
        .filter(|c| c.list != done && !c.archived)
        .take(3)
        .cloned()
        .collect();
    for (i, c) in cards.into_iter().enumerate() {
        let at = Moment(1_000_000 + i64::try_from(i).unwrap() * 600_000);
        let actor = Actor { by: eu(), at };
        let mut moved = c.clone();
        moved.list = done;
        moved.due = Some(Due {
            at,
            done: false,
            remind_before_min: None,
        });
        apply_logged(
            k,
            vec![KanbanOp::put(Record::Card(moved.clone().into()))],
            &actor,
        );
        let mut marked = moved;
        marked.due = marked.due.map(|d| Due { done: true, ..d });
        let actor = Actor {
            by: eu(),
            at: Moment(at.0 + 1000),
        };
        apply_logged(k, vec![KanbanOp::put(Record::Card(marked.into()))], &actor);
    }
}

#[test]
fn j2_a_sugestao_cria_a_regra_e_dispensar_cala_a() {
    let (mut m, id, _) = abre("Write the logline");
    tres_vezes(&mut m, id);
    toca_v(&mut m, id, "Side(AddButton)");
    toca_auto(&mut m, id, "Auto(Tab(Rule))");
    let sug = |m: &mut Mesa| -> Vec<String> {
        linhas(m, id)
            .into_iter()
            .filter(|l| l.starts_with("suggest:"))
            .collect()
    };
    assert_eq!(
        sug(&mut m).len(),
        1,
        "uma sugestão: {:?}",
        linhas(&mut m, id)
    );
    toca_auto(&mut m, id, "Auto(Suggest(0, false))");
    assert!(sug(&mut m).is_empty(), "dispensada");
    assert!(regras(&m, id).is_empty());
    toca_auto(&mut m, id, "Auto(Close)");
    toca_v(&mut m, id, "Side(AddButton)");
    toca_auto(&mut m, id, "Auto(Tab(Rule))");
    assert_eq!(sug(&mut m).len(), 1, "outra vista, a sugestão volta");
    toca_auto(&mut m, id, "Auto(Suggest(0, true))");
    let rs = regras(&m, id);
    assert_eq!(rs.len(), 1, "a sugestão criou a regra");
    assert_eq!(rs[0].1.actions, [Action::MarkDue { done: true }]);
    assert!(
        sug(&mut m).is_empty(),
        "a regra que já faz isto cala a sugestão"
    );
}

#[test]
fn j2_cada_palavra_do_butler_e_cada_separador_tem_traducao() {
    for word in Word::ALL {
        assert_ne!(tr(word.key()), word.key(), "sem tradução: {}", word.key());
    }
    for k in ["rules", "card_buttons", "board_buttons", "calendar", "due"] {
        for p in ["tab", "create"] {
            let key = format!("kanban.auto.ui.{p}.{k}");
            assert_ne!(tr(&key), key, "sem tradução: {key}");
        }
    }
}
