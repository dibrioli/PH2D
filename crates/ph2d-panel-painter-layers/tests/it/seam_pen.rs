//! **O cartão Pen pelo ponteiro** (L2, `docs/Painter/Pinceis/03_MOTORES.md` §4): a mão do pincel nos quatro
//! meios. Cada linha é accionada como o artista a acciona (dígitos numa caixa, clique no chip e na opção, o
//! Reset) e o efeito lê-se no TOOL — a mão do meio de agora (`BrushSettings::mao`). Uma linha que o meio não
//! lê não se pinta.
//!
//! Mutação que sangra (medida 20/10): tirar `PAINTER_PEN_FIELDS` do `number_field::is_param_field`. Tirá-la do
//! `populate` SOBREVIVE — o `paint_num_row` arma o campo ele próprio (`arm_field`); a linha do `populate` é a
//! paridade textual que o `table_driven_chips_are_registered_too` lê.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_panel_painter_layers::{PainterLayersPanel, set_current_mao_do_preset};
use ph2d_tool_painter::{LINHAS_DA_MAO, MAO_DE_FABRICA, PaintMedia, PainterTool, ids};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 6000.0)
}

fn no_meio(m: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(m);
    t
}

fn painted(
    tool: &PainterTool,
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(tool.brush_settings()));
    host.paint::<PainterLayersPanel>(st, viewport())
}

/// O painel pintado com o cartão Pen ABERTO (ele nasce recolhido: o clique no cabeçalho abre-o).
fn novo(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = painted(tool, &mut host, &mut st);
    let cab = rect_of(&rects, ids::PAINTER_PEN_SECTION).expect("o cabeçalho do cartão Pen");
    assert!(
        rect_of(&rects, ids::PAINTER_PEN_FIELDS[0]).is_none(),
        "o cartão Pen nasce recolhido"
    );
    // O dobrar é do painel: nenhum evento chega ao tool (um de rascunho recebe o que vier).
    clica(&mut host, &mut st, &mut PainterTool::default(), cab);
    // A dobra anima: assenta-a no estado que o clique pediu.
    host.settle_section_folds();
    let rects = painted(tool, &mut host, &mut st);
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn entrega(host: &mut MockPanelHost, tool: &mut PainterTool) {
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

fn clica(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    r: Rect,
) {
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    entrega(host, tool);
}

fn digita(tool: &mut PainterTool, id: NodeId, texto: &str) {
    let (mut host, mut st, rects) = novo(tool);
    assert!(
        rect_of(&rects, id).is_some(),
        "a linha {id:?} não está pintada"
    );
    for ev in host.type_into_number(id, texto) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    entrega(&mut host, tool);
}

fn indice(chave: &str) -> usize {
    LINHAS_DA_MAO
        .iter()
        .position(|l| l.chave() == chave)
        .unwrap()
}

/// ⭐ O cartão Pen está nos QUATRO meios, com as linhas que cada meio lê — e só essas.
#[test]
fn o_cartao_pen_pinta_as_linhas_que_o_meio_le() {
    for m in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let (_, _, rects) = novo(&no_meio(m));
        assert!(rect_of(&rects, ids::PAINTER_PEN_SECTION).is_some(), "{m:?}");
        for (l, id) in LINHAS_DA_MAO.into_iter().zip(ids::PAINTER_PEN_FIELDS) {
            let auto = no_meio(m).brush_settings().watercolor_shape_auto;
            assert_eq!(
                rect_of(&rects, id).is_some(),
                l.lida(m, auto),
                "{m:?}: a linha {} pintada ⇔ o meio a lê",
                l.chave()
            );
        }
    }
}

/// ⭐ Digitar uma linha muda a mão do meio de agora: nos antigos os campos do pincel, no Wet o knob.
#[test]
fn digitar_uma_linha_muda_a_mao_do_meio() {
    let i = indice("pressureSize");
    for m in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let mut t = no_meio(m);
        digita(&mut t, ids::PAINTER_PEN_FIELDS[i], "0.5");
        assert_eq!(t.brush_settings().mao[i], 0.5, "{m:?}");
    }
    // No Wet a linha é o knob da caneta: o mesmo valor no mapa do motor.
    let mut t = no_meio(PaintMedia::WetPaint);
    digita(&mut t, ids::PAINTER_PEN_FIELDS[i], "0.4");
    assert_ne!(
        t.brush_settings().wet_knobs,
        no_meio(PaintMedia::WetPaint).brush_settings().wet_knobs
    );
    let o = indice("tiltOpacity");
    let mut t = no_meio(PaintMedia::Digital);
    digita(&mut t, ids::PAINTER_PEN_FIELDS[o], "0.6");
    assert_eq!(t.brush_settings().mao[o], 0.6);
}

/// A rotação é um menu: escolher «Pen tilt» põe a linha em 2.
#[test]
fn a_rotacao_escolhe_se_no_menu() {
    let mut t = no_meio(PaintMedia::Digital);
    let (mut host, mut st, rects) = novo(&t);
    let chip = rect_of(&rects, ids::PAINTER_PEN_ROTATION).expect("o chip da rotação");
    clica(&mut host, &mut st, &mut t, chip);
    let rects = painted(&t, &mut host, &mut st);
    let op = rect_of(&rects, ids::painter_pen_rotation_option_id(2)).expect("a opção Pen tilt");
    clica(&mut host, &mut st, &mut t, op);
    assert_eq!(t.brush_settings().mao[indice("dabRotation")], 2.0);
}

/// O menu da rotação cai na MESMA caixa que as linhas numéricas vizinhas (auditoria 25/10: ia até à borda).
#[test]
fn a_rotacao_alinha_com_as_caixas_do_cartao() {
    let (_, _, rects) = novo(&no_meio(PaintMedia::Digital));
    let chip = rect_of(&rects, ids::PAINTER_PEN_ROTATION).expect("o chip da rotação");
    for chave in ["pulledString", "tiltStart"] {
        let caixa = rect_of(&rects, ids::PAINTER_PEN_FIELDS[indice(chave)]).expect(chave);
        assert!(
            (chip.x - caixa.x).abs() < 0.01 && (chip.w - caixa.w).abs() < 0.01,
            "o chip {chip:?} e a caixa de `{chave}` {caixa:?} têm de partilhar a coluna e a largura"
        );
    }
}

/// O Reset do cartão é um MENU (dono 25/10): o clique no ícone abre-o sem tocar na mão; «Reset to default» devolve
/// a mão de fábrica do meio.
#[test]
fn o_reset_do_cartao_devolve_a_mao_de_fabrica() {
    for m in [PaintMedia::Digital, PaintMedia::WetPaint] {
        let mut t = no_meio(m);
        digita(&mut t, ids::PAINTER_PEN_FIELDS[0], "0.3");
        let mexida = t.brush_settings().mao;
        assert_ne!(mexida, MAO_DE_FABRICA, "{m:?}");
        let (mut host, mut st, rects) = novo(&t);
        let r = rect_of(&rects, ids::PAINTER_PEN_RESET).expect("o Reset do cartão");
        clica(&mut host, &mut st, &mut t, r);
        assert_eq!(
            t.brush_settings().mao,
            mexida,
            "{m:?}: abrir o menu não repõe"
        );
        let rects = painted(&t, &mut host, &mut st);
        let neutro = ids::painter_pen_reset_option_id(ids::PAINTER_PEN_RESET_NEUTRO);
        let r = rect_of(&rects, neutro).expect("o menu abriu com «Reset to default»");
        clica(&mut host, &mut st, &mut t, r);
        let mao = t.brush_settings().mao;
        for (i, l) in LINHAS_DA_MAO.into_iter().enumerate() {
            if l.lida_por(m) {
                assert_eq!(mao[i], MAO_DE_FABRICA[i], "{m:?}: {}", l.chave());
            }
        }
    }
}

/// «Reset to brush preset»: sem preset que traga mão a opção não se alcança (pintada apagada, sem alvo de clique);
/// com ele, o clique despacha a SUA opção ao tool, e o menu fecha.
#[test]
fn o_reset_para_o_preset_despacha_a_sua_opcao() {
    let preset = ids::painter_pen_reset_option_id(ids::PAINTER_PEN_RESET_PRESET);
    let abre = |disponivel: bool| {
        let t = no_meio(PaintMedia::Digital);
        set_current_mao_do_preset(disponivel);
        let (mut host, mut st, rects) = novo(&t);
        let r = rect_of(&rects, ids::PAINTER_PEN_RESET).expect("o Reset do cartão");
        clica(&mut host, &mut st, &mut PainterTool::default(), r);
        let rects = painted(&t, &mut host, &mut st);
        (host, st, rects)
    };
    let (_, _, rects) = abre(false);
    assert!(
        rect_of(
            &rects,
            ids::painter_pen_reset_option_id(ids::PAINTER_PEN_RESET_NEUTRO)
        )
        .is_some(),
        "o menu abriu"
    );
    assert!(
        rect_of(&rects, preset).is_none(),
        "sem preset a opção não tem alvo de clique"
    );
    let (mut host, mut st, rects) = abre(true);
    let r = rect_of(&rects, preset).expect("com preset a opção alcança-se");
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    let pedidos: Vec<PanelEvent> = host
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::ToolPanelEvent(pe) => Some(pe),
            _ => None,
        })
        .collect();
    assert_eq!(
        pedidos,
        vec![PanelEvent::SelectOption(
            ids::PAINTER_PEN_RESET,
            ids::PAINTER_PEN_RESET_PRESET.to_string()
        )],
        "a opção do preset despacha a sua"
    );
    let rects = painted(&no_meio(PaintMedia::Digital), &mut host, &mut st);
    assert!(rect_of(&rects, preset).is_none(), "o menu fechou");
    set_current_mao_do_preset(false);
}

/// O custo de pintar o painel com o cartão Pen ABERTO × fechado, no Digital (o MÍNIMO de 20 pinturas, ms).
#[test]
#[ignore = "sonda: o custo do cartão Pen no painel"]
fn sonda_custo_do_cartao_pen() {
    let t = no_meio(PaintMedia::Digital);
    let mede = |host: &mut MockPanelHost, st: &mut PainterLayersPanelState| {
        (0..20)
            .map(|_| {
                let t0 = std::time::Instant::now();
                std::hint::black_box(painted(&t, host, st));
                t0.elapsed().as_secs_f64() * 1000.0
            })
            .fold(f64::INFINITY, f64::min)
    };
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let fechado = mede(&mut host, &mut st);
    let (mut host, mut st, _) = novo(&t);
    let aberto = mede(&mut host, &mut st);
    eprintln!("painel Digital: Pen fechado {fechado:.2} ms · aberto {aberto:.2} ms (mín. de 20)");
}
