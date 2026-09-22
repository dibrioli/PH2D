//! ⭐⭐⭐ **A secção LIFECYCLE** — o que faz MORRER (TOP-20 #12).
//!
//! # ⚠️ Por que ela saiu do irmão [`super::factory`] (2026-09-18)
//!
//! Elas nasceram no mesmo ficheiro porque nasceram na mesma wave — e o **sujeito** delas é outro,
//! coisa que o cabeçalho de lá já dizia por escrito: *a `Factory` vive em quem fabrica; a
//! `Lifetime` e o `DestroyOutside` vivem na RECEITA*. ⇒ *duas secções, dois ficheiros*.
//!
//! ⛔ O gatilho foi o tecto de LOC (`622/600` ao ganhar a linha da MIRA), e a cura de um tecto é o
//! CORTE — nunca uma entrada nova no `FILE_OVERAGE_OK`.
//!
//! ⚠️ **O pintor de uma linha de aviso é PARTILHADO, e na integração de 20/09 ele mudou de dono:**
//! esta nota dizia *«o `super::factory::warn` fica onde estava»* e a `line/UIUX` **apagou** aquele
//! `warn` local no mesmo dia, substituindo-o pela porta [`super::rows::aviso`] — uma lei, uma porta.
//! As duas metades desta wave fundiram-se: o corpo mudou-se para cá e os chamadores foram
//! **repontados** para a porta. *Duplicá-lo aqui daria duas respostas à pergunta «como se pinta um
//! aviso», e era exactamente por isso que a outra linha o centralizou.*

use super::*;
use ph2d_editor_core::screens::hero::{InspectorFactoryInfo, InspectorLifecycle};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::tr;

/// O corpo do CICLO DE VIDA.
#[allow(clippy::too_many_arguments)]
fn lifecycle_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    l: &InspectorLifecycle,
    info: &InspectorFactoryInfo,
) -> f32 {
    let mut cur_y = y;
    // ⭐⭐ **A coluna do nome é da SECÇÃO** (`line/UIUX`, 2026-09-15): esta secção nasceu
    //    contra a porta antiga (`anchors::field_row`, o nome POR CIMA do campo) e passa à
    //    única que existe. ⚠️ Os nomes são os da secção INTEIRA, inclusive os das linhas que
    //    este quadro não pinta — *uma coluna que salta quando uma linha aparece é uma coluna
    //    por linha com outro nome.*
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.factory.lifetime_s_0_forever"),
            tr("panel.inspector.factory.off_screen_margin_m"),
        ],
    );
    // ⭐⭐ **A metade honesta** — a lei é *a morte só alcança quem nasceu numa corrida*.
    if !info.is_spawned {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(
                "panel.inspector.factory.nothing_is_born_from_this_object_u_put_this_on_the_recipe_a_factory_makes",
            ),
            ColorToken::Text3,
        );
    }
    if l.lifetime_s.is_some() {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.factory.lifetime_s_0_forever"),
            &[crate::ids::INSP_LIFE_SECONDS],
            0.1, // LITERAL-PX-OK: passo em segundos
            Some(ph2d_editor_core::widget::Unit::Seconds),
            seccao,
        );
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            crate::ids::INSP_LIFE_ON_DEATH,
            TextInput::new(crate::ids::INSP_LIFE_ON_DEATH, "")
                .placeholder(ph2d_i18n::tr("panel.factory.on_death")),
        );
    }
    if l.outside_margin.is_some() {
        if !info.has_game_camera {
            cur_y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                tr("panel.inspector.factory.no_game_camera_u_off_screen_has_no_screen_to_measure"),
                ColorToken::Warn,
            );
        }
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.factory.off_screen_margin_m"),
            &[crate::ids::INSP_LIFE_OUTSIDE_MARGIN],
            0.1, // LITERAL-PX-OK: passo em metros
            Some(ph2d_editor_core::widget::Unit::Meters),
            seccao,
        );
    }
    cur_y
}

/// Pinta a secção LIFECYCLE. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_lifecycle_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorFactoryInfo,
) -> f32 {
    let Some(l) = info.lifecycle.as_ref() else {
        return y;
    };
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_LIFECYCLE_SECTION,
        tr("panel.inspector.factory.lifecycle"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_LIFECYCLE_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let cur_y = lifecycle_body(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y + header_h,
        l,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
