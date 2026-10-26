//! **A secção DRAWING RULES** (W4.5, `docs/BlenderConstraints/04_W4_cad.md` §3): as regras de desenho técnico
//! cujo dono é o path escolhido, pela ordem do documento — por regra o nome na cor do estado (a mesma das marcas do
//! canvas: conflito, a mais, REF), o «×» que a tira e, numa medida, o valor com o losango do canal dela. Uma REF só
//! mostra a medida. Os valores vêm do INSTANTÂNEO; o campo que a mão segura mostra o store.

use super::*;
use crate::plano::{Plano, emoldurada};
use ph2d_editor_core::cad_edits::{EstadoDaRegra, InspectorCadInfo, InspectorRegraRow, MAX_REGRAS};
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::property_row::{Seccao, paint_label_row};
use ph2d_editor_core::widget::{Unit, key_mark};
use ph2d_i18n::{TextKey, tr, tr_with};

/// A cor do nome — a lei do `cad_marks` do vetor (o «ok» é texto normal: a marca azul é do canvas).
const fn cor(e: EstadoDaRegra) -> ColorToken {
    match e {
        EstadoDaRegra::Ok => ColorToken::Text1,
        EstadoDaRegra::Ref => ColorToken::Text3,
        EstadoDaRegra::Redundante => ColorToken::Warn,
        EstadoDaRegra::Conflito => ColorToken::Danger,
    }
}

/// O nome da regra e, à direita, o «×». Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn cabeca(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    row: &InspectorRegraRow,
    k: usize,
) -> f32 {
    let id = ids::INSP_CAD_APAGAR[k];
    let caixa = Rect::new(x + w - ROW_H_PX, y, ROW_H_PX, ROW_H_PX);
    hit_index.register(id, caixa);
    paint_button(
        &Button::new(id, tr("panel.inspector.cad.delete"))
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id)),
        caixa,
        scene,
        text_system,
        theme,
    );
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        tr(row.nome),
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w - ROW_H_PX - Spacing::Sm.px(),
        resolve(cor(row.estado), theme),
    );
    y + ph2d_tokens::row_pitch_px()
}

/// A medida (uma lei sem medida não pinta nada): o campo com o losango do canal, ou — REF — só o valor.
#[allow(clippy::too_many_arguments)]
fn linha(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    row: &InspectorRegraRow,
    k: usize,
    seccao: Seccao,
) -> f32 {
    let Some(v) = row.medida else {
        return y;
    };
    let (unit, step) = crate::sync_constraint::unidade_e_passo(crate::sync_cad::unidade(row));
    let chave = TextKey::new("panel.inspector.cad.value");
    if row.referencia {
        let l = paint_label_row(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            ROW_H_PX,
            chave.tr(),
            seccao,
        );
        let font = TypeToken::Sm.px();
        paint_text(
            text_system,
            scene,
            &format!(
                "{} {}",
                format_number(crate::sync_cad::no_ecra_pintado(row, v)),
                unit.map_or("", Unit::suffix)
            ),
            l.control.x,
            l.control.y + (l.control.h - font) * 0.5,
            font,
            l.control.w,
            resolve(ColorToken::Text3, theme),
        );
        return y + ph2d_tokens::row_pitch_px();
    }
    let id = ids::INSP_CAD_MEDIDA[k];
    key_mark::regista_canal(&[id], &[row.canal]);
    ph2d_editor_core::property_row::paint_field_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        chave,
        id,
        step,
        unit,
        seccao,
    )
    .0
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_cad_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorCadInfo,
) -> f32 {
    let (fold, mut cur_y) = match super::nav::cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        core_ids::INSP_LIVE_CAD_SECTION,
        tr("panel.inspector.cad.title"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    let seccao = Seccao::medida(text_system, 1, &[tr("panel.inspector.cad.value")]);
    // `zip` com a tabela de ids: as regras além dela não se pintam umas sobre as outras — a frase diz quantas.
    for (k, (row, _)) in i.rows.iter().zip(ids::INSP_CAD_APAGAR.iter()).enumerate() {
        let tela = (x, w, cur_y);
        cur_y = cabeca(scene, text_system, theme, hit_index, store, tela, row, k);
        let tela = (x, w, cur_y);
        cur_y = linha(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            tela,
            row,
            k,
            seccao,
        );
    }
    let fora = i.rows.len().saturating_sub(MAX_REGRAS);
    if fora > 0 {
        let frase = tr_with("panel.inspector.cad.more", &[("n", &fora.to_string())]);
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &frase,
            ColorToken::Text3,
        );
    }
    fold.finish(store, scene, hit_index, cur_y)
}

/// A moldura: empurra a secção para o [`Plano`], se o path escolhido for dono de alguma regra.
pub(crate) fn push_cad_section<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    info: Option<&'a InspectorCadInfo>,
) {
    let Some(info) = info else {
        return;
    };
    emoldurada(
        plano,
        core_ids::INSP_LIVE_CAD_SECTION,
        store,
        inner_x,
        inner_w,
        header_h,
        move |c, t, y| {
            paint_cad_section(c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, info)
        },
    );
}
