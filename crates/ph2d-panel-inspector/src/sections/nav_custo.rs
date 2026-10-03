//! ⭐⭐ (W7) **O que custa e os ATALHOS** — as secções NAV COST AREA e NAV LINK, e os interruptores do
//! agente (com o *Avoid Harm*). Irmão do [`super::nav`] pelo tecto de LOC do painel.
//!
//! # ⭐ Cada caixa que não faz nada DIZ porquê
//!
//! - *Avoid Harm* num agente sem `Health`: nada o fere, logo ele não evita nada.
//! - Uma área sem colisor não tem forma; num corpo `Dynamic` a ponte não a vê (só recorta parada).
//! - Uma área PROIBIDA é um furo: o custo deixa de ser lido, e a linha dele sai (um controlo morto).
//! - Um atalho sem saída, ou com uma saída que ninguém tem, não leva a lado nenhum.
//!
//! As perguntas vivem nas portas de `nav_edits` (gateadas sem device); aqui só a língua e a cor.

use super::*;
use ph2d_editor_core::nav_edits::{
    CostAreaQueixa, InspectorNavAgent, InspectorNavCostArea, InspectorNavLink, LinkQueixa,
};
use ph2d_editor_core::property_row::{Seccao, paint_check_row};
use ph2d_editor_core::widget::Unit;
use ph2d_i18n::tr;

#[must_use]
const fn chave_da_area(q: CostAreaQueixa) -> &'static str {
    match q {
        CostAreaQueixa::SemForma => "panel.inspector.nav.area_needs_a_collider",
        CostAreaQueixa::CorpoQueAnda => "panel.inspector.nav.area_body_moves",
    }
}

#[must_use]
const fn chave_do_atalho(q: LinkQueixa) -> (&'static str, ColorToken) {
    match q {
        LinkQueixa::SemSaida => ("panel.inspector.nav.link_has_no_exit", ColorToken::Text3),
        LinkQueixa::SaidaPerdida => ("panel.inspector.nav.link_exit_lost", ColorToken::Warn),
    }
}

/// Os três interruptores do agente — *Active*, *Avoid Others* e (W7) *Avoid Harm*, com a frase de
/// quando o último não muda nada. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn interruptores_do_agente(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    a: &InspectorNavAgent,
    seccao: Seccao,
) -> f32 {
    let mut cur_y = y;
    for caixa in [
        (
            crate::ids::INSP_NAV_ACTIVE,
            tr("panel.inspector.nav.active"),
            a.active,
        ),
        (
            crate::ids::INSP_NAV_AVOIDANCE,
            tr("panel.inspector.nav.avoidance"),
            a.avoidance,
        ),
        (
            crate::ids::INSP_NAV_AVOID_HARM,
            tr("panel.inspector.nav.avoid_harm"),
            a.avoid_harm,
        ),
    ] {
        cur_y = paint_check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            caixa,
            seccao,
        );
    }
    if a.evitar_dano_nao_muda_nada() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.nav.no_health_nothing_hurts_it"),
            ColorToken::Text3,
        );
    }
    cur_y
}

/// Pinta a secção NAV COST AREA. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_nav_cost_area_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    c: &InspectorNavCostArea,
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
        core_ids::INSP_LIVE_NAV_COST_AREA_SECTION,
        tr("panel.inspector.nav.nav_cost_area"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    if let Some(q) = c.queixa() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_area(q)),
            ColorToken::Warn,
        );
    }
    let seccao = Seccao::medida(
        text_system,
        2,
        &[
            tr("panel.inspector.nav.cost"),
            tr("panel.inspector.nav.forbidden"),
        ],
    );
    cur_y = paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_NAV_AREA_FORBIDDEN,
            tr("panel.inspector.nav.forbidden"),
            c.forbidden,
        ),
        seccao,
    );
    // ⚠️ **Proibida não LÊ o custo** (é um furo, não um custo infinito) — a linha sai, e a frase
    // diz o que a área faz no lugar dela.
    cur_y = if c.forbidden {
        super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.nav.no_agent_enters"),
            ColorToken::Text3,
        )
    } else {
        super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.nav.cost"),
            &[crate::ids::INSP_NAV_AREA_COST],
            0.05, // LITERAL-PX-OK: passo de arrasto, × chão
            None,
            seccao,
        )
    };
    fold.finish(store, scene, hit_index, cur_y)
}

/// Pinta a secção NAV LINK. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn paint_nav_link_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    l: &InspectorNavLink,
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
        core_ids::INSP_LIVE_NAV_LINK_SECTION,
        tr("panel.inspector.nav.nav_link"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    if let Some(q) = l.queixa() {
        let (chave, cor) = chave_do_atalho(q);
        cur_y = super::rows::aviso(scene, text_system, theme, x, w, cur_y, tr(chave), cor);
    }
    let seccao = Seccao::medida(
        text_system,
        2,
        &[
            tr("panel.inspector.nav.link_exit"),
            tr("panel.inspector.nav.extra_cost"),
            tr("panel.inspector.nav.on_cross"),
        ],
    );
    // ⚠️ **O NOME da saída, pela MESMA porta do alvo `Object`** do agente: o despacho manda o texto
    // cru e o dreno resolve-o para o `stable_name_id` (nunca os bits — o undo respawna o mundo).
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.nav.link_exit"),
        crate::ids::INSP_NAV_LINK_TO,
        TextInput::new(crate::ids::INSP_NAV_LINK_TO, "")
            .placeholder(tr("panel.inspector.nav.exit_object_name_u")),
        seccao,
    );
    for caixa in [
        (
            crate::ids::INSP_NAV_LINK_TELEPORT,
            tr("panel.inspector.nav.teleport"),
            l.teleport,
        ),
        (
            crate::ids::INSP_NAV_LINK_TWO_WAY,
            tr("panel.inspector.nav.two_way"),
            l.two_way,
        ),
    ] {
        cur_y = paint_check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            caixa,
            seccao,
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
        tr("panel.inspector.nav.extra_cost"),
        &[crate::ids::INSP_NAV_LINK_COST],
        0.1, // LITERAL-PX-OK: passo de arrasto em METROS de chão
        Some(Unit::Meters),
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
        tr("panel.inspector.nav.on_cross"),
        crate::ids::INSP_NAV_LINK_ON_CROSSED,
        TextInput::new(crate::ids::INSP_NAV_LINK_ON_CROSSED, "")
            .placeholder(tr("panel.inspector.nav.signal_when_an_agent_crosses_u")),
        seccao,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
