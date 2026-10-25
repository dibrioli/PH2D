//! **As linhas da sprite que eram a secção «Sprite Sheet»** (plano GameEngineFuncional 07 §4: a secção
//! deixou de existir à parte).
//!
//! - Centrado · deslocamento · espelho ([`paint_sprite_pose_rows`]) pintam-se no fim do Render Source.
//! - A grelha colunas × linhas, a célula e «mostrar a folha no ecrã» ([`paint_sheet_grid_rows`]) vão
//!   para o cartão «Animation»; uma sprite SEM animação continua a tê-las no Render Source — sem isso
//!   uma folha parada (um cenário de várias peças) perdia a única porta para escolher a célula.

use super::*;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// Uma linha do deslocamento: o nome do campo (`sprite.offset`) mais o eixo.
fn eixo(key: &str) -> String {
    ph2d_i18n::tr_with(key, &[("offset", &tr("component.field.sprite.10"))])
}

/// ⭐⭐⭐ **A COLUNA desta secção, medida sobre os SETE nomes de linha inteira que ela pinta.**
///
/// ⛔⛔ Até 2026-09-15 ela era a **metade cega** ([`property_row_columns`] sem nome), e o
/// *«Show sheet on canvas»* (`126,1 px` a `Sm`) saía **cortado em todo o curso do dock** — `78,0`
/// no mínimo, `104,6` na largura do dono, `120,0` na de omissão. Medida, a coluna cresce até ele e
/// o corte desaparece nas duas larguras de cima (no mínimo o tecto do campo ganha, que é a troca
/// que o dono escolheu em 2026-05-24).
///
/// ⚠️ **Os dois nomes de MEIA largura (`Flip H`/`Flip V`) ficam de fora** — quando aquela fileira
/// emparelha, a «secção» dela é o PAR (ver [`ph2d_editor_core::property_row::paint_check_rows`]);
/// quando ela parte, cada metade entra nesta coluna na mesma.
pub(crate) fn seccao(text_system: &mut TextSystem) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("component.field.sprite.9"),
            &eixo("panel.inspector.sprite_sheet.offset_x"),
            &eixo("panel.inspector.sprite_sheet.offset_y"),
            tr("component.field.sprite_grid.1"),
            tr("component.field.sprite_grid.2"),
            tr("component.field.sprite_grid.3"),
            tr("panel.inspector.sprite_sheet.show_sheet_on_canvas"),
        ],
    )
}

/// W2 Sprite Inspector v2 — Sprite Sheet section (anatomia §03 §3.4).
/// HFrames / VFrames / Frame integer NumberInputs. Renders today: the
/// extract slices the atlas rect into the grid and selects `frame`'s
/// cell (clamps `hframes`/`vframes >= 1`, `frame < cells` at commit).
/// Uma linha de número com o seu losango (a forma de todas as linhas desta página).
#[allow(clippy::too_many_arguments)]
fn number_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    row_y: f32,
    sec: ph2d_editor_core::property_row::Seccao,
    label: ph2d_editor_core::widget::key_mark::Rotulo<'_>,
    id: NodeId,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let field_h = ROW_H_PX;
    let colunas = ph2d_editor_core::property_row::colunas_da_linha(x, w, row_y, ROW_H_PX, sec);
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        label.texto,
        x,
        row_y + (field_h - label_font) * 0.5,
        label_font,
        colunas.label.w,
        resolve(ColorToken::Text2, theme),
    );
    let rect = Rect::new(colunas.control.x, row_y, colunas.control.w, field_h);
    hit_index.register(id, rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value)
        .step(1.0)
        .visual((state, store.hover_live(id)));
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        store.number_input_mixed(id),
        caret,
        anchor,
        rect,
        scene,
        text_system,
        theme,
    );
    let (_, dot) = ph2d_editor_core::widget::form_row_columns(x, w, row_y, field_h);
    if let Some(alvo) =
        ph2d_editor_core::widget::key_mark::paint_decorator_campo(scene, theme, dot, &[id], label)
    {
        hit_index.register(alvo, dot);
    }
    row_y + field_h + ph2d_tokens::control_gap_px()
}

/// **Centrado · deslocamento X/Y · espelho H/V** — o fim do Render Source. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sprite_pose_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let sec = seccao(text_system);
    let (_, ce_value) = store
        .checkbox(ids::INSP_SPRITE_CENTERED)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Checked));
    let mut cur_y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (
            ids::INSP_SPRITE_CENTERED,
            TextKey::new("component.field.sprite.9"),
            matches!(ce_value, CheckboxValue::Checked),
        ),
        sec,
    );
    for (key, id) in [
        (
            "panel.inspector.sprite_sheet.offset_x",
            ids::INSP_SPRITE_OFFSET_X,
        ),
        (
            "panel.inspector.sprite_sheet.offset_y",
            ids::INSP_SPRITE_OFFSET_Y,
        ),
    ] {
        cur_y = number_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            sec,
            ph2d_editor_core::widget::key_mark::Rotulo::de_campo(
                &eixo(key),
                TextKey::new("component.field.sprite.10"),
            ),
            id,
        );
    }
    paint_flip_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        sec,
    )
}

/// **A grelha da folha**: colunas, linhas, a célula no ecrã e «mostrar a folha no ecrã». O cartão
/// «Animation» pinta-a; o Render Source só a pinta numa sprite sem animação. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sheet_grid_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    hframes: u32,
    vframes: u32,
    with_frame: bool,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = y;
    for (key, id, show) in [
        (
            "component.field.sprite_grid.1",
            ids::INSP_SPRITE_HFRAMES,
            true,
        ),
        (
            "component.field.sprite_grid.2",
            ids::INSP_SPRITE_VFRAMES,
            true,
        ),
        (
            "component.field.sprite_grid.3",
            ids::INSP_SPRITE_FRAME,
            with_frame,
        ),
    ] {
        if show {
            cur_y = number_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                sec,
                TextKey::new(key).into(),
                id,
            );
        }
    }
    sheet_preview_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        sec,
        hframes,
        vframes,
    )
}

#[allow(clippy::too_many_arguments)]
fn sheet_preview_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    sec: ph2d_editor_core::property_row::Seccao,
    hframes: u32,
    vframes: u32,
) -> f32 {
    let cells = u64::from(hframes.max(1)) * u64::from(vframes.max(1));
    if cells <= 1 {
        return y;
    }
    let (_, value) = store
        .checkbox(crate::ids::INSP_SHEET_PREVIEW)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    // ⚠️ **O nome desta linha ENTRA na medida da secção mesmo quando ela não é pintada** — é o mais
    // largo dos sete (`126,1 px` contra `36,3`–`54,2`), e uma coluna que só o conta quando a sprite
    // tem grelha **salta** debaixo do olho do artista no instante em que ele digita `2` em
    // *H Frames*. Ver o doc de [`ph2d_editor_core::property_row::Seccao::medida`].
    ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (
            crate::ids::INSP_SHEET_PREVIEW,
            tr("panel.inspector.sprite_sheet.show_sheet_on_canvas"),
            matches!(value, CheckboxValue::Checked),
        ),
        sec,
    )
}

/// **Espelhar (Flip H / Flip V)** — as duas caixas lado a lado.
#[allow(clippy::too_many_arguments)]
fn paint_flip_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // Logical Flip H / Flip V (Sprite.flip_x/flip_y) — spec §3.4 orders
    // flip with the origin controls (after Offset, before the frame
    // grid). Toggling dispatches an InspectorSpriteEdit and the shader
    // mirrors the sampled UV.
    //
    // ⭐⭐⭐ **As duas partilham uma fileira SE couberem** — spec §6-quater. Elas têm os nomes mais
    //    curtos dos dez booleanos emparelhados do Inspector (`32,4` e `30,8 px`) e mesmo assim não
    //    cabiam em meia linha depois de a marca ganhar caixa: a metade deixa `14,6 px` de coluna a
    //    `273,3` de painel. A tabela medida está no doc da porta.
    let (_, fx_value) = store
        .checkbox(ids::INSP_SPRITE_FLIP_X)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let (_, fy_value) = store
        .checkbox(ids::INSP_SPRITE_FLIP_Y)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        &[
            (
                ids::INSP_SPRITE_FLIP_X,
                TextKey::new("component.field.sprite.2"),
                matches!(fx_value, CheckboxValue::Checked),
            ),
            (
                ids::INSP_SPRITE_FLIP_Y,
                TextKey::new("component.field.sprite.3"),
                matches!(fy_value, CheckboxValue::Checked),
            ),
        ],
        sec,
    )
}
