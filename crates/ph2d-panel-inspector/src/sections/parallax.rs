//! ⭐⭐⭐ **O que o Inspector mostra da PARALAXE** (plano 24, W7).
//!
//! # ⭐⭐ UMA secção, QUATRO componentes — e os três de baixo são BLOCOS
//!
//! A secção existe se o objecto tiver `ScrollFactor` (que é o que liga a lei); o ladrilho, a deriva
//! e a cerca aparecem **só quando o componente deles está lá**. A razão de não serem quatro secções
//! está escrita no [`ph2d_editor_core::parallax_edits`]: *quatro populações, um assunto*.
//!
//! # ⭐ O painel DIZ porque é que nada se mexe, e são DUAS razões
//!
//! | aviso | a cura |
//! |---|---|
//! | `There is no Game Camera in the scene` | pôr uma câmera do jogo |
//! | `This layer moves with the world` | baixar o `Scroll Factor` |
//!
//! ⚠️ **A ordem é a da recusa** (a mais geral primeiro) e a escada vive no
//! [`ph2d_editor_core::parallax_edits::InspectorParallaxInfo::queixa`] — dizer *«esta camada anda
//! com o mundo»* a quem não tem câmera nenhuma manda-o resolver a metade errada.

use super::*;
use ph2d_editor_core::parallax_edits::{InspectorParallaxInfo, ParallaxQueixa, Profundidade};
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::Unit;
use ph2d_i18n::{tr, tr_with};

/// A chave da frase de cada queixa. ⚠️ Um `match` e não uma tabela indexada: a posição de uma
/// variante não é a tag de nada aqui, e uma tabela criaria essa dependência do nada.
const fn chave_da_queixa(q: ParallaxQueixa) -> &'static str {
    match q {
        ParallaxQueixa::SemCamera => "panel.inspector.parallax.there_is_no_game_camera",
        ParallaxQueixa::CameraDesligada => "panel.inspector.parallax.the_game_camera_is_off",
        ParallaxQueixa::Atravessa => {
            "panel.inspector.parallax.the_camera_passes_through_this_layer"
        }
        ParallaxQueixa::OutroMotor => "panel.inspector.parallax.another_motor_moves_this_object",
        ParallaxQueixa::Neutra => "panel.inspector.parallax.this_layer_moves_with_the_world",
    }
}

/// ⭐⭐⭐ **A frase da DISTÂNCIA de um eixo** — a leitura derivada que o plano pede (§W7), e nunca
/// um campo. `None` para o mundo, que não tem nada a dizer (e o neutro já tem a queixa dele).
fn frase_da_profundidade(k: f32) -> Option<String> {
    let um_decimal = |v: f32| format!("{v:.1}");
    match Profundidade::de(k) {
        Profundidade::Mundo => None,
        Profundidade::Ecra => Some(tr("panel.inspector.parallax.pinned_to_the_screen").to_owned()),
        Profundidade::Longe(n) => Some(tr_with(
            "panel.inspector.parallax.n_times_as_far_as_the_world",
            &[("n", &um_decimal(n))],
        )),
        Profundidade::Frente(n) => Some(tr_with(
            "panel.inspector.parallax.in_front_of_the_world_at_n",
            &[("n", &um_decimal(n))],
        )),
        Profundidade::Contra => {
            Some(tr("panel.inspector.parallax.moves_against_the_camera").to_owned())
        }
    }
}

/// ⭐ **As frases a pintar**: uma se os dois eixos contam a mesma distância, uma por eixo (com o
/// nome dele) se não — uma camada pode estar, em profundidade, em dois sítios ao mesmo tempo (o doc
/// do `ScrollFactor::escala`).
fn frases_da_profundidade(k: [f32; 2]) -> Vec<String> {
    if k[0].to_bits() == k[1].to_bits() {
        return frase_da_profundidade(k[0]).into_iter().collect();
    }
    // ⚠️ O nome do eixo e a composição da frase vêm da TABELA (HR-15, auditoria 26 §3): um
    // `format!("{eixo}: {f}")` aqui era o único literal de UI desta crate.
    [
        ("panel.inspector.parallax.axis_x_phrase", k[0]),
        ("panel.inspector.parallax.axis_y_phrase", k[1]),
    ]
    .into_iter()
    .filter_map(|(chave, v)| frase_da_profundidade(v).map(|f| tr_with(chave, &[("f", &f)])))
    .collect()
}

/// O corpo da secção — os CONTROLOS.
///
/// ⚠️ **A tabela é montada ANTES de ser pintada** e é lida DUAS vezes (para medir a coluna mais
/// larga e para pintar) — a lei do [`ph2d_editor_core::property_row::Seccao`], que a secção da
/// câmera já escreve.
#[allow(clippy::too_many_arguments)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorParallaxInfo,
) -> f32 {
    let mut cur_y = y;
    // ⚠️ **A QUEIXA primeiro** — quem não vê nada mexer não quer afinar um ladrilho.
    if let Some(q) = i.queixa() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_queixa(q)),
            ColorToken::Text3,
        );
    }
    // ⭐ **As NOTAS** — a lei corre, e o painel diz o que o artista pode não estar a ver.
    for (vale, chave) in [
        (
            i.nota_pre_visualizacao(),
            "panel.inspector.parallax.the_game_camera_preview_is_off",
        ),
        (
            i.limites_inertes(),
            "panel.inspector.parallax.no_limit_axis_is_active",
        ),
    ] {
        if vale {
            cur_y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                tr(chave),
                ColorToken::Text3,
            );
        }
    }

    // ⚠️ **Cada linha carrega o PRÓPRIO passo e a PRÓPRIA unidade**, e não um partilhado: o factor
    // é adimensional e a `0,05` (um vigésimo do curso útil), o ladrilho e a cerca são METROS, e a
    // deriva é metros por SEGUNDO. *O gate do número mágico pede-o, e pintar «m» num factor seria
    // mentir sobre o que o número é.*
    let mut linhas: Vec<(&'static str, &[ph2d_a11y::NodeId], f64, Option<Unit>)> = vec![(
        tr("panel.inspector.parallax.scroll_factor"),
        &[ids::INSP_PARALLAX_K_X, ids::INSP_PARALLAX_K_Y][..],
        0.05, // LITERAL-PX-OK: fracção adimensional
        None,
    )];
    if i.repeat.is_some() {
        linhas.push((
            tr("panel.inspector.parallax.repeat_m"),
            &[ids::INSP_PARALLAX_TILE_X, ids::INSP_PARALLAX_TILE_Y][..],
            0.5, // LITERAL-PX-OK: passo em metros
            Some(Unit::Meters),
        ));
    }
    if i.motion.is_some() {
        linhas.push((
            tr("panel.inspector.parallax.drift_m_s"),
            &[ids::INSP_PARALLAX_VEL_X, ids::INSP_PARALLAX_VEL_Y][..],
            0.05, // LITERAL-PX-OK: passo em m/s
            // ⚠️ A unidade existe e o doc dizia m/s — a fileira pintava sem nenhuma (auditoria 26).
            Some(Unit::MetersPerSecond),
        ));
    }
    if i.limits.is_some() {
        linhas.push((
            tr("panel.inspector.parallax.limit_min_m"),
            &[ids::INSP_PARALLAX_MIN_X, ids::INSP_PARALLAX_MIN_Y][..],
            0.5, // LITERAL-PX-OK: passo em metros
            Some(Unit::Meters),
        ));
        linhas.push((
            tr("panel.inspector.parallax.limit_max_m"),
            &[ids::INSP_PARALLAX_MAX_X, ids::INSP_PARALLAX_MAX_Y][..],
            0.5, // LITERAL-PX-OK: passo em metros
            Some(Unit::Meters),
        ));
    }
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        2,
        &linhas.iter().map(|t| t.0).collect::<Vec<_>>(),
    );
    for (n, (label, ids2, step, unit)) in linhas.into_iter().enumerate() {
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
            ids2,
            step,
            unit,
            seccao,
        );
        // ⭐ A DISTÂNCIA pinta-se COLADA ao factor (a 1.ª fileira), que é o número que ela lê.
        if n == 0 {
            for frase in frases_da_profundidade(i.factor) {
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
        }
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_parallax_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorParallaxInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PARALLAX_SECTION,
        tr("panel.inspector.parallax.parallax"),
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
        ph2d_editor_core::ids::INSP_LIVE_PARALLAX_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    if info.selected_count > 1 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.parallax.editing_the_primary_selection_only"),
            ColorToken::Text3,
        );
    }
    cur_y = corpo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}

#[cfg(test)]
mod tests {
    use super::frases_da_profundidade;

    /// ⭐⭐ **As cinco espécies, e a metade dos DOIS EIXOS.** ⚠️ `0,5` e não um número qualquer: é
    /// o único em que `1/k` e `k` coincidiriam num erro de inversão (`2,0` contra `0,5`), logo ele
    /// separa a lei da mutação que esquecer o `1/`.
    #[test]
    fn a_distancia_diz_o_que_o_factor_significa() {
        assert!(
            frases_da_profundidade([1.0, 1.0]).is_empty(),
            "o mundo nao tem frase"
        );
        let longe = frases_da_profundidade([0.5, 0.5]);
        assert_eq!(longe.len(), 1);
        assert!(
            longe[0].contains("2.0"),
            "k = 0,5 e' o DOBRO da distancia: {longe:?}"
        );
        let frente = frases_da_profundidade([2.0, 2.0]);
        assert!(
            frente[0].contains("0.5"),
            "k = 2 esta' a METADE da distancia: {frente:?}"
        );
        assert_ne!(frases_da_profundidade([0.0, 0.0]), longe);
        assert_ne!(frases_da_profundidade([-1.0, -1.0]), frente);
        let dois = frases_da_profundidade([0.5, 1.0]);
        assert_eq!(dois.len(), 1, "o eixo Y e' o mundo e cala-se: {dois:?}");
        assert!(
            dois[0].starts_with("X: "),
            "com eixos diferentes a frase NOMEIA o eixo"
        );
    }
}
