//! A W6 pela costura do hero (2/2): os MODELOS, os ESPELHOS, o ENVELHECIMENTO e os ATALHOS.

use crate::Mesa;
use crate::verso::{cartao, ctrl, eu, sem};
use crate::w6::{alvo, foca, tecla, toca};
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_kanban_model::Aging;

#[test]
fn w6_modelos_make_template_pinta_o_selo_o_rodape_cria_do_modelo_e_new_kanban_from() {
    let (mut m, id, c) = crate::verso::abre("Fox concept sheet");
    crate::verso::toca(&mut m, id, "Side(Template)");
    assert!(m.kanban(id).cards[&c].template, "Make template liga o modelo");
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    // O modelo pinta o selo e perde o distintivo da entrega (estava atrasado).
    let selos = m.docs().probe_badges(id.0, c);
    assert!(selos.iter().any(|s| s == "This card is a template"), "{selos:?}");
    assert!(!selos.iter().any(|s| s.contains("Oct") || s.contains("Yesterday")), "{selos:?}");
    // O ícone do rodapé da «Backlog»: «Create card from template».
    let backlog = m.lista(id, "Backlog");
    let r = m.docs().probe_template_button(id.0, backlog).expect("o ícone do modelo");
    m.clica((r.x + r.w / 2.0, r.y + r.h / 2.0));
    assert!(m.docs().probe_w6(id.0).1.is_some_and(|d| d.starts_with("FromTemplate")));
    toca(&mut m, id, "Dlg(Template(");
    toca(&mut m, id, "Dlg(Confirm)");
    let novo = m
        .kanban(id)
        .cards
        .values()
        .filter(|x| x.title == "Fox concept sheet" && x.list == backlog)
        .map(|x| x.template)
        .collect::<Vec<_>>();
    assert_eq!(novo, vec![false], "o cartão novo nasce do modelo e já não é modelo");
    // «New Kanban from…» no menu File.
    let antes = m.hero.documents.kanbans().iter().count();
    assert!(m.docs().family_command(ph2d_app_kanban::CMD_FROM));
    m.quadro();
    assert!(m.docs().probe_w6(id.0).1.is_some_and(|d| d == "NewKanbanFrom"));
    toca(&mut m, id, "Dlg(Confirm)");
    assert_eq!(m.hero.documents.kanbans().iter().count(), antes + 1, "um Kanban novo");
}

#[test]
fn w6_espelho_mirror_mostra_o_mesmo_cartao_noutra_lista_abre_o_original_e_arquivar_o_tira() {
    let (mut m, id) = Mesa::cena1();
    let c = cartao(&m, id, "Animate shot 12");
    let done = m.lista(id, "Done");
    foca(&mut m, id, c);
    assert!(tecla(&mut m, 'e'));
    toca(&mut m, id, "Quick(Mirror)");
    toca(&mut m, id, "Dlg(Open(List))");
    toca(&mut m, id, &format!("Dlg(List({done:?}))"));
    toca(&mut m, id, "Dlg(Confirm)");
    let espelhos = m.docs().probe_mirrors(id.0);
    assert_eq!(espelhos.len(), 1, "um espelho");
    let (row, of) = espelhos[0];
    assert_eq!(of.card, c);
    assert_eq!(m.docs().probe_rows(id.0, done)[0], row, "no topo da «Done»");
    assert!(m.docs().probe_badges(id.0, row).iter().any(|s| s == "Mirror"), "o selo do espelho");
    // Clicar no espelho abre o verso do ORIGINAL.
    let r = m.rect_cartao(id, row);
    m.clica(crate::pega(r));
    assert_eq!(m.docs().probe_back(id.0), Some(c));
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    // Arquivar o original (C) tira o espelho; o desfazer devolve os dois.
    foca(&mut m, id, c);
    assert!(tecla(&mut m, 'c'));
    assert!(m.docs().probe_mirrors(id.0).is_empty(), "sem original não há espelho");
    m.desfaz();
    assert_eq!(m.docs().probe_mirrors(id.0).len(), 1, "o desfazer devolve o espelho");
}

#[test]
fn w6_envelhecimento_card_aging_liga_regular_e_pirata_pelo_menu_do_kanban() {
    let (mut m, id) = Mesa::cena1();
    assert!(m.docs().probe_ages(id.0).is_empty(), "desligado: nada envelhece");
    let menu = m.docs().probe_rect(id.0, "menu").expect("menu");
    m.clica((menu.x + menu.w / 2.0, menu.y + menu.h / 2.0));
    let linha = m
        .docs()
        .probe_menu_rows(id.0)
        .into_iter()
        .find(|(n, _)| n == "Aging")
        .expect("Card aging… no menu")
        .1;
    m.clica((linha.x + linha.w / 2.0, linha.y + linha.h / 2.0));
    toca(&mut m, id, "Dlg(Aging(1))");
    assert_eq!(m.kanban(id).header.aging, Some(Aging::Regular));
    m.quadro();
    // A cena nasce em 1970: todos os cartões estão no nível 3.
    let ages = m.docs().probe_ages(id.0);
    assert!(!ages.is_empty() && ages.iter().all(|(_, n)| *n == 3), "{ages:?}");
    toca(&mut m, id, "Dlg(Aging(2))");
    assert_eq!(m.kanban(id).header.aging, Some(Aging::Pirate));
    m.quadros(2);
    toca(&mut m, id, "Dlg(Aging(0))");
    assert_eq!(m.kanban(id).header.aging, None);
}

#[test]
fn w6_atalhos_ajuda_menu_espaco_seguir_votar_etiqueta_copiar_colar_e_z() {
    let (mut m, id) = Mesa::cena1();
    assert!(tecla(&mut m, '?'));
    assert_eq!(m.docs().probe_w6(id.0).0.as_deref(), Some("Help"));
    assert!(m.tecla(BoardKey::Escape, sem(), None));
    m.quadro();
    assert!(tecla(&mut m, 'w'));
    assert!(m.docs().probe_overlays(id.0).0, "W abre o menu do Kanban");
    assert!(tecla(&mut m, 'w'));
    let c = cartao(&m, id, "Write the logline");
    let backlog = m.lista(id, "Backlog");
    foca(&mut m, id, c);
    let me = eu();
    assert!(tecla(&mut m, ' '), "Space atribui-me");
    assert!(m.kanban(id).cards[&c].members.contains(&me));
    assert!(tecla(&mut m, 's'));
    assert!(m.kanban(id).cards[&c].watchers.contains(&me), "S segue");
    assert!(tecla(&mut m, 'v'));
    assert!(m.kanban(id).cards[&c].votes.contains(&me), "V vota");
    let n = m.kanban(id).cards[&c].labels.len();
    assert!(tecla(&mut m, '2'));
    assert_ne!(m.kanban(id).cards[&c].labels.len(), n, "2 alterna a 2.ª etiqueta");
    assert!(tecla(&mut m, 'z'), "Z desfaz");
    assert_eq!(m.kanban(id).cards[&c].labels.len(), n);
    // Ctrl+C / Ctrl+V: uma cópia logo abaixo.
    let out = m.tecla(BoardKey::Char('c'), ctrl(), Some("c"));
    assert!(out);
    assert_eq!(m.docs().probe_clip().map(|r| r.card), Some(c));
    assert!(m.tecla(BoardKey::Char('v'), ctrl(), Some("v")));
    m.quadro();
    let ordem = m.cartoes(id, backlog);
    assert_eq!(m.kanban(id).cards[&ordem[1]].title, "Write the logline", "a cópia abaixo");
    // `L` abre o verso nas etiquetas.
    assert!(tecla(&mut m, 'l'));
    assert_eq!(m.docs().probe_back(id.0), Some(c));
}

#[test]
fn w6_arroba_cardinal_e_circunflexo_completam_e_viram_membro_etiqueta_e_posicao() {
    let (mut m, id) = Mesa::cena1();
    let backlog = m.lista(id, "Backlog");
    let c = cartao(&m, id, "Write the logline");
    foca(&mut m, id, c);
    assert!(tecla(&mut m, 'n'), "N abre o compositor por baixo");
    m.escreve("Shot @an");
    m.quadro();
    alvo(&mut m, id, "Suggest(0)");
    m.escreve("a #anim");
    // Enter com sugestões completa (não grava); sem sugestões, grava.
    m.enter();
    m.escreve("^to");
    m.enter();
    m.enter();
    let k = m.kanban(id);
    let first = k.cards_in(backlog)[0];
    assert_eq!(first.title, "Shot", "as palavras que casam saem do título");
    assert_eq!(first.members.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(), vec!["Ana"]);
    let anim = k.labels.values().find(|l| l.name == "Animation").expect("l").id;
    assert_eq!(first.labels, vec![anim]);
}
