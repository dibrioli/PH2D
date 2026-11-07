//! ⭐ O modal «Collision Layers» (onda 8, dono 07/11), pelo gesto real: o ponteiro no rect pintado, o Esc da
//! tecla, o duplo clique. A janela é larga como a do app: as 32 camadas cabem sem rolar.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_physics::state::{PhysicsPanelState, PhysicsSnapshot};
use ph2d_panel_physics::{PhysicsIntent, PhysicsPanel, cards, drain_intents, ids, modal};
use ph2d_physics_ecs::{LayerMatrix, LayerNames, MAX_LAYERS, PhysicsSettings};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};

struct Mesa {
    host: MockPanelHost,
    st: PhysicsPanelState,
    rects: Vec<(NodeId, Rect)>,
    relogio: u64,
}

impl Mesa {
    fn nova(nomes: LayerNames, classica: bool) -> Self {
        cards::force_cards_on_this_thread(Some(!classica));
        ph2d_panel_physics::set_current_physics(Some(PhysicsSnapshot::default()));
        ph2d_panel_physics::set_layer_names(nomes);
        ph2d_panel_physics::set_visible_layers(1);
        modal::fecha();
        let _ = drain_intents();
        let mut m = Self {
            host: MockPanelHost::with_panel::<PhysicsPanel>(),
            st: PhysicsPanelState,
            rects: Vec::new(),
            relogio: 0,
        };
        m.host.set_collapsed(ids::PHYSICS_SEC_LAYERS, false);
        m.pinta();
        m
    }

    fn pinta(&mut self) {
        for _ in 0..30 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<PhysicsPanel>(st, JANELA),
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

    fn centro(&self, id: NodeId) -> (f32, f32) {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    fn aplica(&mut self, ev: Vec<WidgetEvent>) -> Vec<PhysicsIntent> {
        for e in ev {
            let _ = self
                .host
                .apply_panel_event::<PhysicsPanel>(&mut self.st, e);
        }
        drain_intents()
    }

    fn clica(&mut self, id: NodeId) -> Vec<PhysicsIntent> {
        let (x, y) = self.centro(id);
        assert_eq!(self.host.hit_at(x, y), Some(id), "outro id no centro");
        let ev = self.host.click_at(x, y);
        self.aplica(ev)
    }

    fn duplo(&mut self, id: NodeId) {
        let (x, y) = self.centro(id);
        // O ponteiro acerta no rótulo em ecrã; o despacho promove o 2.º clique a duplo (como os testes do Flip).
        assert_eq!(
            self.host.hit_at(x, y),
            Some(id),
            "outro id no rótulo: nome20 {:?} linha20 {:?} col2 {:?} linha2 {:?} card {:?} scrim {:?} edit {:?} rect {:?}",
            ids::PHYSICS_LAYER_NAME[20],
            ids::PHYSICS_LAYER_ROW_LABEL[20],
            ids::PHYSICS_LAYER_COL_LABEL[2],
            ids::PHYSICS_LAYER_ROW_LABEL[2],
            ids::PHYSICS_MATRIX_CARD,
            ids::PHYSICS_MATRIX_SCRIM,
            modal::editando(),
            self.rect(id)
        );
        let ev = vec![WidgetEvent::DoubleClick(id)];
        let _ = self.aplica(ev);
        self.pinta();
    }

    fn pairar(&mut self, x: f32, y: f32) {
        let _ = self.host.dispatch_pointer_event(PointerEvent {
            kind: PointerKind::Move,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 0.0,
            timestamp_ns: 0,
        });
    }
}

fn celula(a: usize, b: usize) -> NodeId {
    ids::PHYSICS_LAYER_CELL[ids::physics_layer_cell_index(a, b)]
}

fn mundo_de(intents: &[PhysicsIntent]) -> LayerMatrix {
    intents
        .iter()
        .rev()
        .find_map(|i| match i {
            PhysicsIntent::SetSettings(s, _) => Some(LayerMatrix::from_rows(s.layer_matrix)),
            _ => None,
        })
        .expect("a célula escreve")
}

/// Abre pelo painel; as 528 células e os 64 rótulos estão dentro da janela; a célula alta (25, 20) escreve
/// os dois lados do par e só ele; Esc fecha; e com o modal fechado nenhuma célula aceita clique.
#[test]
fn abre_pelo_painel_mostra_as_32_e_a_celula_alta_escreve_o_par() {
    let mut mesa = Mesa::nova(LayerNames::default(), false);
    assert!(
        mesa.rect(celula(0, 0)).is_none(),
        "a grelha saiu do painel"
    );
    let _ = mesa.clica(ids::PHYSICS_MATRIX_OPEN);
    mesa.pinta();
    assert!(modal::aberto());
    for a in 0..MAX_LAYERS {
        for b in 0..=a {
            let r = mesa
                .rect(celula(a, b))
                .unwrap_or_else(|| panic!("({a},{b}) não pintada"));
            assert!(
                r.x >= JANELA.x && r.y >= JANELA.y && r.x + r.w <= JANELA.w && r.y + r.h <= JANELA.h,
                "({a},{b}) fora da janela: {r:?}"
            );
        }
        assert!(mesa.rect(ids::PHYSICS_LAYER_ROW_LABEL[a]).is_some());
        assert!(mesa.rect(ids::PHYSICS_LAYER_COL_LABEL[a]).is_some());
    }
    let m = mundo_de(&mesa.clica(celula(25, 20)));
    assert!(!m.collides(25, 20) && !m.collides(20, 25), "os dois lados");
    for a in 0..MAX_LAYERS {
        for b in 0..MAX_LAYERS {
            if (a, b) != (25, 20) && (a, b) != (20, 25) {
                assert!(m.collides(a, b), "({a},{b}) não devia mudar");
            }
        }
    }
    let ev = mesa.host.esc_key();
    assert_eq!(ev, [WidgetEvent::Click(ids::PHYSICS_MATRIX_CLOSE)]);
    let _ = mesa.aplica(ev);
    mesa.pinta();
    assert!(!modal::aberto(), "Esc fecha");
    assert!(mesa.rect(celula(25, 20)).is_none());
}

/// O «Edit matrix…» do Inspector abre o modal com a linha da camada do objeto acesa, mesmo com o painel
/// fechado; o X e o clique fora fecham; o clique no cartão (fora de um controlo) não fecha.
#[test]
fn abre_pelo_inspector_com_a_linha_em_destaque_e_fecha_pelo_x_e_por_fora() {
    let mut mesa = Mesa::nova(LayerNames::default(), false);
    mesa.host.set_panel_visible("physics", false);
    ph2d_panel_physics::abre_a_matriz(&mut mesa.host, 20);
    assert_eq!(modal::linha_em_destaque(), Some(20));
    assert!(!mesa.host.panel_visible("physics"), "o painel não abre");
    mesa.pinta();
    assert!(mesa.rect(celula(20, 20)).is_some(), "a linha 20 à vista");
    let carta = mesa.rect(ids::PHYSICS_MATRIX_CARD).expect("o cartão");
    let ev = mesa.host.click_at(carta.x + 2.0, carta.y + carta.h - 2.0);
    let _ = mesa.aplica(ev);
    assert!(modal::aberto(), "o cartão engole o clique");
    let _ = mesa.clica(ids::PHYSICS_MATRIX_CLOSE);
    assert!(!modal::aberto(), "o X fecha");

    ph2d_panel_physics::abre_a_matriz(&mut mesa.host, 3);
    mesa.pinta();
    let ev = mesa.host.click_at(JANELA.x + 2.0, JANELA.y + 2.0);
    let _ = mesa.aplica(ev);
    assert!(!modal::aberto(), "fora fecha");
    assert_eq!(modal::linha_em_destaque(), None);
}

/// Pairar sobre a célula (Props × Ground) em ecrã põe-na sob o ponteiro: o pintor acende a linha e a coluna dela;
/// a dica diz os dois nomes por extenso; sem nome, «Layer n».
#[test]
fn pairar_e_a_dica_dizem_os_dois_nomes() {
    let mut nomes = LayerNames::default();
    nomes.0[1] = "Ground".into();
    nomes.0[3] = "Props".into();
    let mut mesa = Mesa::nova(nomes, false);
    modal::abre(None);
    mesa.pinta();
    let (x, y) = mesa.centro(celula(3, 1));
    mesa.pairar(x, y);
    mesa.pinta();
    assert_eq!(
        mesa.host.store().button_state(celula(3, 1)),
        Some(ph2d_editor_core::widget::ButtonState::Hovered)
    );
    assert_eq!(
        mesa.host.store().tooltip_for(celula(3, 1)),
        Some("Props × Ground: click or drag to switch their collision")
    );
    assert_eq!(
        mesa.host.store().tooltip_for(celula(25, 20)),
        Some("Layer 25 × Layer 20: click or drag to switch their collision")
    );
    modal::fecha();
}

/// Duplo clique no rótulo da esquerda abre o campo; Enter grava UM pedido; Esc desiste sem pedido; vazio volta a
/// «Layer n» (o nome gravado é vazio, nunca «Layer n»). O rótulo de cima faz o mesmo.
#[test]
fn duplo_clique_renomeia_nos_dois_rotulos_enter_esc_e_vazio() {
    let mut nomes = LayerNames::default();
    nomes.0[2] = "Ground".into();
    let mut mesa = Mesa::nova(nomes, false);
    modal::abre(None);
    mesa.pinta();
    let _ = mesa.host.drained_actions();

    mesa.duplo(ids::PHYSICS_LAYER_ROW_LABEL[20]);
    assert_eq!(modal::editando(), Some(20));
    let campo = ids::PHYSICS_LAYER_NAME[20];
    assert_eq!(mesa.host.store().focus_id(), Some(campo));
    mesa.host.set_text(campo, "Enemy");
    let _ = mesa.aplica(vec![WidgetEvent::TextChanged(campo)]);
    assert!(mesa.host.drained_actions().is_empty(), "escrever não grava");
    let ev = mesa
        .host
        .key_down(ph2d_editor_core::interaction::dispatch::keymap::KEY_ENTER);
    let _ = mesa.aplica(ev);
    assert_eq!(
        mesa.host.drained_actions(),
        [EditorAction::LayerRename {
            layer: 20,
            name: "Enemy".into()
        }],
        "Enter = um passo"
    );
    assert_eq!(modal::editando(), None);
    assert!(modal::aberto(), "Enter não fecha o modal");

    // Esc desiste: nenhum pedido, e o modal continua aberto (o 1.º Esc é do campo).
    mesa.duplo(ids::PHYSICS_LAYER_COL_LABEL[2]);
    assert_eq!(modal::editando(), Some(2));
    assert_eq!(
        mesa.host.store().text(ids::PHYSICS_LAYER_NAME[2]),
        Some("Ground"),
        "o campo abre com o nome dado"
    );
    mesa.host.set_text(ids::PHYSICS_LAYER_NAME[2], "Mud");
    let ev = mesa.host.esc_key();
    let _ = mesa.aplica(ev);
    assert!(mesa.host.drained_actions().is_empty(), "Esc nunca grava");
    assert!(modal::aberto() && modal::editando().is_none());
    mesa.pinta();

    // Vazio: o nome volta a «Layer 2» — o pedido grava vazio.
    mesa.duplo(ids::PHYSICS_LAYER_ROW_LABEL[2]);
    mesa.host.set_text(ids::PHYSICS_LAYER_NAME[2], "   ");
    let ev = mesa
        .host
        .key_down(ph2d_editor_core::interaction::dispatch::keymap::KEY_ENTER);
    let _ = mesa.aplica(ev);
    assert_eq!(
        mesa.host.drained_actions(),
        [EditorAction::LayerRename {
            layer: 2,
            name: String::new()
        }]
    );
    let mut sem = LayerNames::default();
    assert_eq!(modal::rotulo(&sem, 20), "Layer 20");
    sem.0[20] = "Enemy".into();
    assert_eq!(modal::rotulo(&sem, 20), "Enemy", "o nome do artista vence");
    modal::fecha();
}

/// Um projeto sem nomes mostra «Layer n» e não tem bytes de nomes para gravar.
#[test]
fn projeto_sem_nomes_mostra_layer_n_e_nao_grava_nomes() {
    let sem = LayerNames::default();
    for n in 0..MAX_LAYERS {
        assert_eq!(modal::rotulo(&sem, n), format!("Layer {n}"));
    }
    assert!(sem.to_bytes().is_empty(), "o «Layer n» nunca vai ao ficheiro");
}

/// A face clássica (`PH2D_CARDS=0`) mostra o resumo e o mesmo «Edit matrix…», que abre o MESMO modal.
#[test]
fn a_face_classica_abre_o_mesmo_modal() {
    let mut mesa = Mesa::nova(LayerNames::default(), true);
    assert!(mesa.rect(celula(0, 0)).is_none(), "sem grelha no painel");
    let _ = mesa.clica(ids::PHYSICS_MATRIX_OPEN);
    mesa.pinta();
    assert!(modal::aberto());
    assert!(mesa.rect(celula(31, 31)).is_some());
    let _ = mesa.aplica(vec![WidgetEvent::Click(ids::PHYSICS_MATRIX_CLOSE)]);
    cards::force_cards_on_this_thread(None);
}

/// O arrasto que pinta atravessa a linha 7 no modal: cada célula cruzada muda, e só ela.
#[test]
fn arrastar_no_modal_pinta_cada_celula_cruzada() {
    let mut mesa = Mesa::nova(LayerNames::default(), false);
    modal::abre(None);
    mesa.pinta();
    let (x0, y0) = mesa.centro(celula(7, 0));
    let (x1, y1) = mesa.centro(celula(7, 7));
    let ev = mesa.host.drag_at(x0, y0, x1, y1);
    let m = mundo_de(&mesa.aplica(ev));
    for j in 0..8 {
        assert!(!m.collides(7, j), "o par (7,{j}) ficou por pintar");
    }
    assert!(m.collides(6, 0), "fora do caminho não muda");
    let _ = PhysicsSettings::default();
    modal::fecha();
}

/// O losango da matriz (canal MUNDO) vive no canto dos rótulos do modal, com as células por campos.
#[test]
fn o_losango_da_matriz_mora_no_modal() {
    use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_target, set_key_marks};
    set_key_marks(std::collections::BTreeMap::from([(
        celula(0, 0),
        KeyMark::Chave,
    )]));
    let mut mesa = Mesa::nova(LayerNames::default(), false);
    modal::abre(None);
    mesa.pinta();
    let tem = mesa
        .rects
        .iter()
        .filter_map(|(alvo, _)| key_mark_target(*alvo))
        .any(|c| c.fields.contains(&celula(0, 0)));
    set_key_marks(std::collections::BTreeMap::new());
    modal::fecha();
    assert!(tem, "a matriz perdeu a marca de animação no modal");
}
