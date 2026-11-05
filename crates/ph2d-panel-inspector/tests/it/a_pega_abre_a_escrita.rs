//! ⭐ Um clique ou duplo clique no ícone de um campo de cartão não perde o que se escreve a seguir
//! (janela real, 05/11: `Scale X`, dois cliques no «X», `Ctrl+A`, `2.5`, `Enter` não mudava nada e as
//! teclas caíam nos atalhos do app). A varredura é a família: todo campo com pega pintado no Inspector.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::aquecimento::{PASSO_DE_QUADRO_S, com_passo_fixo};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_editor_core::widget::card_field::handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids::{
    INSP_TRANSFORM_POS_X, INSP_TRANSFORM_ROT, INSP_TRANSFORM_SCALE_X, INSP_TRANSFORM_SKEW_X,
};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 2000.0,
};

/// Dentro da janela do duplo clique.
const ENTRE_TOQUES_NS: u128 = 60_000_000;
/// Fora dela.
const NOVO_GESTO_NS: u128 = 2_000_000_000;

struct Mesa {
    host: MockPanelHost,
    st: InspectorState,
    rects: Vec<(NodeId, Rect)>,
    relogio: u128,
}

impl Mesa {
    fn nova() -> Self {
        set_cards_on(true);
        set_current_inspector_transform(Some(InspectorTransformInfo {
            entity_bits: 7,
            translation: [40.0, 50.0],
            rotation_rad: 0.5,
            scale: [2.5, 1.5],
            skew_rad: [0.0, 0.0],
        }));
        let mut m = Self {
            host: MockPanelHost::with_panel::<InspectorPanel>(),
            st: InspectorState::default(),
            rects: Vec::new(),
            relogio: NOVO_GESTO_NS,
        };
        m.host.settle_section_folds();
        for _ in 0..60 {
            m.quadro();
        }
        m
    }

    fn quadro(&mut self) {
        let (host, st) = (&mut self.host, &mut self.st);
        self.rects = com_passo_fixo(PASSO_DE_QUADRO_S, || {
            host.paint::<InspectorPanel>(st, JANELA)
        });
    }

    fn pega(&self, id: NodeId) -> (f32, f32) {
        let r = self
            .rects
            .iter()
            .rev()
            .find(|(i, q)| *i == id && q.w > 0.0)
            .map(|(_, q)| *q)
            .unwrap_or_else(|| panic!("{id:?} não se pintou"));
        let h = handle_rect(r);
        (h.x + h.w * 0.5, h.y + h.h * 0.5)
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) {
        let evs: Vec<WidgetEvent> = self.host.dispatch_pointer_event(PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: self.relogio,
        });
        for e in evs {
            self.host
                .apply_panel_event::<InspectorPanel>(&mut self.st, e);
        }
    }

    fn toque(&mut self, (x, y): (f32, f32), passo: u128) {
        self.relogio += passo;
        self.ponteiro(PointerKind::Down, x, y);
        self.ponteiro(PointerKind::Up, x, y);
    }

    fn escreve(&mut self, id: NodeId, texto: &str) {
        for e in self.host.type_keys_then_enter(id, texto) {
            self.host
                .apply_panel_event::<InspectorPanel>(&mut self.st, e);
        }
        assert!(
            self.host
                .drained_actions()
                .iter()
                .any(|a| matches!(a, EditorAction::InspectorTransformEdit(_))),
            "{id:?}: o que se escreveu não chegou ao objecto"
        );
    }

    fn valor(&self, id: NodeId) -> f64 {
        self.host.store().number_value(id).expect("campo numérico")
    }

    /// Todo campo de cartão com pega que esta pintura mostra.
    fn com_pega(&self) -> Vec<NodeId> {
        let mut ids: Vec<NodeId> = self
            .rects
            .iter()
            .filter(|(id, r)| {
                r.w > 0.0
                    && self.host.store().card_field(*id).is_some()
                    && self.host.store().tool_row_slot(*id) != Some(*r)
            })
            .map(|(id, _)| *id)
            .collect();
        ids.sort_by_key(|i| i.0);
        ids.dedup();
        ids
    }
}

impl Drop for Mesa {
    fn drop(&mut self) {
        set_current_inspector_transform(None);
        set_cards_on(false);
    }
}

/// Clicar no ícone abre a escrita, com o valor intacto, em cada campo da família.
#[test]
fn clicar_na_pega_abre_a_escrita_em_todo_campo() {
    let mut m = Mesa::nova();
    let ids = m.com_pega();
    assert!(ids.len() >= 7, "a varredura viu só {} campos", ids.len());
    for id in ids {
        let antes = m.valor(id);
        let p = m.pega(id);
        m.toque(p, NOVO_GESTO_NS);
        assert_eq!(m.host.store().focus_id(), Some(id), "{id:?} sem foco");
        assert_eq!(m.valor(id), antes, "{id:?}: o clique mudou o valor");
        m.host.store_mut().set_focus(None);
        m.quadro();
    }
}

/// Duplo clique no ícone seguido de escrever: o que se escreve fica, com e sem valor de fábrica.
#[test]
fn duplo_clique_na_pega_nao_perde_o_que_se_escreve() {
    for (id, escrito, esperado) in [
        (INSP_TRANSFORM_SCALE_X, "4.5", 4.5),
        (INSP_TRANSFORM_POS_X, "55", 55.0),
        (INSP_TRANSFORM_ROT, "66", 66.0),
        (INSP_TRANSFORM_SKEW_X, "12", 12.0),
    ] {
        let mut m = Mesa::nova();
        let p = m.pega(id);
        m.toque(p, NOVO_GESTO_NS);
        m.toque(p, ENTRE_TOQUES_NS);
        assert_eq!(m.host.store().focus_id(), Some(id), "{id:?} sem foco");
        m.escreve(id, escrito);
        assert_eq!(m.valor(id), esperado, "{id:?}");
    }
}

/// O duplo clique continua a repor o valor de fábrica (02 §3).
#[test]
fn duplo_clique_na_pega_continua_a_repor_a_fabrica() {
    let mut m = Mesa::nova();
    assert_eq!(m.valor(INSP_TRANSFORM_SCALE_X), 2.5);
    let p = m.pega(INSP_TRANSFORM_SCALE_X);
    m.toque(p, NOVO_GESTO_NS);
    m.toque(p, ENTRE_TOQUES_NS);
    assert_eq!(m.valor(INSP_TRANSFORM_SCALE_X), 1.0);
}

/// Arrastar a pega continua a ser scrub, sem abrir a escrita.
#[test]
fn arrastar_a_pega_nao_abre_a_escrita() {
    let mut m = Mesa::nova();
    let (x, y) = m.pega(INSP_TRANSFORM_POS_X);
    let antes = m.valor(INSP_TRANSFORM_POS_X);
    m.relogio += NOVO_GESTO_NS;
    for (kind, px) in [
        (PointerKind::Down, x),
        (PointerKind::Move, x + 20.0),
        (PointerKind::Move, x + 40.0),
        (PointerKind::Up, x + 40.0),
    ] {
        m.ponteiro(kind, px, y);
    }
    assert_ne!(m.valor(INSP_TRANSFORM_POS_X), antes, "o scrub não mexeu");
    assert_ne!(
        m.host.store().focus_id(),
        Some(INSP_TRANSFORM_POS_X),
        "o scrub abriu a escrita"
    );
}
