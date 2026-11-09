//! **O topo e os básicos do painel Painter em linhas de cartão, fatia H2** (`docs/CartoesUI/13_PAINTER_PLANO.md`):
//! Paint Mode, Preset, Sync, Blend, Color, Size, Strength, Flow, Accumulate. Pelo gesto real, e o que chega ao
//! barramento é o MESMO do painel de antes. Nenhum motor, preset ou pincel muda.

use super::h1_os_cartoes::{Mesa, PAINEL, esc, pincel, saiu_de_proposito, valor_enviado};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::grip_of;
use ph2d_host::PointerKind;
use ph2d_panel_painter_layers::{
    VistaPreset, VistaPresets, set_current_wet_presets, set_current_wet_preview,
};
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, PaintMedia};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) const ROUND: &str = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
pub(super) const WASH: &str = "WetPainter/Watercolor/Fluid/Wet Wash.ph2d-brush.png";

fn vista(ativo: Option<&str>) -> Arc<VistaPresets> {
    let e = |rel: &str, nome: &str| VistaPreset {
        relativo: rel.to_owned(),
        nome: nome.to_owned(),
        categoria: "Watercolor".to_owned(),
        favorito: rel == ROUND,
        apagavel: rel == WASH,
    };
    Arc::new(VistaPresets {
        entradas: vec![e(ROUND, "Wet Round"), e(WASH, "Wet Wash")],
        ativo: ativo.map(str::to_owned),
        politica: 1,
        ilegiveis: 0,
        ordem_manual: true,
    })
}

/// Os presets e a prévia são do fio: cada teste semeia e limpa os dele.
pub(super) struct ComPresets;

impl ComPresets {
    pub(super) fn com(ativo: Option<&str>) -> Self {
        set_current_wet_presets(Some(vista(ativo)));
        Self
    }
}

impl Drop for ComPresets {
    fn drop(&mut self) {
        set_current_wet_presets(None);
        set_current_wet_preview(None);
    }
}

pub(super) fn previa() {
    let (l, a) = (8u32, 4u32);
    let px = vec![200u8; (l * a * 4) as usize];
    set_current_wet_preview(Some((Arc::new(px), l, a)));
}

const IDS_DA_FATIA: [NodeId; 20] = [
    p::PAINTER_BRUSH_MEDIA,
    p::PAINTER_WETPAINT_PRESET,
    p::PAINTER_WETPAINT_PRESET_FAVORITE,
    p::PAINTER_WETPAINT_PRESET_SAVE_NEW,
    p::PAINTER_WETPAINT_PRESET_SAVE_DEFAULT,
    p::PAINTER_WETPAINT_PRESET_RESET,
    p::PAINTER_WETPAINT_PRESET_IMPORT,
    p::PAINTER_WETPAINT_PRESET_EXPORT,
    p::PAINTER_WETPAINT_PRESET_RESET_ORDER,
    p::PAINTER_WETPAINT_PRESET_POLICY,
    p::PAINTER_BRUSH_SYNC,
    p::PAINTER_BRUSH_BLEND,
    p::PAINTER_BRUSH_BLEND_ON_EMPTY,
    core_ids::PAINTER_COLOR_THUMB,
    // O Size é a caixa «Size (px)» em todo meio (dono 08/11).
    p::PAINTER_WETPAINT_VOL_SIZE,
    p::PAINTER_BRUSH_STRENGTH_CHIP,
    p::PAINTER_BRUSH_FLOW_CHIP,
    p::PAINTER_BRUSH_ACCUMULATE,
    p::PAINTER_BRUSH_WASH,
    p::PAINTER_BRUSH_SMEAR_FIO,
];

/// ⭐ As mesmas condições, nas DUAS direcções: em cada ferramenta e meio, estes ids aparecem onde apareciam, e só aí.
#[test]
fn the_top_and_the_basics_show_the_same_controls_in_each_tool_and_medium() {
    let _p = ComPresets::com(Some(WASH));
    let casos = [
        (PaintMedia::Digital, "brush"),
        (PaintMedia::Watercolor, "brush"),
        (PaintMedia::Impasto, "brush"),
        (PaintMedia::WetPaint, "brush"),
        (PaintMedia::Digital, "eraser"),
        (PaintMedia::Digital, "blur"),
        (PaintMedia::Digital, "smear"),
        (PaintMedia::Digital, "clone"),
        (PaintMedia::Digital, "mask"),
        (PaintMedia::Digital, "inpaint"),
        (PaintMedia::Digital, "sculpt"),
    ];
    let mut vistos = BTreeSet::new();
    for (meio, modo) in casos {
        let (brush, _) = pincel(meio, modo);
        let antes = Mesa::nova(false, brush, false).ids();
        let agora = Mesa::nova(true, brush, false).ids();
        for id in IDS_DA_FATIA {
            assert_eq!(
                antes.contains(&id),
                agora.contains(&id),
                "{modo} {meio:?}: {id:?} antes {} agora {}",
                antes.contains(&id),
                agora.contains(&id)
            );
            if agora.contains(&id) {
                vistos.insert(id);
            }
        }
        let faltam: Vec<_> = antes
            .iter()
            .filter(|i| !agora.contains(i) && !saiu_de_proposito(**i))
            .collect();
        assert!(faltam.is_empty(), "{modo} {meio:?}: sumiram {faltam:?}");
    }
    assert_eq!(vistos.len(), IDS_DA_FATIA.len(), "a régua não viu todos");
}

/// O mesmo clique dá a mesma acção.
#[test]
fn every_click_of_the_top_and_basics_sends_the_same_action() {
    let _p = ComPresets::com(Some(WASH));
    let (digital, _) = pincel(PaintMedia::Digital, "brush");
    let (smear, _) = pincel(PaintMedia::Digital, "smear");
    let casos = [
        (digital, p::PAINTER_WETPAINT_PRESET_FAVORITE),
        (digital, p::PAINTER_WETPAINT_PRESET_SAVE_NEW),
        (digital, p::PAINTER_WETPAINT_PRESET_SAVE_DEFAULT),
        (digital, p::PAINTER_WETPAINT_PRESET_RESET),
        (digital, p::PAINTER_WETPAINT_PRESET_IMPORT),
        (digital, p::PAINTER_WETPAINT_PRESET_EXPORT),
        (digital, p::PAINTER_WETPAINT_PRESET_RESET_ORDER),
        (digital, p::PAINTER_BRUSH_SYNC),
        (digital, p::PAINTER_BRUSH_ACCUMULATE),
        (digital, p::PAINTER_BRUSH_WASH),
        (smear, p::PAINTER_BRUSH_SMEAR_FIO),
    ];
    for (brush, id) in casos {
        let velho = Mesa::nova(false, brush, false).clica(id);
        let novo = Mesa::nova(true, brush, false).clica(id);
        assert!(!novo.is_empty(), "{id:?}: nada no barramento");
        assert_eq!(velho, novo, "{id:?}");
    }
    // O Delete abre a janela de confirmação: o mesmo botão dela aparece nas duas caras.
    let abre = |cartoes: bool| {
        let mut m = Mesa::nova(cartoes, digital, false);
        let _ = m.clica(p::PAINTER_WETPAINT_PRESET_DELETE);
        m.assenta();
        let [apaga, cancela, ..] = p::PAINTER_WETPAINT_PRESET_DELETE_DIALOG;
        let aberta = m.rect(apaga).is_some();
        let _ = m.clica(cancela);
        aberta
    };
    assert_eq!(abre(false), abre(true), "a janela do Delete");
    assert!(abre(true), "o Delete não abriu a janela");
}

/// Cada lista abre debaixo da caixa dela, e a opção envia o mesmo.
#[test]
fn each_list_opens_under_its_box_and_its_option_sends_the_same() {
    let _p = ComPresets::com(Some(ROUND));
    let (digital, _) = pincel(PaintMedia::Digital, "brush");
    let casos = [
        (p::PAINTER_BRUSH_MEDIA, p::painter_brush_media_option_id(1)),
        (p::PAINTER_BRUSH_BLEND, p::painter_brush_blend_option_id(3)),
        (
            p::PAINTER_WETPAINT_PRESET,
            p::painter_wetpaint_preset_option_id(2, WASH),
        ),
        (
            p::PAINTER_WETPAINT_PRESET_POLICY,
            p::painter_wetpaint_policy_option_id(2),
        ),
    ];
    for (caixa, opcao) in casos {
        let escolhe = |cartoes: bool| {
            let mut mesa = Mesa::nova(cartoes, digital, false);
            let chip = mesa.rect(caixa).expect("a caixa");
            let _ = mesa.clica(caixa);
            mesa.assenta();
            let r = mesa.rect(opcao).expect("a opção aberta");
            assert!(
                r.y >= chip.y && r.x < chip.x + chip.w && r.x + r.w > chip.x,
                "{caixa:?} abriu longe: {chip:?} → {r:?}"
            );
            mesa.clica(opcao)
        };
        let (velho, novo) = (escolhe(false), escolhe(true));
        assert!(!novo.is_empty(), "{caixa:?}: nada no barramento");
        assert_eq!(velho, novo, "{caixa:?}");
    }
}

/// Strength e Flow: o scrub chega pela barra de sempre, só ao seu campo, e o Esc devolve. (O Size é a caixa
/// «Size (px)» desde 08/11, sem barra: `size_px_e_dab_fixo`.)
#[test]
fn scrubbing_a_basic_number_reaches_its_own_bar_and_escape_gives_it_back() {
    let (digital, _) = pincel(PaintMedia::Digital, "brush");
    let mut base: BrushSettings = digital;
    base.strength = 0.4;
    base.flow = 0.5;
    let trio = [
        (
            p::PAINTER_BRUSH_STRENGTH_SLIDER,
            p::PAINTER_BRUSH_STRENGTH_CHIP,
            0.4,
        ),
        (
            p::PAINTER_BRUSH_FLOW_SLIDER,
            p::PAINTER_BRUSH_FLOW_CHIP,
            0.5,
        ),
    ];
    for (barra, chip, semente) in trio {
        let mut mesa = Mesa::nova(true, base, false);
        let antes = mesa.host.store().number_value(chip).expect("semeado");
        assert!((antes - semente).abs() < 1e-3, "{chip:?} semente {antes}");
        let ev = mesa.arrasta_a_pega(chip, 40.0);
        let no_gesto = mesa.host.store().number_value(chip).expect("campo");
        assert!((no_gesto - antes).abs() > 1e-3, "{chip:?} não se moveu");
        let accoes = mesa.aplica(ev);
        let enviado = valor_enviado(&accoes, barra).expect("a barra avisa o painel");
        assert!((enviado - no_gesto).abs() < 1e-3, "{enviado} × {no_gesto}");
        for (_, outro, s) in trio.iter().filter(|t| t.1 != chip) {
            let v = mesa.host.store().number_value(*outro).expect("campo");
            assert!((v - s).abs() < 1e-3, "{outro:?} mexeu-se: {v}");
        }
        // Um quadro com o pincel ainda antigo: a semente não puxa o número de volta.
        mesa.quadro();
        let a_meio = mesa.host.store().number_value(chip).expect("campo");
        assert!(
            (a_meio - no_gesto).abs() < 1e-6,
            "a semente puxou: {a_meio}"
        );
        let ao_soltar = esc(&mut mesa);
        let voltou = mesa.host.store().number_value(chip).expect("campo");
        assert!((voltou - antes).abs() < 1e-6, "o Esc deu {voltou}");
        let de_volta = valor_enviado(&ao_soltar, barra).expect("o Esc avisa a barra");
        assert!(
            (de_volta - antes).abs() < 1e-3,
            "a barra ficou em {de_volta}"
        );
    }
}

/// Escrever nos números manda o mesmo número pela mesma barra.
#[test]
fn typing_a_basic_number_sends_the_same_action_as_the_old_panel() {
    let (digital, _) = pincel(PaintMedia::Digital, "brush");
    for (chip, texto) in [
        (p::PAINTER_BRUSH_SIZE_CHIP, "0.6"),
        (p::PAINTER_BRUSH_STRENGTH_CHIP, "0.7"),
        (p::PAINTER_BRUSH_FLOW_CHIP, "0.8"),
        (p::PAINTER_BRUSH_BLEND_ON_EMPTY, "1"),
    ] {
        let escreve = |cartoes: bool| {
            let mut mesa = Mesa::nova(cartoes, digital, false);
            let ev = mesa.host.type_into_number(chip, texto);
            mesa.aplica(ev)
        };
        let (velho, novo) = (escreve(false), escreve(true));
        assert!(!novo.is_empty(), "{chip:?}: nada no barramento");
        assert_eq!(velho, novo, "{chip:?}");
    }
}

/// Cada caixa mostra o valor do pincel de agora, e um pincel novo muda-a.
#[test]
fn the_boxes_follow_the_brush() {
    let (mut b, _) = pincel(PaintMedia::Digital, "brush");
    b.size_px = 12.5;
    b.strength = 0.35;
    b.flow = 0.45;
    b.blend_sobre_vazio = true;
    let mut mesa = Mesa::nova(true, b, false);
    let le = |m: &Mesa, id| m.host.store().number_value(id).expect("semeado");
    // O «Size (px)» mostra o diâmetro: raio 12,5 → 25 px.
    assert!((le(&mesa, p::PAINTER_WETPAINT_VOL_SIZE) - 25.0).abs() < 1e-3);
    assert!((le(&mesa, p::PAINTER_BRUSH_STRENGTH_CHIP) - 0.35).abs() < 1e-3);
    assert!((le(&mesa, p::PAINTER_BRUSH_FLOW_CHIP) - 0.45).abs() < 1e-3);
    assert!((le(&mesa, p::PAINTER_BRUSH_BLEND_ON_EMPTY) - 1.0).abs() < 1e-3);
    b.flow = 0.9;
    ph2d_panel_painter_layers::state::set_current_brush(Some(b));
    mesa.quadro();
    assert!((le(&mesa, p::PAINTER_BRUSH_FLOW_CHIP) - 0.9).abs() < 1e-3);
}

/// A faixa do campo é a da barra: o scrub não passa de 1, e o `On empty` só tem dois valores.
#[test]
fn the_boxes_keep_the_ranges_of_the_old_panel() {
    let (mut b, _) = pincel(PaintMedia::Digital, "brush");
    b.flow = 0.95;
    let mut mesa = Mesa::nova(true, b, false);
    let ev = mesa.arrasta_a_pega(p::PAINTER_BRUSH_FLOW_CHIP, 4000.0);
    let _ = mesa.aplica(ev);
    let v = mesa
        .host
        .store()
        .number_value(p::PAINTER_BRUSH_FLOW_CHIP)
        .expect("campo");
    assert!(v <= 1.0 + 1e-6, "passou da faixa: {v}");
    let mut mesa = Mesa::nova(true, b, false);
    let ev = mesa.arrasta_a_pega(p::PAINTER_BRUSH_BLEND_ON_EMPTY, 4000.0);
    let _ = mesa.aplica(ev);
    let v = mesa
        .host
        .store()
        .number_value(p::PAINTER_BRUSH_BLEND_ON_EMPTY)
        .expect("campo");
    assert!((v - 1.0).abs() < 1e-6, "On empty foi a {v}");
}

/// A amostra abre o seletor partilhado; com a cor por camada, vê-se e não tem alvo, e o seletor aberto fecha.
#[test]
fn the_colour_box_opens_the_picker_and_per_layer_colour_turns_it_off() {
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let id = core_ids::PAINTER_COLOR_THUMB;
    let abre = |cartoes: bool| {
        let mut mesa = Mesa::nova(cartoes, b, false);
        let accoes = mesa.clica(id);
        (accoes, mesa.host.store().picker_target())
    };
    let (velho, novo) = (abre(false), abre(true));
    assert_eq!(velho, novo);
    assert_eq!(novo.1, Some(id), "o seletor não abriu");

    let mut inativa = b;
    inativa.color_inactive = true;
    let mut mesa = Mesa::nova(true, inativa, false);
    assert!(mesa.rect(id).is_none(), "a cor inativa continua clicável");
    mesa.host.store_mut().set_picker_target(Some(id));
    mesa.quadro();
    assert_eq!(mesa.host.store().picker_target(), None);
    let desenho = |brush: BrushSettings| {
        let mut m = Mesa::nova(true, brush, false);
        let (h, st) = (&mut m.host, &mut m.st);
        h.paint_draw_data::<ph2d_panel_painter_layers::PainterLayersPanel>(st, m.janela)
    };
    assert_ne!(desenho(b), desenho(inativa), "a cor inativa não se esmaece");
}

/// O seletor aberto na amostra conduz o pincel ao vivo, como antes; fechado, a amostra segue o pincel.
#[test]
fn the_open_picker_drives_the_brush_colour() {
    let (mut b, _) = pincel(PaintMedia::Digital, "brush");
    b.color = [0.0, 0.0, 1.0];
    let id = core_ids::PAINTER_COLOR_THUMB;
    let conduz = |cartoes: bool| {
        let mut mesa = Mesa::nova(cartoes, b, false);
        assert_eq!(
            mesa.host.store().widget_color(id),
            Some([0, 0, 255, 255]),
            "a amostra não segue o pincel"
        );
        let _ = mesa.host.drained_actions();
        let store = mesa.host.store_mut();
        store.set_picker_target(Some(id));
        store.set_widget_color(id, [10, 20, 30, 255]);
        mesa.quadro();
        let accoes: Vec<String> = mesa
            .host
            .drained_actions()
            .iter()
            .map(|a| format!("{a:?}"))
            .collect();
        assert_eq!(
            mesa.host.store().widget_color(id),
            Some([10, 20, 30, 255]),
            "o pincel antigo pisou o seletor"
        );
        accoes
    };
    let (velho, novo) = (conduz(false), conduz(true));
    assert!(novo.iter().any(|a| a.contains("10,20,30")), "{novo:?}");
    assert_eq!(velho, novo);
}

/// A cor da caixa é a do pincel: trocar a cor muda o desenho, e o código.
#[test]
fn the_colour_box_shows_the_brush_colour() {
    let (mut b, _) = pincel(PaintMedia::Digital, "brush");
    b.color = [1.0, 0.0, 0.0];
    let desenho = |brush: BrushSettings| {
        let mut m = Mesa::nova(true, brush, false);
        let (h, st) = (&mut m.host, &mut m.st);
        h.paint_draw_data::<ph2d_panel_painter_layers::PainterLayersPanel>(st, m.janela)
    };
    let vermelho = desenho(b);
    assert!(
        vermelho.contains(&0xff00_00ff),
        "o vermelho do pincel não está na caixa"
    );
    b.color = [0.0, 1.0, 0.0];
    assert!(
        desenho(b).contains(&0xff00_ff00),
        "o verde não está na caixa"
    );
}

/// As caixas da lei esmaecem e continuam editáveis; a dica diz o que as acorda, e sai quando acordam.
#[test]
fn a_box_with_a_dimming_law_dims_and_keeps_working() {
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_tool_painter::{Inercias, PainterTool};
    let mut tool = PainterTool::default();
    let mut estados = vec![tool.brush_settings()];
    tool.handle_panel_event(PanelEvent::Click(p::PAINTER_BRUSH_ACCUMULATE));
    estados.push(tool.brush_settings());
    tool.handle_panel_event(PanelEvent::SetValue(p::PAINTER_BRUSH_STRENGTH_SLIDER, 1.0));
    estados.push(tool.brush_settings());
    for id in [
        p::PAINTER_BRUSH_ACCUMULATE,
        p::PAINTER_BRUSH_WASH,
        p::PAINTER_BRUSH_BLEND_ON_EMPTY,
    ] {
        let apagada = *estados
            .iter()
            .find(|b| b.inercias.dica_de(id).is_some())
            .unwrap_or_else(|| panic!("{id:?}: nenhum estado o deixa inerte"));
        let mut acesa = apagada;
        acesa.inercias = Inercias::NENHUMA;
        let desenho = |b: BrushSettings, dica: bool| {
            let mut m = Mesa::nova(true, b, false);
            assert_eq!(
                m.host
                    .store()
                    .tooltip_for(id)
                    .is_some_and(|t| !t.is_empty()),
                dica,
                "{id:?}: dica"
            );
            assert!(m.rect(id).is_some(), "{id:?} deixou de ter alvo");
            // Só este cartão: a camada da linha esmaecida é o que acrescenta entradas.
            m.so_abertos(&[p::PAINTER_CARD_BASICOS]);
            let (h, st) = (&mut m.host, &mut m.st);
            h.paint_draw_data::<ph2d_panel_painter_layers::PainterLayersPanel>(st, m.janela)
        };
        let viva = desenho(acesa, false);
        let morta = desenho(apagada, true);
        assert!(
            morta.len() > viva.len(),
            "{id:?} não esmaeceu: {} × {}",
            morta.len(),
            viva.len()
        );
    }
}

/// Nenhum outro id destes cartões tem linha esmaecida: o dia em que ganhar, o corpo tem de a pintar.
#[test]
fn no_other_native_id_of_the_top_has_a_dimmed_law() {
    let com_lei = [
        p::PAINTER_BRUSH_ACCUMULATE,
        p::PAINTER_BRUSH_WASH,
        p::PAINTER_BRUSH_BLEND_ON_EMPTY,
    ];
    let outros = IDS_DA_FATIA
        .iter()
        .chain([
            &p::PAINTER_BRUSH_SIZE_SLIDER,
            &p::PAINTER_BRUSH_STRENGTH_SLIDER,
            &p::PAINTER_BRUSH_FLOW_SLIDER,
            &p::PAINTER_WETPAINT_PRESET_DELETE,
        ])
        .filter(|i| !com_lei.contains(i));
    for id in outros {
        for d in ph2d_tool_painter::Dependente::TODOS {
            assert!(!d.cobre(*id), "{id:?} ganhou a lei {d:?}: pinte-a no corpo");
        }
    }
}

/// A prévia continua viva dentro do cartão do topo: o pino é onde a prévia diz, e fica fora dele quando fixada.
#[test]
fn the_preview_is_a_piece_of_the_top_and_the_pin_is_where_it_says() {
    let _p = ComPresets::com(Some(ROUND));
    previa();
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mesa = Mesa::nova(true, b, false);
    let pino = mesa.rect(p::PAINTER_BRUSH_PREVIEW_PIN).expect("o pino");
    let onde = ph2d_panel_painter_layers::rect_da_preview().expect("a prévia");
    let certo = ph2d_panel_painter_layers::rect_do_pino(onde);
    assert!(
        (pino.x - certo.x).abs() < 1.0 && (pino.y - certo.y).abs() < 1.0,
        "{pino:?} × {certo:?}"
    );
    let chip = mesa.rect(p::PAINTER_WETPAINT_PRESET).expect("o Preset");
    let guardar = mesa
        .rect(p::PAINTER_WETPAINT_PRESET_SAVE_NEW)
        .expect("Save");
    assert!(
        onde.y + onde.h <= chip.y && chip.y + chip.h <= guardar.y,
        "a prévia saiu do lugar: {chip:?} {onde:?} {guardar:?}"
    );
}

/// Os volumes do Wet Paint continuam lá, por capturar até à fatia deles: mesmo clique, mesmo valor.
#[test]
fn the_wet_volumes_are_still_there_and_send_the_same() {
    let (b, _) = pincel(PaintMedia::WetPaint, "brush");
    let id = p::PAINTER_WETPAINT_DILUTION;
    let escreve = |cartoes: bool| {
        let mut mesa = Mesa::nova(cartoes, b, false);
        assert!(mesa.rect(p::PAINTER_WETPAINT_VOL_SIZE).is_some(), "Size px");
        let ev = mesa.host.type_into_number(id, "0.4");
        mesa.aplica(ev)
    };
    let (velho, novo) = (escreve(false), escreve(true));
    assert!(!novo.is_empty());
    assert_eq!(velho, novo);
}

/// O cartão Basics fica de fora onde não há nenhuma das suas linhas; e a pega do topo é a do id de sempre.
#[test]
fn the_cards_keep_their_ids() {
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mesa = Mesa::nova(true, b, false);
    assert!(mesa.rect(grip_of(p::PAINTER_CARD_TOPO)).is_some());
    assert!(mesa.rect(grip_of(p::PAINTER_CARD_BASICOS)).is_some());
    assert!(mesa.host.store().panel_rect(PAINEL).is_some());
}

/// Com o pino, a prévia mora na faixa presa, por cima dos cartões, e a faixa tem a altura dela.
#[test]
fn a_pinned_preview_sits_in_the_strip_above_the_cards() {
    let _p = ComPresets::com(Some(ROUND));
    previa();
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mut mesa = Mesa::nova(true, b, false);
    let solta = mesa.rect(grip_of(p::PAINTER_CARD_TOPO)).expect("a pega").y;
    let _ = mesa.clica(p::PAINTER_BRUSH_PREVIEW_PIN);
    mesa.assenta();
    assert!(mesa.rect(p::PAINTER_BRUSH_PREVIEW_PIN).is_some(), "o pino");
    let onde = ph2d_panel_painter_layers::rect_da_preview().expect("a prévia");
    let topo = mesa.rect(grip_of(p::PAINTER_CARD_TOPO)).expect("a pega").y;
    assert!(topo > solta + 1.0, "a faixa não cresceu: {solta} → {topo}");
    assert!(
        onde.y + onde.h <= topo,
        "a prévia ficou por cima: {onde:?} × {topo}"
    );
}

/// A voar, o cartão do topo não muda a coluna que os pintores usam: os rects guardados continuam no ecrã.
#[test]
fn a_flying_card_does_not_move_what_the_painters_remember() {
    let _p = ComPresets::com(Some(ROUND));
    previa();
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mut mesa = Mesa::nova(true, b, false);
    let antes = ph2d_panel_painter_layers::rect_da_preview().expect("a prévia");
    let g = mesa.rect(grip_of(p::PAINTER_CARD_TOPO)).expect("a pega");
    let (x, y0) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    for i in 1..=15 {
        let _ = mesa.ponteiro(PointerKind::Move, x, y0 + (i * 20) as f32);
        mesa.quadro();
    }
    assert!(
        mesa.rect(p::PAINTER_BRUSH_SYNC).is_none(),
        "o cartão não está a voar"
    );
    let a_voar = ph2d_panel_painter_layers::rect_da_preview().expect("a prévia");
    assert!(
        (a_voar.x - antes.x).abs() < 1.0,
        "a coluna foi com o voo: {antes:?} → {a_voar:?}"
    );
    let _ = mesa.ponteiro(PointerKind::Up, x, y0 + 300.0);
}
