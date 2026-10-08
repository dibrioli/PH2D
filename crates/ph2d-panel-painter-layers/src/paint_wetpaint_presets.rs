//! **O bloco de presets do Wet Painter** (W1, `docs/Painter/NewFeaturesWetPaint/02 §5`): no Wet Paint
//! ele toma o lugar da fileira «Preset» dos meios. De cima para baixo: o chip Preset (o navegador:
//! Favoritos · Recentes · os grupos, que são as pastas), Favorite, o preview VIVO, Save as New ·
//! Save as Default · Reset, os volumes VIVOS (Size px = o diâmetro do anel · Water) com os valores
//! rápidos, e a política dos volumes. Opacity, Pressure e Length não têm leitor no Wet Paint: não
//! aparecem (gate `a_opacidade_nao_tem_leitor_no_wet_paint`).
//!
//! O que escreve onde: os botões e as escolhas viram um `PedidoPreset` no tool, que a ponte da app
//! executa (o disco é dela); os volumes vão directos ao pincel (`route_wet_preset_event`).

use crate::number_field;
use crate::paint_brush_rows::{paint_dropdown_chip, paint_dropdown_row};
use crate::paint_brush_top::paint_checkbox_row;
use crate::state_wet_presets::{self as st, VistaPresets};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tool_painter::{BRUSH_SIZE_MAX_PX, BRUSH_SIZE_MIN_PX, BrushSettings, ids};
use ph2d_vector::ImageQuality;

/// As secções do popover do Preset (o id de cada linha leva a secção: um preset aparece até três vezes).
const SECAO_FAVORITOS: u8 = 0;
const SECAO_RECENTES: u8 = 1;
const SECAO_GRUPO: u8 = 2;
const SECOES: [u8; 3] = [SECAO_FAVORITOS, SECAO_RECENTES, SECAO_GRUPO];
const SIZE_STEP: f64 = 1.0; // LITERAL-PX-OK: Size px volume step (1 px of diameter)
const PERCENT: f32 = 100.0; // LITERAL-PX-OK: 0..1 volume shown as percent in the quick list
/// O volume Size px é o DIÂMETRO; o pincel guarda o raio (05 §1.1).
const DIAMETRO_POR_RAIO: f32 = 2.0; // LITERAL-PX-OK: diameter = 2 × radius, a law, not a size

/// Pinta o bloco; devolve o `y` seguinte.
pub(crate) fn paint_wetpaint_presets(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let vista = st::current_wet_presets();
    let mut y = y;
    if let Some(v) = vista.as_deref() {
        y = paint_navegador(ctx, theme, x, content_w, y, v);
    } else {
        y = paint_preview(ctx, x, content_w, y);
    }
    y = volume_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        0,
        "panel.painter_layers.wetpaint.vol_size",
        ids::PAINTER_WETPAINT_VOL_SIZE,
        brush.size_px * DIAMETRO_POR_RAIO,
        (
            BRUSH_SIZE_MIN_PX * DIAMETRO_POR_RAIO,
            BRUSH_SIZE_MAX_PX * DIAMETRO_POR_RAIO,
        ),
        SIZE_STEP,
        0,
    );
    y = volume_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        1,
        "panel.painter_layers.wetpaint.water",
        ids::PAINTER_WETPAINT_WATER,
        brush.wet_knobs.water as f32,
        (0.0, 1.0),
        number_field::FINE_STEP,
        2,
    );
    if let Some(v) = vista.as_deref() {
        let (ny, open) = paint_dropdown_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "panel.painter_layers.wetpaint.preset_policy",
            ids::PAINTER_WETPAINT_PRESET_POLICY,
            v.politica,
            nome_da_politica(v.politica),
        );
        if let Some(r) = open {
            st::PENDING_POLICY_DD.with(|c| c.set(Some((r, v.politica))));
        }
        y = ny;
    }
    y
}

fn paint_navegador(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    v: &VistaPresets,
) -> f32 {
    let ativo = v.ativo.as_deref().and_then(|r| v.procurar(r));
    let nome = ativo.map_or_else(
        || tr("panel.painter_layers.wetpaint.preset_none"),
        |a| a.nome.as_str(),
    );
    let (mut y, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.wetpaint.preset",
        ids::PAINTER_WETPAINT_PRESET,
        0,
        nome,
    );
    if let Some(r) = open {
        st::PENDING_PRESET_DD.with(|c| c.set(Some(r)));
    }
    if let Some(a) = ativo {
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            ids::PAINTER_WETPAINT_PRESET_FAVORITE,
            "panel.painter_layers.wetpaint.preset_favorite",
            a.favorito,
        );
    }
    y = paint_preview(ctx, x, content_w, y);
    // Sem preset activo não há o que guardar por cima nem o que repor: só «Save as New».
    let mut botoes = vec![(
        ids::PAINTER_WETPAINT_PRESET_SAVE_NEW,
        tr("panel.painter_layers.wetpaint.preset_save_new").to_owned(),
    )];
    if ativo.is_some() {
        botoes.push((
            ids::PAINTER_WETPAINT_PRESET_SAVE_DEFAULT,
            tr("panel.painter_layers.wetpaint.preset_save_default").to_owned(),
        ));
        botoes.push((
            ids::PAINTER_WETPAINT_PRESET_RESET,
            tr("panel.painter_layers.wetpaint.preset_reset").to_owned(),
        ));
    }
    crate::paint_seg_row::seg_row_owned(
        ctx,
        theme,
        x,
        content_w,
        y,
        ids::PAINTER_WETPAINT_PRESET_SAVE_NEW,
        tr("panel.painter_layers.wetpaint.preset_actions"),
        &botoes,
        usize::MAX,
    )
}

/// O preview VIVO: o pincel de agora sobre a obra de agora, o quadro que a ponte publica (ela repinta-o
/// numa thread a cada mudança e toca a animação do traço uma vez).
fn paint_preview(ctx: &mut PaintCtx, x: f32, content_w: f32, y: f32) -> f32 {
    let Some((px, w, h)) = st::current_wet_preview().filter(|(_, w, _)| *w > 0) else {
        return y;
    };
    let alto = content_w * h as f32 / w as f32;
    ctx.scene.draw_image_rgba(
        &px,
        w,
        h,
        (
            f64::from(x),
            f64::from(y),
            f64::from(x + content_w),
            f64::from(y + alto),
        ),
        ImageQuality::Medium,
    );
    y + alto + ph2d_tokens::control_gap_px()
}

/// Um volume: a fileira numérica e, à direita, o chip dos valores rápidos.
#[allow(clippy::too_many_arguments)]
fn volume_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    vol: usize,
    chave: &str,
    id: ph2d_a11y::NodeId,
    value: f32,
    (min, max): (f32, f32),
    step: f64,
    decimals: usize,
) -> f32 {
    let chip_w = ph2d_tokens::ICON_BTN_SIZE_PX;
    let gap = ph2d_tokens::control_gap_px();
    let sec = crate::seccoes::seccao_da_chave(ctx.text_system, chave);
    let ny = number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w - chip_w - gap,
        y,
        tr(chave),
        id,
        value,
        min,
        max,
        step,
        decimals,
        sec,
    );
    let rect = Rect::new(x + content_w - chip_w, y, chip_w, ph2d_tokens::ROW_H_PX);
    if paint_dropdown_chip(
        ctx,
        theme,
        ids::PAINTER_WETPAINT_VOL_QUICK[vol],
        0,
        "",
        rect,
    ) {
        st::PENDING_QUICK_DD.with(|c| c.set(Some((rect, vol))));
    }
    ny
}

fn nome_da_politica(i: u8) -> &'static str {
    match i {
        0 => tr("panel.painter_layers.wetpaint.policy_volumes_and_mode"),
        1 => tr("panel.painter_layers.wetpaint.policy_volumes"),
        _ => tr("panel.painter_layers.wetpaint.policy_none"),
    }
}

/// As linhas do popover do Preset: títulos (desactivados) e presets, com o índice corrido como valor.
fn opcoes_do_preset(v: &VistaPresets) -> (Vec<DropdownOption<usize>>, usize) {
    let mut out = Vec::new();
    let mut atual = usize::MAX;
    let titulo = |out: &mut Vec<DropdownOption<usize>>, texto: String| {
        let k = out.len();
        out.push(
            DropdownOption::new(ids::painter_wetpaint_preset_title_id(k), k, texto).disabled(true),
        );
    };
    if v.ilegiveis > 0 {
        titulo(
            &mut out,
            tr_with(
                "panel.painter_layers.wetpaint.preset_unreadable",
                &[("n", &v.ilegiveis)],
            ),
        );
    }
    let mut linha = |out: &mut Vec<DropdownOption<usize>>, secao: u8, e: &st::VistaPreset| {
        let k = out.len();
        if secao == SECAO_GRUPO && v.ativo.as_deref() == Some(e.relativo.as_str()) {
            atual = k;
        }
        out.push(DropdownOption::new(
            ids::painter_wetpaint_preset_option_id(secao, &e.relativo),
            k,
            e.nome.clone(),
        ));
    };
    let favoritos: Vec<&st::VistaPreset> = v.entradas.iter().filter(|e| e.favorito).collect();
    if !favoritos.is_empty() {
        titulo(
            &mut out,
            tr("panel.painter_layers.wetpaint.preset_favorites").to_owned(),
        );
        for e in favoritos {
            linha(&mut out, SECAO_FAVORITOS, e);
        }
    }
    let recentes: Vec<&st::VistaPreset> = v.recentes.iter().filter_map(|r| v.procurar(r)).collect();
    if !recentes.is_empty() {
        titulo(
            &mut out,
            tr("panel.painter_layers.wetpaint.preset_recent").to_owned(),
        );
        for e in recentes {
            linha(&mut out, SECAO_RECENTES, e);
        }
    }
    let mut grupo_atual: Option<&str> = None;
    for e in &v.entradas {
        if grupo_atual != Some(e.grupo.as_str()) {
            grupo_atual = Some(e.grupo.as_str());
            titulo(&mut out, e.grupo.clone());
        }
        linha(&mut out, SECAO_GRUPO, e);
    }
    (out, atual)
}

/// Os popovers abertos do bloco — pintados por cima do corpo, depois do clip (como os outros).
pub(crate) fn paint_wetpaint_presets_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some(r) = st::PENDING_PRESET_DD.with(std::cell::Cell::take)
        && let Some(v) = st::current_wet_presets()
    {
        let (opcoes, atual) = opcoes_do_preset(&v);
        crate::dropdown_popover::paint_dropdown_popover(
            ctx,
            theme,
            ids::PAINTER_WETPAINT_PRESET,
            opcoes,
            r,
            atual,
        );
    }
    if let Some((r, cur)) = st::PENDING_POLICY_DD.with(std::cell::Cell::take) {
        let opcoes = (0..ids::PAINTER_WETPAINT_PRESET_POLICY_COUNT)
            .map(|i| {
                DropdownOption::new(
                    ids::painter_wetpaint_policy_option_id(i),
                    i,
                    nome_da_politica(i),
                )
            })
            .collect();
        crate::dropdown_popover::paint_dropdown_popover(
            ctx,
            theme,
            ids::PAINTER_WETPAINT_PRESET_POLICY,
            opcoes,
            r,
            cur,
        );
    }
    if let Some((r, vol)) = st::PENDING_QUICK_DD.with(std::cell::Cell::take) {
        let opcoes = ids::vol_quick_values(vol)
            .iter()
            .enumerate()
            .map(|(i, &q)| {
                let rotulo = if vol == 0 {
                    tr_with("panel.painter_layers.wetpaint.quick_px", &[("n", &q)])
                } else {
                    tr_with(
                        "panel.painter_layers.wetpaint.quick_percent",
                        &[("n", &(q * PERCENT))],
                    )
                };
                DropdownOption::new(
                    ids::painter_wetpaint_vol_quick_option_id(vol, i as u8),
                    i as u8,
                    rotulo,
                )
            })
            .collect();
        crate::dropdown_popover::paint_dropdown_popover(
            ctx,
            theme,
            ids::PAINTER_WETPAINT_VOL_QUICK[vol],
            opcoes,
            r,
            u8::MAX,
        );
    }
}

/// O preset que uma linha do popover nomeia (qualquer secção), pela vista publicada.
pub(crate) fn preset_escolhido(id: ph2d_a11y::NodeId) -> Option<String> {
    let v = st::current_wet_presets()?;
    v.entradas.iter().find_map(|e| {
        SECOES
            .iter()
            .any(|&s| ids::painter_wetpaint_preset_option_id(s, &e.relativo) == id)
            .then(|| e.relativo.clone())
    })
}

/// Fecha o chip do Preset e manda o preset escolhido ao tool (que o regista para a ponte).
pub(crate) fn escolher_preset(host: &mut dyn PanelHostInternal, relativo: String) {
    if let Some(InteractiveState::Dropdown { open, .. }) =
        host.store_mut().get_mut(ids::PAINTER_WETPAINT_PRESET)
    {
        *open = false;
    }
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            ids::PAINTER_WETPAINT_PRESET,
            relativo,
        )));
}
