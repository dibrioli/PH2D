//! Smoke do dono 09/11 (`PH2D_GAME_SMOKE=5`): «Two Sheets» e «Aligned Sheets» — Animated Sprites com
//! ícones de componente e o grupo de clips, as últimas linhas de uma lista longa e rolada, repintada a
//! cada quadro — seleccionam-se com UM clique, por cima do nome, com um quadro entre Down e Up.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_component_desc::ComponentCategory;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest, SelectModifier};
use ph2d_editor_core::hier_rows::{ContentGroup, ContentItem, ContentOpens, RowRole};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::PANEL_TIMELINE;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_panel_hierarchy::{HierarchyPanel, cards};
use ph2d_ui_testkit::MockPanelHost;

const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const PAINEL: NodeId = core_ids::HIER_PANEL;
const N: usize = 40;

fn id_de(n: usize) -> NodeId {
    NodeId(100_000 + n as u64)
}

fn clips() -> ContentGroup {
    ContentGroup {
        label_key: "hier_rows.content.clips",
        icon: IconId::Film,
        item_icon: IconId::Layer,
        opens: ContentOpens::Panel(PANEL_TIMELINE),
        items: [(1, "walk"), (2, "run")]
            .iter()
            .map(|(key, name)| ContentItem {
                key: *key,
                name: (*name).into(),
                depth: 0,
                active: false,
            })
            .collect(),
    }
}

fn linha(n: usize, selected: bool) -> HierarchyEntity {
    let animado = n >= N - 2;
    HierarchyEntity {
        name: match n {
            x if x == N - 2 => "Two Sheets".into(),
            x if x == N - 1 => "Aligned Sheets".into(),
            _ => format!("Object {n}"),
        },
        icon: IconId::Sprite,
        indent: 0,
        badge: None,
        swatch: None,
        visible: true,
        selected,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
        role: RowRole::Object,
        components: if animado {
            vec![
                ComponentCategory::Image,
                ComponentCategory::Animation,
                ComponentCategory::Physics,
            ]
        } else {
            Vec::new()
        },
        content: if animado { vec![clips()] } else { Vec::new() },
        has: Default::default(),
        entity: Some(0xBEEF_0000 + n as u64),
    }
}

struct Mesa {
    host: MockPanelHost,
    st: HierarchyState,
    rects: Vec<(NodeId, Rect)>,
    t: u128,
    sel: Option<usize>,
    /// Quanto a mão anda (em x) entre Down e Up.
    tremor: f32,
    /// A caneta que pousa sem se aproximar: o Down chega sem um Move nem um quadro antes.
    sem_aproximar: bool,
}

impl Mesa {
    fn nova(cards_on: bool, sel: Option<usize>) -> Self {
        cards::force_cards_on_this_thread(Some(cards_on));
        let mut m = Self {
            host: MockPanelHost::with_panel::<HierarchyPanel>(),
            st: HierarchyState::default(),
            rects: Vec::new(),
            t: 1_000_000_000,
            sel,
            tremor: 0.0,
            sem_aproximar: false,
        };
        m.host.set_panel_visible(PANEL_TIMELINE, false);
        for _ in 0..10 {
            m.quadro();
        }
        m
    }

    /// Um quadro com a animação a tocar: a shell volta a sincronizar as mesmas linhas.
    fn quadro(&mut self) {
        let ordem: Vec<NodeId> = (0..N).map(id_de).collect();
        let mapa: BTreeMap<_, _> = (0..N)
            .map(|i| (id_de(i), linha(i, self.sel == Some(i))))
            .collect();
        ph2d_panel_hierarchy::sync_from_hierarchy(self.host.store_mut(), &ordem, mapa);
        let (h, st) = (&mut self.host, &mut self.st);
        self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || h.paint::<HierarchyPanel>(st, VIEW),
        );
        self.t += 16_000_000;
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    fn ponteiro(&mut self, kind: PointerKind, (x, y): (f32, f32)) -> Vec<WidgetEvent> {
        self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: self.t,
        })
    }

    fn rola_ao_fundo(&mut self) {
        let l = self.host.store().panel_rect(PAINEL).expect("lista");
        for _ in 0..40 {
            let _ = self.host.wheel_at(l.x + l.w * 0.5, l.y + l.h * 0.5, -120.0);
            self.quadro();
        }
    }

    /// Clique real: move, quadro, Down, `quadros` quadros com `hold_ns` no total, Up.
    fn clique(
        &mut self,
        p: (f32, f32),
        quadros: usize,
        hold_ns: u128,
    ) -> (Vec<WidgetEvent>, Vec<EditorAction>) {
        let mut ev = Vec::new();
        if !self.sem_aproximar {
            ev.extend(self.ponteiro(PointerKind::Move, p));
            self.quadro();
        }
        ev.extend(self.ponteiro(PointerKind::Down, p));
        let t0 = self.t;
        for _ in 0..quadros {
            self.quadro();
        }
        self.t = t0 + hold_ns;
        let q = (p.0 + self.tremor, p.1);
        if self.tremor != 0.0 {
            ev.extend(self.ponteiro(PointerKind::Move, q));
        }
        ev.extend(self.ponteiro(PointerKind::Up, q));
        for e in ev.clone() {
            let _ = self
                .host
                .apply_panel_event::<HierarchyPanel>(&mut self.st, e);
        }
        let acts = self.host.drained_actions();
        self.quadro();
        (ev, acts)
    }
}

fn seleciona(row: NodeId) -> EditorAction {
    EditorAction::Hierarchy(HierRequest::SelectRow {
        row,
        modifier: SelectModifier::Replace,
    })
}

/// O ponto do NOME: a fracção `fx` da largura que fica fora dos ícones/galo.
fn um_clique(cards_on: bool, sel: Option<usize>, alvo: usize, fx: f32, hold_ns: u128) {
    um_clique_com_tremor(cards_on, sel, alvo, fx, hold_ns, 0.0);
}

fn um_clique_com_tremor(
    cards_on: bool,
    sel: Option<usize>,
    alvo: usize,
    fx: f32,
    hold_ns: u128,
    tremor: f32,
) {
    let mut m = Mesa::nova(cards_on, sel);
    m.tremor = tremor;
    m.rola_ao_fundo();
    let r = m
        .rect(id_de(alvo))
        .unwrap_or_else(|| panic!("linha {alvo} não pintada"));
    let p = (r.x + r.w * fx, r.y + r.h * 0.5);
    let hit = m.host.hit_at(p.0, p.1);
    let (ev, acts) = m.clique(p, 3, hold_ns);
    assert!(
        acts.contains(&seleciona(id_de(alvo))),
        "UM clique em {alvo} (cards {cards_on}, sel {sel:?}, fx {fx}, hold {hold_ns}) não \
         seleccionou; hit {hit:?}, linha {r:?}: eventos {ev:?} · acções {acts:?}"
    );
}

#[test]
fn um_clique_no_nome_de_um_animated_sprite_no_fundo_da_lista() {
    for cards_on in [true, false] {
        for sel in [None, Some(0)] {
            for alvo in [N - 2, N - 1] {
                for fx in [0.2, 0.35, 0.5] {
                    um_clique(cards_on, sel, alvo, fx, 120_000_000);
                }
            }
        }
    }
}

/// ⚠️ A pressão que dura mais que `LONG_PRESS_THRESHOLD_NS` (400 ms, carimbada pela SHELL com
/// `SystemTime::now()` ao PROCESSAR o evento) vira `LongPress` e o clique é suprimido — e ninguém
/// consome `LongPress` desde que o renomear passou ao duplo clique.
#[test]
fn uma_pressao_de_450_ms_ainda_seleciona() {
    um_clique(true, None, N - 1, 0.35, 450_000_000);
}

/// ⚠️ Uma mão (ou caneta) que treme 6 px entre Down e Up activa o arrasto de reparentar
/// (`update_hierarchy_drag`, limiar 5 px), o Up cai em cima da própria linha (`over_self`) e
/// `dispatch_up` devolve ANTES do `apply_click`.
#[test]
fn um_clique_com_6_px_de_tremor_ainda_seleciona() {
    um_clique_com_tremor(true, None, N - 1, 0.35, 120_000_000, 6.0);
}

/// ⚠️ A caneta (e o primeiro clique vindo de outro painel) pousa SEM aproximar: o Down chega sem Move nem
/// quadro antes — a linha ainda não está apontada (ícones à direita só com hover, 11/11).
#[test]
fn um_clique_sem_aproximar_ainda_seleciona() {
    for cards_on in [true, false] {
        for fx in [0.2, 0.35, 0.5] {
            let mut m = Mesa::nova(cards_on, None);
            m.sem_aproximar = true;
            let r = m.rect(id_de(1)).expect("linha 1");
            let p = (r.x + r.w * fx, r.y + r.h * 0.5);
            let (ev, acts) = m.clique(p, 3, 120_000_000);
            assert!(
                acts.contains(&seleciona(id_de(1))),
                "cards {cards_on} fx {fx}: {ev:?} · {acts:?}"
            );
        }
    }
}

/// O centro do ícone de TIPO da linha `n` (o glifo à esquerda do nome), como foi pintado.
fn centro_do_icone(m: &Mesa, n: usize) -> (f32, f32) {
    let r = m
        .rect(ph2d_panel_hierarchy::ids::hier_icon_companion(id_de(n)))
        .unwrap_or_else(|| panic!("ícone da linha {n} não pintado"));
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐ Smoke do dono 13/11: «clicar uma vez no ícone deve selecionar» — o mesmo pedido do nome.
#[test]
fn um_clique_no_icone_seleciona() {
    for cards_on in [true, false] {
        for (rolar, alvo) in [(false, 1), (true, N - 1)] {
            let mut m = Mesa::nova(cards_on, None);
            if rolar {
                m.rola_ao_fundo();
            }
            let p = centro_do_icone(&m, alvo);
            let (ev, acts) = m.clique(p, 3, 120_000_000);
            assert!(
                acts.contains(&seleciona(id_de(alvo))),
                "cards {cards_on} linha {alvo}: um clique no ícone não seleccionou: {ev:?} · {acts:?}"
            );
        }
    }
}

/// ⭐ Smoke do dono 13/11: «arrastar através do ícone deve ser possível» — o arrasto da LINHA (o mesmo
/// do nome: reparentar dentro do painel; fora dele, a shell arma a carga `Object` por `hierarchy_row_grip`).
#[test]
fn arrastar_pelo_icone_arrasta_o_objeto() {
    for cards_on in [true, false] {
        let mut m = Mesa::nova(cards_on, None);
        let p = centro_do_icone(&m, 1);
        let destino = m.rect(id_de(5)).expect("linha 5");
        let icone = ph2d_panel_hierarchy::ids::hier_icon_companion(id_de(1));
        assert_eq!(m.host.store().hierarchy_row_grip(icone), Some(id_de(1)));
        let _ = m.ponteiro(PointerKind::Move, p);
        m.quadro();
        let mut ev = m.ponteiro(PointerKind::Down, p);
        let fim = (p.0, destino.y + destino.h * 0.5);
        for i in 1..=6 {
            let y = p.1 + (fim.1 - p.1) * i as f32 / 6.0;
            ev.extend(m.ponteiro(PointerKind::Move, (p.0, y)));
            m.quadro();
        }
        let drag = m.host.store().hierarchy_drag();
        assert!(
            drag.is_some_and(|d| d.dragged == id_de(1) && d.active),
            "cards {cards_on}: o arrasto pelo ícone não pegou a linha: {drag:?}"
        );
        ev.extend(m.ponteiro(PointerKind::Up, fim));
        assert!(
            ev.iter().any(|e| matches!(
                e,
                WidgetEvent::HierReparent { dragged, .. } if *dragged == id_de(1)
            )),
            "cards {cards_on}: soltar sobre a linha 5 não reparentou: {ev:?}"
        );
    }
}

/// O duplo clique no ícone continua a enquadrar o objecto (2026-05-26) — e não abre o renomear.
#[test]
fn duplo_clique_no_icone_ainda_enquadra() {
    for cards_on in [true, false] {
        let mut m = Mesa::nova(cards_on, None);
        let p = centro_do_icone(&m, 1);
        let (_, mut acts) = m.clique(p, 1, 50_000_000);
        let (ev, segundo) = m.clique(p, 1, 50_000_000);
        acts.extend(segundo);
        let foco = EditorAction::SetViewFocus {
            kind: ph2d_editor_core::screens::hero::ViewFocusKind::Selected,
        };
        assert!(
            acts.contains(&foco) && acts.contains(&seleciona(id_de(1))),
            "cards {cards_on}: o duplo clique no ícone não enquadrou: {ev:?} · {acts:?}"
        );
        assert!(
            !acts
                .iter()
                .any(|a| matches!(a, EditorAction::Hierarchy(HierRequest::RenameSeed { .. }))),
            "cards {cards_on}: o duplo clique no ícone abriu o renomear: {acts:?}"
        );
    }
}
