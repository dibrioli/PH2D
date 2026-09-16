//! ⭐⭐⭐ **A PILHA DAS CAMADAS DE AJUSTE: cada barra é uma CAIXA ÚNICA com o número na unidade do
//! artista, e cada nome sai da TABELA de strings.**
//!
//! ⛔⛔ Até 2026-09-16 a barra era `nome | trilho nu`: o nome numa coluna literal de `44 px` pintada
//! em `Base` (**17 dos 44** nomes cortados em toda largura, o `Contrast` do próprio comentário
//! incluído) e **nenhum número** — pôr o contraste em `+25` era arrastar até parecer certo. E os
//! nomes vinham da crate de efeitos em inglês abreviado (`Shad Amt`, `Preserve Lum.`), invisíveis ao
//! censo de texto do painel porque o literal mora do outro lado da fronteira.
//!
//! ⚠️ **As três perguntas são três gates**, porque são três defeitos diferentes:
//!
//! 1. **o número ESCREVE o que diz** — o chip projecta pela unidade do slot, e uma unidade errada
//!    edita o valor errado em silêncio. O ORÁCULO é a leitura dos campos do ajuste, escrita aqui à
//!    mão (⛔ nunca pela mesma unidade: um gate que compara duas construções é cego a uma mutação
//!    partilhada — a lição que o gate irmão do card Line pagou);
//! 2. **cada rótulo tem nome** na tabela, e os nomes de uma pilha são distintos;
//! 3. **o nome do interruptor cabe na coluna** dele, pela escada do dock.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::adjust_nomes;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_dock_shows_layers, set_current_layers,
};
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};
use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_tool_painter::{
    AdjustmentKind, AdjustmentParams, LayerId, LayerKind, PainterTool, SliderNumber,
};
use ph2d_ui_testkit::MockPanelHost;

const BARRAS: [PainterLayerWidget; 8] = [
    PainterLayerWidget::AdjParam0,
    PainterLayerWidget::AdjParam1,
    PainterLayerWidget::AdjParam2,
    PainterLayerWidget::AdjParam3,
    PainterLayerWidget::AdjParam4,
    PainterLayerWidget::AdjParam5,
    PainterLayerWidget::AdjParam6,
    PainterLayerWidget::AdjParam7,
];

const CHIPS: [PainterLayerWidget; 8] = [
    PainterLayerWidget::AdjChip0,
    PainterLayerWidget::AdjChip1,
    PainterLayerWidget::AdjChip2,
    PainterLayerWidget::AdjChip3,
    PainterLayerWidget::AdjChip4,
    PainterLayerWidget::AdjChip5,
    PainterLayerWidget::AdjChip6,
    PainterLayerWidget::AdjChip7,
];

/// Os rótulos e os números de uma pilha, das MESMAS funções que o pintor lê, com o separador activo
/// no de omissão (o primeiro).
fn pilha(params: &AdjustmentParams) -> (Vec<&'static str>, Vec<SliderNumber>) {
    let (barras, numeros) = match params {
        AdjustmentParams::ChannelMixer(m) => (
            ph2d_tool_painter::channel_mixer_slider_params(m, 0),
            ph2d_tool_painter::channel_mixer_slider_numbers().to_vec(),
        ),
        AdjustmentParams::SelectiveColor(s) => (
            ph2d_tool_painter::selective_color_slider_params(s, 0),
            ph2d_tool_painter::selective_color_slider_numbers().to_vec(),
        ),
        AdjustmentParams::GradientMap(g) => (
            ph2d_tool_painter::gradient_stop_color_params(g, 0),
            ph2d_tool_painter::gradient_stop_color_numbers().to_vec(),
        ),
        p => (
            ph2d_tool_painter::adjustment_slider_params(p),
            ph2d_tool_painter::adjustment_slider_numbers(p),
        ),
    };
    (barras.into_iter().map(|(n, _)| n).collect(), numeros)
}

/// A chave de um rótulo de barra, pela família que o pintor usa.
fn chave(params: &AdjustmentParams, fonte: &str) -> Option<&'static str> {
    match params {
        AdjustmentParams::ChannelMixer(_) => adjust_nomes::chave_do_misturador(fonte),
        AdjustmentParams::SelectiveColor(_) => adjust_nomes::chave_da_seletiva(fonte),
        AdjustmentParams::GradientMap(_) => adjust_nomes::chave_do_gradiente(fonte),
        p => adjust_nomes::chave_da_barra(p, fonte),
    }
}

/// Uma ferramenta com UMA camada de ajuste de `especie` — com o tinte ligado no preto-e-branco,
/// para as duas barras que só existem com ele.
fn ferramenta(especie: AdjustmentKind) -> (PainterTool, LayerId) {
    let mut tool = PainterTool::default();
    let id = tool
        .add_adjustment_layer(especie)
        .expect("a camada de ajuste nasce");
    if especie == AdjustmentKind::BlackAndWhite {
        tool.flip_adjustment_toggle(id, 0);
    }
    (tool, id)
}

fn params_de(tool: &PainterTool, id: LayerId) -> AdjustmentParams {
    match &tool.layers().get(id).expect("a camada existe").kind {
        LayerKind::Adjustment(adj) => adj.params.clone(),
        _ => panic!("não é uma camada de ajuste"),
    }
}

/// Pinta o painel com a pilha da ferramenta, num dock de `dock` px.
fn pintar(
    tool: &PainterTool,
    dock: f32,
) -> (
    MockPanelHost,
    PainterLayersPanelState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    set_current_dock_shows_layers(true);
    set_current_layers(Some(tool.layers().clone()));
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let layout = HeroLayout::for_viewport_bands(
        viewport,
        false,
        ChromeBands {
            right_dock_w: dock,
            ..ChromeBands::DEFAULT
        },
        CenterSplit::None,
        DockSides::BOTH,
    );
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint_with_layout::<PainterLayersPanel>(&mut st, layout, viewport);
    (host, st, rects)
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// ⛔⛔ **O ORÁCULO — o valor do slot na unidade do artista, lido dos CAMPOS do ajuste.**
///
/// Escrito à mão e de propósito: ele não usa a unidade da crate de efeitos. `%` = fracção × 100,
/// `°` = voltas × 360 ou radianos em graus, `px` e níveis como estão.
#[allow(clippy::too_many_lines)]
fn na_unidade_do_artista(params: &AdjustmentParams, slot: usize) -> f32 {
    use AdjustmentParams as P;
    let pct = |v: f32| v * 100.0;
    match (params, slot) {
        (P::HueSaturationBrightness(p), 0) => p.h * 360.0,
        (P::HueSaturationBrightness(p), 1) => pct(p.s),
        (P::HueSaturationBrightness(p), 2) => pct(p.b),
        (P::BrightnessContrast(p), 0) => pct(p.brightness),
        (P::BrightnessContrast(p), 1) => pct(p.contrast),
        (P::Exposure(p), 0) => p.exposure_ev,
        (P::Exposure(p), 1) => p.offset,
        (P::Exposure(p), 2) => 1.0 + p.gamma_correction,
        (P::Vibrance(p), 0) => pct(p.vibrance),
        (P::Vibrance(p), 1) => pct(p.saturation),
        (P::Posterize(p), 0) => f32::from(p.levels),
        (P::Threshold(p), 0) => f32::from(p.threshold),
        (P::PhotoFilter(p), 0) => pct(p.temperature),
        (P::PhotoFilter(p), 1) => pct(p.density),
        (P::ColorBalance(p), 0) => pct(p.cyan_red),
        (P::ColorBalance(p), 1) => pct(p.magenta_green),
        (P::ColorBalance(p), 2) => pct(p.yellow_blue),
        (P::BlackAndWhite(p), s @ 0..=5) => {
            pct([p.reds, p.yellows, p.greens, p.cyans, p.blues, p.magentas][s])
        }
        (P::BlackAndWhite(p), 6) => p.tint_color.map_or(f32::NAN, |t| t.h),
        (P::BlackAndWhite(p), 7) => pct(p.tint_amount),
        (P::Levels(p), 0) => p.black_point * 255.0,
        (P::Levels(p), 2) => p.white_point * 255.0,
        (P::Levels(p), 3) => p.output_black * 255.0,
        (P::Levels(p), 4) => p.output_white * 255.0,
        (P::GaussianBlur(p), 0) => p.radius,
        (P::MotionBlur(p), 0) => p.distance,
        (P::MotionBlur(p), 1) => p.angle.to_degrees(),
        (P::Sharpen(p), 0) => pct(p.amount),
        (P::Sharpen(p), 1) => p.radius,
        (P::ChromaticAberration(p), 0) => p.red_shift,
        (P::ChromaticAberration(p), 1) => p.green_shift,
        (P::ChromaticAberration(p), 2) => p.blue_shift,
        (P::Noise(p), 0) => pct(p.amount),
        (P::Halftone(p), 0) => p.dot_size,
        (P::Halftone(p), 1) => p.angle.to_degrees(),
        #[allow(clippy::cast_precision_loss)]
        (P::ColorLookupLut(p), 0) => p.lut_3d.0 as f32,
        (P::ColorLookupLut(p), 1) => pct(p.intensity),
        (P::Bloom(p), 0) => pct(p.threshold),
        (P::Bloom(p), 1) => pct(p.intensity),
        (P::Bloom(p), 2) => p.radius,
        (P::Bloom(p), 3) => pct(p.falloff),
        (P::ShadowsHighlights(p), 0) => pct(p.shadows_amount),
        (P::ShadowsHighlights(p), 1) => pct(p.shadows_tonal_width),
        (P::ShadowsHighlights(p), 2) => p.shadows_radius,
        (P::ShadowsHighlights(p), 3) => pct(p.highlights_amount),
        (P::ShadowsHighlights(p), 4) => pct(p.highlights_tonal_width),
        (P::ShadowsHighlights(p), 5) => p.highlights_radius,
        (P::ShadowsHighlights(p), 6) => pct(p.color_correction),
        (P::ShadowsHighlights(p), 7) => pct(p.midtone_contrast),
        // As pilhas próprias, no separador de omissão (saída vermelha · grupo dos vermelhos · 1.ª
        // paragem).
        (P::ChannelMixer(p), s @ 0..=3) => pct(p.red_out[s]),
        (P::SelectiveColor(p), 0) => pct(p.reds.cyan),
        (P::SelectiveColor(p), 1) => pct(p.reds.magenta),
        (P::SelectiveColor(p), 2) => pct(p.reds.yellow),
        (P::SelectiveColor(p), 3) => pct(p.reds.black),
        (P::GradientMap(p), s @ 0..=2) => f32::from(p.stops[0].color[s]),
        (p, s) => panic!("o oráculo não conhece o slot {s} de {p:?}"),
    }
}

/// ⭐⭐⭐ **O número de cada barra escreve o que diz.**
///
/// Digita no chip de cada barra afim um valor que NÃO é o default nem um extremo (onde um `clamp`
/// esconderia uma unidade errada), drena até à ferramenta pelo caminho do produto e lê o campo pelo
/// oráculo.
#[test]
fn o_numero_de_cada_barra_de_ajuste_escreve_o_que_diz() {
    let mut provadas = 0usize;
    let mut so_leitura = 0usize;
    for especie in AdjustmentKind::ALL {
        if especie == AdjustmentKind::Curves {
            continue;
        }
        let (tool0, id) = ferramenta(especie);
        let (_, numeros) = pilha(&params_de(&tool0, id));
        for (slot, numero) in numeros.iter().enumerate() {
            let SliderNumber::Affine {
                scale,
                offset,
                integer,
            } = *numero
            else {
                so_leitura += 1;
                continue;
            };
            let (mut tool, id) = ferramenta(especie);
            let (mut host, mut st, rects) = pintar(&tool, 304.0);
            let chip = painter_layer_widget_id(id.0, CHIPS[slot]);
            assert!(
                rect_de(&rects, chip).is_some(),
                "{especie:?} slot {slot}: o número editável não é pintado"
            );
            let alvo = if integer {
                offset + (0.37 * scale).round()
            } else {
                offset + 0.37 * scale
            };
            for ev in host.type_into_number(chip, &format!("{alvo}")) {
                host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
            }
            for action in host.drained_actions() {
                if let EditorAction::ToolPanelEvent(pe) = action {
                    tool.handle_panel_event(pe);
                }
            }
            set_current_layers(None);
            let gravado = na_unidade_do_artista(&params_de(&tool, id), slot);
            assert!(
                (gravado - alvo).abs() <= 1e-3 * scale.abs().max(1.0),
                "{especie:?} slot {slot}: digitei {alvo} e o ajuste ficou com {gravado}"
            );
            provadas += 1;
        }
    }
    // Piso de população: as 22 pilhas com barras somam 65 barras afins (medido 2026-09-16) (o Gamma do Levels é o
    // único número que só se mostra).
    assert!(
        provadas >= 65,
        "só {provadas} barras provadas — a varredura partiu-se"
    );
    assert_eq!(
        so_leitura, 1,
        "o número que só se mostra é o Gamma do Levels — mudou a população"
    );
}

/// ⭐⭐ **Cada barra é pintada, e o chip existe exactamente quando o número se escreve.**
#[test]
fn cada_barra_de_ajuste_e_uma_caixa_unica() {
    let mut barras = 0usize;
    for especie in AdjustmentKind::ALL {
        if especie == AdjustmentKind::Curves {
            continue;
        }
        let (tool, id) = ferramenta(especie);
        let (_, numeros) = pilha(&params_de(&tool, id));
        let (_h, _s, rects) = pintar(&tool, 304.0);
        set_current_layers(None);
        for (slot, numero) in numeros.iter().enumerate() {
            let slider = rect_de(&rects, painter_layer_widget_id(id.0, BARRAS[slot]))
                .unwrap_or_else(|| panic!("{especie:?} slot {slot}: a barra não foi pintada"));
            let chip = rect_de(&rects, painter_layer_widget_id(id.0, CHIPS[slot]));
            match numero {
                SliderNumber::Affine { .. } => {
                    let chip = chip.unwrap_or_else(|| {
                        panic!("{especie:?} slot {slot}: a barra não tem número editável")
                    });
                    assert!(
                        chip.y >= slider.y - 0.5 && chip.y + chip.h <= slider.y + slider.h + 0.5,
                        "{especie:?} slot {slot}: o número {chip:?} não está na caixa {slider:?}"
                    );
                }
                SliderNumber::Shown(_) => assert!(
                    chip.is_none(),
                    "{especie:?} slot {slot}: um número que não se escreve registou um chip"
                ),
            }
            barras += 1;
        }
    }
    assert!(barras >= 60, "só {barras} barras pintadas");
}

/// ⭐⭐ **Cada rótulo da crate de efeitos tem nome na tabela — e os nomes de uma pilha são
/// distintos.**
///
/// ⚠️ Varre também as saídas do misturador e os grupos da cor selectiva, porque os rótulos delas
/// são por separador.
#[test]
fn cada_rotulo_de_ajuste_tem_nome_na_tabela() {
    let mut vistos = 0usize;
    let mut expandidos = 0usize;
    for especie in AdjustmentKind::ALL {
        let (tool, id) = ferramenta(especie);
        let params = params_de(&tool, id);
        let (barras, numeros) = pilha(&params);
        assert_eq!(
            barras.len(),
            numeros.len(),
            "{especie:?}: a crate de efeitos dá {} nomes e {} números",
            barras.len(),
            numeros.len()
        );
        let mut nomes = Vec::new();
        for fonte in &barras {
            let k = chave(&params, fonte)
                .unwrap_or_else(|| panic!("{especie:?}: o rótulo {fonte:?} não tem chave"));
            let nome = ph2d_i18n::tr(k);
            assert_ne!(nome, k, "{especie:?}: a chave {k} não está declarada");
            if nome != *fonte {
                expandidos += 1;
            }
            nomes.push(nome);
            vistos += 1;
        }
        for fonte in ph2d_tool_painter::adjustment_toggle_params(&params)
            .iter()
            .map(|(f, _)| *f)
        {
            let k = adjust_nomes::chave_do_interruptor(fonte)
                .unwrap_or_else(|| panic!("{especie:?}: o interruptor {fonte:?} não tem chave"));
            nomes.push(ph2d_i18n::tr(k));
            vistos += 1;
        }
        if let Some((opcoes, _)) = ph2d_tool_painter::adjustment_segment_params(&params) {
            for fonte in opcoes {
                assert!(
                    adjust_nomes::chave_do_segmento(fonte).is_some(),
                    "{especie:?}: a opção {fonte:?} não tem chave"
                );
                vistos += 1;
            }
        }
        let mut unicos = nomes.clone();
        unicos.sort_unstable();
        unicos.dedup();
        assert_eq!(
            unicos.len(),
            nomes.len(),
            "{especie:?}: dois controlos da mesma pilha têm o mesmo nome: {nomes:?}"
        );
        let nome_da_especie = ph2d_i18n::tr(adjust_nomes::chave_da_especie(especie));
        assert_ne!(
            nome_da_especie,
            adjust_nomes::chave_da_especie(especie),
            "{especie:?}: o nome da espécie não está declarado"
        );
    }
    // As outras duas saídas do misturador e os outros oito grupos da cor selectiva.
    let mixer = ph2d_tool_painter::AdjustmentParams::neutral_for(AdjustmentKind::ChannelMixer);
    let sel = ph2d_tool_painter::AdjustmentParams::neutral_for(AdjustmentKind::SelectiveColor);
    if let (AdjustmentParams::ChannelMixer(m), AdjustmentParams::SelectiveColor(s)) = (&mixer, &sel)
    {
        for out in 0..3 {
            for (fonte, _) in ph2d_tool_painter::channel_mixer_slider_params(m, out) {
                assert!(adjust_nomes::chave_do_misturador(fonte).is_some());
            }
        }
        for bucket in 0..ph2d_tool_painter::SELCOLOR_BUCKETS.len() {
            for (fonte, _) in ph2d_tool_painter::selective_color_slider_params(s, bucket) {
                assert!(adjust_nomes::chave_da_seletiva(fonte).is_some());
            }
        }
    }
    assert!(vistos >= 80, "só {vistos} rótulos vistos");
    // ⚠️ O CONTROLO: a tabela não é a identidade — os nomes abreviados foram expandidos.
    assert!(
        expandidos >= 20,
        "só {expandidos} nomes diferem do rótulo da crate — a tabela está a devolver o rótulo cru"
    );
}

/// ⏳ **QUANTOS NOMES DE INTERRUPTOR ELIDEM, POR LARGURA DO DOCK — e só ENCOLHE.**
///
/// ⚠️ **Os dois do mínimo são o TECTO da porta, não uma falha nova:** `Keep Luminosity` (Color
/// Balance, Photo Filter, `95,4 px`) mede mais do que os `78,0` que a coluna tem a `220`, porque a
/// porta reserva ao controlo o piso de um CAMPO ([`ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX`]),
/// como a linha de marcar. A contagem é a mesma de antes desta onda (o `Preserve Lum.` abreviado
/// também cortava ali). ⛔ O nome do Photoshop, `Preserve Luminosity` (`117,3`), cortava TAMBÉM a
/// `245` — medido e trocado por palavras inteiras que cabem.
const INTERRUPTORES_ELIDEM: &[(f32, usize)] = &[
    (220.0, 2),
    (245.0, 0),
    // Amostra DATADA da largura do dono (lida em 2026-09-14).
    (273.3, 0),
    (304.0, 0),
    (720.0, 0),
];

/// ⭐ **O nome de cada interruptor cabe na coluna dele** — medido no painel pintado: a coluna é o
/// espaço entre o início da pilha (um recuo depois do olho da camada) e o interruptor.
#[test]
fn cada_nome_de_interruptor_cabe_na_coluna() {
    let mut ts = TextSystem::without_system_fonts();
    let fonte = TypeToken::Sm.px();
    const TOGGLES: [PainterLayerWidget; 2] = [
        PainterLayerWidget::AdjToggle0,
        PainterLayerWidget::AdjToggle1,
    ];
    let mut medidos = 0usize;
    for (dock, tecto) in INTERRUPTORES_ELIDEM {
        let mut cortados = Vec::new();
        for especie in AdjustmentKind::ALL {
            let (tool, id) = ferramenta(especie);
            let params = params_de(&tool, id);
            let toggles = ph2d_tool_painter::adjustment_toggle_params(&params);
            if toggles.is_empty() {
                continue;
            }
            let (_h, _s, rects) = pintar(&tool, *dock);
            set_current_layers(None);
            let olho = rect_de(
                &rects,
                painter_layer_widget_id(id.0, PainterLayerWidget::Visibility),
            )
            .expect("a linha da camada é pintada");
            let x = olho.x + ph2d_tokens::list_indent_px();
            for (slot, (f, _)) in toggles.iter().enumerate() {
                let r = rect_de(&rects, painter_layer_widget_id(id.0, TOGGLES[slot]))
                    .unwrap_or_else(|| panic!("{especie:?}: o interruptor {f:?} não foi pintado"));
                let nome = ph2d_i18n::tr(adjust_nomes::chave_do_interruptor(f).expect("tem chave"));
                let coluna = r.x - x - Spacing::Md.px();
                medidos += 1;
                if ts.prefix_width(nome, fonte) > coluna + 0.5 {
                    cortados.push(format!("{especie:?} «{nome}» numa coluna de {coluna:.1}"));
                }
            }
        }
        assert!(
            cortados.len() <= *tecto,
            "a {dock}: {} nomes de interruptor elidem e o tecto é {tecto}:\n  {}",
            cortados.len(),
            cortados.join("\n  ")
        );
        assert!(
            cortados.len() == *tecto,
            "a {dock}: elidem {} e a tabela ainda diz {tecto} — aperte o número",
            cortados.len()
        );
    }
    assert!(medidos >= 5 * 5, "só {medidos} interruptores medidos");
}

/// Os glifos que o painel pinta com uma camada de `especie`, num dock largo (sem elisão).
fn glifos_com(especie: AdjustmentKind) -> u32 {
    let (tool, _) = ferramenta(especie);
    set_current_dock_shows_layers(true);
    set_current_layers(Some(tool.layers().clone()));
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let layout = HeroLayout::for_viewport_bands(
        viewport,
        false,
        ChromeBands {
            right_dock_w: 720.0,
            ..ChromeBands::DEFAULT
        },
        CenterSplit::None,
        DockSides::BOTH,
    );
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let (glifos, _) =
        host.paint_and_count_geometry_with_layout::<PainterLayersPanel>(&mut st, layout, viewport);
    set_current_layers(None);
    glifos
}

/// ⭐⭐ **E o PINTOR usa a tabela** — o gate de cima prova que ela existe, este que ela chega ao
/// ecrã.
///
/// ⚠️ O arnês não lê texto, conta glifos. A pilha das *Sombras/Luzes* tem os nomes que mais
/// cresceram (`Shad Amt` → `Shadows Amount`): os oito nomes inteiros somam mais glifos do que os
/// oito abreviados, os números e a diferença de nome de camada juntos. ⇒ a diferença para uma
/// camada SEM linhas (*Invert*) tem de cobrir, no mínimo, os glifos dos nomes inteiros — pintar o
/// rótulo cru da crate fica abaixo disso.
#[test]
fn o_pintor_pinta_os_nomes_da_tabela() {
    let params =
        ph2d_tool_painter::AdjustmentParams::neutral_for(AdjustmentKind::ShadowsHighlights);
    let (barras, _) = pilha(&params);
    let inteiros: usize = barras
        .iter()
        .map(|f| {
            ph2d_i18n::tr(chave(&params, f).expect("tem chave"))
                .chars()
                .filter(|c| !c.is_whitespace())
                .count()
        })
        .sum();
    let crus: usize = barras
        .iter()
        .map(|f| f.chars().filter(|c| !c.is_whitespace()).count())
        .sum();
    // O controlo da fixtura: os nomes cresceram o bastante para a diferença ser mensurável.
    assert!(
        inteiros >= crus + 40,
        "os nomes inteiros ({inteiros}) já não crescem o bastante sobre os crus ({crus})"
    );
    let com = glifos_com(AdjustmentKind::ShadowsHighlights);
    let sem = glifos_com(AdjustmentKind::Invert);
    let diferenca = com.saturating_sub(sem) as usize;
    assert!(
        diferenca >= inteiros,
        "a pilha das Sombras/Luzes pinta só {diferenca} glifos a mais do que uma pilha vazia, e os \
         oito nomes inteiros somam {inteiros} — o pintor não está a pintar os nomes da tabela"
    );
}
