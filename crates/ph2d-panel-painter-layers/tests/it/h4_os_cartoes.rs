//! **Os cartões Watercolor e Impasto em linhas de cartão, fatia H4, e os botões sem «…»**
//! (`docs/CartoesUI/13_PAINTER_PLANO.md`, `05` §4 lei 7): pelo gesto real, e o que chega ao barramento é o MESMO do painel
//! de antes. Nenhum motor, preset ou pincel muda.

use super::h1_os_cartoes::{Mesa, esc, pincel, valor_enviado};
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, PaintMedia};
use std::collections::BTreeSet;

const AGUA: NodeId = p::PAINTER_WATERCOLOR_SECTION;
const IMPASTO: NodeId = p::PAINTER_IMPASTO_SECTION;
const WET: NodeId = p::PAINTER_WETPAINT_SECTION;
const TOPO: NodeId = p::PAINTER_CARD_TOPO;

const CORES: [NodeId; 2] = [p::PAINTER_IMPASTO_WAX_COLOR, p::PAINTER_IMPASTO_LIGHT_COLOR];

fn aguada() -> BrushSettings {
    pincel(PaintMedia::Watercolor, "brush").0
}

fn impasto() -> BrushSettings {
    pincel(PaintMedia::Impasto, "brush").0
}

/// A faca na mão: o cartão Knife aparece, e a lâmpada 2 dá o Enable.
fn faca() -> BrushSettings {
    let mut b = impasto();
    b.impasto_tool = 1;
    b.impasto_rig.selected = 1;
    b
}

fn so(secao: NodeId, cartoes: bool, b: BrushSettings) -> Mesa {
    let mut m = Mesa::nova(cartoes, b, false);
    if cartoes {
        m.so_abertos(&[secao, TOPO]);
    }
    m
}

fn numeros_da_agua() -> BTreeSet<NodeId> {
    [
        p::PAINTER_WATERCOLOR_FILL,
        p::PAINTER_WATERCOLOR_DEPTH,
        p::PAINTER_WATERCOLOR_OPACITY,
        p::PAINTER_WATERCOLOR_EDGE,
        p::PAINTER_WATERCOLOR_SPREAD,
        p::PAINTER_WATERCOLOR_WARP,
        p::PAINTER_WATERCOLOR_FLOW_SIZE,
        p::PAINTER_WATERCOLOR_FLOW_ANGLE,
        p::PAINTER_WATERCOLOR_PAPER_EDGE,
        p::PAINTER_WATERCOLOR_CHARGE,
        p::PAINTER_WATERCOLOR_DILUTION,
        p::PAINTER_WATERCOLOR_PULL,
        p::PAINTER_WATERCOLOR_WET,
        p::PAINTER_WATERCOLOR_SMUDGE,
        p::PAINTER_WATERCOLOR_DRY_TIME,
        p::PAINTER_WATERCOLOR_WET_PREVIEW,
    ]
    .into_iter()
    .collect()
}

fn numeros_do_impasto() -> BTreeSet<NodeId> {
    p::PAINTER_IMPASTO_FIELDS.iter().copied().collect()
}

fn cliques_da_agua() -> [NodeId; 4] {
    [
        p::PAINTER_WATERCOLOR_SMOOTH_EDGES,
        p::PAINTER_WATERCOLOR_DRY_NOW,
        p::PAINTER_WATERCOLOR_WET_NOW,
        p::PAINTER_WATERCOLOR_RESET,
    ]
}

/// Todo id de controlo das duas secções: os números, os cliques, a lista do Flow.
fn universo() -> BTreeSet<NodeId> {
    let mut u = numeros_da_agua();
    u.extend(numeros_do_impasto());
    u.extend(cliques_da_agua());
    u.extend(p::PAINTER_IMPASTO_CLICKS);
    u.extend(p::PAINTER_IMPASTO_TOOL_IDS);
    u.extend(CORES);
    u.insert(p::PAINTER_WATERCOLOR_FLOW_KIND);
    u
}

fn estados() -> Vec<(&'static str, BrushSettings)> {
    let mut esculpe = impasto();
    esculpe.is_sculpt = true;
    esculpe.sculpt_knob_family = 1;
    esculpe.sculpt_is_chisel = true;
    esculpe.sculpt_filters = true;
    vec![
        ("aguada", aguada()),
        ("impasto", impasto()),
        ("faca e lâmpada 2", faca()),
        ("esculpir", esculpe),
    ]
}

/// ⭐ As mesmas condições, nas DUAS direcções, em cada estado.
#[test]
fn watercolor_and_impasto_show_the_same_controls_as_the_old_panel() {
    let nosso = universo();
    let mut vistos = BTreeSet::new();
    for (nome, b) in estados() {
        let so_nossos = |m: &Mesa| -> BTreeSet<NodeId> {
            m.ids().into_iter().filter(|i| nosso.contains(i)).collect()
        };
        let antes = so_nossos(&Mesa::nova(false, b, false));
        let agora = so_nossos(&Mesa::nova(true, b, false));
        let so_antes: Vec<_> = antes.difference(&agora).collect();
        let so_agora: Vec<_> = agora.difference(&antes).collect();
        assert!(
            so_antes.is_empty() && so_agora.is_empty(),
            "{nome}: só antes {so_antes:?}, só agora {so_agora:?}"
        );
        vistos.extend(agora);
    }
    assert!(vistos.len() > 45, "a régua viu pouco: {}", vistos.len());
}

fn numeros_em(secao: NodeId, b: BrushSettings, nosso: &BTreeSet<NodeId>) -> Vec<NodeId> {
    let m = so(secao, true, b);
    m.ids().into_iter().filter(|i| nosso.contains(i)).collect()
}

/// Escrever em cada número manda o mesmo número pelo mesmo evento, com a faixa de antes.
#[test]
fn typing_each_number_sends_the_same_action_as_the_old_panel() {
    for (secao, b, nosso, piso) in [
        (AGUA, aguada(), numeros_da_agua(), 16),
        (IMPASTO, faca(), numeros_do_impasto(), 12),
    ] {
        let ids = numeros_em(secao, b, &nosso);
        assert!(ids.len() >= piso, "viu {} números", ids.len());
        let (mut velha, mut nova) = (so(secao, false, b), so(secao, true, b));
        for id in ids {
            for texto in ["0.04", "500", "-500"] {
                let ev = velha.host.type_into_number(id, texto);
                let a = velha.aplica(ev);
                let ev = nova.host.type_into_number(id, texto);
                let n = nova.aplica(ev);
                assert!(!n.is_empty(), "{id:?}: nada no barramento");
                assert_eq!(a, n, "{id:?} com {texto}");
            }
        }
    }
}

/// O scrub de cada número move só o seu campo, a semente não o puxa de volta, e o Esc devolve.
#[test]
fn scrubbing_each_number_moves_only_it_and_escape_gives_it_back() {
    for (secao, b, nosso) in [
        (AGUA, aguada(), numeros_da_agua()),
        (IMPASTO, faca(), numeros_do_impasto()),
    ] {
        let mut mesa = so(secao, true, b);
        let ids = numeros_em(secao, b, &nosso);
        for id in &ids {
            let le = |m: &Mesa, i: NodeId| m.host.store().number_value(i).expect("semeado");
            let antes = le(&mesa, *id);
            let outros: Vec<(NodeId, f64)> = ids
                .iter()
                .filter(|o| *o != id)
                .map(|o| (*o, le(&mesa, *o)))
                .collect();
            let mut ev = mesa.arrasta_a_pega(*id, 60.0);
            let mut no_gesto = le(&mesa, *id);
            if (no_gesto - antes).abs() < 1e-9 {
                mesa = so(secao, true, b);
                ev = mesa.arrasta_a_pega(*id, -60.0);
                no_gesto = le(&mesa, *id);
            }
            assert!((no_gesto - antes).abs() > 1e-9, "{id:?} não se moveu");
            let accoes = mesa.aplica(ev);
            let enviado = valor_enviado(&accoes, *id).expect("o painel avisa a ferramenta");
            assert!((enviado - no_gesto).abs() < 1e-3, "{id:?} {enviado}");
            for (o, v) in &outros {
                assert!(
                    (le(&mesa, *o) - v).abs() < 1e-9,
                    "{o:?} mexeu-se com {id:?}"
                );
            }
            mesa.quadro();
            assert!((le(&mesa, *id) - no_gesto).abs() < 1e-6, "a semente puxou");
            let _ = esc(&mut mesa);
            assert!((le(&mesa, *id) - antes).abs() < 1e-6, "{id:?}: o Esc");
        }
    }
}

/// Cada caixa, botão e opção: o mesmo clique, a mesma acção.
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    for (secao, b, ids) in [
        (AGUA, aguada(), cliques_da_agua().to_vec()),
        (
            IMPASTO,
            faca(),
            p::PAINTER_IMPASTO_CLICKS
                .iter()
                .chain(&p::PAINTER_IMPASTO_TOOL_IDS)
                .chain(&CORES)
                .copied()
                .collect::<Vec<_>>(),
        ),
    ] {
        let vistos = so(secao, true, b).ids();
        for id in ids {
            if !vistos.contains(&id) {
                assert!(
                    !so(secao, false, b).ids().contains(&id),
                    "{id:?}: o painel de antes mostrava-o"
                );
                continue;
            }
            let velho = so(secao, false, b).clica(id);
            let novo = so(secao, true, b).clica(id);
            assert_eq!(velho, novo, "{id:?}");
        }
    }
}

/// A lista do Flow abre debaixo da caixa, e a opção envia o mesmo.
#[test]
fn the_flow_list_opens_under_its_box_and_its_option_sends_the_same() {
    let b = aguada();
    let caixa = p::PAINTER_WATERCOLOR_FLOW_KIND;
    let opcao = p::painter_flow_kind_option_id(1);
    let escolhe = |cartoes: bool| {
        let mut mesa = so(AGUA, cartoes, b);
        let chip = mesa.rect(caixa).expect("a caixa");
        let _ = mesa.clica(caixa);
        mesa.assenta();
        let r = mesa.rect(opcao).expect("a opção aberta");
        assert!(r.y >= chip.y, "abriu acima: {chip:?} → {r:?}");
        mesa.clica(opcao)
    };
    let (velho, novo) = (escolhe(false), escolhe(true));
    assert!(!novo.is_empty(), "nada no barramento");
    assert_eq!(velho, novo);
}

/// A linha sob Flow Angle saía cortada: agora nada do corpo se sobrepõe, e tudo cai dentro do cartão.
#[test]
fn no_row_of_the_watercolor_card_overlaps_another() {
    let nosso = numeros_da_agua();
    let m = so(AGUA, true, aguada());
    let mut rects: Vec<(NodeId, ph2d_editor_core::zones::Rect)> = nosso
        .iter()
        .map(|id| (*id, m.rect(*id).expect("pintado")))
        .collect();
    rects.push((
        p::PAINTER_WATERCOLOR_SMOOTH_EDGES,
        m.rect(p::PAINTER_WATERCOLOR_SMOOTH_EDGES).expect("caixa"),
    ));
    for (i, (a, ra)) in rects.iter().enumerate() {
        for (b, rb) in &rects[i + 1..] {
            let separados = ra.x + ra.w <= rb.x
                || rb.x + rb.w <= ra.x
                || ra.y + ra.h <= rb.y
                || rb.y + rb.h <= ra.y;
            assert!(separados, "{a:?} por cima de {b:?}: {ra:?} {rb:?}");
        }
    }
    let angulo = m
        .rect(p::PAINTER_WATERCOLOR_FLOW_ANGLE)
        .expect("Flow Angle");
    let borda = m
        .rect(p::PAINTER_WATERCOLOR_PAPER_EDGE)
        .expect("Paper Edge");
    assert!(
        borda.y >= angulo.y + angulo.h,
        "a linha sob Flow Angle: {angulo:?} {borda:?}"
    );
}

/// Irmãos curtos dividem a linha (05 §4, lei 1).
#[test]
fn short_siblings_share_a_row() {
    let par = |secao, b, a: NodeId, c: NodeId| {
        let m = so(secao, true, b);
        let (ra, rc) = (m.rect(a).expect("a"), m.rect(c).expect("c"));
        assert!(
            (ra.y - rc.y).abs() < 0.5 && ra.x + ra.w <= rc.x,
            "{a:?} e {c:?}: {ra:?} {rc:?}"
        );
    };
    par(
        AGUA,
        aguada(),
        p::PAINTER_WATERCOLOR_FILL,
        p::PAINTER_WATERCOLOR_DEPTH,
    );
    par(
        AGUA,
        aguada(),
        p::PAINTER_WATERCOLOR_PAPER_EDGE,
        p::PAINTER_WATERCOLOR_SMOOTH_EDGES,
    );
    par(
        AGUA,
        aguada(),
        p::PAINTER_WATERCOLOR_DRY_NOW,
        p::PAINTER_WATERCOLOR_WET_NOW,
    );
    par(
        IMPASTO,
        impasto(),
        p::PAINTER_IMPASTO_WAX,
        p::PAINTER_IMPASTO_WAX_COLOR,
    );
    par(
        IMPASTO,
        impasto(),
        p::PAINTER_IMPASTO_LIGHT_POWER,
        p::PAINTER_IMPASTO_LIGHT_COLOR,
    );
}

/// Os dez nomes da lista TOOL descem em linhas, e cada um cabe no seu segmento.
#[test]
fn the_tool_list_wraps_and_no_segment_is_narrower_than_its_name() {
    let m = so(IMPASTO, true, impasto());
    let mut linhas = BTreeSet::new();
    let mut medidor = ph2d_text::TextSystem::new();
    let nomes = [
        "Deposit", "Knife", "Smooth", "Sharpen", "Flatten", "Scrape", "Fill", "Chisel", "Layer",
        "Inflate",
    ];
    for (id, nome) in p::PAINTER_IMPASTO_TOOL_IDS.iter().zip(nomes) {
        let r = m.rect(*id).expect("o segmento");
        linhas.insert(r.y as i32);
        let w = medidor.prefix_width(nome, ph2d_card_stack::metrics::SEG_TEXT_PX);
        assert!(r.w >= w, "{nome}: {w} num segmento de {}", r.w);
    }
    assert!(linhas.len() >= 2, "dez nomes numa linha: {linhas:?}");
}

/// ⭐ A queixa do dono: «Wet canv… Dry canv…». Os três na mesma linha, e cada nome cabe com o respiro.
#[test]
fn the_whole_canvas_buttons_share_a_row_and_show_their_whole_name() {
    let mut b = pincel(PaintMedia::WetPaint, "brush").0;
    b.wet_oleo_real = false;
    let m = so(WET, true, b);
    let mut medidor = ph2d_text::TextSystem::new();
    let mut y = None;
    for (id, nome) in [
        (p::PAINTER_WETPAINT_WETCANVAS, "Wet"),
        (p::PAINTER_WETPAINT_DRYCANVAS, "Dry"),
        (p::PAINTER_WETPAINT_FASTDRY, "Fast dry"),
    ] {
        let r = m.rect(id).expect("o botão");
        let w = medidor.prefix_width(nome, ph2d_card_stack::metrics::VALUE_PX);
        assert!(
            w + 2.0 * ph2d_card_stack::metrics::FIELD_PAD_R <= r.w,
            "{nome}: {w} em {}",
            r.w
        );
        assert_eq!(*y.get_or_insert(r.y), r.y, "{nome} desceu de linha");
        assert!(
            m.host
                .store()
                .tooltip_for(id)
                .is_some_and(|t| !t.is_empty()),
            "{nome} sem dica"
        );
    }
}

/// A cor abre o seletor partilhado; com ele aberto, o que muda volta à ferramenta pelo evento de antes.
#[test]
fn each_colour_opens_the_picker_and_sends_the_pick_back() {
    for id in CORES {
        let mut mesa = so(IMPASTO, true, impasto());
        let _ = mesa.clica(id);
        assert_eq!(mesa.host.store().picker_target(), Some(id), "{id:?}");
        mesa.host
            .store_mut()
            .set_widget_color(id, [12, 34, 56, 255]);
        let _ = mesa.host.drained_actions();
        mesa.quadro();
        let accoes: Vec<String> = mesa
            .host
            .drained_actions()
            .iter()
            .map(|a| format!("{a:?}"))
            .collect();
        assert!(
            accoes.iter().any(|a| a.contains("12,34,56")),
            "{id:?}: {accoes:?}"
        );
    }
}

/// Reset no cabeçalho do cartão, com o mesmo id e o mesmo efeito.
#[test]
fn the_reset_of_each_section_lives_in_the_card_header() {
    for (secao, b, id) in [
        (AGUA, aguada(), p::PAINTER_WATERCOLOR_RESET),
        (IMPASTO, impasto(), p::PAINTER_IMPASTO_RESET),
    ] {
        let (velho, novo) = (so(secao, false, b).clica(id), so(secao, true, b).clica(id));
        assert!(!novo.is_empty(), "{id:?}: nada no barramento");
        assert_eq!(velho, novo, "{id:?}");
    }
}
