//! ⭐⭐⭐ **O IMPACTO na secção HEALTH** (plano 28, W5) — a pausa da morte, o piscar, quanto do
//! empurrão esta vida aceita e os números de dano.
//!
//! ⚠️ **Irmão do [`super::vida`] por CAP de LOC** (aquele está a `442` de `600`, e estas linhas são
//! um bloco com um assunto só). Ele importa de lá a coluna de números e as caixas — nunca as copia.
//!
//! # ⭐ Linhas que SOMEM conforme os números (a lei do `uses_arg`)
//!
//! - o **piscar** só com invencibilidade: ele dura a janela dela, e sem janela o campo é morto;
//! - a **cor** e a **altura** dos números só com os números ligados.

use super::vida::{caixa, numeros};
use super::*;
use ph2d_editor_core::vida_edits::InspectorHealthInfo;
use ph2d_editor_core::widget::Unit;
use ph2d_i18n::tr;

/// As linhas do IMPACTO. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn corpo_impacto(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    h: &InspectorHealthInfo,
) -> f32 {
    // ⚠️ A coluna mede os rótulos do bloco INTEIRO, inclusive os que este quadro não pinta — uma
    // coluna que salta quando uma linha aparece é uma coluna por linha com outro nome.
    let rotulos = [
        tr("panel.inspector.vida.death_hitstop"),
        tr("panel.inspector.vida.blink"),
        tr("panel.inspector.vida.knockback_taken"),
        tr("panel.inspector.vida.numbers_size"),
    ];
    let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &rotulos);
    let s = Some(Unit::Seconds);
    let mut linhas: Vec<(&str, NodeId, f64, Option<Unit>)> =
        vec![(rotulos[0], ids::INSP_VIDA_DEATH_HITSTOP, 0.01, s)]; // LITERAL-PX-OK: segundos
    if h.invincible_s > 0.0 {
        linhas.push((rotulos[1], ids::INSP_VIDA_BLINK, 0.01, s)); // LITERAL-PX-OK: segundos
    }
    linhas.push((rotulos[2], ids::INSP_VIDA_KNOCKBACK_TAKEN, 0.05, None)); // LITERAL-PX-OK: fracção
    let mut cur_y = numeros(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        &linhas,
        seccao,
    );
    cur_y = caixa(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_VIDA_NUMBERS,
        tr("panel.inspector.vida.numbers"),
        h.numbers,
    );
    if !h.numbers {
        return cur_y;
    }
    let cell = Rect::new(x, cur_y, w, ph2d_tokens::ROW_H_PX);
    super::color_tint::paint_tint_swatch_cell(
        cell,
        tr("panel.inspector.vida.numbers_color"),
        ids::INSP_VIDA_NUMBERS_COLOR,
        crate::state_tint::tint_f32_to_u8(h.numbers_color),
        false,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    cur_y += ph2d_tokens::row_pitch_px();
    numeros(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[(
            rotulos[3],
            ids::INSP_VIDA_NUMBERS_SIZE,
            0.05, // LITERAL-PX-OK: metros
            Some(Unit::Meters),
        )],
        seccao,
    )
}
