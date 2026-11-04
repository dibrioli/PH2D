//! ⭐⭐ **O painel Macros em cartões: nenhum controlo morreu.** O inventário é o da tabela `BOTOES`/`CAMPOS`;
//! cada id é provado pelo gesto REAL nos rects pintados.

use super::seam_macros::{BITS, info};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::macro_edits::{
    FamiliaDaMacro, MacroDaBibliotecaView, MacroPanelInfo, MacroRequest, OndeAplicar, PropsDaEtapa,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_macros::state::Escolhida;
use ph2d_panel_macros::{MacrosPanel, MacrosPanelState, cards, ids, set_current_macros};
use ph2d_ui_testkit::MockPanelHost;

const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 340.0,
    h: 2400.0,
};

fn com_biblioteca() -> MacroPanelInfo {
    MacroPanelInfo {
        biblioteca: vec![MacroDaBibliotecaView {
            nome: "Tree".into(),
            familias: vec![FamiliaDaMacro::Painter],
            duracao_s: 2.5,
            tela: Some((96, 64)),
        }],
        pode_guardar: true,
        por_cima_de: Some("Sky".into()),
        por_cima_alvo: Some(7),
        ..info(Some(FamiliaDaMacro::Painter), None)
    }
}

struct Mesa {
    host: MockPanelHost,
    st: MacrosPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova(i: MacroPanelInfo) -> Self {
        cards::force_cards_on_this_thread(Some(true));
        set_current_macros(i);
        let mut mesa = Self {
            host: MockPanelHost::with_panel::<MacrosPanel>(),
            st: MacrosPanelState::default(),
            rects: Vec::new(),
        };
        mesa.host.set_panel_visible("macros", true);
        mesa.assenta();
        mesa
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<MacrosPanel>(st, COLUNA),
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
            let _ = self.host.apply_panel_event::<MacrosPanel>(&mut self.st, e);
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
        self.aplica(ev)
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

    /// Escolhe a 1.ª etapa, pelo clique na linha.
    fn escolhe(&mut self) {
        let _ = self.clica(ids::row_id(0, 0));
        self.assenta();
    }
}

fn so(a: &[EditorAction], r: MacroRequest) {
    assert_eq!(a, [EditorAction::Macro(r)]);
}

/// ⭐ O inventário: todo botão e campo pintado num estado alcançável.
#[test]
fn every_id_of_the_old_panel_is_painted_in_some_reachable_state() {
    let mut pintados = std::collections::BTreeSet::new();
    let mut parado = Mesa::nova(com_biblioteca());
    parado.escolhe();
    let _ = parado.clica(ids::lib_row_id("Tree"));
    parado.assenta();
    pintados.extend(parado.rects.iter().map(|(id, _)| *id));
    let gravando = Mesa::nova(info(None, Some(FamiliaDaMacro::Painter)));
    pintados.extend(gravando.rects.iter().map(|(id, _)| *id));
    let mut param = info(Some(FamiliaDaMacro::Motion), None);
    if let Some(o) = &mut param.objeto {
        for e in &mut o.etapas {
            e.familia = FamiliaDaMacro::Motion;
            e.props = PropsDaEtapa::Param { valor: 2.0 };
        }
    }
    let mut mesa = Mesa::nova(param);
    mesa.escolhe();
    pintados.extend(mesa.rects.iter().map(|(id, _)| *id));
    let inventario = [
        ids::MACROS_CLOSE,
        ids::MACROS_RECORD,
        ids::MACROS_STOP,
        ids::MACROS_RENAME,
        ids::MACROS_DELETE,
        ids::MACROS_HINT,
        ids::MACROS_LIB_SAVE,
        ids::MACROS_LIB_SAVE_ASSET,
        ids::MACROS_LIB_IMPORT,
        ids::MACROS_LIB_APPLY_NEW,
        ids::MACROS_LIB_APPLY_OVER,
        ids::MACROS_LIB_EXPORT,
        ids::MACROS_LIB_DELETE,
        ids::MACROS_WAIT,
        ids::MACROS_SIZE,
        ids::MACROS_VALUE,
        ids::MACROS_COLOR,
        ids::row_id(0, 0),
        ids::lib_row_id("Tree"),
    ];
    let mortos: Vec<NodeId> = inventario
        .into_iter()
        .filter(|id| !pintados.contains(id))
        .collect();
    assert!(mortos.is_empty(), "nunca pintados: {mortos:?}");
}

/// Cada verbo pelo clique real: Gravar, os três que guardam, e os da macro escolhida.
#[test]
fn every_verb_answers_a_real_click() {
    let mut mesa = Mesa::nova(com_biblioteca());
    so(&mesa.clica(ids::MACROS_RECORD), MacroRequest::Grava);
    so(
        &mesa.clica(ids::MACROS_LIB_SAVE),
        MacroRequest::Guarda { entity_bits: BITS },
    );
    so(
        &mesa.clica(ids::MACROS_LIB_SAVE_ASSET),
        MacroRequest::GuardaComoAsset { entity_bits: BITS },
    );
    so(&mesa.clica(ids::MACROS_LIB_IMPORT), MacroRequest::Importa);
    assert!(
        mesa.rect(ids::MACROS_LIB_APPLY_NEW).is_none(),
        "sem escolhida"
    );
    let _ = mesa.clica(ids::lib_row_id("Tree"));
    mesa.assenta();
    assert_eq!(mesa.st.lib_escolhida.as_deref(), Some("Tree"));
    let a = mesa.clica(ids::MACROS_LIB_APPLY_NEW);
    assert!(
        matches!(&a[..], [EditorAction::Macro(MacroRequest::Aplica { nome, onde: OndeAplicar::Novo { .. } })] if nome == "Tree"),
        "{a:?}"
    );
    assert!(
        !mesa.clica(ids::MACROS_LIB_APPLY_OVER).is_empty(),
        "por cima"
    );
    assert!(!mesa.clica(ids::MACROS_LIB_EXPORT).is_empty(), "exportar");
    assert!(!mesa.clica(ids::MACROS_LIB_DELETE).is_empty(), "apagar");
    let _ = mesa.clica(ids::MACROS_CLOSE);
    assert!(!mesa.host.panel_visible("macros"), "o «×» fecha");

    let mut grava = Mesa::nova(info(None, Some(FamiliaDaMacro::Painter)));
    so(&grava.clica(ids::MACROS_STOP), MacroRequest::Para);
}

/// Sem gravação que se guarde, os dois «Save» pintam-se apagados: sem rect, sem clique.
#[test]
fn a_verb_the_table_cannot_ask_is_off_and_takes_no_click() {
    let mesa = Mesa::nova(MacroPanelInfo {
        pode_guardar: false,
        ..com_biblioteca()
    });
    assert!(mesa.rect(ids::MACROS_LIB_SAVE).is_none());
    assert!(mesa.rect(ids::MACROS_LIB_SAVE_ASSET).is_none());
    assert!(mesa.rect(ids::MACROS_LIB_IMPORT).is_some(), "o controlo");
}

/// A etapa escolhida abre o seu cartão: o scrub da espera e do tamanho pedem a edição da SUA etapa, e
/// o clique na amostra abre o selector.
#[test]
fn the_chosen_step_opens_its_card_and_its_fields_ask_for_edits() {
    let mut mesa = Mesa::nova(com_biblioteca());
    assert!(
        mesa.rect(ids::MACROS_WAIT).is_none(),
        "sem escolhida, sem cartão"
    );
    mesa.escolhe();
    assert_eq!(
        mesa.st.escolhida,
        Some(Escolhida {
            entity_bits: BITS,
            trilha: 0,
            k: 0
        })
    );
    for id in [ids::MACROS_WAIT, ids::MACROS_SIZE] {
        let r = mesa.rect(id).expect("campo");
        let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
        let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
        ev.extend(mesa.ponteiro(PointerKind::Move, x + 30.0, y));
        ev.extend(mesa.ponteiro(PointerKind::Move, x + 60.0, y));
        let a = mesa.aplica(ev);
        assert!(
            a.iter()
                .any(|t| matches!(t, EditorAction::Macro(MacroRequest::Edita { k: 0, .. }))),
            "{id:?}: {a:?}"
        );
        let _ = mesa.ponteiro(PointerKind::Up, x + 60.0, y);
        mesa.assenta();
    }
    let ev = mesa.host.type_into_number(ids::MACROS_SIZE, "20");
    let a = mesa.aplica(ev);
    assert!(
        a.iter()
            .any(|t| matches!(t, EditorAction::Macro(MacroRequest::Edita { .. }))),
        "{a:?}"
    );
    let _ = mesa.clica(ids::MACROS_COLOR);
    assert_eq!(mesa.host.store().picker_target(), Some(ids::MACROS_COLOR));
}

/// O campo de espera anda em centésimos e o tamanho em inteiros: o passo é da grandeza.
/// *Mutação: sem `mark_card_field` ⇒ não há campo de cartão e o passo é o de fora.*
#[test]
fn each_field_of_a_step_declares_the_step_of_its_quantity() {
    let mut mesa = Mesa::nova(com_biblioteca());
    mesa.escolhe();
    let store = mesa.host.store();
    let passo = |id| store.card_field(id).map(|c| c.step);
    assert_eq!(passo(ids::MACROS_SIZE), Some(1.0));
    let espera = passo(ids::MACROS_WAIT).expect("a espera é campo de cartão");
    assert!(espera < 1.0, "a espera anda em frações: {espera}");
}

/// Dobrar, reordenar e o botão direito no título; todo ícone mudo tem dica.
#[test]
fn headers_fold_reorder_and_open_the_theme_menu() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    let mut mesa = Mesa::nova(com_biblioteca());
    let (a, b) = (ids::MACROS_SEC_RECORD, ids::MACROS_SEC_STEPS);
    let r = mesa.rect(a).expect("Record");
    assert!(mesa.rect(ids::MACROS_RECORD).is_some());
    let _ = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    assert!(mesa.host.store().is_collapsed(a));
    mesa.assenta();
    assert!(
        mesa.rect(ids::MACROS_RECORD).is_none(),
        "dobrado não regista"
    );
    let _ = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    mesa.assenta();

    let ra = mesa.rect(a).expect("A");
    let rb = mesa.rect(b).expect("B");
    let (x, y0) = (ra.x + ra.w * 0.4, ra.y + m::CARD_HEAD_H * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    let _ = mesa.ponteiro(PointerKind::Move, x, y0 + m::CARD_HEAD_H);
    mesa.assenta();
    let alvo = rb.y + rb.h * 2.0;
    let _ = mesa.ponteiro(PointerKind::Move, x, alvo);
    mesa.assenta();
    let _ = mesa.ponteiro(PointerKind::Up, x, alvo);
    mesa.assenta();
    let ordem = mesa.host.store().section_order().to_vec();
    let pos = |id| ordem.iter().position(|&k| k == id);
    assert!(pos(a) > pos(b), "não desceu: {ordem:?}");

    let r = mesa.rect(b).expect("B");
    let _ = mesa
        .host
        .right_click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    let menu = mesa.host.store().context_menu().map(|c| c.kind);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == b),
        "{menu:?}"
    );

    let store = mesa.host.store();
    let panel = ph2d_editor_core::ids::MACROS_PANEL;
    let mudos: Vec<NodeId> = dicas::icon_only_ids(panel)
        .into_iter()
        .chain([
            ids::MACROS_CLOSE,
            ph2d_editor_core::panel::cards::list_menu_of(panel),
            ph2d_editor_core::ids::grip_de(ids::MACROS_SEC_RECORD),
        ])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
}

/// `PH2D_CARDS=0`: o corpo de antes, com o seu rect de dock.
#[test]
fn cards_off_paints_the_old_body() {
    cards::force_cards_on_this_thread(Some(false));
    set_current_macros(com_biblioteca());
    let mut host = MockPanelHost::with_panel::<MacrosPanel>();
    host.set_panel_visible("macros", true);
    let mut st = MacrosPanelState::default();
    let rects = host.paint::<MacrosPanel>(&mut st, COLUNA);
    assert!(rects.iter().any(|(id, _)| *id == ids::MACROS_RECORD));
    assert!(
        !host
            .store()
            .is_floating_panel(ph2d_editor_core::ids::MACROS_PANEL)
    );
    assert!(!rects.iter().any(|(id, _)| *id == ids::MACROS_SEC_RECORD));
}
