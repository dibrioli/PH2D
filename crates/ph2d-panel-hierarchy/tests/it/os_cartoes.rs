//! ⭐ **A Hierarquia em cartões: a árvore continua a mesma.** Cada gesto pelo ponteiro REAL, nos rects
//! que a pintura registou; `cards_off` prova que `PH2D_CARDS=0` é o painel de antes. O custo por quadro
//! é o `#[ignore]` no fim, medido com as duas pinturas intercaladas no mesmo processo.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_component_desc::ComponentCategory;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest, SelectModifier};
use ph2d_editor_core::hier_rows::{ContentGroup, ContentItem, ContentOpens, RowRole};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_panel_hierarchy::{HierarchyPanel, cards, ids};
use ph2d_tokens::ROW_H_PX;
use ph2d_ui_testkit::MockPanelHost;

const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const PAINEL: NodeId = core_ids::HIER_PANEL;

fn linha(n: usize, indent: u8) -> HierarchyEntity {
    HierarchyEntity {
        name: format!("Object {n}"),
        icon: IconId::Sprite,
        indent,
        badge: None,
        swatch: None,
        visible: true,
        selected: false,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
        role: RowRole::Object,
        components: if n == 0 {
            vec![ComponentCategory::Physics]
        } else {
            Vec::new()
        },
        content: Vec::new(),
        has: Default::default(),
        entity: Some(0xBEEF_0000 + n as u64),
    }
}

fn id_de(n: usize) -> NodeId {
    NodeId(100_000 + n as u64)
}

struct Mesa {
    host: MockPanelHost,
    st: HierarchyState,
    rects: Vec<(NodeId, Rect)>,
    /// O relógio dos cliques: o `click_at` do arnês demora o bastante para uma linha ler «long press».
    t: u128,
}

impl Mesa {
    /// `n` objectos soltos, o 2.º filho do 1.º.
    fn nova(n: usize) -> Self {
        Self::com(n, |_| {})
    }

    fn com(n: usize, muda: impl Fn(&mut BTreeMap<NodeId, HierarchyEntity>)) -> Self {
        cards::force_cards_on_this_thread(Some(true));
        let mut mesa = Self {
            host: MockPanelHost::with_panel::<HierarchyPanel>(),
            st: HierarchyState::default(),
            rects: Vec::new(),
            t: 1_000_000_000,
        };
        mesa.semeia(n, muda);
        mesa.assenta();
        mesa
    }

    fn semeia(&mut self, n: usize, muda: impl Fn(&mut BTreeMap<NodeId, HierarchyEntity>)) {
        let ordem: Vec<NodeId> = (0..n).map(id_de).collect();
        let mut mapa: BTreeMap<_, _> = (0..n)
            .map(|i| (id_de(i), linha(i, u8::from(i == 1))))
            .collect();
        muda(&mut mapa);
        ph2d_panel_hierarchy::sync_from_hierarchy(self.host.store_mut(), &ordem, mapa);
    }

    fn pinta(&mut self) {
        let (h, st) = (&mut self.host, &mut self.st);
        self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || h.paint::<HierarchyPanel>(st, VIEW),
        );
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            self.pinta();
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

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<EditorAction> {
        for e in eventos {
            let _ = self
                .host
                .apply_panel_event::<HierarchyPanel>(&mut self.st, e);
        }
        self.host.drained_actions()
    }

    /// O clique pelo dispatcher, com a contraprova de que é este id que está por cima.
    fn clica(&mut self, id: NodeId) -> Vec<EditorAction> {
        let (x, y) = self.centro(id);
        assert_eq!(self.host.hit_at(x, y), Some(id), "outro id por cima");
        self.t += 2_000_000_000;
        let mut ev = self.ponteiro_em(PointerKind::Down, (x, y), self.t);
        ev.extend(self.ponteiro_em(PointerKind::Up, (x, y), self.t + 1_000_000));
        assert!(
            ev.contains(&WidgetEvent::Click(id)),
            "{id:?} morto sob o dedo: {ev:?}"
        );
        self.aplica(ev)
    }

    fn ponteiro(&mut self, kind: PointerKind, p: (f32, f32)) -> Vec<WidgetEvent> {
        self.ponteiro_em(kind, p, 0)
    }

    fn ponteiro_em(&mut self, kind: PointerKind, (x, y): (f32, f32), t: u128) -> Vec<WidgetEvent> {
        self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: t,
        })
    }

    fn lista(&self) -> Rect {
        self.host
            .store()
            .panel_rect(PAINEL)
            .expect("lista publicada")
    }

    fn linhas_pintadas(&self) -> usize {
        self.rects
            .iter()
            .filter(|(id, _)| id.0 >= 100_000 && id.0 < 200_000)
            .count()
    }
}

fn seleciona(row: NodeId, modifier: SelectModifier) -> EditorAction {
    EditorAction::Hierarchy(HierRequest::SelectRow { row, modifier })
}

/// A árvore tem a densidade de uma árvore, não a altura de um campo, e cabe no cartão.
#[test]
fn as_linhas_tem_a_densidade_de_uma_arvore_e_tocam_se() {
    let mesa = Mesa::nova(4);
    let (a, b) = (
        mesa.rect(id_de(0)).expect("linha 0"),
        mesa.rect(id_de(2)).expect("linha 2"),
    );
    assert_eq!(a.h, ROW_H_PX);
    assert!(a.h < m::FIELD_H, "a linha ficou com a altura de um campo");
    assert!(b.y > a.y && a.x == b.x && a.w == b.w);
    let l = mesa.lista();
    assert!(
        a.x >= l.x && a.x + a.w <= l.x + l.w,
        "{a:?} sai da lista {l:?}"
    );
}

/// A lista flutua: publica o rect dela; o clique dentro é dela, o clique abaixo vai ao canvas.
#[test]
fn a_lista_flutua_e_o_clique_abaixo_dela_vai_ao_canvas() {
    let mesa = Mesa::nova(4);
    let store = mesa.host.store();
    let l = mesa.lista();
    assert!(store.is_floating_panel(PAINEL));
    assert!(
        l.h < VIEW.h * 0.5,
        "com 5 linhas a lista abraça o conteúdo: {l:?}"
    );
    assert_eq!(store.panel_at(l.x + 1.0, l.y + l.h - 1.0), Some(PAINEL));
    assert_eq!(store.panel_at(l.x + 1.0, l.y + l.h + 20.0), None);
    assert_eq!(store.panel_at(l.x + l.w + 3.0, l.y + 10.0), None);
}

/// Clique, Shift e Ctrl: o despacho é o de antes, com os mesmos ids.
#[test]
fn clique_ctrl_e_shift_selecionam_pela_linha() {
    let mut mesa = Mesa::nova(4);
    let got = mesa.clica(id_de(2));
    assert!(
        got.contains(&seleciona(id_de(2), SelectModifier::Replace)),
        "{got:?}"
    );
    mesa.host.store_mut().set_cmd_held(true);
    let got = mesa.clica(id_de(3));
    assert!(
        got.contains(&seleciona(id_de(3), SelectModifier::Toggle)),
        "{got:?}"
    );
    mesa.host.store_mut().set_cmd_held(false);
    mesa.host.store_mut().set_shift_held(true);
    let got = mesa.clica(id_de(0));
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::RangeSelect {
            row: id_de(0)
        })),
        "{got:?}"
    );
}

/// O «+» no cabeçalho da lista, o olho, o cadeado e o grupo: cada um o seu pedido.
#[test]
fn mais_olho_cadeado_e_grupo_respondem_nos_rects_pintados() {
    let mut mesa = Mesa::nova(4);
    let row = id_de(2);
    let got = mesa.clica(core_ids::HIERARCHY_ADD);
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::AddRoot)),
        "{got:?}"
    );
    let got = mesa.clica(core_ids::hier_eye_companion(row));
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::ToggleVisibility {
            row
        })),
        "{got:?}"
    );
    mesa.pinta();
    let got = mesa.clica(ids::hier_lock_companion(row));
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::ToggleLock { row })),
        "{got:?}"
    );
    mesa.pinta();
    let got = mesa.clica(ids::hier_group_companion(row));
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::ToggleGroup { row })),
        "{got:?}"
    );
}

/// O galo recolhe: o filho deixa de se pintar, e volta.
#[test]
fn o_galo_recolhe_e_expande() {
    let mut mesa = Mesa::nova(4);
    assert!(mesa.rect(id_de(1)).is_some(), "o filho nasce à vista");
    let galo = core_ids::hier_expand_companion(id_de(0));
    let _ = mesa.clica(galo);
    mesa.assenta();
    assert!(mesa.rect(id_de(1)).is_none(), "recolher não tirou o filho");
    let _ = mesa.clica(galo);
    mesa.assenta();
    assert!(
        mesa.rect(id_de(1)).is_some(),
        "expandir não devolveu o filho"
    );
}

/// Duplo clique no nome abre o campo POR CIMA do nome, dentro da linha e antes dos ícones.
#[test]
fn renomear_abre_a_caixa_por_cima_do_nome() {
    let mut mesa = Mesa::nova(4);
    let row = id_de(2);
    mesa.st.rename_target_row = Some(row);
    mesa.pinta();
    let caixa = mesa
        .rect(core_ids::HIER_RENAME_INPUT)
        .expect("sem caixa de renomear");
    let r = mesa.rect(row).expect("a linha");
    let cadeado = mesa.rect(ids::hier_lock_companion(row)).expect("cadeado");
    assert!(
        caixa.y >= r.y && caixa.y + caixa.h <= r.y + r.h,
        "{caixa:?} fora de {r:?}"
    );
    let grupo = mesa.rect(ids::hier_group_companion(row)).expect("grupo");
    assert!(
        caixa.x + caixa.w <= cadeado.x.min(grupo.x) + 0.5,
        "tapa os ícones: {caixa:?}"
    );
    let (x, y) = mesa.centro(core_ids::HIER_RENAME_INPUT);
    assert_eq!(mesa.host.hit_at(x, y), Some(core_ids::HIER_RENAME_INPUT));
}

/// A linha voa, a marca aparece, e largar é o `HierReparent` de sempre; fora da lista não reparenta.
#[test]
fn arrastar_reparente_pela_mao_e_fora_da_lista_nao() {
    let mut mesa = Mesa::nova(5);
    let (de, para) = (mesa.centro(id_de(3)), mesa.rect(id_de(2)).expect("alvo"));
    let parado = mesa
        .host
        .paint_and_count_geometry::<HierarchyPanel>(&mut mesa.st, VIEW);
    let _ = mesa.ponteiro(PointerKind::Down, de);
    let meio = (de.0, para.y + para.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Move, meio);
    let voando = mesa
        .host
        .paint_and_count_geometry::<HierarchyPanel>(&mut mesa.st, VIEW);
    assert!(mesa.host.store().hierarchy_drag().is_some_and(|d| d.active));
    assert!(
        voando.0 > parado.0,
        "nem voo nem marca: {voando:?} contra {parado:?}"
    );
    let ev = mesa.ponteiro(PointerKind::Up, meio);
    assert!(
        ev.iter().any(
            |e| matches!(e, WidgetEvent::HierReparent { dragged, new_parent, .. }
            if *dragged == id_de(3) && *new_parent == Some(id_de(2)))
        ),
        "largar no meio da linha entra dentro dela: {ev:?}"
    );

    let mut mesa = Mesa::nova(5);
    let de = mesa.centro(id_de(3));
    let _ = mesa.ponteiro(PointerKind::Down, de);
    let fora = (mesa.lista().x + mesa.lista().w + 40.0, de.1);
    let _ = mesa.ponteiro(PointerKind::Move, fora);
    let ev = mesa.ponteiro(PointerKind::Up, fora);
    assert!(
        !ev.iter()
            .any(|e| matches!(e, WidgetEvent::HierReparent { .. })),
        "largar no canvas reparentou: {ev:?}"
    );
}

/// ⭐ Com 5 000 linhas só se regista o que se vê; a roda rola; o que passa da lista não apanha o dedo.
#[test]
fn cinco_mil_linhas_pintam_so_o_que_se_ve_e_a_roda_rola() {
    let mut mesa = Mesa::nova(5000);
    let vistas = mesa.linhas_pintadas();
    let cabem = (VIEW.h / ROW_H_PX) as usize + 2;
    assert!(
        vistas > 10 && vistas <= cabem,
        "pintou {vistas} de 5000 (cabem {cabem})"
    );
    let l = mesa.lista();
    assert!(
        l.y + l.h <= VIEW.y + VIEW.h,
        "a lista passa do encaixe: {l:?}"
    );
    assert!(mesa.rect(id_de(10)).is_some() && mesa.rect(id_de(4000)).is_none());
    let (x, y) = (l.x + l.w * 0.5, l.y + l.h * 0.5);
    let _ = mesa.host.wheel_at(x, y, -2200.0);
    mesa.assenta();
    assert!(
        mesa.host.store().panel_scroll(PAINEL) > 2000.0,
        "a roda não rolou"
    );
    assert!(
        mesa.rect(id_de(10)).is_none(),
        "o que subiu continua a apanhar o dedo"
    );
    assert!(
        mesa.rect(id_de(110)).is_some(),
        "a linha 110 tinha de estar à vista"
    );
    assert!(mesa.linhas_pintadas() <= cabem);
}

/// A busca fica presa quando a árvore rola; o título e o «+» também.
#[test]
fn a_busca_e_o_mais_nao_rolam_com_a_arvore() {
    let mut mesa = Mesa::nova(600);
    let (busca, mais) = (
        mesa.rect(ids::HIER_SEARCH).expect("busca"),
        mesa.rect(core_ids::HIERARCHY_ADD).expect("mais"),
    );
    let l = mesa.lista();
    let _ = mesa
        .host
        .wheel_at(l.x + l.w * 0.5, l.y + l.h * 0.5, -3000.0);
    mesa.assenta();
    assert_eq!(mesa.rect(ids::HIER_SEARCH), Some(busca));
    assert_eq!(mesa.rect(core_ids::HIERARCHY_ADD), Some(mais));
    let (x, y) = (busca.x + 5.0, busca.y + 5.0);
    assert_eq!(mesa.host.hit_at(x, y), Some(ids::HIER_SEARCH));
    let abaixo = mesa
        .rects
        .iter()
        .filter(|(id, r)| id.0 >= 100_000 && id.0 < 200_000 && r.y < busca.y + busca.h)
        .count();
    let (hx, hy) = (busca.x + 1.0, busca.y + busca.h + 1.0);
    assert_ne!(
        mesa.host
            .hit_at(hx, hy)
            .map(|i| i.0 >= 100_000 && i.0 < 200_000),
        Some(true)
    );
    assert_eq!(abaixo, 0, "linhas por baixo da busca registadas");
}

/// A mão na borda de baixo rola a lista, e a linha da borda continua alvo.
#[test]
fn arrastar_na_borda_rola_a_lista() {
    let mut mesa = Mesa::nova(600);
    let de = mesa.centro(id_de(3));
    let _ = mesa.ponteiro(PointerKind::Down, de);
    let l = mesa.lista();
    let _ = mesa.ponteiro(PointerKind::Move, (de.0, l.y + l.h - 12.0));
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        mesa.pinta();
    }
    assert!(
        mesa.host.store().panel_scroll_target(PAINEL) > 0.0,
        "a mão na borda não rolou"
    );
}

/// ⭐ A selecção vinda do canvas traz a linha à vista; o clique numa linha à vista não mexe a lista.
#[test]
fn a_selecao_de_fora_rola_ate_a_linha_e_a_de_dentro_nao() {
    let mut mesa = Mesa::nova(600);
    assert!(mesa.rect(id_de(500)).is_none());
    mesa.semeia(600, |m| {
        if let Some(e) = m.get_mut(&id_de(500)) {
            e.selected = true;
        }
    });
    mesa.assenta();
    let r = mesa.rect(id_de(500)).expect("a selecção não veio à vista");
    let l = mesa.lista();
    assert!(
        r.y >= l.y && r.y + r.h <= l.y + l.h,
        "{r:?} fora da lista {l:?}"
    );
    let antes = mesa.host.store().panel_scroll_target(PAINEL);
    let vizinha = id_de(495);
    mesa.semeia(600, |m| {
        if let Some(e) = m.get_mut(&vizinha) {
            e.selected = true;
        }
    });
    mesa.assenta();
    assert_eq!(
        mesa.host.store().panel_scroll_target(PAINEL),
        antes,
        "a linha já à vista mexeu a lista"
    );
}

/// Canvas escolhe um objecto dentro de um ramo recolhido: o ramo abre. Recolher depois com a mesma
/// selecção fica recolhido. *Mutação: sem `abre_os_ramos` ⇒ a linha nunca aparece.*
#[test]
fn uma_selecao_em_ramo_recolhido_abre_o_ramo_uma_vez() {
    let mut mesa = Mesa::nova(5);
    let galo = core_ids::hier_expand_companion(id_de(0));
    let _ = mesa.clica(galo);
    mesa.assenta();
    assert!(mesa.rect(id_de(1)).is_none(), "o filho está recolhido");
    mesa.semeia(5, |m| {
        if let Some(e) = m.get_mut(&id_de(1)) {
            e.selected = true;
        }
    });
    mesa.assenta();
    assert!(mesa.rect(id_de(1)).is_some(), "o ramo não abriu");
    let _ = mesa.clica(galo);
    mesa.assenta();
    assert!(
        mesa.rect(id_de(1)).is_none(),
        "recolher à mão reabriu sozinho"
    );
}

/// Abaixo de uma lista curta, na coluna dela, largar é o fim da raiz; ao lado continua a ser o canvas.
/// *Mutação: a guarda de `hierarchy_drop_in_own_panel` fora ⇒ não há reparent.*
#[test]
fn largar_abaixo_de_uma_lista_curta_vai_ao_fim_da_raiz() {
    let mut mesa = Mesa::nova(5);
    let de = mesa.centro(id_de(1));
    let l = mesa.lista();
    let abaixo = (l.x + l.w * 0.5, l.y + l.h + 60.0);
    let _ = mesa.ponteiro(PointerKind::Down, de);
    let _ = mesa.ponteiro(PointerKind::Move, abaixo);
    assert!(mesa.host.store().hierarchy_drag_in_own_panel());
    let ev = mesa.ponteiro(PointerKind::Up, abaixo);
    assert!(
        ev.iter().any(
            |e| matches!(e, WidgetEvent::HierReparent { dragged, new_parent: None, .. }
            if *dragged == id_de(1))
        ),
        "largar abaixo da lista não foi ao fim da raiz: {ev:?}"
    );

    let mut mesa = Mesa::nova(5);
    let de = mesa.centro(id_de(1));
    let l = mesa.lista();
    let ao_lado = (l.x + l.w + 30.0, l.y + l.h + 60.0);
    let _ = mesa.ponteiro(PointerKind::Down, de);
    let _ = mesa.ponteiro(PointerKind::Move, ao_lado);
    let ev = mesa.ponteiro(PointerKind::Up, ao_lado);
    assert!(
        !ev.iter()
            .any(|e| matches!(e, WidgetEvent::HierReparent { .. })),
        "fora da coluna é o canvas: {ev:?}"
    );
}

fn textos(mesa: &mut Mesa) -> Vec<String> {
    let (h, st) = (&mut mesa.host, &mut mesa.st);
    h.medindo_a_pintura::<HierarchyPanel>(st, VIEW)
        .into_iter()
        .map(|m| m.texto)
        .collect()
}

/// Agarrar uma linha de uma multisselecção mostra todas no voo; acima do tecto, «+n more».
/// *Mutação: `leva_selecao` fora de `outras` ⇒ só a agarrada voa.*
#[test]
fn o_voo_de_uma_multisselecao_mostra_o_que_leva() {
    let escolhe = |quantas: usize, m: &mut BTreeMap<NodeId, HierarchyEntity>| {
        for i in 2..2 + quantas {
            if let Some(e) = m.get_mut(&id_de(i)) {
                e.selected = true;
            }
        }
    };
    let mut mesa = Mesa::com(12, |m| escolhe(3, m));
    let (de, para) = (mesa.centro(id_de(3)), mesa.centro(id_de(8)));
    let _ = mesa.ponteiro(PointerKind::Down, de);
    let _ = mesa.ponteiro(PointerKind::Move, para);
    let t = textos(&mut mesa);
    let vezes = |nome: &str| t.iter().filter(|x| x.as_str() == nome).count();
    assert_eq!(vezes("Object 3"), 2, "a agarrada: linha e voo. {t:?}");
    assert_eq!(vezes("Object 2"), 2, "as escolhidas voam com ela. {t:?}");
    assert_eq!(vezes("Object 4"), 2, "as escolhidas voam com ela. {t:?}");
    assert_eq!(vezes("Object 5"), 1, "a não escolhida fica. {t:?}");

    let mut mesa = Mesa::com(12, |m| escolhe(8, m));
    let (de, para) = (mesa.centro(id_de(3)), mesa.centro(id_de(11)));
    let _ = mesa.ponteiro(PointerKind::Down, de);
    let _ = mesa.ponteiro(PointerKind::Move, para);
    let t = textos(&mut mesa);
    assert!(t.iter().any(|x| x == "+4 more"), "{t:?}");
}

/// A cor de preenchimento como o desenho a grava.
fn palavra_da_cor(c: ph2d_vector::Color) -> u32 {
    let mut s = ph2d_vector::VectorScene::new();
    ph2d_editor_core::paint::fill_rounded_rect(&mut s, VIEW, 1.0, c);
    s.inner().encoding().draw_data[0]
}

/// A lista aberta do filtro tem a superfície das do cartão, não a do Inspector de antes.
/// *Mutação: sem o `card_ink::com` ⇒ sai `BgElev` e a moldura.*
#[test]
fn a_lista_do_filtro_tem_a_tinta_do_cartao() {
    use ph2d_editor_core::panel::PanelHost;
    use ph2d_editor_core::widget::card_ink::CardInk;
    use ph2d_tokens::ColorToken;
    let mut mesa = Mesa::nova(6);
    let theme = mesa.host.theme();
    let tinta = CardInk::of_card(None, theme);
    let do_cartao = palavra_da_cor(tinta.color(ColorToken::KanbanCardRaised).with_alpha(1.0));
    let antes = mesa
        .host
        .paint_draw_data::<HierarchyPanel>(&mut mesa.st, VIEW);
    let vezes = |d: &[u32]| d.iter().filter(|w| **w == do_cartao).count();
    let (x, y) = mesa.centro(ids::HIER_FILTER_DD);
    let ev = mesa.host.click_at(x, y);
    let _ = mesa.aplica(ev);
    mesa.pinta();
    let aberta = mesa
        .host
        .paint_draw_data::<HierarchyPanel>(&mut mesa.st, VIEW);
    assert!(
        vezes(&aberta) > vezes(&antes),
        "a superfície da lista não é a do cartão"
    );
    assert!(
        !aberta.contains(&palavra_da_cor(
            ph2d_editor_core::paint::resolve(ColorToken::BgElev, theme).with_alpha(1.0)
        )),
        "ainda tem a superfície do Inspector de antes"
    );
}

/// «1 entity · 1 component», nunca «1 entities». *Mutação: `{entities} entities` de volta.*
#[test]
fn um_objecto_fala_no_singular() {
    let mut mesa = Mesa::nova(1);
    let t = textos(&mut mesa);
    assert!(
        t.iter().any(|x| x.starts_with("1 entity")),
        "o contador: {t:?}"
    );
    let mut mesa = Mesa::nova(3);
    let t = textos(&mut mesa);
    assert!(
        t.iter().any(|x| x.starts_with("3 entities")),
        "o contador: {t:?}"
    );
}

/// Escrever na busca filtra e o filtro abre pelos mesmos ids.
#[test]
fn a_busca_filtra_e_o_filtro_abre() {
    let mut mesa = Mesa::nova(30);
    mesa.host.set_text(ids::HIER_SEARCH, "Object 12");
    mesa.assenta();
    assert!(mesa.rect(id_de(12)).is_some() && mesa.rect(id_de(13)).is_none());
    mesa.host.set_text(ids::HIER_SEARCH, "");
    mesa.assenta();
    let (x, y) = mesa.centro(ids::HIER_FILTER_DD);
    assert_eq!(mesa.host.hit_at(x, y), Some(ids::HIER_FILTER_DD));
    let ev = mesa.host.click_at(x, y);
    let _ = mesa.aplica(ev);
    mesa.pinta();
    assert_eq!(mesa.host.dropdown_is_open(ids::HIER_FILTER_DD), Some(true));
    let _ = mesa.clica(ids::hier_filter_opt(0));
    assert_eq!(mesa.host.dropdown_is_open(ids::HIER_FILTER_DD), Some(false));
    assert_eq!(mesa.st.component_filter, None, "«All» continua todos");
}

/// O botão direito na linha continua a abrir o menu de objecto.
#[test]
fn o_botao_direito_abre_o_menu_da_linha() {
    let mut mesa = Mesa::nova(4);
    let (x, y) = mesa.centro(id_de(2));
    let ev = mesa.host.right_click_at(x, y);
    let _ = mesa.aplica(ev);
    assert!(
        mesa.host.store().context_menu().is_some(),
        "o botão direito na linha não abriu nada"
    );
}

/// Os interruptores pintam-se arrastando: da linha 2 à 3 pela coluna do olho.
#[test]
fn arrastar_pela_coluna_do_olho_pinta_as_outras_linhas() {
    let mut mesa = Mesa::nova(5);
    let (a, b) = (id_de(2), id_de(3));
    let (de, para) = (
        mesa.centro(core_ids::hier_eye_companion(a)),
        mesa.centro(core_ids::hier_eye_companion(b)),
    );
    let ev = mesa.host.drag_at(de.0, de.1, para.0, para.1);
    let got = mesa.aplica(ev);
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::ToggleVisibility {
            row: a
        })),
        "{got:?}"
    );
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::ToggleVisibility {
            row: b
        })),
        "{got:?}"
    );
}

/// O ícone de componente e a raiz Cena: a linha mostra-os e o clique responde.
#[test]
fn o_icone_de_componente_e_a_cena_respondem() {
    let mut mesa = Mesa::nova(4);
    let icone = ids::hier_component_icon(id_de(0), ComponentCategory::Physics);
    let got = mesa.clica(icone);
    assert!(
        got.contains(&seleciona(id_de(0), SelectModifier::Replace)),
        "{got:?}"
    );
    assert!(
        mesa.host.store().inspector_reveal().is_some(),
        "não pediu o cartão"
    );
    mesa.pinta();
    let got = mesa.clica(ids::HIER_SCENE_ROW);
    assert!(
        got.contains(&EditorAction::ClearSelection),
        "a Cena não limpou a selecção: {got:?}"
    );
}

/// Um objecto arrastado para um «drop» de asset: o rect da linha é o que a queda lê.
#[test]
fn o_drop_de_asset_acha_a_linha_sob_o_cursor() {
    let mesa = Mesa::nova(4);
    let (x, y) = mesa.centro(id_de(2));
    assert_eq!(mesa.host.hit_at(x, y), Some(id_de(2)));
    assert!(mesa.host.store().is_hierarchy_row(id_de(2)));
    assert!(!mesa.host.store().is_hierarchy_row(ids::HIER_SCENE_ROW));
}

/// Os Clips: o conteúdo nasce recolhido, o galo abre-o, e a linha do clip responde.
#[test]
fn o_conteudo_de_um_objecto_abre_pelo_galo() {
    let mut mesa = Mesa::com(3, |m| {
        if let Some(e) = m.get_mut(&id_de(2)) {
            e.content = vec![ContentGroup {
                label_key: "hier_rows.content.layers",
                icon: IconId::Layers,
                item_icon: IconId::Layer,
                opens: ContentOpens::Mode(ObjectMode::Draw),
                items: vec![ContentItem {
                    key: 7,
                    name: "Ink".into(),
                    depth: 0,
                    active: false,
                }],
            }];
        }
    });
    let (grupo, item) = (
        ids::hier_content_row(id_de(2), 0, None),
        ids::hier_content_row(id_de(2), 0, Some(7)),
    );
    assert!(mesa.rect(grupo).is_some() && mesa.rect(item).is_none());
    let galo = core_ids::hier_expand_companion(grupo);
    assert!(
        mesa.rect(galo).is_some(),
        "o grupo de conteúdo perdeu o galo"
    );
    let _ = mesa.clica(galo);
    mesa.assenta();
    assert!(
        mesa.rect(item).is_some(),
        "o galo do grupo não abriu o conteúdo"
    );
    let _ = mesa.clica(galo);
    mesa.assenta();
    assert!(
        mesa.rect(item).is_none(),
        "o galo do grupo não fechou o conteúdo"
    );
    let _ = mesa.clica(grupo);
    mesa.assenta();
    assert!(
        mesa.rect(item).is_some(),
        "o clique no grupo não abriu o conteúdo"
    );
}

/// O galo da raiz Cena recolhe a cena inteira (era um id acima da guarda dos companheiros).
#[test]
fn o_galo_da_cena_recolhe_tudo() {
    let mut mesa = Mesa::nova(3);
    let galo = core_ids::hier_expand_companion(ids::HIER_SCENE_ROW);
    let _ = mesa.clica(galo);
    mesa.assenta();
    assert!(
        mesa.rect(id_de(2)).is_none(),
        "a Cena recolhida ainda mostra os objectos"
    );
    let _ = mesa.clica(galo);
    mesa.assenta();
    assert!(mesa.rect(id_de(2)).is_some());
}

/// O rato por cima acende a linha; a pintura muda (a contraprova é a linha em repouso).
#[test]
fn o_rato_por_cima_acende_a_linha() {
    let mut mesa = Mesa::nova(4);
    let repouso = mesa
        .host
        .paint_and_count_geometry::<HierarchyPanel>(&mut mesa.st, VIEW);
    let p = mesa.centro(id_de(3));
    let _ = mesa.ponteiro(PointerKind::Move, p);
    assert_eq!(
        mesa.host.store().hot_id(),
        Some(id_de(3)),
        "a linha não soube do rato"
    );
    let acesa = mesa
        .host
        .paint_and_count_geometry::<HierarchyPanel>(&mut mesa.st, VIEW);
    assert!(
        acesa.1 > repouso.1,
        "o realce não pintou: {acesa:?} × {repouso:?}"
    );
}

/// Todo ícone mudo pintado tem dica (T31).
#[test]
fn todo_icone_mudo_tem_dica() {
    let mut mesa = Mesa::nova(4);
    mesa.assenta();
    let mudos = dicas::icon_only_ids(PAINEL);
    assert!(mudos.len() > 8, "só {} ícones no censo", mudos.len());
    for id in mudos {
        assert!(
            mesa.host.store().tooltip_for(id).is_some(),
            "{id:?} sem dica"
        );
    }
    assert!(
        mesa.host
            .store()
            .tooltip_for(core_ids::HIERARCHY_ADD)
            .is_some()
    );
}

/// `PH2D_CARDS=0`: o painel de antes, na coluna inteira, sem lista.
#[test]
fn cards_off_e_o_painel_de_antes() {
    let mut mesa = Mesa::nova(4);
    let cartao = mesa.lista();
    cards::force_cards_on_this_thread(Some(false));
    mesa.host.store_mut().clear_panel_rect(PAINEL);
    mesa.assenta();
    let velho = mesa.lista();
    assert!(
        velho.h > cartao.h,
        "o de antes ocupa a coluna inteira: {velho:?}"
    );
    assert!(mesa.rect(core_ids::HIERARCHY_ADD).is_some());
    cards::force_cards_on_this_thread(Some(true));
}

/// ⭐ O custo por quadro das duas pinturas, intercaladas no mesmo processo, resumo = o mínimo
/// (`docs/DevOps/MEDIR_VELOCIDADE.md`). `--profile smoke`, `--ignored --nocapture`.
#[test]
#[ignore = "medição: corre à mão com --profile smoke"]
fn custo_por_quadro_com_cinco_mil_linhas() {
    let mut mesa = Mesa::nova(5000);
    let mut melhor = [f64::MAX; 2];
    for ronda in 0..60 {
        let on = ronda % 2 == 0;
        cards::force_cards_on_this_thread(Some(on));
        let t = std::time::Instant::now();
        mesa.pinta();
        let s = t.elapsed().as_secs_f64() * 1e6;
        let i = usize::from(!on);
        melhor[i] = melhor[i].min(s);
    }
    cards::force_cards_on_this_thread(Some(true));
    println!(
        "5000 linhas, pintura do painel, mínimo de 30: cartões {:.0} µs · painel de antes {:.0} µs",
        melhor[0], melhor[1]
    );
}
