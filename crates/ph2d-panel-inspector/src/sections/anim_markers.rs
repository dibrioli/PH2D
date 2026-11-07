//! **Os marcadores em quadro da animação aberta** (SFX Lab W1), ao lado de On Finish e On Loop: «ao entrar neste
//! quadro, grita este sinal». Na tela o quadro conta de `1`; no dado o `step` conta de `0`.

use super::*;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::corpo;
use ph2d_editor_core::property_row::Seccao;
use ph2d_editor_core::screens::hero::{InspectorAnimInfo, InspectorAnimRow};
use ph2d_editor_core::widget::card_ink::card_body;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::{TextKey, tr, tr_with};

/// A caixa do quadro: cabe `999` ao lado da letra de arrasto.
const FRAME_W: f32 = m::FIELD_H * 2.5; // LITERAL-PX-OK: múltiplo da altura de campo

/// Os rótulos sobre as duas colunas: um par por grelha, não um por linha.
#[allow(clippy::too_many_arguments)]
fn colunas(scene: &mut VectorScene, text_system: &mut TextSystem, x: f32, w: f32, y: f32) -> f32 {
    let Some(ink) = card_body() else {
        return y;
    };
    let font = TypeToken::Sm.px();
    let cor = ink.color(ColorToken::Text3);
    let topo = y + m::FIELD_ROW_GAP;
    let segunda = x + FRAME_W + m::FIELD_GAP;
    ph2d_editor_core::paint::paint_text(
        text_system,
        scene,
        tr("card.anim.marker_frame"),
        x,
        topo,
        font,
        FRAME_W,
        cor,
    );
    ph2d_editor_core::paint::paint_text(
        text_system,
        scene,
        tr("card.anim.marker_signal"),
        segunda,
        topo,
        font,
        (w - FRAME_W - m::FIELD_GAP).max(0.0),
        cor,
    );
    topo + font
}

/// Uma linha: «Frame», «Signal» e o ×. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn marker_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: usize,
    seccao: Seccao,
) -> f32 {
    let (frame, signal, del) = (
        ids::INSP_ANIM_MARKER_FRAME[i],
        ids::INSP_ANIM_MARKER_SIGNAL[i],
        ids::INSP_ANIM_MARKER_DEL[i],
    );
    let Some(ink) = card_body() else {
        let side = ROW_H_PX;
        super::anim::icon_button(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            Rect::new(x + w - side, y, side, side),
            del,
            IconId::Close,
            tr("card.anim.marker_del_tip"),
        );
        let y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w - side - ph2d_tokens::control_gap_px(),
            y,
            Rotulo::vista(TextKey::new("card.anim.marker_frame")),
            &[frame],
            1.0,
            None,
            seccao,
        );
        return super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y,
            tr("card.anim.marker_signal"),
            signal,
            TextInput::new(signal, "").placeholder(tr("card.anim.marker_signal_placeholder")),
            seccao,
        );
    };
    let (gap, h) = (m::FIELD_GAP, m::FIELD_H);
    let y = y + m::FIELD_ROW_GAP;
    corpo::fields_in(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        Rect::new(x, y, FRAME_W, h),
        tr("card.anim.marker_frame"),
        &[frame],
        1.0,
    );
    let sx = x + FRAME_W + gap;
    corpo::text_in(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        Rect::new(sx, y, (x + w - h - gap - sx).max(0.0), h),
        signal,
        tr("card.anim.marker_signal_placeholder"),
    );
    super::anim::icon_button(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        Rect::new(x + w - h, y, h, h),
        del,
        IconId::Close,
        tr("card.anim.marker_del_tip"),
    );
    y + h
}

/// A lista dos marcadores da animação aberta e «+ Marker». Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn marker_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorAnimRow,
    seccao: Seccao,
) -> f32 {
    let mut cur_y =
        super::rows::titulo_de_grupo(scene, text_system, theme, x, w, y, tr("card.anim.markers"));
    if !row.markers.is_empty() {
        cur_y = colunas(scene, text_system, x, w, cur_y);
    }
    for (i, marker) in row
        .markers
        .iter()
        .take(ids::MAX_ANIM_MARKER_ROWS)
        .enumerate()
    {
        cur_y = marker_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            i,
            seccao,
        );
        // Um quadro que passou do fim depois de encolher a animação fica na lista, e a linha diz porquê.
        if marker.step >= row.steps {
            cur_y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                &tr_with(
                    "card.anim.marker_past_end",
                    &[("frame", &(marker.step + 1)), ("n", &row.steps)],
                ),
                ColorToken::Warn,
            );
        }
    }
    let fora = row.markers.len().saturating_sub(ids::MAX_ANIM_MARKER_ROWS);
    if fora > 0 {
        return super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with("card.anim.markers_hidden", &[("n", &fora)]),
            ColorToken::Warn,
        );
    }
    super::anim_card::button_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_ANIM_MARKER_ADD,
        tr("card.anim.marker_add"),
    )
}

/// O resumo do cartão: os marcadores moram no editor (ficha 02, decisão 4), e este botão abre-o.
#[allow(clippy::too_many_arguments)]
pub(super) fn marker_summary(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorAnimRow,
) -> f32 {
    let n = row.markers.len();
    let texto = if n == 0 {
        tr("card.anim.marker_add_open").to_owned()
    } else {
        tr_with("card.anim.markers_open", &[("n", &n)])
    };
    super::anim_card::button_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_ANIM_MARKER_ADD,
        &texto,
    )
}

/// Semeia as linhas todo quadro, menos o campo sob o dedo: uma linha removida desloca as de baixo.
pub(crate) fn sync_marker_fields(
    host: &mut dyn PanelHostInternal,
    an: &InspectorAnimInfo,
    selected: usize,
) {
    let Some(row) = an.rows.get(selected) else {
        return;
    };
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    let teto = f64::from(row.steps.max(1));
    for (i, marker) in row
        .markers
        .iter()
        .take(ids::MAX_ANIM_MARKER_ROWS)
        .enumerate()
    {
        let frame = ids::INSP_ANIM_MARKER_FRAME[i];
        host.store_mut().set_number_range(frame, 1.0, teto, 1.0);
        if focus != Some(frame) && drag != Some(frame) {
            crate::unidade_mundo::semeia(host, frame, f64::from(marker.step) + 1.0);
        }
        let signal = ids::INSP_ANIM_MARKER_SIGNAL[i];
        if focus != Some(signal) && host.store().text(signal) != Some(marker.signal.as_str()) {
            crate::sync_sections::write_text(host, signal, &marker.signal);
        }
    }
}
