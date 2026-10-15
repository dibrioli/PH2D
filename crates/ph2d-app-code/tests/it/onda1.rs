//! Os itens da Onda 1 que atravessam o hero: os comandos nos menus e no `Ctrl+K` (E13), a árvore de
//! acessibilidade do ecrã (E12), o cursor que pisca, o Shift+roda, gravar o projeto com a CENA activa
//! e o erro de declaração que não some da calha enquanto há ●.

use crate::{AREA, Mesa, centro, ctrl};
use ph2d_app_code::commands::{CMD_GOTO, CMD_SAVE};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::screens::hero::{document_tabs, global_palette, surface_input};
use ph2d_host::Modifiers;
use ph2d_panel_code::{FindField, FindHit};

/// Um quadro do hero como a pintura o começa (as abas e os comandos publicados).
fn segue(m: &mut Mesa) {
    m.quadro();
    document_tabs::follow_surfaces(&mut m.hero);
}

fn rotulos(m: &Mesa) -> Vec<String> {
    global_palette::build_global_model(&m.hero)
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|i| i.label.clone())
        .collect()
}

/// ⭐ **E13**: com a aba de código activa, «Go to Line» está no menu *Edit* (com o atalho) e na
/// paleta `Ctrl+K`; escolhê-lo abre o campo. Controlo: com a CENA activa não há nenhum dos dois.
#[test]
fn a_paleta_lista_go_to_line_com_a_aba_activa_e_escolher_abre_o_campo() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "local a = 1\n")]);
    m.pede(ids[0]);
    segue(&mut m);
    let goto = ph2d_i18n::tr("code.cmd.goto");
    assert!(goto.contains("Cmd+G"), "o atalho no rótulo: {goto}");
    let edit: Vec<String> = m
        .hero
        .store
        .doc_menu_rows(ContextMenuKind::MenuBarEdit)
        .map(|(_, l)| l.to_owned())
        .collect();
    assert!(edit.iter().any(|l| l == goto), "{edit:?}");
    assert!(
        m.hero
            .store
            .doc_menu_rows(ContextMenuKind::MenuBarFile)
            .any(|(id, _)| id == CMD_SAVE)
    );
    assert!(rotulos(&m).iter().any(|l| l == goto), "o Ctrl+K lista-o");

    assert!(global_palette::route_global_pick(&mut m.hero, CMD_GOTO));
    m.pinta(ids[0]);
    assert!(
        m.docs()
            .find_target(ids[0], FindHit::Field(FindField::GoTo, 0))
            .is_some(),
        "escolher «Go to Line» abre o campo"
    );

    // Controlo: a cena activa não tem os comandos da aba.
    m.hero.documents.activate(None);
    document_tabs::follow_surfaces(&mut m.hero);
    assert!(!rotulos(&m).iter().any(|l| l == goto));
    assert_eq!(
        m.hero
            .store
            .doc_menu_rows(ContextMenuKind::MenuBarEdit)
            .count(),
        0
    );
}

/// ⭐ **E12**: com a aba activa, a árvore do ecrã tem o `Document` filho do `Window` e um nó por linha
/// visível com o texto exacto; rolar troca os nós (os que saíram, saem).
#[test]
fn a_arvore_do_ecra_tem_um_no_por_linha_visivel_e_rolar_troca_os_nos() {
    use ph2d_a11y::{NodeId, Role, Tree};
    let texto: String = (0..200).map(|i| format!("local v{i} = {i}\n")).collect();
    let (mut m, ids) = Mesa::new(&[("longo.luau", texto.as_str())]);
    m.pede(ids[0]);
    segue(&mut m);
    m.pinta(ids[0]);
    let mut tree = Tree::new(NodeId(1));
    let runs = |tree: &Tree| -> Vec<String> {
        tree.ids()
            .filter_map(|id| tree.get(id))
            .filter(|n| n.role() == Role::TextRun)
            .map(|n| n.value().unwrap_or_default().to_owned())
            .collect()
    };
    surface_input::sync_a11y(&m.hero, &mut tree, AREA);
    let window = tree.get(NodeId(1)).expect("a raiz");
    assert_eq!(window.role(), Role::Window);
    let doc = tree
        .ids()
        .find(|id| tree.get(*id).map(|n| n.role()) == Some(Role::Document))
        .expect("o Document da aba");
    assert_eq!(
        window.children(),
        [doc.into()],
        "o Document é filho do Window"
    );
    let vis = m.docs().visible_lines(ids[0]).expect("vista");
    let antes = runs(&tree);
    assert_eq!(antes.len(), vis.len(), "um nó por linha visível");
    assert_eq!(antes[0], "local v0 = 0");

    // Rolar: as primeiras linhas saem da árvore, as novas entram.
    let (x, y) = centro(AREA);
    assert!(
        m.docs()
            .wheel(ids[0].0, x + AREA.w * 0.25, y, -600.0, false)
    );
    m.pinta(ids[0]);
    surface_input::sync_a11y(&m.hero, &mut tree, AREA);
    let depois = runs(&tree);
    let vis = m.docs().visible_lines(ids[0]).expect("vista");
    assert!(vis.start > 0, "rolou");
    assert_eq!(depois.len(), vis.len());
    assert!(
        !depois.contains(&"local v0 = 0".to_owned()),
        "a linha 0 saiu"
    );
    assert_eq!(depois[0], format!("local v{0} = {0}", vis.start));

    // E a aba fechada leva os nós: só o Window fica.
    m.hero.documents.activate(None);
    surface_input::sync_a11y(&m.hero, &mut tree, AREA);
    assert_eq!(tree.len(), 1);
}

/// O cursor PISCA (meio-período = o token `Slow`), fica aceso enquanto se escreve, e o «movimento
/// reduzido» deixa-o fixo.
#[test]
fn o_cursor_pisca_e_fica_aceso_enquanto_se_escreve() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "-- bob\n")]);
    m.pede(ids[0]);
    let ms = 1_000_000u128;
    let meio = u128::from(ph2d_tokens::Duration::Slow.ms() as u32) * ms;
    let t0 = 10_000 * ms;
    m.docs().set_clock(t0, false);
    m.escreve(ids[0], "x");
    assert!(m.docs().caret_lit(ids[0]), "aceso logo depois da tecla");
    m.docs().set_clock(t0 + meio, false);
    assert!(!m.docs().caret_lit(ids[0]), "apagado no 2.º meio-período");
    m.docs().set_clock(t0 + meio * 2, false);
    assert!(m.docs().caret_lit(ids[0]), "aceso de novo");
    // A escrever (teclas mais depressa que o piscar): sempre aceso.
    for k in 1..6u128 {
        m.docs().set_clock(t0 + meio * 2 + k * meio / 2, false);
        m.escreve(ids[0], "y");
        assert!(m.docs().caret_lit(ids[0]));
    }
    // Movimento reduzido: fixo.
    m.docs().set_clock(t0 + meio * 99, true);
    assert!(m.docs().caret_lit(ids[0]));
    // O relógio do frame é o do chrome.
    m.hero.store.note_clock(t0 + meio * 101);
    m.quadro();
    let lit = m.docs().caret_lit(ids[0]);
    m.hero.store.note_clock(t0 + meio * 102);
    m.quadro();
    assert_ne!(
        m.docs().caret_lit(ids[0]),
        lit,
        "o frame lê o relógio do chrome"
    );
}

/// Shift+roda rola na HORIZONTAL, pela porta do hero (o Shift vem da cache que a shell actualiza).
#[test]
fn shift_roda_rola_na_horizontal() {
    let linha = "x".repeat(400);
    let texto = format!("{linha}\n{linha}\n");
    let (mut m, ids) = Mesa::new(&[("largo.luau", texto.as_str())]);
    m.pede(ids[0]);
    segue(&mut m);
    m.pinta(ids[0]);
    let (x, y) = centro(AREA);
    let x = x + AREA.w * 0.25;
    m.hero.store.set_shift_held(true);
    assert!(ph2d_editor_core::screens::hero::board_view::wheel(
        &mut m.hero,
        x,
        y,
        -120.0
    ));
    let (sx, sy) = m.docs().scroll_of(ids[0]).expect("aberto");
    assert!(sx > 0.0 && sy == 0.0, "rolou na horizontal: ({sx}, {sy})");
    // Controlo: sem Shift, a roda não mexe na horizontal.
    m.hero.store.set_shift_held(false);
    let _ = ph2d_editor_core::screens::hero::board_view::wheel(&mut m.hero, x, y, -120.0);
    assert_eq!(m.docs().scroll_of(ids[0]).expect("aberto").0, sx);
}

/// ⭐ **Gravar o projeto com a CENA activa aplica as abas com ●** (decisão do dono: «salvar =
/// guardado»): o texto vai para a biblioteca e o ● sai; uma aba sem ● não é tocada.
#[test]
fn gravar_com_a_cena_activa_aplica_as_abas_com_alteracoes() {
    let (m, ids) = Mesa::new(&[("bob.luau", "-- bob\n"), ("ana.luau", "-- ana\n")]);
    let mut m = m.com_cena(ids[0], 1);
    m.pede(ids[1]);
    m.pede(ids[0]);
    m.escreve(ids[0], "-- novo\n");
    assert!(m.docs().is_dirty(ids[0]));
    m.hero.documents.activate(None);
    assert!(m.hero.documents.scene_active());
    // Controlo: um quadro SEM gravar não aplica nada.
    m.quadro();
    assert!(m.docs().is_dirty(ids[0]));
    assert_eq!(m.texto_salvo(ids[0]), "-- bob\n");
    // File ▸ Save / Ctrl+S com a cena activa (a bandeira do menu).
    m.hero.file_menu.save = true;
    m.quadro();
    let texto = m.docs().text_of(ids[0]).expect("aberto");
    assert_eq!(m.texto_salvo(ids[0]), texto);
    assert!(!m.docs().is_dirty(ids[0]));
    assert_eq!(m.texto_salvo(ids[1]), "-- ana\n");
    let _ = ctrl();
}

/// ⭐ **Um erro de DECLARAÇÃO (`ph2d.property`) não some da calha enquanto há ●** — e segue a linha
/// quando o texto acima dela muda. Controlo: um erro de SINTAXE salvo sai com o ● (o tree-sitter
/// mostra-o vivo).
#[test]
fn o_erro_de_declaracao_fica_na_calha_enquanto_ha_alteracoes() {
    let texto = "-- um\nph2d.property(\"a\", 1)\nph2d.property(\"a\", 2)\n";
    let (m, ids) = Mesa::new(&[("dup.luau", texto)]);
    let mut m = m.com_cena(ids[0], 1);
    m.cena();
    m.pede(ids[0]);
    m.quadro();
    let linhas = |m: &mut Mesa| -> Vec<usize> {
        m.docs()
            .problems_of(ids[0])
            .iter()
            .map(|p| p.line)
            .collect()
    };
    assert_eq!(
        linhas(&mut m),
        [2],
        "o Luau acusa a 2.ª declaração (linha 3)"
    );
    // Escrever uma linha no topo: ● — o erro fica, uma linha abaixo.
    m.tecla(ids[0], Key::Home, ctrl(), None);
    m.escreve(ids[0], "-- novo\n");
    m.quadro();
    assert!(m.docs().is_dirty(ids[0]));
    assert_eq!(linhas(&mut m), [3], "segue o texto");

    // Controlo: um erro de sintaxe salvo sai quando o texto o corrige (sem salvar).
    let (m2, ids2) = Mesa::new(&[("q.luau", "local x = = 1\n")]);
    let mut m2 = m2.com_cena(ids2[0], 1);
    m2.cena();
    m2.pede(ids2[0]);
    m2.quadro();
    assert!(!m2.docs().problems_of(ids2[0]).is_empty());
    m2.tecla(ids2[0], Key::Char('a'), ctrl(), Some("a"));
    m2.escreve(ids2[0], "local x = 1\n");
    m2.quadro();
    assert!(m2.docs().is_dirty(ids2[0]));
    assert!(
        m2.docs().problems_of(ids2[0]).is_empty(),
        "corrigido: nada na calha"
    );
}
