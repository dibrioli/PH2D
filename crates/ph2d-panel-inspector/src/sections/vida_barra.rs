//! ⭐⭐⭐ **O que o Inspector mostra da BARRA DE VIDA** (plano 28, W4) — a terceira secção da família.
//!
//! ⚠️ **Irmão do [`super::vida_dano`] pelo mesmo motivo**: outra secção, de outro componente, que só
//! partilha com as irmãs a moldura, as caixas e a coluna de números — e essas ele importa de lá.
//!
//! # ⭐⭐ A PRIMEIRA linha diz DE QUEM é a vida que ela mostra
//!
//! A barra lê a vida de outro objecto pelo NOME, e um nome escrito errado desenha **nada** — que se
//! lê exactamente como uma barra partida. ⇒ antes dos números a secção diz o que a barra encontrou
//! (ninguém com esse nome · alguém sem vida · `Shows 30 of 100`), e diz-o pelas MESMAS portas que a
//! ponte usa ao desenhar ([`ph2d_editor_core::vida_edits::BarraAlvo`]).

use super::vida::{cabecalho, caixa, nome, numeros};
use super::*;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::vida_edits::{BarraAlvo, InspectorBarInfo, InspectorVidaInfo};
use ph2d_editor_core::widget::Unit;
use ph2d_i18n::{tr, tr_with};

/// **A frase do que a barra encontrou**, e a cor dela — `Danger` quando ela não desenha nada.
fn o_que_encontrou(b: &InspectorBarInfo) -> (String, ColorToken) {
    let nome = b.target.trim();
    match b.alvo {
        BarraAlvo::SemAlvo => (
            tr_with("panel.inspector.vida.bar_no_target", &[("name", &nome)]),
            ColorToken::Danger,
        ),
        BarraAlvo::SemVida if nome.is_empty() => (
            tr("panel.inspector.vida.bar_own_no_health").to_string(),
            ColorToken::Danger,
        ),
        BarraAlvo::SemVida => (
            tr_with(
                "panel.inspector.vida.bar_target_no_health",
                &[("name", &nome)],
            ),
            ColorToken::Danger,
        ),
        BarraAlvo::Mostra { pontos, max } => (
            tr_with(
                "panel.inspector.vida.bar_shows",
                &[
                    ("now", &format_number(f64::from(pontos))),
                    ("max", &format_number(f64::from(max))),
                ],
            ),
            ColorToken::Text1,
        ),
    }
}

/// O corpo da secção HEALTH BAR.
#[allow(clippy::too_many_arguments)]
fn corpo_barra(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    b: &InspectorBarInfo,
) -> f32 {
    let (frase, cor) = o_que_encontrou(b);
    let mut cur_y = super::rows::aviso(scene, text_system, theme, x, w, y, &frase, cor);
    // ⚠️ O ALVO vem logo a seguir à frase que fala dele — é o campo que a cura de duas das três
    // queixas pede.
    cur_y = nome(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_BARRA_TARGET,
        tr("panel.inspector.vida.bar_target_hint"),
    );
    let rotulos = [
        tr("panel.inspector.vida.bar_width"),
        tr("panel.inspector.vida.bar_height"),
        tr("panel.inspector.vida.bar_offset_x"),
        tr("panel.inspector.vida.bar_offset_y"),
        tr("panel.inspector.vida.bar_trail_delay"),
        tr("panel.inspector.vida.bar_trail_speed"),
    ];
    let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &rotulos);
    let m = Some(Unit::Meters);
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
            (rotulos[0], ids::INSP_BARRA_WIDTH, 0.05, m), // LITERAL-PX-OK: metros
            (rotulos[1], ids::INSP_BARRA_HEIGHT, 0.01, m), // LITERAL-PX-OK: metros
            (rotulos[2], ids::INSP_BARRA_OFFSET_X, 0.05, m), // LITERAL-PX-OK: metros
            (rotulos[3], ids::INSP_BARRA_OFFSET_Y, 0.05, m), // LITERAL-PX-OK: metros
            (
                rotulos[4],
                ids::INSP_BARRA_TRAIL_DELAY,
                0.05, // LITERAL-PX-OK: segundos
                Some(Unit::Seconds),
            ),
            (rotulos[5], ids::INSP_BARRA_TRAIL_SPEED, 0.1, None), // LITERAL-PX-OK: barras/s
        ],
        seccao,
    );
    let [fill, trail, back] = ids::INSP_BARRA_CORES;
    for (id, chave, rgba) in [
        (fill, "panel.inspector.vida.bar_fill", b.fill),
        (trail, "panel.inspector.vida.bar_trail", b.trail),
        (back, "panel.inspector.vida.bar_back", b.back),
    ] {
        let cell = Rect::new(x, cur_y, w, ph2d_tokens::ROW_H_PX);
        super::color_tint::paint_tint_swatch_cell(
            cell,
            tr(chave),
            id,
            crate::state_tint::tint_f32_to_u8(rgba),
            false,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        cur_y += ph2d_tokens::row_pitch_px();
    }
    caixa(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_BARRA_HIDE_FULL,
        tr("panel.inspector.vida.bar_hide_when_full"),
        b.hide_when_full,
    )
}

/// Pinta a secção HEALTH BAR. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_health_bar_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorVidaInfo,
    b: &InspectorBarInfo,
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
        ph2d_editor_core::ids::INSP_LIVE_HEALTH_BAR_SECTION,
        tr("panel.inspector.vida.health_bar"),
        // ⚠️ O aviso da selecção múltipla só uma vez: quem está por cima já o disse.
        if info.health.is_some() || info.damage.is_some() {
            1
        } else {
            info.selected_count
        },
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    let cur_y = corpo_barra(scene, text_system, theme, hit_index, store, x, w, cur_y, b);
    fold.finish(store, scene, hit_index, cur_y)
}
