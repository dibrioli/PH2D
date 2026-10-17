//! ⭐⭐ **As ABAS do título e o grupo Advanced** (F17, dono 16/10 —
//! `docs/UI_New_and_Simple/vetor_nova_ui/28_plano_F17_abas_desenho_interface.md` §6).
//!
//! Os gestos são REAIS (Down+Up sobre o rect que o painel pintou): o sintético pularia a
//! focabilidade no store, que é a metade que faz um controlo pintado ficar morto sob o ponteiro.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{Aba, ComponentState, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_tool_vector::ids as t;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// Um Down+Up REAL sobre o que o painel pintou para `id`; os eventos seguem para o painel.
fn clica(host: &mut MockPanelHost, id: ph2d_a11y::NodeId, what: &str) {
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
}

fn pintada(host: &mut MockPanelHost, id: ph2d_a11y::NodeId) -> Option<Rect> {
    host.painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, id)
}

/// ⭐⭐ **Cada aba pinta só as suas secções, e o Encaixe nas duas.** Com um MESTRE publicado: em
/// Drawing a Component não pinta e o Fill pinta; o clique na aba «Interface» inverte. *Mutações:
/// o filtro da aba fora do laço ⇒ a Component pinta em Drawing; a aba sem `register` ⇒ o clique
/// não chega.*
#[test]
fn cada_aba_pinta_so_as_suas_seccoes() {
    state::set_component_state(Some(ComponentState {
        is_main: true,
        ..ComponentState::default()
    }));
    let mut host = crate::host_aberto();
    assert_eq!(state::aba(), Aba::Drawing, "o painel nasce no desenho");
    assert!(pintada(&mut host, t::VECTOR_SECTION_FILL).is_some());
    assert!(
        pintada(&mut host, t::VECTOR_SECTION_SNAP).is_some(),
        "o Snap é das duas abas"
    );
    assert!(
        pintada(&mut host, ids::VECTOR_SECTION_COMPONENT).is_none(),
        "a Component pintou na aba Drawing"
    );
    clica(&mut host, ids::VECTOR_TAB_INTERFACE, "a aba Interface");
    assert_eq!(state::aba(), Aba::Interface);
    assert!(pintada(&mut host, ids::VECTOR_SECTION_COMPONENT).is_some());
    assert!(
        pintada(&mut host, t::VECTOR_SECTION_SNAP).is_some(),
        "o Snap é das duas abas"
    );
    assert!(
        pintada(&mut host, t::VECTOR_SECTION_FILL).is_none(),
        "o Fill pintou na aba Interface"
    );
    clica(&mut host, ids::VECTOR_TAB_DRAWING, "a aba Drawing");
    assert_eq!(state::aba(), Aba::Drawing);
    state::set_component_state(None);
}

/// ⭐⭐ **As duas portas escrevem a MESMA aba** — o clique na aba e o do interruptor da ①; o `on`
/// do interruptor e o tinto do palco seguem-na. *Mutação: o interruptor com um estado próprio ⇒ a
/// aba e o `on` discordam.*
#[test]
fn as_duas_portas_escrevem_a_mesma_aba() {
    let mut host = crate::host_aberto();
    let interruptor = |host: &MockPanelHost| {
        let bars = ph2d_panel_vector::bars::tool_bars(host.store(), None, Vec::new());
        let ultimo = bars.column.last().and_then(|g| g.first()).cloned();
        let b = ultimo.expect("a coluna ① perdeu o último grupo");
        assert_eq!(
            b.id,
            ids::VECTOR_VIEW_INTERFACE,
            "o interruptor não é o último da ①"
        );
        (b.on, bars.stage_tint)
    };
    assert_eq!(interruptor(&host), (false, None));
    host.apply_panel_event::<VectorPanel>(
        &mut VectorPanelState,
        WidgetEvent::Click(ids::VECTOR_VIEW_INTERFACE),
    );
    assert_eq!(
        state::aba(),
        Aba::Interface,
        "o interruptor não abriu a Interface"
    );
    assert_eq!(
        interruptor(&host),
        (true, Some(ph2d_tokens::ColorToken::StageTintUi))
    );
    clica(&mut host, ids::VECTOR_TAB_DRAWING, "a aba Drawing");
    assert_eq!(
        interruptor(&host),
        (false, None),
        "a aba e o interruptor discordam"
    );
    host.apply_panel_event::<VectorPanel>(
        &mut VectorPanelState,
        WidgetEvent::Click(ids::VECTOR_VIEW_INTERFACE),
    );
    host.apply_panel_event::<VectorPanel>(
        &mut VectorPanelState,
        WidgetEvent::Click(ids::VECTOR_VIEW_INTERFACE),
    );
    assert_eq!(
        state::aba(),
        Aba::Drawing,
        "o 2.º clique no interruptor volta ao desenho"
    );
}

/// ⭐⭐ **O grupo Advanced dobra os sete juntos, e um membro nunca foge dele.** Fixture: uma ordem
/// gravada que põe o Fill ENTRE o Blend e o Morph. Dobrado (de nascença) ⇒ nenhum membro pinta;
/// o clique no cabeçalho ⇒ os membros pintam LOGO a seguir a ele, e nada de fora entre eles.
/// *Mutação: pintar os membros pela ordem geral ⇒ o Blend sobe para junto do Fill.*
#[test]
fn o_grupo_avancado_dobra_os_sete_juntos() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    host.store_mut().set_section_order(vec![
        t::VECTOR_SECTION_BLEND,
        t::VECTOR_SECTION_FILL,
        t::VECTOR_SECTION_MORPH,
    ]);
    let membros = [t::VECTOR_SECTION_BLEND, t::VECTOR_SECTION_MORPH];
    // ⚠️ Os dois que pintam sem seleção (as escolhas do Pick Shapes são uma 2.ª seleção — ver
    // `the_blend_and_morph_survive…`); os outros cinco escondem-se sozinhos sem forma.
    let cab = pintada(&mut host, ids::VECTOR_SECTION_ADVANCED).expect("o grupo não pintou");
    for m in membros {
        assert!(
            pintada(&mut host, m).is_none(),
            "{m:?} pintou com o grupo dobrado"
        );
    }
    clica(
        &mut host,
        ids::VECTOR_SECTION_ADVANCED,
        "o cabeçalho Advanced",
    );
    assert!(!host.store().is_collapsed(ids::VECTOR_SECTION_ADVANCED));
    let ys: Vec<f32> = membros
        .iter()
        .map(|m| {
            pintada(&mut host, *m)
                .unwrap_or_else(|| panic!("{m:?} não pintou"))
                .y
        })
        .collect();
    let (lo, hi) = ys
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(*y), b.max(*y)));
    assert!(lo > cab.y, "um membro pintou ACIMA do cabeçalho do grupo");
    let fill = pintada(&mut host, t::VECTOR_SECTION_FILL).expect("o Fill sumiu");
    assert!(fill.y < cab.y, "o Fill caiu dentro do grupo");
    let mut st = VectorPanelState;
    let estranhas: Vec<_> = host
        .paint::<VectorPanel>(&mut st, VIEWPORT)
        .into_iter()
        .filter(|(id, r)| {
            ids::VECTOR_SECTIONS.contains(id)
                && !membros.contains(id)
                && *id != ids::VECTOR_SECTION_ADVANCED
                && r.y > lo
                && r.y < hi
        })
        .collect();
    assert!(
        estranhas.is_empty(),
        "secções de fora entre os membros: {estranhas:?}"
    );
}

/// ⭐⭐ **A ordem gravada do DONO não esconde a ordem do Figma** (época, `28_plano_F17…` §2). Fixture:
/// a linha `order=` REAL de `~/.ph2d/sections.txt` (16/10), que tem `vector.section.fill · stroke ·
/// snap · blend` (e o antigo `tool`) pela ordem de antes. Depois de UMA pintura a ordem pintada é a
/// natural; e um arrasto feito depois disso sobrevive à pintura seguinte. *Mutação: sem a chamada da
/// época no `paint` ⇒ o Snap pinta DEPOIS do Stroke.*
#[test]
fn a_ordem_gravada_do_dono_cede_a_ordem_do_figma_uma_vez() {
    const DONO: &str = "order=13531293219780341756,7974693498053883245,6417323300296073960,\
462361958089524258,17878912805899712198,256748903521374071,14429391564959515535,\
7272866814588566086,4487367034743066984,6039181550140646391,533158348381115479,\
7835137614430501909,5489823267695691391,6404651399025619991,5475466507587415736,\
16209176553101174232,6247592538361872117,7854175722016653081,12859062204754217185,\
17560411704787829902,3855126674888680695,7790154217309205455,11085371548993854718,\
6440797432030781278\n";
    let mut host = crate::host_aberto();
    ph2d_editor_core::interaction::restore_section_prefs_text(host.store_mut(), DONO);
    let antes = host.store().section_order().len();
    let y = |host: &mut MockPanelHost, id| pintada(host, id).map(|r| r.y).expect("não pintou");
    let natural = [
        t::VECTOR_SECTION_SNAP,
        t::VECTOR_SECTION_FILL,
        t::VECTOR_SECTION_STROKE,
    ];
    let ys: Vec<f32> = natural.iter().map(|id| y(&mut host, *id)).collect();
    assert!(
        ys.windows(2).all(|w| w[0] < w[1]),
        "a ordem gravada do dono ainda manda: Snap/Fill/Stroke a {ys:?}"
    );
    assert_eq!(
        host.store().section_order().len(),
        antes - 4,
        "só as quatro secções do vetor saem da ordem gravada (Inspector e Painter ficam)"
    );
    // O artista arrasta o Stroke para cima do Snap DEPOIS da época: o arrasto manda.
    host.store_mut()
        .set_section_order(vec![t::VECTOR_SECTION_STROKE, t::VECTOR_SECTION_SNAP]);
    assert!(
        y(&mut host, t::VECTOR_SECTION_STROKE) < y(&mut host, t::VECTOR_SECTION_SNAP),
        "o arrasto depois da época foi esquecido"
    );
}
