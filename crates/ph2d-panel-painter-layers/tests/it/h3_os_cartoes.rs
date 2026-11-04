//! **O cartão Wet Paint em linhas de cartão, fatia H3** (`docs/CartoesUI/13_PAINTER_PLANO.md`): pelo gesto real, nos rects
//! pintados, e o que chega ao barramento é o MESMO do painel de antes. Nenhum motor, preset ou pincel muda.

use super::censo_dos_controlos::Bancada;
use super::h1_os_cartoes::{Mesa, esc, pincel, valor_enviado};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::grip_of;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::widget::key_mark::{KeyMark, add_key_mark, key_mark_target, set_key_marks};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_painter_layers::FALLBACK_BRUSH;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, PaintMedia, WetTool};
use std::collections::BTreeSet;

const WET: NodeId = p::PAINTER_WETPAINT_SECTION;
const TOPO: NodeId = p::PAINTER_CARD_TOPO;
const CHIPS_RAPIDOS: [NodeId; 3] = p::PAINTER_WETPAINT_VOL_QUICK;

fn molhado() -> BrushSettings {
    pincel(PaintMedia::WetPaint, "brush").0
}

fn com_oleo() -> BrushSettings {
    let mut b = molhado();
    b.wet_oleo_real = true;
    b
}

/// O Wet e o topo abertos, o resto dobrado: numa lista alta, o que passa da janela não regista alvo.
fn so_o_wet(cartoes: bool, b: BrushSettings) -> Mesa {
    let mut m = Mesa::nova(cartoes, b, false);
    if cartoes {
        m.so_abertos(&[WET, TOPO]);
    }
    m
}

/// Todo id de controlo deste cartão e dos volumes.
fn universo() -> BTreeSet<NodeId> {
    p::PAINTER_WETPAINT_FIELDS
        .iter()
        .chain(&p::PAINTER_WETPAINT_OLEO_FIELDS)
        .chain(&p::PAINTER_WETPAINT_CLICKS)
        .chain(&CHIPS_RAPIDOS)
        .chain(&[
            p::PAINTER_WETPAINT_MEDIUM,
            p::PAINTER_WETPAINT_OLEO_AMBIENTE,
            p::PAINTER_WETPAINT_TILT_PAD,
            p::PAINTER_WETPAINT_VOL_SIZE,
            p::PAINTER_WETPAINT_VOL_OPACITY,
            p::PAINTER_WETPAINT_DILUTION,
        ])
        .copied()
        .collect()
}

/// O id lido na mensagem: a tabela e o lugar nela.
fn onde(id: NodeId) -> String {
    let em = |t: &[NodeId]| t.iter().position(|i| *i == id);
    if let Some(n) = em(&p::PAINTER_WETPAINT_FIELDS) {
        format!("FIELDS[{n}]")
    } else if let Some(n) = em(&p::PAINTER_WETPAINT_OLEO_FIELDS) {
        format!("OLEO[{n}]")
    } else if let Some(n) = em(&p::PAINTER_WETPAINT_CLICKS) {
        format!("CLICKS[{n}]")
    } else {
        format!("{id:?}")
    }
}

fn estados() -> Vec<(String, BrushSettings)> {
    let mut casos = vec![
        ("pincel".to_owned(), molhado()),
        ("com óleo".to_owned(), com_oleo()),
    ];
    let mut na_tela = molhado();
    na_tela.wet_oleo_na_tela = true;
    casos.push(("óleo só na tela".to_owned(), na_tela));
    casos.push((
        "borracha".to_owned(),
        pincel(PaintMedia::WetPaint, "eraser").0,
    ));
    for t in [
        WetTool::Smear,
        WetTool::Blend,
        WetTool::Wet,
        WetTool::Dry,
        WetTool::Blow,
    ] {
        let mut b = molhado();
        b.wet_tool = t;
        casos.push((format!("{t:?}"), b));
    }
    casos
}

/// ⭐ As mesmas condições, nas DUAS direcções, em cada ferramenta da água e com o óleo.
#[test]
fn the_wet_card_shows_the_same_controls_in_each_water_tool_and_oil_state() {
    let nosso = universo();
    let mut vistos = BTreeSet::new();
    for (nome, b) in estados() {
        let so_nossos = |m: &Mesa| -> BTreeSet<NodeId> {
            m.ids().into_iter().filter(|i| nosso.contains(i)).collect()
        };
        let antes = so_nossos(&Mesa::nova(false, b, false));
        let agora = so_nossos(&Mesa::nova(true, b, false));
        let so_antes: Vec<String> = antes.difference(&agora).map(|i| onde(*i)).collect();
        let so_agora: Vec<String> = agora.difference(&antes).map(|i| onde(*i)).collect();
        assert!(
            so_antes.is_empty() && so_agora.is_empty(),
            "{nome}: só antes {so_antes:?}, só agora {so_agora:?}"
        );
        vistos.extend(agora);
    }
    assert!(vistos.len() > 60, "a régua viu pouco: {}", vistos.len());
}

/// Os números que cada estado mostra, com um valor dentro da faixa de todos.
fn numeros_em(b: BrushSettings) -> Vec<NodeId> {
    let nosso: BTreeSet<NodeId> = p::PAINTER_WETPAINT_FIELDS
        .iter()
        .chain(&p::PAINTER_WETPAINT_OLEO_FIELDS)
        .copied()
        .collect();
    let m = so_o_wet(true, b);
    m.ids().into_iter().filter(|i| nosso.contains(i)).collect()
}

/// Escrever em cada número manda o mesmo número pelo mesmo evento, com a faixa de antes.
#[test]
fn typing_each_number_sends_the_same_action_as_the_old_panel() {
    let b = com_oleo();
    let ids = numeros_em(b);
    assert!(ids.len() >= 30, "viu {} números", ids.len());
    let (mut velha, mut nova) = (so_o_wet(false, b), so_o_wet(true, b));
    for id in ids {
        for texto in ["0.04", "500"] {
            let ev = velha.host.type_into_number(id, texto);
            let a = velha.aplica(ev);
            let ev = nova.host.type_into_number(id, texto);
            let n = nova.aplica(ev);
            assert!(!n.is_empty(), "{id:?}: nada no barramento");
            assert_eq!(a, n, "{id:?} com {texto}");
        }
    }
}

/// O scrub de cada número move só o seu campo, o pincel antigo não o puxa de volta, e o Esc devolve.
#[test]
fn scrubbing_each_number_moves_only_it_and_escape_gives_it_back() {
    let b = com_oleo();
    let mut mesa = so_o_wet(true, b);
    let ids = numeros_em(b);
    for id in &ids {
        let le = |m: &Mesa, i: NodeId| m.host.store().number_value(i).expect("semeado");
        let antes = le(&mesa, *id);
        let outros: Vec<(NodeId, f64)> = ids
            .iter()
            .filter(|o| *o != id)
            .map(|o| (*o, le(&mesa, *o)))
            .collect();
        // Para baixo no que já está no topo da faixa.
        let mut ev = mesa.arrasta_a_pega(*id, 60.0);
        let mut no_gesto = le(&mesa, *id);
        if (no_gesto - antes).abs() < 1e-9 {
            // Outra mesa: dois toques seguidos na pega são o duplo clique que volta à fábrica.
            mesa = so_o_wet(true, b);
            ev = mesa.arrasta_a_pega(*id, -60.0);
            no_gesto = le(&mesa, *id);
        }
        assert!(
            (no_gesto - antes).abs() > 1e-9,
            "{} não se moveu de {antes}",
            onde(*id)
        );
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

/// O duplo clique do campo volta à fábrica do motor, a mesma tabela.
#[test]
fn each_number_knows_its_factory_value() {
    let mut b = com_oleo();
    b.wet_knobs = ph2d_tool_painter::WetKnobs::default();
    b.wet_grid_ratio = FALLBACK_BRUSH.wet_grid_ratio;
    let m = so_o_wet(true, b);
    for id in [
        p::PAINTER_WETPAINT_PICKUP,
        p::PAINTER_WETPAINT_GRID,
        p::PAINTER_WETPAINT_GRAVITY,
    ] {
        let f = m.host.store().card_field(id).expect("marcado").default;
        let v = m.host.store().number_value(id).expect("semeado");
        assert!((f.expect("fábrica") - v).abs() < 1e-3, "{id:?}");
    }
}

/// Cada caixa e cada verbo: o mesmo clique, a mesma acção.
#[test]
fn every_click_of_the_wet_card_sends_the_same_action() {
    let b = com_oleo();
    for id in [
        p::PAINTER_WETPAINT_TILT_TOGGLE,
        p::PAINTER_WETPAINT_WETCANVAS,
        p::PAINTER_WETPAINT_DRYCANVAS,
        p::PAINTER_WETPAINT_FASTDRY,
        p::PAINTER_WETPAINT_SHOWWET,
        p::PAINTER_WETPAINT_PAPER_VISUAL,
        p::PAINTER_WETPAINT_OLEO_REAL,
        p::PAINTER_WETPAINT_PIGMENT_MIXING,
        p::PAINTER_WETPAINT_LIVE_WATER,
        p::PAINTER_WETPAINT_TUNING,
        p::PAINTER_WETPAINT_RESET,
    ] {
        let velho = so_o_wet(false, b).clica(id);
        let novo = so_o_wet(true, b).clica(id);
        assert!(!novo.is_empty(), "{id:?}: nada no barramento");
        assert_eq!(velho, novo, "{id:?}");
    }
}

/// Paint type, Environment e os valores rápidos: abrem debaixo da caixa, e a opção envia o mesmo.
#[test]
fn each_wet_list_opens_under_its_box_and_its_option_sends_the_same() {
    let b = com_oleo();
    let casos = [
        (
            p::PAINTER_WETPAINT_MEDIUM,
            p::painter_wetpaint_medium_option_id(1),
        ),
        (
            p::PAINTER_WETPAINT_OLEO_AMBIENTE,
            p::painter_wetpaint_oleo_ambiente_option_id(2),
        ),
        (
            CHIPS_RAPIDOS[0],
            p::painter_wetpaint_vol_quick_option_id(0, 3),
        ),
        (
            CHIPS_RAPIDOS[2],
            p::painter_wetpaint_vol_quick_option_id(2, 3),
        ),
    ];
    for (caixa, opcao) in casos {
        let escolhe = |cartoes: bool| {
            let mut mesa = so_o_wet(cartoes, b);
            let chip = mesa.rect(caixa).expect("a caixa");
            let _ = mesa.clica(caixa);
            mesa.assenta();
            let r = mesa.rect(opcao).expect("a opção aberta");
            assert!(r.y >= chip.y, "{caixa:?} abriu acima: {chip:?} → {r:?}");
            assert!(
                r.x < chip.x + chip.w && r.x + r.w > chip.x,
                "{caixa:?} abriu ao lado: {chip:?} → {r:?}"
            );
            mesa.clica(opcao)
        };
        let (velho, novo) = (escolhe(false), escolhe(true));
        assert!(!novo.is_empty(), "{caixa:?}: nada no barramento");
        assert_eq!(velho, novo, "{caixa:?}");
    }
}

/// Size, Dilution e Volume: o valor do pincel, e o Size é o diâmetro.
#[test]
fn the_volumes_show_the_brush_values() {
    let mut b = molhado();
    b.size_px = 20.0;
    let m = so_o_wet(true, b);
    let v = m
        .host
        .store()
        .number_value(p::PAINTER_WETPAINT_VOL_SIZE)
        .expect("semeado");
    assert!((v - 40.0).abs() < 1e-3, "Size {v}");
    let velha = so_o_wet(false, b);
    for id in [
        p::PAINTER_WETPAINT_DILUTION,
        p::PAINTER_WETPAINT_VOL_OPACITY,
    ] {
        let (a, n) = (
            velha.host.store().number_value(id).expect("velha"),
            m.host.store().number_value(id).expect("nova"),
        );
        assert!((a - n).abs() < 1e-9, "{id:?}: {a} × {n}");
    }
}

/// A ferramenta Wet pousa só água: o Water dela, com o chip rápido dele, no lugar da Dilution e do Volume.
#[test]
fn the_water_tool_keeps_its_water_volume() {
    let mut b = molhado();
    b.wet_tool = WetTool::Wet;
    let m = so_o_wet(true, b);
    assert!(m.rect(p::PAINTER_WETPAINT_WATER).is_some());
    assert!(m.rect(CHIPS_RAPIDOS[1]).is_some());
    assert!(m.rect(p::PAINTER_WETPAINT_DILUTION).is_none());
}

/// A marca de animação: um número por número, como no painel de antes (Auto Key grava tudo).
#[test]
fn each_number_keeps_its_animation_mark() {
    let b = com_oleo();
    let conta = |cartoes: bool, ids: &[NodeId]| {
        set_key_marks(std::collections::BTreeMap::new());
        for id in ids {
            add_key_mark(*id, KeyMark::Chave);
        }
        let m = so_o_wet(cartoes, b);
        let alvos: Vec<NodeId> = m
            .ids()
            .into_iter()
            .filter(|i| key_mark_target(*i).is_some())
            .collect();
        set_key_marks(std::collections::BTreeMap::new());
        alvos.len()
    };
    let mut ids = numeros_em(b);
    ids.extend([
        p::PAINTER_WETPAINT_MEDIUM,
        p::PAINTER_WETPAINT_OLEO_AMBIENTE,
    ]);
    let (velha, nova) = (conta(false, &ids), conta(true, &ids));
    assert_eq!(velha, nova, "marcas de antes × de agora");
    assert!(nova >= ids.len(), "só {nova} marcas para {} ids", ids.len());
}

/// O Heavy Edge inerte sem o Tilt: esmaecido, com dica, e continua a responder.
#[test]
fn the_heavy_edge_dims_without_the_tilt_and_keeps_working() {
    let id = p::PAINTER_WETPAINT_HEAVY_EDGE;
    let mut banca = Bancada::nova(PaintMedia::WetPaint, &[]);
    let mut apagada = banca.tool.brush_settings();
    if apagada.wet_tilt_on {
        banca
            .tool
            .handle_panel_event(PanelEvent::Click(p::PAINTER_WETPAINT_TILT_TOGGLE));
        apagada = banca.tool.brush_settings();
    }
    assert!(
        apagada.inercias.dica_de(id).is_some(),
        "o estado da régua não o deixa inerte"
    );
    let mut acesa = apagada;
    acesa.inercias = ph2d_tool_painter::Inercias::NENHUMA;
    let desenho = |b: BrushSettings| {
        let mut m = so_o_wet(true, b);
        assert!(m.rect(id).is_some(), "deixou de ter alvo");
        let dica = m
            .host
            .store()
            .tooltip_for(id)
            .is_some_and(|t| !t.is_empty());
        let (h, st) = (&mut m.host, &mut m.st);
        let d = h.paint_draw_data::<ph2d_panel_painter_layers::PainterLayersPanel>(st, m.janela);
        (d.len(), dica)
    };
    let (viva, dica_viva) = desenho(acesa);
    let (morta, dica_morta) = desenho(apagada);
    assert!(morta > viva, "não esmaeceu: {morta} × {viva}");
    assert!(dica_morta && !dica_viva, "a dica é da lei");
}

fn solta(mesa: &mut Mesa, sec: NodeId) {
    let g = mesa.rect(grip_of(sec)).expect("a pega");
    let (x, y0) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    for i in 1..=20 {
        let _ = mesa.ponteiro(PointerKind::Move, x - (i * 30) as f32, y0 + 40.0);
        mesa.quadro();
    }
    let _ = mesa.ponteiro(PointerKind::Up, x - 600.0, y0 + 40.0);
    mesa.assenta();
    assert!(
        mesa.host
            .store()
            .is_loose_in(sec, core_ids::PAINTER_LAYERS_PANEL),
        "o cartão não se soltou"
    );
}

fn tela_do_visor(m: &Mesa) -> Rect {
    match m.host.store().get(p::PAINTER_WETPAINT_TILT_PAD) {
        Some(InteractiveState::CurvePoint { canvas, .. }) => *canvas,
        _ => panic!("o visor não registou a tela"),
    }
}

/// Arrasta no visor, a `f` da largura e da altura dele, e devolve o que o painel mandou.
fn arrasta_no_visor(m: &mut Mesa, (fx, fy): (f32, f32)) -> Vec<String> {
    let r = m.rect(p::PAINTER_WETPAINT_TILT_PAD).expect("o visor");
    let (x, y) = (r.x + r.w * fx, r.y + r.h * fy);
    let mut ev = m.ponteiro(PointerKind::Down, r.x + r.w * 0.5, r.y + r.h * 0.5);
    ev.extend(m.ponteiro(PointerKind::Move, x, y));
    ev.extend(m.ponteiro(PointerKind::Up, x, y));
    let a = m.aplica(ev);
    m.quadro();
    let mut b = m.aplica(Vec::new());
    let mut t = a;
    t.append(&mut b);
    t
}

/// ⭐ Tilt e visor iguais (E10): o mesmo arrasto dá o mesmo anel e o mesmo raio, na lista e num cartão solto.
#[test]
fn the_tilt_dial_is_the_same_on_the_list_and_on_a_loose_card() {
    let b = molhado();
    let alvo = (0.8, 0.3);
    let velho = arrasta_no_visor(&mut so_o_wet(false, b), alvo);
    let mut mesa = so_o_wet(true, b);
    let r = mesa.rect(p::PAINTER_WETPAINT_TILT_PAD).expect("o visor");
    let tela = tela_do_visor(&mesa);
    assert!((r.x - tela.x).abs() < 0.5 && (r.y - tela.y).abs() < 0.5);
    assert!((r.w - 128.0).abs() < 0.5, "o visor mudou de tamanho: {r:?}");
    let novo = arrasta_no_visor(&mut mesa, alvo);
    assert!(valor_enviado(&novo, p::PAINTER_WETPAINT_TILT_RING).is_some());
    assert_eq!(velho, novo, "na lista");
    let mut solta_ = so_o_wet(true, b);
    solta(&mut solta_, WET);
    let r = solta_.rect(p::PAINTER_WETPAINT_TILT_PAD).expect("o visor");
    let tela = tela_do_visor(&solta_);
    assert!(
        (r.x - tela.x).abs() < 0.5 && (r.y - tela.y).abs() < 0.5,
        "a tela do visor ficou na coluna: {tela:?} × {r:?}"
    );
    let solto = arrasta_no_visor(&mut solta_, alvo);
    assert_eq!(velho, solto, "solto");
}

/// O que sobra de um par fica com a linha inteira, e a caixa divide a linha só se o nome cabe inteiro.
#[test]
fn what_is_left_of_a_pair_takes_the_whole_line() {
    let m = so_o_wet(true, com_oleo());
    let r = |id| m.rect(id).expect("pintado");
    let (par, sobra) = (
        r(p::PAINTER_WETPAINT_COVER),
        r(p::PAINTER_WETPAINT_WET_SOFTEN),
    );
    assert!((sobra.x - par.x).abs() < 0.5, "{sobra:?} × {par:?}");
    assert!(sobra.w > 1.5 * par.w, "{sobra:?} × {par:?}");
    // A caixa do nome comprido divide a linha só onde o nome cabe inteiro; sem marca de animação, sobra coluna.
    let mistura = r(p::PAINTER_WETPAINT_PIGMENT_MIXING);
    let nome = ph2d_text::TextSystem::new()
        .prefix_width("Pigment mixing", ph2d_card_stack::metrics::VALUE_PX);
    let pede = ph2d_card_stack::metrics::CHECK_BOX + ph2d_card_stack::metrics::CHECK_GAP + nome;
    assert!(pede <= mistura.w, "{pede} em {mistura:?}");
}

/// O Reset é o do cabeçalho, e o título é o de sempre.
#[test]
fn the_wet_card_keeps_its_reset_and_only_one() {
    let m = so_o_wet(true, molhado());
    let n = m
        .rects
        .iter()
        .filter(|(i, _)| *i == p::PAINTER_WETPAINT_RESET)
        .count();
    assert_eq!(n, 1, "Reset registado {n} vezes");
    let r = m.rect(p::PAINTER_WETPAINT_RESET).expect("o Reset");
    let g = m.rect(grip_of(WET)).expect("a pega");
    assert!(
        (r.y + r.h * 0.5 - (g.y + g.h * 0.5)).abs() < g.h,
        "fora da cabeça"
    );
}

/// ⭐ Um solto não muda a coluna dos outros: o que um corpo capturado guarda em coordenadas (a tela de uma curva)
/// continua igual ao alvo dele. Sem a guarda, o Falloff passava a pintar-se na coluna do solto.
#[test]
fn a_loose_card_leaves_the_stored_canvas_of_the_others_in_their_column() {
    let sec = p::PAINTER_BRUSH_STROKE_SECTION;
    let curva = p::PAINTER_SHAPE_SECTION;
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    mesa.so_abertos(&[sec, curva]);
    solta(&mut mesa, sec);
    let telas: Vec<(NodeId, Rect)> = mesa
        .rects
        .iter()
        .filter_map(|(i, r)| match mesa.host.store().get(*i) {
            Some(InteractiveState::CurvePoint { canvas, .. }) if r.w > 0.0 => Some((*i, *canvas)),
            _ => None,
        })
        .collect();
    assert!(!telas.is_empty(), "a régua não viu nenhuma curva");
    let g = mesa.rect(grip_of(curva)).expect("a pega do Shape");
    for (i, t) in telas {
        assert!(
            t.x >= g.x - 1.0 && t.x < g.x + g.w,
            "{i:?}: a tela saiu da coluna: {t:?} × {g:?}"
        );
    }
}
