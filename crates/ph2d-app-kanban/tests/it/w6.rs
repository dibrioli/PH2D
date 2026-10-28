//! A W6 pela costura do hero (1/2): o FILTRO, a PESQUISA, a EDIÇÃO RÁPIDA e copiar/mover — cada
//! gesto entra pelo rato e pelo teclado do hero; o efeito lê-se no documento e nas sondas da vista.

use crate::verso::{cartao, ctrl, sem};
use crate::{Mesa, pega};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, KanbanId, KanbanOp, Record};

/// O rect pintado do alvo W6 cujo nome começa por `nome` (o de cima).
pub(crate) fn alvo(m: &mut Mesa, id: KanbanId, nome: &str) -> Rect {
    m.docs()
        .probe_w6_hits(id.0)
        .into_iter()
        .rev()
        .find(|(n, _)| n.starts_with(nome))
        .unwrap_or_else(|| panic!("o alvo {nome} não foi pintado"))
        .1
}

/// Um clique no meio do alvo W6 `nome`.
pub(crate) fn toca(m: &mut Mesa, id: KanbanId, nome: &str) {
    let r = alvo(m, id, nome);
    m.clica((r.x + r.w / 2.0, r.y + r.h / 2.0));
}

/// Uma tecla com texto (`f`, `/`, `?`…) pelo hero, e um quadro.
pub(crate) fn tecla(m: &mut Mesa, c: char) -> bool {
    let k = if c.is_ascii_alphanumeric() {
        BoardKey::Char(c.to_ascii_lowercase())
    } else {
        BoardKey::Other
    };
    let t = c.to_string();
    let out = m.tecla(k, sem(), Some(&t));
    m.quadro();
    out
}

/// O foco do teclado no cartão `c` (um clique abre o verso; o `Esc` fecha-o e o foco fica).
pub(crate) fn foca(m: &mut Mesa, id: KanbanId, c: CardId) {
    let r = m.rect_cartao(id, c);
    m.clica(pega(r));
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    assert_eq!(m.docs().probe_focus(id.0), Some(c));
}

/// Rola o painel aberto (a roda sobre ele) até o alvo `nome` se pintar.
pub(crate) fn rola_ate(m: &mut Mesa, id: KanbanId, nome: &str) {
    for _ in 0..40 {
        if m.docs()
            .probe_w6_hits(id.0)
            .iter()
            .any(|(n, _)| n.starts_with(nome))
        {
            return;
        }
        let r = alvo(m, id, "Inside");
        m.roda(r.x + r.w / 2.0, r.y + r.h / 2.0, -48.0);
    }
    panic!("o alvo {nome} não apareceu a rolar");
}

/// Todas as linhas mostradas, por todas as listas.
pub(crate) fn mostradas(m: &mut Mesa, id: KanbanId) -> Vec<CardId> {
    let lists: Vec<_> = m.kanban(id).lists.keys().copied().collect();
    lists
        .into_iter()
        .flat_map(|l| m.docs().probe_rows(id.0, l))
        .collect()
}

/// Um segundo Kanban (as listas do `id`, sem cartões), com um cartão `titulo` na 1.ª lista.
pub(crate) fn segundo(m: &mut Mesa, id: KanbanId, titulo: Option<&str>) -> KanbanId {
    let shared = m.hero.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let k = set.create_from(id, "Second", false).expect("kanban");
    if let Some(t) = titulo {
        let mut c = set
            .get(id)
            .expect("k")
            .cards
            .values()
            .next()
            .expect("c")
            .clone();
        let doc = set.doc_mut(k).expect("doc");
        let list = doc.lists_in_order()[0].id;
        c.id = CardId(doc.mint());
        c.list = list;
        c.pos = doc.pos_after_cards(list);
        c.title = t.into();
        c.labels.clear();
        c.members.clear();
        ph2d_kanban_model::apply_batch(doc, vec![KanbanOp::put(Record::Card(c.into()))]);
    }
    drop(set);
    m.quadro();
    k
}

#[test]
fn w6_filtro_f_abre_o_painel_a_palavra_e_a_etiqueta_tiram_cartoes_e_o_clear_all_devolve() {
    let (mut m, id) = Mesa::cena1();
    let antes = m.bytes();
    let todos = mostradas(&mut m, id).len();
    assert!(tecla(&mut m, 'f'), "F abre o filtro");
    assert_eq!(m.docs().probe_w6(id.0).0.as_deref(), Some("Filter"));
    // A palavra escreve-se logo (o teclado está na caixa): só os cartões com «fox» ficam.
    m.escreve("fox");
    m.quadro();
    let fox = mostradas(&mut m, id);
    let titulos: Vec<String> = fox
        .iter()
        .map(|c| m.kanban(id).cards[c].title.clone())
        .collect();
    assert!(
        !fox.is_empty() && fox.len() < todos,
        "o filtro tira cartões: {titulos:?}"
    );
    assert!(
        titulos.iter().all(|t| t.to_lowercase().contains("fox")),
        "{titulos:?}"
    );
    assert_eq!(m.docs().probe_w6(id.0).3, 1, "o botão conta uma escolha");
    // Uma etiqueta: «Character» (os dois cartões da raposa têm-na) — ainda casam.
    let character = m
        .kanban(id)
        .labels
        .values()
        .find(|l| l.name == "Character")
        .expect("etiqueta")
        .id;
    rola_ate(&mut m, id, &format!("Label({character:?})"));
    toca(&mut m, id, &format!("Label({character:?})"));
    assert_eq!(m.docs().probe_w6(id.0).3, 2);
    let fox = mostradas(&mut m, id);
    assert!(fox.len() == 2, "fox E Character: os dois cartões da raposa");
    // O botão «Filters» e o «Clear all» estão no cabeçalho; o filtro NÃO entra no documento.
    alvo(&mut m, id, "FilterButton");
    assert_eq!(m.bytes(), antes, "o filtro é da vista, nunca do documento");
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    assert_eq!(m.docs().probe_w6(id.0).0, None, "Esc fecha o painel");
    assert_eq!(
        mostradas(&mut m, id),
        fox,
        "o filtro fica depois de fechar o painel"
    );
    toca(&mut m, id, "ClearAll");
    assert_eq!(mostradas(&mut m, id).len(), todos, "Clear all devolve tudo");
    // `Q`: os meus cartões; `X` limpa.
    assert!(tecla(&mut m, 'q'));
    assert_eq!(m.docs().probe_w6(id.0).3, 1);
    assert!(tecla(&mut m, 'x'));
    assert_eq!(m.docs().probe_w6(id.0).3, 0);
}

#[test]
fn w6_pesquisa_barra_procura_em_todos_os_kanbans_e_o_enter_abre_o_cartao_na_aba_dele() {
    let (mut m, id) = Mesa::cena1();
    // Um segundo Kanban com um cartão que só ele tem.
    let outro = segundo(&mut m, id, Some("Zebra crossing"));
    assert!(tecla(&mut m, '/'), "/ abre a pesquisa");
    assert_eq!(m.docs().probe_w6(id.0).0.as_deref(), Some("Search"));
    m.escreve("zebra");
    m.quadro();
    let found = m.docs().probe_found(id.0);
    assert_eq!(found.len(), 1, "encontra no OUTRO Kanban: {found:?}");
    assert_eq!(found[0].kanban, outro);
    alvo(&mut m, id, "Result(");
    m.enter();
    m.quadro();
    assert_eq!(m.activa(), Some(outro.0), "abriu a aba do Kanban do cartão");
    assert_eq!(
        m.docs().probe_back(outro.0),
        Some(found[0].card),
        "com o verso aberto"
    );
}

#[test]
fn w6_edicao_rapida_e_abre_no_lugar_enter_grava_o_titulo_e_archive_arquiva() {
    let (mut m, id) = Mesa::cena1();
    let c = cartao(&m, id, "Design the fox");
    foca(&mut m, id, c);
    assert!(tecla(&mut m, 'e'), "E abre a edição rápida");
    assert_eq!(m.docs().probe_w6(id.0).2, Some(c));
    for q in [
        "Open", "Labels", "Members", "Cover", "Dates", "Move", "Copy", "Mirror", "Archive",
    ] {
        alvo(&mut m, id, &format!("Quick({q})"));
    }
    alvo(&mut m, id, "QuickSave");
    m.escreve("Design the wolf");
    m.enter();
    assert_eq!(m.kanban(id).cards[&c].title, "Design the wolf");
    assert_eq!(m.docs().probe_w6(id.0).2, None, "Enter grava e fecha");
    // O lápis (e o botão direito) abrem-na também; «Archive» arquiva.
    assert!(tecla(&mut m, 'e'));
    toca(&mut m, id, "Quick(Archive)");
    assert!(m.kanban(id).cards[&c].archived, "arquivado");
    assert_eq!(m.docs().probe_w6(id.0).2, None);
}

#[test]
fn w6_copiar_com_titulo_e_mover_para_outro_kanban_que_o_ctrl_z_desfaz_inteiro() {
    let (mut m, id) = Mesa::cena1();
    let c = cartao(&m, id, "Storyboard act 1");
    let todo = m.lista(id, "To Do");
    foca(&mut m, id, c);
    assert!(tecla(&mut m, 'e'));
    toca(&mut m, id, "Quick(Copy)");
    assert!(
        m.docs()
            .probe_w6(id.0)
            .1
            .is_some_and(|d| d.starts_with("CopyCard"))
    );
    for k in [
        "Dlg(Keep(0))",
        "Dlg(Open(Kanban))",
        "Dlg(Open(List))",
        "Dlg(Open(Position))",
    ] {
        alvo(&mut m, id, k);
    }
    // O título abre selecionado? Não: o cursor no fim — escreve-se a seguir.
    m.escreve(" v2");
    toca(&mut m, id, "Dlg(Confirm)");
    let copia = m
        .kanban(id)
        .cards
        .values()
        .find(|x| x.title == "Storyboard act 1 v2")
        .map(|x| (x.id, x.list))
        .expect("a cópia com o título novo");
    assert_eq!(copia.1, todo, "na mesma lista");
    let ordem = m.cartoes(id, todo);
    let i = ordem.iter().position(|x| *x == c).expect("o original");
    assert_eq!(
        ordem[i + 1],
        copia.0,
        "logo abaixo do original (a posição do Trello)"
    );
    // Mover para outro Kanban.
    let outro = segundo(&mut m, id, None);
    foca(&mut m, id, c);
    assert!(tecla(&mut m, 'e'));
    toca(&mut m, id, "Quick(Move)");
    toca(&mut m, id, "Dlg(Open(Kanban))");
    toca(&mut m, id, &format!("Dlg(Kanban({outro:?}))"));
    toca(&mut m, id, "Dlg(Confirm)");
    assert!(
        !m.kanban(id).cards.get(&c).is_some_and(|x| !x.meta.deleted),
        "saiu deste Kanban"
    );
    let la = m.kanban(outro);
    assert!(
        la.cards
            .values()
            .any(|x| x.title == "Storyboard act 1" && !x.meta.deleted)
    );
    // Um Ctrl+Z desfaz o gesto nos DOIS Kanbans.
    m.desfaz();
    assert!(
        m.kanban(id).cards.get(&c).is_some_and(|x| !x.meta.deleted),
        "voltou"
    );
    let la = m.kanban(outro);
    assert!(
        !la.cards
            .values()
            .any(|x| x.title == "Storyboard act 1" && !x.meta.deleted)
    );
    let _ = ctrl();
}
