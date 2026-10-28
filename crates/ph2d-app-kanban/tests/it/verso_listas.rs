//! O VERSO (W3): a checklist (adicionar, marcar, item → cartão), a etiqueta nova, o comentário com
//! menção na atividade e o histórico que o gesto regista.

use crate::verso::{abre, alvo, cartao, eu, sem, toca};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_kanban_model::{LabelColor, Subject};

#[test]
fn a_checklist_adiciona_marca_e_converte_um_item_em_cartao() {
    let (mut m, id, c) = abre("Write the logline");
    toca(&mut m, id, "Side(Checklist)");
    toca(&mut m, id, "Pop(Save)");
    let k = m.kanban(id);
    let cl = k
        .checklists
        .values()
        .find(|x| x.card == c)
        .expect("a checklist nasceu")
        .id;
    toca(&mut m, id, &format!("AddItem({cl:?})"));
    m.escreve("Rig the fox");
    m.enter();
    m.escreve("Paint it");
    m.enter();
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    let k = m.kanban(id);
    let mut itens: Vec<_> = k
        .items
        .values()
        .filter(|i| i.checklist == cl && !i.meta.deleted)
        .collect();
    itens.sort_by(|a, b| a.pos.cmp(&b.pos));
    let textos: Vec<_> = itens.iter().map(|i| i.text.as_str()).collect();
    assert_eq!(
        textos,
        ["Rig the fox", "Paint it"],
        "o compositor fica aberto para o seguinte"
    );
    let (rig, paint) = (itens[0].id, itens[1].id);
    toca(&mut m, id, &format!("ItemBox({paint:?})"));
    assert!(m.kanban(id).items[&paint].done, "marcado");
    let feed = m.docs().probe_back_feed(id.0);
    assert!(
        feed.is_empty() || !feed.iter().any(|l| l.contains("completed")),
        "detalhe escondido"
    );

    // Pairar o item mostra as ações; «Convert to card».
    let r = alvo(&mut m, id, &format!("ItemText({rig:?})"));
    m.move_(r.x + 4.0, r.y + r.h / 2.0);
    m.quadro();
    toca(&mut m, id, &format!("ItemConvert({rig:?})"));
    let k = m.kanban(id);
    assert!(k.items[&rig].meta.deleted, "o item saiu");
    let novo = cartao(&m, id, "Rig the fox");
    let backlog = m.lista(id, "Backlog");
    let ordem = m.cartoes(id, backlog);
    let i = ordem.iter().position(|x| *x == c).expect("origem");
    assert_eq!(ordem[i + 1], novo, "o cartão novo nasce logo abaixo");
    assert_eq!(m.kanban(id).cards[&novo].origin, Some(c));
    // O histórico do cartão novo diz de onde veio, em nome de «eu».
    let k = m.kanban(id);
    let h = k.activity_of(Subject::Card(novo));
    assert_eq!(h.first().map(|a| a.by.clone()), Some(eu()));
}

#[test]
fn uma_etiqueta_nova_com_nome_e_cor_entra_no_cartao() {
    let (mut m, id, c) = abre("Write the logline");
    assert!(
        m.tecla(BoardKey::Char('l'), sem(), Some("l")),
        "L abre as etiquetas"
    );
    m.quadro();
    toca(&mut m, id, "Pop(LabelCreate)");
    m.escreve("Rigging");
    toca(&mut m, id, "Pop(Color(3))");
    toca(&mut m, id, "Pop(Save)");
    let k = m.kanban(id);
    let l = k
        .labels
        .values()
        .find(|l| l.name == "Rigging")
        .expect("a etiqueta nasceu");
    assert_eq!(l.color, Some(LabelColor::all()[3]));
    assert!(k.cards[&c].labels.contains(&l.id), "e entrou no cartão");
    assert_eq!(
        m.docs().probe_back_state(id.0).0.as_deref(),
        Some("Labels"),
        "volta à lista das etiquetas"
    );
    // Clicar na etiqueta da lista tira-a do cartão.
    let lid = l.id;
    toca(&mut m, id, &format!("Pop(Label({lid:?}))"));
    assert!(!m.kanban(id).cards[&c].labels.contains(&lid));
}

#[test]
fn um_comentario_com_mencao_aparece_na_atividade() {
    let (mut m, id, c) = abre("Write the logline");
    toca(&mut m, id, "CommentArea");
    assert!(m.docs().probe_back_state(id.0).3, "a caixa em edição");
    m.escreve("@an");
    m.quadro();
    let lista: Vec<_> = m
        .docs()
        .probe_back_hits(id.0)
        .into_iter()
        .filter(|(n, _)| n.starts_with("Mention("))
        .collect();
    assert_eq!(lista.len(), 1, "o @ completa: só a Ana casa com «an»");
    m.enter();
    m.escreve("check the timing");
    toca(&mut m, id, "CommentSave");
    let k = m.kanban(id);
    let cm = k
        .comments
        .values()
        .find(|x| x.card == c && !x.meta.deleted && x.author == eu())
        .expect("o comentário gravou-se");
    assert_eq!(cm.text.trim_end(), "@Ana check the timing");
    m.quadro();
    let feed = m.docs().probe_back_feed(id.0);
    assert_eq!(
        feed.first().map(String::as_str),
        Some(format!("comment:{}:{}", eu().0, cm.text).as_str()),
        "o mais novo primeiro: {feed:?}"
    );
    // Mover pelo verso regista no histórico, intercalado.
    toca(&mut m, id, "ListLink");
    let todo = m.lista(id, "To Do");
    toca(&mut m, id, &format!("Pop(List({todo:?}))"));
    assert_eq!(m.kanban(id).cards[&c].list, todo);
    m.quadro();
    let feed = m.docs().probe_back_feed(id.0);
    assert!(
        feed.iter()
            .any(|l| l.starts_with("activity:") && l.contains("Backlog") && l.contains("To Do")),
        "o movimento aparece: {feed:?}"
    );
}
