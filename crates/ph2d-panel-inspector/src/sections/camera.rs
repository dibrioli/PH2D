//! ⭐⭐⭐ **A secção CAMERA** — o enquadramento do jogo (TOP-20 #7, W3).
//!
//! # ⚠️ Ela nasce COM a wave, e isso é a lição que o `Timers` custou
//!
//! O `Timers` shipou anexável e sem linha de edição, e o report do dono foi *«timer sumiu do modal
//! de componente»*. *Um componente anexável sem painel é indistinguível de um que não foi anexado.*
//!
//! # ⚠️ Uma secção, TRÊS corpos
//!
//! `GameCamera` · `+ CameraFollow` · `+ CameraLimits` — três componentes, uma pergunta só para o
//! artista. Ver o doc de [`ph2d_editor_core::ids::inspector_camera`].
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! Quatro avisos, e cada um responde a uma forma diferente de *«mexo nos números e nada acontece»*,
//! que é a única pergunta que um artista faz sobre uma câmera:
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `not active` | `Active` desligado ⇒ ela nunca concorre | ligar a caixa |
//! | `another camera commands` | há várias e esta não é a de maior prioridade | subir a `Priority` |
//! | `target not found` | o nome do alvo não é de ninguém na cena | escrever o nome certo |
//! | `limits smaller than the view` | a cerca não cabe na janela ⇒ a câmera **fixa no centro dela** | alargar a cerca, ou baixar a `Height` |
//!
//! ⚠️ **O quarto é o que só o snapshot pode dizer**: ele é geometria da JANELA (proporção do ecrã ×
//! altura da câmera), e não dos quatro números da cerca. *Nenhum artista adivinha, a olhar para
//! `min` e `max`, que a câmera parou de seguir porque a caixa é mais estreita que o ecrã.*

use super::*;
use ph2d_editor_core::screens::hero::{
    InspectorCameraFollow, InspectorCameraInfo, InspectorCameraLimits, InspectorGameCamera,
};
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_editor_core::widget::{BitmaskGrid32, paint_bitmask_grid32};
use ph2d_i18n::tr;

// ⭐ **A caixa de marcar desta secção MUDOU-SE para a porta** (2026-09-15) — este ficheiro tinha a
//    quarta cópia do mesmo `register` + `checkbox_visual` + `bool → CheckboxValue`, e a lei *«o
//    valor vem do SNAPSHOT, nunca do store»* estava escrita em cada uma delas.
//    Ver [`ph2d_editor_core::property_row::paint_check_row`].

/// O corpo da CÂMERA. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn camera_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    cam: &InspectorGameCamera,
    info: &InspectorCameraInfo,
) -> f32 {
    let mut cur_y = y;

    // ⚠️ **Os avisos vêm ANTES dos números**, e é deliberado: quem não vê a câmera reagir não quer
    // afinar um amortecimento — quer saber porquê.
    if !cam.active {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.camera.active_is_off_this_camera"),
            ColorToken::Warn,
        );
    } else if !info.is_active_camera && info.camera_count > 1 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.camera.another_camera_commands_raise_priority"),
            ColorToken::Warn,
        );
    }

    // ⚠️ **Cada linha carrega o PRÓPRIO passo**, e não um partilhado: a `Priority` é uma CONTAGEM,
    // e um arrasto de décimo em décimo sobre um inteiro é um controlo que mente sobre o que guarda.
    // *O gate do número mágico é que o disse — a primeira redacção partilhava `0,1` entre metros e
    // uma contagem.*
    let linhas = [
        (
            tr("panel.inspector.camera.height_m"),
            &[ids::INSP_CAMERA_HEIGHT][..],
            0.5,
            Some(ph2d_editor_core::widget::Unit::Meters),
        ), // LITERAL-PX-OK: passo em metros
        (
            tr("panel.inspector.camera.offset_m"),
            &[ids::INSP_CAMERA_OFFSET_X, ids::INSP_CAMERA_OFFSET_Y][..],
            0.1, // LITERAL-PX-OK: passo em metros
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
        (
            tr("panel.inspector.camera.priority"),
            &[ids::INSP_CAMERA_PRIORITY][..],
            1.0,
            None,
        ), // LITERAL-PX-OK: uma prioridade de cada vez
    ];
    // ⭐⭐ **A coluna é da SECÇÃO, medida uma vez sobre a TABELA que ela pinta** — ver
    //    [`ph2d_editor_core::property_row::Seccao`]. ⛔ A tabela deixou de ser um literal dentro do
    //    `for` porque ela é lida DUAS vezes: para medir o nome mais largo e para pintar.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        2,
        &linhas.iter().map(|t| t.0).collect::<Vec<_>>(),
    );
    for (label, ids3, step, unit) in linhas {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            ids3,
            step,
            unit,
            seccao,
        );
    }

    // ⭐⭐⭐ **As duas caixas partilham uma fileira SE couberem** — spec §6-quater; a tabela medida
    //    está no doc da porta. ⚠️ A segunda é **o interruptor da PRÉ-VISUALIZAÇÃO**, e o
    //    `look_through` é o nome mais largo dos dez booleanos emparelhados do Inspector (`79,8 px`):
    //    é ele que decide quando esta fileira parte.
    ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[
            (
                ids::INSP_CAMERA_ACTIVE,
                tr("panel.inspector.camera.active"),
                cam.active,
            ),
            (
                ids::INSP_CAMERA_PREVIEW,
                tr("panel.inspector.camera.look_through"),
                info.preview_on,
            ),
        ],
        seccao,
    )
}

/// O corpo de QUEM ELA SEGUE. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn follow_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    f: &InspectorCameraFollow,
) -> f32 {
    let linhas = [
        (
            tr("panel.inspector.camera.damping_1_s"),
            [ids::INSP_CAMERA_DAMP_X, ids::INSP_CAMERA_DAMP_Y],
            0.5, // LITERAL-PX-OK: passo em 1/s
            Some(ph2d_editor_core::widget::Unit::PerSecond),
        ),
        (
            tr("panel.inspector.camera.dead_zone"),
            [ids::INSP_CAMERA_DEAD_X, ids::INSP_CAMERA_DEAD_Y],
            0.05, // LITERAL-PX-OK: fracção da meia-janela
            None,
        ),
        (
            tr("panel.inspector.camera.lookahead_s"),
            [ids::INSP_CAMERA_LOOK_X, ids::INSP_CAMERA_LOOK_Y],
            0.05, // LITERAL-PX-OK: passo em segundos
            Some(ph2d_editor_core::widget::Unit::Seconds),
        ),
        (
            tr("panel.inspector.camera.follow_offset_m"),
            [ids::INSP_CAMERA_FOLLOW_OFF_X, ids::INSP_CAMERA_FOLLOW_OFF_Y],
            0.1, // LITERAL-PX-OK: passo em metros
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
    ];
    // ⭐⭐ **A coluna é da SECÇÃO, medida uma vez sobre a TABELA que ela pinta** — ver
    //    [`ph2d_editor_core::property_row::Seccao`]. ⛔ A tabela deixou de ser um literal dentro do
    //    `for` porque ela é lida DUAS vezes: para medir o nome mais largo e para pintar.
    //
    // ⛔⛔ **E o nome do ALVO entra na MESMA medição** (2026-09-23, varredura `onde_comeca_o_valor`):
    //    até aqui ele vinha numa declaração PRÓPRIA do chamador (`campos = 1`, só o nome dele), e a
    //    caixa do alvo arrancava a `136` enquanto as seis linhas de baixo arrancavam a `111` — duas
    //    colunas dentro de UMA secção. *Duas declarações para o mesmo corpo são duas colunas.*
    let nomes: Vec<&str> = std::iter::once(tr("panel.inspector.camera.target_label"))
        .chain(linhas.iter().map(|t| t.0))
        .collect();
    let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 2, &nomes);
    let mut cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.camera.target_label"),
        ids::INSP_CAMERA_TARGET,
        TextInput::new(ids::INSP_CAMERA_TARGET, "")
            .placeholder(tr("panel.inspector.camera.object_name")),
        seccao,
    );

    if !f.target.trim().is_empty() && !f.target_found {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.camera.nothing_in_the_scene_has"),
            ColorToken::Danger,
        );
    }

    for (label, ids2, step, unit) in linhas {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &ids2,
            step,
            unit,
            seccao,
        );
    }
    cur_y
}

/// O corpo da CERCA. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn limits_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    l: &InspectorCameraLimits,
) -> f32 {
    let mut cur_y = y;
    if l.smaller_than_view {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.camera.limits_are_smaller_than_the"),
            ColorToken::Warn,
        );
    }
    let linhas = [
        (
            tr("panel.inspector.camera.min_m"),
            [ids::INSP_CAMERA_MIN_X, ids::INSP_CAMERA_MIN_Y],
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
        (
            tr("panel.inspector.camera.max_m"),
            [ids::INSP_CAMERA_MAX_X, ids::INSP_CAMERA_MAX_Y],
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
    ];
    // ⭐⭐ **A coluna é da SECÇÃO, medida uma vez sobre a TABELA que ela pinta** — ver
    //    [`ph2d_editor_core::property_row::Seccao`]. ⛔ A tabela deixou de ser um literal dentro do
    //    `for` porque ela é lida DUAS vezes: para medir o nome mais largo e para pintar.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        2,
        &linhas.iter().map(|t| t.0).collect::<Vec<_>>(),
    );
    for (label, ids2, unit) in linhas {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &ids2,
            0.5, // LITERAL-PX-OK: passo em metros
            unit,
            seccao,
        );
    }
    cur_y
}

/// A máscara de camadas — a mesma grade `4×8` da visibilidade, com ids PRÓPRIOS.
///
/// ⚠️ Ela nasce **RECOLHIDA**, como a irmã: são 32 caixas, e a pergunta é avançada.
#[allow(clippy::too_many_arguments)]
fn cull_mask(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    mask: u32,
) -> f32 {
    let h = ROW_H_PX;
    let row_gap = Spacing::Xs.px();
    let mut yy = close_section(scene, theme, x, w, y);
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ids::INSP_CAMERA_CULL_HEADER,
        tr("panel.inspector.camera.cull_mask"),
    )
    .open_t(store.section_open_live(ids::INSP_CAMERA_CULL_HEADER));
    let header_rect = Rect::new(x, yy, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_CAMERA_CULL_HEADER, header_rect);
    yy += header_h;
    match SectionFold::begin(
        store,
        ids::INSP_CAMERA_CULL_HEADER,
        x,
        w,
        yy,
        scene,
        hit_index,
    ) {
        None => yy + row_gap,
        Some(fold) => {
            let grid = BitmaskGrid32::new(
                core_ids::INSP_LIVE_CAMERA_SECTION,
                tr("panel.inspector.camera.cull_mask"),
                ids::INSP_CAMERA_CULL_BIT,
                mask,
            );
            for (bit, id) in ids::INSP_CAMERA_CULL_BIT.iter().enumerate() {
                hit_index.register(*id, BitmaskGrid32::cell_rect(x, yy, w, h, bit));
            }
            paint_bitmask_grid32(&grid, x, yy, w, h, scene, text_system, theme);
            let inner = yy + BitmaskGrid32::grid_height(h) + row_gap;
            fold.finish(store, scene, hit_index, inner)
        }
    }
}

/// A secção inteira — cabeçalho, dobra e os três corpos. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_camera_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorCameraInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let color_id = core_ids::INSP_LIVE_CAMERA_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: acento neutro por omissão
    let header = section_header(
        store,
        core_ids::INSP_LIVE_CAMERA_SECTION,
        tr("panel.inspector.camera.camera"),
    )
    .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_CAMERA_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;

    cur_y = camera_body(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &info.camera,
        info,
    );
    if let Some(f) = info.follow.as_ref() {
        cur_y = follow_body(scene, text_system, theme, hit_index, store, x, w, cur_y, f);
    }
    if let Some(l) = info.limits.as_ref() {
        cur_y = limits_body(scene, text_system, theme, hit_index, store, x, w, cur_y, l);
    }
    cur_y = cull_mask(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info.camera.cull_mask,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
