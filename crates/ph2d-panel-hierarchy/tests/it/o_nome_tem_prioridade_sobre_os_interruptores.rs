//! ⭐⭐ **O nome tem prioridade sobre os interruptores** (defeito do dono 11/11, Play na cena do jogo:
//! «Collisio…», «Ground L…»). Pintado pela porta real do painel, em cartões, com a coluna da esquerda
//! no PISO (`PANEL_MIN_W_PX`) — a largura que o dono tem. Controlo: o objecto escondido mostra o olho
//! fechado sem hover; a linha apontada devolve os três interruptores; o nome que nem assim cabe sai
//! com «…» e o balão da linha diz o nome inteiro.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_component_desc::ComponentCategory as C;
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::dock_seam::ChromeBands;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::screens::layout::{CenterSplit, DockSides, HeroLayout};
use ph2d_editor_core::text_elide::elisao::{self, Medido};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_panel_hierarchy::{HierarchyPanel, cards, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const LONGO: &str = "Collision Shape for the very long moving platform (12)";

/// A cena do jogo (`PH2D_GAME_SMOKE=10`): nome, recuo, componentes, à vista.
const CENA: &[(&str, u8, &[C], bool)] = &[
    ("Ground Left", 0, &[C::Physics], true),
    ("Collision Shape", 1, &[C::Physics], true),
    (
        "Coin 1",
        0,
        &[C::Animation, C::Physics, C::Logic, C::Audio],
        true,
    ),
    ("Collision Shape (6)", 1, &[C::Physics], true),
    ("Hills", 0, &[C::Animation], false),
    (LONGO, 0, &[C::Physics], true),
];

fn id_de(i: usize) -> NodeId {
    NodeId(100_000 + i as u64)
}

fn entidade(i: usize) -> HierarchyEntity {
    let (nome, indent, comps, visible) = CENA[i];
    HierarchyEntity {
        name: nome.into(),
        icon: IconId::Sprite,
        indent,
        badge: None,
        swatch: None,
        visible,
        selected: false,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
        role: RowRole::Object,
        components: comps.to_vec(),
        content: Vec::new(),
        has: Default::default(),
        entity: Some(0xC0DE_0000 + i as u64),
    }
}

struct Mesa {
    host: MockPanelHost,
    st: HierarchyState,
    layout: HeroLayout,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova() -> Self {
        cards::force_cards_on_this_thread(Some(true));
        let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
        let ordem: Vec<NodeId> = (0..CENA.len()).map(id_de).collect();
        let mapa: BTreeMap<_, _> = (0..CENA.len()).map(|i| (id_de(i), entidade(i))).collect();
        ph2d_panel_hierarchy::sync_from_hierarchy(host.store_mut(), &ordem, mapa);
        let bandas = ChromeBands {
            left_dock_w: ph2d_tokens::PANEL_MIN_W_PX,
            ..ChromeBands::DEFAULT
        };
        let layout =
            HeroLayout::for_viewport_bands(VIEW, false, bandas, CenterSplit::None, DockSides::BOTH);
        let mut mesa = Self {
            host,
            st: HierarchyState::default(),
            layout,
            rects: Vec::new(),
        };
        for _ in 0..30 {
            mesa.pinta();
        }
        mesa
    }

    fn pinta(&mut self) -> Vec<Medido> {
        let (h, st, lay) = (&mut self.host, &mut self.st, self.layout);
        let (rects, medidos) = elisao::medindo(|| {
            ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint_with_layout::<HierarchyPanel>(st, lay, VIEW),
            )
        });
        self.rects = rects;
        medidos
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0)
            .map(|(_, r)| *r)
    }

    /// A linha `row` passa a escolhida (sem rato em lado nenhum).
    fn escolhe(&mut self, row: NodeId) {
        let ordem: Vec<NodeId> = (0..CENA.len()).map(id_de).collect();
        let mut mapa: BTreeMap<_, _> = (0..CENA.len()).map(|i| (id_de(i), entidade(i))).collect();
        mapa.get_mut(&row).expect("linha").selected = true;
        ph2d_panel_hierarchy::sync_from_hierarchy(self.host.store_mut(), &ordem, mapa);
        for _ in 0..30 {
            let _ = self.pinta();
        }
    }

    fn aponta(&mut self, row: NodeId) {
        let r = self.rect(row).expect("linha pintada");
        let _ = self.host.dispatch_pointer_event(PointerEvent {
            kind: PointerKind::Move,
            x: r.x + r.w * 0.5,
            y: r.y + r.h * 0.5,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: 0,
        });
    }
}

fn nome<'a>(medidos: &'a [Medido], texto: &str) -> &'a Medido {
    medidos
        .iter()
        .rev()
        .find(|m| m.texto == texto)
        .unwrap_or_else(|| panic!("{texto:?} não foi pintado"))
}

#[test]
fn na_largura_do_dono_os_nomes_da_cena_do_jogo_saem_inteiros_sem_hover() {
    let mut mesa = Mesa::nova();
    let medidos = mesa.pinta();
    // Medido 11/11 no piso (220): com os interruptores e o ícone de componente o nome tinha 54 px
    // («Collisio…»); sem eles, 118 — «Collision Shape (6)» cabe.
    for i in [0, 1, 2, 3] {
        let m = nome(&medidos, CENA[i].0);
        eprintln!(
            "{:?}: orçamento {:.1} px → {:?}",
            m.texto, m.largura, m.pintado
        );
        assert!(
            m.coube(),
            "{:?} saiu {:?} em {:.1} px",
            m.texto,
            m.pintado,
            m.largura
        );
        assert!(
            mesa.rect(core_ids::hier_eye_companion(id_de(i))).is_none(),
            "o olho à vista da linha {i} pinta-se sem hover e come o nome"
        );
    }
    // ⛔ Controlo: o escondido mostra o olho fechado sempre, e é clicável.
    assert!(
        mesa.rect(core_ids::hier_eye_companion(id_de(4))).is_some(),
        "o olho FECHADO tem de ficar à vista sem hover"
    );
    // Smoke do dono 11/11: em repouso, nenhum ícone de componente — nem no escondido.
    for i in 0..CENA.len() {
        for c in CENA[i].2 {
            assert!(
                mesa.rect(ids::hier_component_icon(id_de(i), *c)).is_none(),
                "o ícone {c:?} da linha {:?} pinta-se sem hover",
                CENA[i].0
            );
        }
    }
}

/// Com o rato na Coin 1 voltam os quatro ícones de componente e os três interruptores, nas mesmas
/// posições que a linha escolhida tem (todos juntos, nada desliza).
#[test]
fn com_hover_os_icones_de_componente_voltam_todos_nas_posicoes_de_sempre() {
    let coin = id_de(2);
    let todos: Vec<NodeId> = CENA[2]
        .2
        .iter()
        .map(|c| ids::hier_component_icon(coin, *c))
        .chain([
            core_ids::hier_eye_companion(coin),
            ids::hier_lock_companion(coin),
            ids::hier_group_companion(coin),
        ])
        .collect();
    let mut mesa = Mesa::nova();
    mesa.aponta(coin);
    let _ = mesa.pinta();
    let com_hover: Vec<Option<Rect>> = todos.iter().map(|id| mesa.rect(*id)).collect();
    let mut escolhida = Mesa::nova();
    escolhida.escolhe(coin);
    let de_sempre: Vec<Option<Rect>> = todos.iter().map(|id| escolhida.rect(*id)).collect();
    assert!(
        com_hover.iter().all(Option::is_some),
        "faltou ícone com hover: {com_hover:?}"
    );
    assert_eq!(
        com_hover, de_sempre,
        "os ícones mudaram de sítio entre hover e escolha"
    );
}

#[test]
fn a_linha_apontada_devolve_os_tres_interruptores() {
    let mut mesa = Mesa::nova();
    let row = id_de(1);
    let antes = nome(&mesa.pinta(), CENA[1].0).largura;
    mesa.aponta(row);
    let medidos = mesa.pinta();
    for id in [
        core_ids::hier_eye_companion(row),
        ids::hier_lock_companion(row),
        ids::hier_group_companion(row),
    ] {
        assert!(mesa.rect(id).is_some(), "{id:?} não voltou com o hover");
    }
    let m = nome(&medidos, CENA[1].0);
    eprintln!(
        "{:?}: sem hover {antes:.1} px, com hover {:.1} px → {:?}",
        m.texto, m.largura, m.pintado
    );
    assert!(
        mesa.rect(core_ids::hier_eye_companion(id_de(0))).is_none(),
        "o hover de uma linha acendeu os interruptores de outra"
    );
}

#[test]
fn o_nome_que_nao_cabe_sai_com_reticencia_e_o_balao_diz_o_nome_inteiro() {
    let mut mesa = Mesa::nova();
    let row = id_de(5);
    let medidos = mesa.pinta();
    let m = nome(&medidos, LONGO);
    assert!(
        !m.coube() && m.pintado.ends_with('…'),
        "{:?} devia sair com «…»: {:?}",
        m.texto,
        m.pintado
    );
    let _ = mesa.pinta();
    assert_eq!(mesa.host.store().tooltip_for(row), Some(LONGO));
    assert_eq!(
        mesa.host.store().tooltip_for(id_de(0)),
        None,
        "nome inteiro não tem balão"
    );
}
