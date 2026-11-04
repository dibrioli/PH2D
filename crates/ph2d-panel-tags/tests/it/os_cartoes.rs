//! ⭐ **O painel Tags em cartões: a lista continua lista.** Cada verbo, cada linha, o renomear e o
//! arrastar para aninhar, pelo ponteiro REAL, nos rects que a pintura registou.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{dicas, list_menu_of};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{TagTreeEdit, TagsPanelInfo, TagsPanelRow};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_tags::{TagsPanel, TagsPanelState, cards, ids, set_current_tags};
use ph2d_ui_testkit::MockPanelHost;

const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 340.0,
    h: 1400.0,
};
const ENEMY: u64 = 7;
const FLYING: u64 = 8;
const STATUE: u64 = 9;
const PAINEL: NodeId = ph2d_editor_core::ids::TAGS_PANEL;

fn tag(id: u64, label: &str, depth: usize, members: usize, subtree: usize) -> TagsPanelRow {
    TagsPanelRow {
        id,
        label: label.into(),
        depth,
        members,
        subtree,
    }
}

fn arvore() -> TagsPanelInfo {
    TagsPanelInfo {
        rows: vec![
            tag(ENEMY, "Enemy", 0, 5, 2),
            tag(FLYING, "Flying", 1, 3, 1),
            tag(STATUE, "Statue", 0, 1, 1),
        ],
        problem: None,
    }
}

struct Mesa {
    host: MockPanelHost,
    st: TagsPanelState,
    rects: Vec<(NodeId, Rect)>,
    area: Rect,
}

impl Mesa {
    fn em(info: TagsPanelInfo, area: Rect) -> Self {
        cards::force_cards_on_this_thread(Some(true));
        set_current_tags(info);
        let mut mesa = Self {
            host: MockPanelHost::with_panel::<TagsPanel>(),
            st: TagsPanelState::default(),
            rects: Vec::new(),
            area,
        };
        mesa.assenta();
        mesa
    }

    fn nova() -> Self {
        Self::em(arvore(), COLUNA)
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st, area) = (&mut self.host, &mut self.st, self.area);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<TagsPanel>(st, area),
            );
        }
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<EditorAction> {
        for e in eventos {
            let _ = self.host.apply_panel_event::<TagsPanel>(&mut self.st, e);
        }
        self.host.drained_actions()
    }

    fn clica(&mut self, id: NodeId) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "{id:?} morto sob o dedo: {ev:?}"
        );
        self.aplica(ev)
    }

    fn escolhe(&mut self, tag: u64) {
        let _ = self.clica(ids::row_id(tag));
        assert_eq!(self.st.focus, Some(tag));
        self.assenta();
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
        self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: 0,
        })
    }
}

fn um_so(accoes: &[EditorAction], edit: TagTreeEdit) {
    assert_eq!(accoes, [EditorAction::TagTreeEdit { edit }]);
}

/// O «+» do cabeçalho faz o que o `+ New` fazia, sem linha em mãos.
#[test]
fn the_plus_on_the_header_creates_a_root_tag() {
    let mut mesa = Mesa::nova();
    let out = mesa.clica(ids::TAGS_NEW);
    um_so(&out, TagTreeEdit::Create { parent: None });
}

/// O inventário: sem linha em mãos só o «+» e o «×»; com ela, os verbos dela; uma raiz não muda de
/// pai. Cada verbo chega ao barramento pelo clique.
#[test]
fn every_verb_of_the_chosen_row_answers_a_real_click() {
    let mesa = Mesa::nova();
    for id in [
        ids::TAGS_CHILD,
        ids::TAGS_RENAME,
        ids::TAGS_UNPARENT,
        ids::TAGS_SELECT,
        ids::TAGS_DELETE,
    ] {
        assert!(mesa.rect(id).is_none(), "{id:?} oferecido sem sujeito");
    }
    assert!(mesa.rect(ids::TAGS_CLOSE).is_some(), "o × da lista");
    for (id, esperado) in [
        (
            ids::TAGS_CHILD,
            TagTreeEdit::Create {
                parent: Some(FLYING),
            },
        ),
        (ids::TAGS_DELETE, TagTreeEdit::Delete { id: FLYING }),
        (ids::TAGS_SELECT, TagTreeEdit::SelectTagged { id: FLYING }),
        (
            ids::TAGS_UNPARENT,
            TagTreeEdit::Move {
                id: FLYING,
                parent: None,
            },
        ),
    ] {
        let mut mesa = Mesa::nova();
        mesa.escolhe(FLYING);
        let out = mesa.clica(id);
        um_so(&out, esperado);
    }
    let mut raiz = Mesa::nova();
    raiz.escolhe(STATUE);
    assert!(raiz.rect(ids::TAGS_UNPARENT).is_none());
}

/// O «×» da lista fecha o painel: o id de sempre, despachado como antes.
#[test]
fn the_close_of_the_list_hides_the_panel() {
    let mut mesa = Mesa::nova();
    let _ = mesa.clica(ids::TAGS_CLOSE);
    assert!(
        !mesa
            .host
            .panel_visible(<TagsPanel as ph2d_editor_core::panel::Panel>::ID)
    );
}

/// `Rename` abre o campo por cima do nome da linha, semeado; a contagem continua à vista; o Enter
/// escreve no documento.
#[test]
fn rename_opens_over_the_row_and_the_submit_reaches_the_bus() {
    let mut mesa = Mesa::nova();
    mesa.escolhe(ENEMY);
    let _ = mesa.clica(ids::TAGS_RENAME);
    mesa.assenta();
    let linha = mesa.rect(ids::row_id(ENEMY)).expect("linha");
    let campo = mesa.rect(ids::TAGS_RENAME_INPUT).expect("o campo abriu");
    assert!(campo.y == linha.y && campo.x + campo.w < linha.x + linha.w);
    assert_eq!(
        mesa.host.store().text(ids::TAGS_RENAME_INPUT),
        Some("Enemy")
    );
    mesa.host.set_text(ids::TAGS_RENAME_INPUT, "Inimigo");
    let out = mesa.aplica(vec![WidgetEvent::Submit(ids::TAGS_RENAME_INPUT)]);
    um_so(
        &out,
        TagTreeEdit::Rename {
            id: ENEMY,
            label: "Inimigo".into(),
        },
    );
}

/// ⭐ Arrastar uma tag para o meio de outra aninha-a, pelo gesto real; no ar, a linha voa sem
/// registar clique por cima da árvore.
#[test]
fn dragging_a_row_onto_another_nests_it_and_it_flies_meanwhile() {
    let mut mesa = Mesa::nova();
    let de = mesa.rect(ids::row_id(STATUE)).expect("Statue");
    let para = mesa.rect(ids::row_id(ENEMY)).expect("Enemy");
    let (x, y0) = (de.x + de.w * 0.5, de.y + de.h * 0.5);
    let meio = para.y + para.h * 0.5;
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    let _ = mesa.ponteiro(PointerKind::Move, x, y0 - 8.0);
    let _ = mesa.ponteiro(PointerKind::Move, x, meio);
    let sem_voo = mesa.rects.iter().filter(|(_, r)| r.w > 0.0).count();
    mesa.assenta();
    assert_eq!(
        mesa.rects.iter().filter(|(_, r)| r.w > 0.0).count(),
        sem_voo,
        "o voo não pode registar clique"
    );
    let ev = mesa.ponteiro(PointerKind::Up, x, meio);
    let out = mesa.aplica(ev);
    um_so(
        &out,
        TagTreeEdit::Move {
            id: STATUE,
            parent: Some(ENEMY),
        },
    );
}

/// A recusa aparece debaixo da linha dela e empurra a linha seguinte; contraprova: sem recusa não.
#[test]
fn a_refusal_pushes_the_next_row_down() {
    let limpo = Mesa::nova();
    let mut com = arvore();
    com.problem = Some((ENEMY, "That name is taken".into()));
    let queixa = Mesa::em(com, COLUNA);
    let y = |m: &Mesa| m.rect(ids::row_id(FLYING)).expect("Flying").y;
    assert!(
        y(&queixa) > y(&limpo) + m::LABEL_PX,
        "a recusa não abriu lugar"
    );
}

/// O projecto vazio diz o gesto que o enche, e os verbos não aparecem.
#[test]
fn an_empty_project_still_offers_the_plus() {
    let mut mesa = Mesa::em(TagsPanelInfo::default(), COLUNA);
    let out = mesa.clica(ids::TAGS_NEW);
    um_so(&out, TagTreeEdit::Create { parent: None });
}

/// Dobra pelo cabeçalho; dobrado, as linhas saem do clique.
#[test]
fn the_header_folds_the_tree() {
    let mut mesa = Mesa::nova();
    let r = mesa.rect(ids::TAGS_SEC_TREE).expect("cartão");
    let _ = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    mesa.assenta();
    assert!(mesa.host.store().is_collapsed(ids::TAGS_SEC_TREE));
    assert!(mesa.rect(ids::row_id(ENEMY)).is_none());
}

/// Muitas tags num encaixe baixo: a lista rola e a linha de baixo responde.
#[test]
fn a_long_tree_scrolls_and_the_lower_row_answers() {
    let muitas = TagsPanelInfo {
        rows: (0..60)
            .map(|i| tag(100 + i, &format!("Tag {i}"), 0, 0, 1))
            .collect(),
        problem: None,
    };
    let baixa = Rect::new(0.0, 0.0, 340.0, 500.0);
    let mut mesa = Mesa::em(muitas, baixa);
    let alvo = ids::row_id(159);
    let visivel = |m: &Mesa| m.rect(alvo).is_some_and(|r| r.y + r.h < baixa.h);
    assert!(!visivel(&mesa), "a última começa abaixo da dobra");
    let _ = mesa.host.wheel_at(baixa.w * 0.5, baixa.h * 0.5, -5000.0);
    mesa.assenta();
    assert!(visivel(&mesa), "a roda não trouxe a última linha");
    let _ = mesa.clica(alvo);
    assert_eq!(mesa.st.focus, Some(159));
}

/// Cartões reordenam-se pelo cabeçalho, e reordenar não mexe no documento.
#[test]
fn dragging_a_card_reorders_the_list() {
    let mut mesa = Mesa::nova();
    let a = mesa.rect(ids::TAGS_SEC_TREE).expect("árvore");
    let b = mesa.rect(ids::TAGS_SEC_SELECTED).expect("verbos");
    let (x, y0) = (a.x + a.w * 0.4, a.y + m::CARD_HEAD_H * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    let _ = mesa.ponteiro(PointerKind::Move, x, y0 + m::CARD_HEAD_H);
    mesa.assenta();
    let alvo = b.y + b.h * 2.0;
    let _ = mesa.ponteiro(PointerKind::Move, x, alvo);
    mesa.assenta();
    let _ = mesa.ponteiro(PointerKind::Up, x, alvo);
    mesa.assenta();
    let ordem = mesa.host.store().section_order().to_vec();
    let pos = |id| ordem.iter().position(|&k| k == id);
    assert!(
        pos(ids::TAGS_SEC_TREE) > pos(ids::TAGS_SEC_SELECTED),
        "{ordem:?}"
    );
    assert!(mesa.host.drained_actions().is_empty());
}

/// Todo ícone sem texto tem dica.
#[test]
fn every_mute_control_has_a_tooltip() {
    let mesa = Mesa::nova();
    let store = mesa.host.store();
    let mudos: Vec<NodeId> = dicas::icon_only_ids(PAINEL)
        .into_iter()
        .chain([ids::TAGS_NEW, ids::TAGS_CLOSE, list_menu_of(PAINEL)])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
}

/// `PH2D_CARDS=0`: a barra de verbos de antes, com o `+ New` em texto e sem secções de cartão.
#[test]
fn cards_off_paints_the_old_body() {
    cards::force_cards_on_this_thread(Some(false));
    set_current_tags(arvore());
    let mut host = MockPanelHost::with_panel::<TagsPanel>();
    let mut st = TagsPanelState::default();
    let rects = host.paint::<TagsPanel>(&mut st, COLUNA);
    assert!(rects.iter().any(|(id, _)| *id == ids::TAGS_NEW));
    assert!(!rects.iter().any(|(id, _)| *id == ids::TAGS_SEC_TREE));
}
