//! ⭐⭐⭐ **O que o Inspector mostra do DANO** (plano 28, W3) — a metade de QUEM BATE.
//!
//! ⚠️ **Irmão do [`super::vida`] por CAP de LOC** (aquele chegou a `602` contra `600`), e o corte é
//! o certo por RESPONSABILIDADE: a secção DAMAGE é outra secção, de outro componente, que só partilha
//! com a irmã a moldura e as caixas — e essas ele importa de lá, nunca copia.

use super::vida::{cabecalho, caixa, chave_da_queixa, nome, numeros};
use super::*;
use ph2d_editor_core::vida_edits::{InspectorDamageInfo, InspectorVidaInfo, VidaQueixa};
use ph2d_i18n::tr;

/// O corpo da secção DAMAGE.
#[allow(clippy::too_many_arguments)]
fn corpo_dano(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorVidaInfo,
    d: &InspectorDamageInfo,
) -> f32 {
    let mut cur_y = y;
    let queixa = match i.queixa() {
        // ⚠️ *Sem corpo* já foi dito pela secção de vida, se ela está lá.
        Some(VidaQueixa::SemCorpo) if i.health.is_none() => Some(VidaQueixa::SemCorpo),
        Some(VidaQueixa::NaoFere) => Some(VidaQueixa::NaoFere),
        _ => None,
    };
    if let Some(q) = queixa {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_queixa(q)),
            ColorToken::Danger,
        );
    }
    let rotulo = if d.per_second {
        tr("panel.inspector.vida.amount_per_second")
    } else {
        tr("panel.inspector.vida.amount")
    };
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.vida.amount"),
            tr("panel.inspector.vida.amount_per_second"),
            tr("panel.inspector.vida.hitstop"),
            tr("panel.inspector.vida.knockback"),
            tr("panel.inspector.vida.knockback_lift"),
            tr("panel.inspector.vida.team"),
        ],
    );
    let v = Some(ph2d_editor_core::widget::Unit::MetersPerSecond);
    cur_y = numeros(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[
            (rotulo, ids::INSP_DANO_AMOUNT, 1.0, None), // LITERAL-PX-OK: pontos
            // ⭐ O IMPACTO de quem bate (plano 28, W5).
            (
                tr("panel.inspector.vida.hitstop"),
                ids::INSP_DANO_HITSTOP,
                0.01, // LITERAL-PX-OK: segundos
                Some(ph2d_editor_core::widget::Unit::Seconds),
            ),
            (
                tr("panel.inspector.vida.knockback"),
                ids::INSP_DANO_KNOCKBACK,
                0.5, // LITERAL-PX-OK: m/s
                v,
            ),
            (
                tr("panel.inspector.vida.knockback_lift"),
                ids::INSP_DANO_KNOCKBACK_LIFT,
                0.5, // LITERAL-PX-OK: m/s
                v,
            ),
        ],
        seccao,
    );
    for (id, chave, ligada) in [
        (
            ids::INSP_DANO_PER_SECOND,
            "panel.inspector.vida.per_second",
            d.per_second,
        ),
        (
            ids::INSP_DANO_IGNORES_SHIELD,
            "panel.inspector.vida.ignores_shield",
            d.ignores_shield,
        ),
        (
            ids::INSP_DANO_IGNORES_ARMOR,
            "panel.inspector.vida.ignores_armor",
            d.ignores_armor,
        ),
        (
            ids::INSP_DANO_VANISH,
            "panel.inspector.vida.vanish",
            d.vanish,
        ),
    ] {
        cur_y = caixa(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            id,
            tr(chave),
            ligada,
        );
    }
    nome(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.vida.team"),
        ids::INSP_DANO_TEAM,
        tr("panel.inspector.vida.damage_team_hint"),
        seccao,
    )
}

/// Pinta a secção DAMAGE. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_damage_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorVidaInfo,
    d: &InspectorDamageInfo,
) -> f32 {
    let (fold, cur_y) = match cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ph2d_editor_core::ids::INSP_LIVE_DAMAGE_SECTION,
        tr("panel.inspector.vida.damage"),
        // ⚠️ O aviso da selecção múltipla só uma vez: se há vida por cima, ela já o disse.
        if info.health.is_some() {
            1
        } else {
            info.selected_count
        },
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    let cur_y = corpo_dano(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        d,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
