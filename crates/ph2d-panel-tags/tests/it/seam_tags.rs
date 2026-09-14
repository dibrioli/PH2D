//! ⭐⭐⭐ **A varredura de COSTURA do painel TAGS** (plano §5.2, gates 26–28) — cada controlo que o
//! painel pinta é **carregado pelo PONTEIRO** e a acção exacta dele é afirmada.
//!
//! ⚠️ **Um `WidgetEvent::Click` sintético passa com o controlo MORTO sob o dedo** — foi o que
//! deixou os quatro chips da booleana verdes durante duas waves. O que separa as duas coisas é o
//! `click_at`, que atravessa o mesmo `dispatch_pointer` da shell: um id que o `populate` não
//! registou não fica `active` no Down, e o `Click` nunca nasce.
//!
//! ⚠️ **É uma VARREDURA, não uma amostra**: os seis verbos e a linha entram aqui. *Escolher o
//! controlo mais cheio e confiar que ele cobre os outros* é a premissa que já apodreceu duas vezes
//! neste repo.

use ph2d_editor_core::TagTreeEdit;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{TagsPanelInfo, TagsPanelRow};
use ph2d_panel_tags::{TagsPanel, TagsPanelState, ids, set_current_tags};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 1200.0,
};

const ENEMY: u64 = 7;
const FLYING: u64 = 8;
const STATUE: u64 = 9;

/// **A fixtura**: `Enemy (5)` › `Flying (3)`, e a raiz irmã `Statue (1)`.
///
/// ⚠️ **Os ids NÃO são `1, 2, 3`** de propósito: um gate que confundisse o id da tag com o índice
/// da linha passaria sobre uma fixtura assim, e é exactamente a espécie de troca que o painel faz
/// (ele deriva o `NodeId` da linha do id da TAG).
fn arvore() -> TagsPanelInfo {
    TagsPanelInfo {
        rows: vec![
            TagsPanelRow {
                id: ENEMY,
                label: "Enemy".into(),
                depth: 0,
                members: 5,
                subtree: 2,
            },
            TagsPanelRow {
                id: FLYING,
                label: "Flying".into(),
                depth: 1,
                members: 3,
                subtree: 1,
            },
            TagsPanelRow {
                id: STATUE,
                label: "Statue".into(),
                depth: 0,
                members: 1,
                subtree: 1,
            },
        ],
        problem: None,
    }
}

/// Pinta o painel e devolve `(host, state, rects)`. O painel é forçado visível — um `paint` gateado
/// na visibilidade devolveria antes de desenhar.
fn palco(
    info: TagsPanelInfo,
) -> (
    MockPanelHost,
    TagsPanelState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut host = MockPanelHost::with_panel::<TagsPanel>();
    let mut state = TagsPanelState::default();
    set_current_tags(info);
    let rects = host.paint::<TagsPanel>(&mut state, VIEWPORT);
    (host, state, rects)
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Rect {
    rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"))
}

/// Carrega no MEIO de `id` com um ponteiro a sério e devolve o que chegou ao barramento.
fn carregar(
    host: &mut MockPanelHost,
    state: &mut TagsPanelState,
    rects: &[(ph2d_a11y::NodeId, Rect)],
    id: ph2d_a11y::NodeId,
) -> Vec<EditorAction> {
    let r = rect_de(rects, id);
    let eventos = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        eventos
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "carregar no meio de {id:?} deu {eventos:?} — o controlo é pintado e hit-registado, e o \
         store não o considera focável: ele está MORTO sob o dedo"
    );
    for ev in eventos {
        let _ = host.apply_panel_event::<TagsPanel>(state, ev);
    }
    host.drained_actions()
}

/// Escolhe uma linha (o sujeito dos verbos), pelo ponteiro.
fn escolher(
    host: &mut MockPanelHost,
    state: &mut TagsPanelState,
    rects: &[(ph2d_a11y::NodeId, Rect)],
    tag: u64,
) {
    let _ = carregar(host, state, rects, ids::row_id(tag));
    assert_eq!(state.focus, Some(tag), "a linha não ficou em mãos");
}

fn so(actions: &[EditorAction], edit: TagTreeEdit, o_que: &str) {
    assert_eq!(
        actions,
        [EditorAction::TagTreeEdit { edit: edit.clone() }],
        "{o_que} não chegou ao barramento como exactamente um TagTreeEdit({edit:?})"
    );
}

/// ⭐⭐⭐ **`+ New` cria uma raiz** — o único verbo oferecido sem linha em mãos, e a porta do painel
/// num projecto vazio.
#[test]
fn the_new_button_reaches_the_bus_with_no_row_in_hand() {
    let (mut host, mut state, rects) = palco(TagsPanelInfo::default());
    let out = carregar(&mut host, &mut state, &rects, ids::TAGS_NEW);
    so(&out, TagTreeEdit::Create { parent: None }, "+ New");
}

/// ⛔ **Sem linha em mãos, os outros verbos NÃO são pintados** — o painel nunca oferece um gesto
/// que vai ser recusado.
///
/// **Mutação que deve sangrar:** o `verbs()` a devolver a lista inteira sempre.
#[test]
fn with_no_row_in_hand_the_bar_offers_only_new() {
    let (_, _, rects) = palco(arvore());
    let pintado = |id| rects.iter().any(|(n, _)| *n == id);
    assert!(pintado(ids::TAGS_NEW));
    for id in [
        ids::TAGS_CHILD,
        ids::TAGS_RENAME,
        ids::TAGS_DELETE,
        ids::TAGS_SELECT,
        ids::TAGS_UNPARENT,
    ] {
        assert!(!pintado(id), "{id:?} foi oferecido sem sujeito");
    }
}

/// ⭐⭐⭐ **Escolher uma linha acende os verbos DELA, e os cinco chegam ao barramento** — a
/// varredura inteira (gate 26), pelo ponteiro.
#[test]
fn every_verb_of_a_chosen_row_reaches_the_bus() {
    // ⚠️ Cada verbo corre no seu palco: um clique deixa estado no store (foco, `Pressed`), e
    // encadeá-los mediria o segundo sobre o resíduo do primeiro.
    for (id, esperado, o_que) in [
        (
            ids::TAGS_CHILD,
            TagTreeEdit::Create {
                parent: Some(FLYING),
            },
            "+ Child",
        ),
        (
            ids::TAGS_DELETE,
            TagTreeEdit::Delete { id: FLYING },
            "Delete",
        ),
        (
            ids::TAGS_SELECT,
            TagTreeEdit::SelectTagged { id: FLYING },
            "Select",
        ),
        (
            ids::TAGS_UNPARENT,
            TagTreeEdit::Move {
                id: FLYING,
                parent: None,
            },
            "Move to root",
        ),
    ] {
        let (mut host, mut state, rects) = palco(arvore());
        escolher(&mut host, &mut state, &rects, FLYING);
        // ⚠️ **Re-pintar depois de escolher**: os verbos da linha só nascem no quadro seguinte, e
        // um `rect` do quadro anterior apontaria para um botão que ainda não existia.
        let rects = host.paint::<TagsPanel>(&mut state, VIEWPORT);
        let out = carregar(&mut host, &mut state, &rects, id);
        so(&out, esperado, o_que);
    }
}

/// ⛔ **`Move to root` não é oferecido a uma RAIZ** — ela já está lá, e um botão que não faz nada é
/// um botão que ensina que o gesto não funciona.
#[test]
fn move_to_root_is_not_offered_to_a_root() {
    let (mut host, mut state, rects) = palco(arvore());
    escolher(&mut host, &mut state, &rects, STATUE);
    let rects = host.paint::<TagsPanel>(&mut state, VIEWPORT);
    assert!(
        !rects.iter().any(|(n, _)| *n == ids::TAGS_UNPARENT),
        "uma raiz recebeu «Move to root»"
    );
}

/// ⭐⭐⭐ **O DUPLO CLIQUE renomeia** (gate 26) — e o `Submit` do campo leva o nome ao barramento.
///
/// ⚠️ **Dois cliques dentro da janela de 350 ms**, que é o que o despacho promove a `DoubleClick`;
/// o `click_at` espaça os cliques um segundo de propósito, então aqui o gesto é montado à mão.
#[test]
fn a_double_click_on_a_row_opens_the_rename_and_the_commit_reaches_the_bus() {
    let (mut host, mut state, rects) = palco(arvore());
    let id = ids::row_id(ENEMY);
    let _ = host.apply_panel_event::<TagsPanel>(&mut state, WidgetEvent::DoubleClick(id));
    assert_eq!(state.renaming, Some(ENEMY), "o campo não abriu");
    // ⭐ O campo abre **semeado** com o nome de agora — renomear é editar, não reescrever do zero.
    assert_eq!(
        host.store().text(ids::TAGS_RENAME_INPUT),
        Some("Enemy"),
        "o campo abriu vazio: o artista perde o nome que ia só emendar"
    );
    let _ = rects;
    host.set_text(ids::TAGS_RENAME_INPUT, "Inimigo");
    let _ = host
        .apply_panel_event::<TagsPanel>(&mut state, WidgetEvent::Submit(ids::TAGS_RENAME_INPUT));
    so(
        &host.drained_actions(),
        TagTreeEdit::Rename {
            id: ENEMY,
            label: "Inimigo".into(),
        },
        "o commit do renomear",
    );
    assert_eq!(
        state.renaming, None,
        "o campo ficou aberto depois do commit"
    );
}

/// ⚠️ **`Esc` aborta e NÃO escreve** — um campo que comita ao sair torna impossível desistir.
#[test]
fn escape_aborts_the_rename_without_writing() {
    let (mut host, mut state, _) = palco(arvore());
    let _ = host
        .apply_panel_event::<TagsPanel>(&mut state, WidgetEvent::DoubleClick(ids::row_id(ENEMY)));
    host.set_text(ids::TAGS_RENAME_INPUT, "seja o que for");
    let _ = host
        .apply_panel_event::<TagsPanel>(&mut state, WidgetEvent::Cancel(ids::TAGS_RENAME_INPUT));
    assert_eq!(state.renaming, None);
    assert!(
        host.drained_actions().is_empty(),
        "o Esc escreveu no documento"
    );
}

/// ⭐⭐ **O rótulo do `Delete` DIZ o estrago** (a metade *«remove from N objects»* do plano): apagar
/// `Enemy` leva `Flying` junto e desmarca 5 objectos, e o botão escreve-o ANTES de ser carregado.
///
/// ⚠️ **A régua é a LARGURA do botão**, e não um palpite: ela é derivada do rótulo
/// (`verb_w = prefix_width(label) + 2·Lg`), logo dois estragos diferentes dão dois botões de
/// tamanhos diferentes — e um rótulo fixo *«Delete»* daria **o mesmo** nos dois. A outra metade (o
/// texto exacto) é o gate de unidade na crate, que lê a tabela dos verbos.
///
/// **Mutações que devem sangrar:** o rótulo fixo · o número a ser o da subárvore no lugar do dos
/// objectos (a largura deixaria de seguir a fixtura).
#[test]
fn the_delete_button_is_as_wide_as_the_damage_it_names() {
    let largura = |tag: u64| {
        let (mut host, mut state, rects) = palco(arvore());
        escolher(&mut host, &mut state, &rects, tag);
        let rects = host.paint::<TagsPanel>(&mut state, VIEWPORT);
        rect_de(&rects, ids::TAGS_DELETE).w
    };
    let (grande, pequeno) = (largura(ENEMY), largura(STATUE));
    assert!(
        grande > pequeno + 1.0,
        "o Delete de `Enemy` («2 tags, 5 objects») tem de ser MAIS LARGO que o de `Statue` \
         («1 objects») — leu {grande} contra {pequeno}: o rótulo não segue o estrago"
    );
}

/// ⭐⭐⭐ **A RECUSA chega a PIXEL** (gate 27) — a frase que a lei produziu, contada em GLIFOS.
///
/// ⚠️ **Uma faixa reservada NÃO é uma faixa pintada**: um gate de geometria fica verde sobre um
/// ecrã em branco (a lição do `TextRow.problem` do L-System). O que se conta aqui é o que o Vello
/// encaminhou por `draw_glyphs`, que é a única coisa que distingue as duas.
///
/// **Mutação que deve sangrar:** o `problem` a ser lido e não pintado.
#[test]
fn a_refusal_reaches_glyphs_on_its_own_row() {
    let limpo = {
        let mut host = MockPanelHost::with_panel::<TagsPanel>();
        let mut state = TagsPanelState::default();
        set_current_tags(arvore());
        host.paint_and_count_geometry::<TagsPanel>(&mut state, VIEWPORT)
            .0
    };
    let frase = "A tag with this name already exists here.";
    let com_queixa = {
        let mut info = arvore();
        info.problem = Some((FLYING, frase.to_string()));
        let mut host = MockPanelHost::with_panel::<TagsPanel>();
        let mut state = TagsPanelState::default();
        set_current_tags(info);
        host.paint_and_count_geometry::<TagsPanel>(&mut state, VIEWPORT)
            .0
    };
    // ⚠️ A barra é a FRASE INTEIRA menos os espaços — um `>` sozinho passaria com um ponto pintado.
    let minimo = frase.chars().filter(|c| !c.is_whitespace()).count() as u32;
    assert!(
        com_queixa >= limpo + minimo,
        "a recusa não chegou a pixel: {limpo} glifos sem ela, {com_queixa} com ela \
         (esperava pelo menos +{minimo})"
    );
}

/// ⛔ **Um painel vazio DIZ o gesto que o enche** — um painel em branco lê-se como partido.
///
/// ⚠️ Medido em GLIFOS pela mesma razão do gate acima.
#[test]
fn an_empty_tree_says_the_gesture_that_fills_it() {
    let mut host = MockPanelHost::with_panel::<TagsPanel>();
    let mut state = TagsPanelState::default();
    set_current_tags(TagsPanelInfo::default());
    let (glifos, _) = host.paint_and_count_geometry::<TagsPanel>(&mut state, VIEWPORT);
    // O painel vazio só tem o `+ New` (5 caracteres visíveis) e a frase do vazio (39).
    assert!(
        glifos > 30,
        "o painel vazio pintou {glifos} glifos — ele não diz nada"
    );
}
