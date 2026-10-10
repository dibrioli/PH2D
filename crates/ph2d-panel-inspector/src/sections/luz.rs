//! ⭐ **O que o Inspector mostra da LUZ** (Luz 2D, W4 — `docs/Luz2D/01_plano.md` §4): Tipo · Cor ·
//! Intensidade · Tamanho · Alcance, e os AVISOS da luz do quadro (a recusa do solver, a lâmpada que
//! a grade não vê, os sprites que entram pelo quad) — recusa que só o terminal vê é botão mudo.
//!
//! A Ambiente não tem posição, tamanho nem alcance: a secção esconde-os em vez de mostrar campos
//! que não fazem nada.

use super::tween_editor::grupo;
use super::*;
use ph2d_editor_core::luz_edits::{AvisoDaLuz, InspectorLuzInfo, TIPOS};
use ph2d_editor_core::widget::{SectionFold, Unit};
use ph2d_i18n::{tr, tr_with};

/// A frase de um aviso, e a cor dele.
fn frase(a: AvisoDaLuz) -> (String, ColorToken) {
    match a {
        AvisoDaLuz::GradeGrandeDemais { lado, limite } => (
            tr_with(
                "panel.inspector.luz.grid_too_big",
                &[("lado", &lado), ("limite", &limite)],
            ),
            ColorToken::Warn,
        ),
        AvisoDaLuz::SolverIndisponivel => (
            tr("panel.inspector.luz.solver_unavailable").to_string(),
            ColorToken::Warn,
        ),
        AvisoDaLuz::PontoSemTamanho => (
            tr("panel.inspector.luz.point_without_size").to_string(),
            ColorToken::Warn,
        ),
        AvisoDaLuz::PequenaDemaisParaOZoom => (
            tr("panel.inspector.luz.too_small_for_zoom").to_string(),
            ColorToken::Warn,
        ),
        AvisoDaLuz::ForaDaVista => (
            tr("panel.inspector.luz.outside_view").to_string(),
            ColorToken::Text3,
        ),
        AvisoDaLuz::SpritesPeloQuad { com_quad, total } => (
            tr_with(
                "panel.inspector.luz.sprites_by_box_n_of_total",
                &[("n", &com_quad), ("total", &total)],
            ),
            ColorToken::Text3,
        ),
    }
}

/// Os avisos, do mais grave para o mais brando (a ordem é a da família), e a frase do quanto a luz
/// acende — o número que o artista pensa («branco a 1 m»), não o que a lei come.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    mut y: f32,
    i: &InspectorLuzInfo,
) -> f32 {
    if i.selected_count > 1 {
        y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            &tr_with(
                "panel.inspector.luz.editing_primary_of_n",
                &[("n", &i.selected_count)],
            ),
            ColorToken::Text3,
        );
    }
    for a in &i.avisos {
        let (texto, cor) = frase(*a);
        y = super::rows::aviso(scene, text_system, theme, x, w, y, &texto, cor);
    }
    let pleno = if i.ambiente() {
        tr_with(
            "panel.inspector.luz.ambient_n_percent",
            &[("n", &format!("{:.0}", i.luz_a_1m * 100.0))], // LITERAL-PX-OK: fração → %
        )
    } else {
        tr_with(
            "panel.inspector.luz.white_at_d_m",
            &[("d", &format!("{:.2}", i.luz_a_1m))],
        )
    };
    y = super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &pleno,
        ColorToken::Text2,
    );
    let Some((nivel, auto, custo)) = i.qualidade else {
        return y;
    };
    let modo = tr(if auto {
        "panel.inspector.luz.automatic"
    } else {
        "panel.inspector.luz.manual"
    });
    let texto = match custo {
        Some(ms) => tr_with(
            "panel.inspector.luz.quality_cost",
            &[
                ("nivel", &tr(nivel.chave())),
                ("modo", &modo),
                ("ms", &format!("{ms:.1}")),
            ],
        ),
        None => tr_with(
            "panel.inspector.luz.quality",
            &[("nivel", &tr(nivel.chave())), ("modo", &modo)],
        ),
    };
    super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &texto,
        ColorToken::Text3,
    )
}

/// O corpo — os CONTROLOS.
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
    i: &InspectorLuzInfo,
) -> f32 {
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.luz.type"),
            tr("panel.inspector.luz.color"),
            tr("panel.inspector.luz.intensity"),
            tr("panel.inspector.luz.size"),
            tr("panel.inspector.luz.limit_range"),
            tr("panel.inspector.luz.range"),
        ],
    );
    let rotulos: Vec<&str> = TIPOS.iter().map(|k| tr(k)).collect();
    let mut cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.luz.type"),
        &crate::ids::INSP_LUZ_TIPO,
        &rotulos,
        usize::from(i.tipo),
        seccao,
    );
    let [r, g, b] = i.cor_srgb;
    cur_y = super::color_tint::paint_tint_swatch_cell(
        Rect::new(x, cur_y, w, ph2d_tokens::ROW_H_PX),
        tr("panel.inspector.luz.color"),
        crate::ids::INSP_LUZ_COR,
        [r, g, b, u8::MAX],
        false,
        store,
        hit_index,
        scene,
        text_system,
        theme,
        seccao,
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.luz.intensity"),
        &[crate::ids::INSP_LUZ_INTENSIDADE],
        0.1, // LITERAL-PX-OK: passo de scrub da fluência (2π = branco a 1 m)
        None,
        seccao,
    );
    if i.ambiente() {
        return cur_y;
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
        tr("panel.inspector.luz.size"),
        &[crate::ids::INSP_LUZ_TAMANHO],
        0.01, // LITERAL-PX-OK: passo de scrub em METROS do raio da lâmpada
        Some(Unit::Meters),
        seccao,
    );
    let limitado = i.alcance_m.is_finite();
    cur_y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_LUZ_ALCANCE_LIGADO,
            tr("panel.inspector.luz.limit_range"),
            limitado,
        ),
        seccao,
    );
    if !limitado {
        return cur_y;
    }
    super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.luz.range"),
        &[crate::ids::INSP_LUZ_ALCANCE],
        0.1, // LITERAL-PX-OK: passo de scrub em METROS
        Some(Unit::Meters),
        seccao,
    )
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_luz_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorLuzInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_LUZ_SECTION,
        tr("panel.inspector.luz.title"),
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
        ph2d_editor_core::ids::INSP_LIVE_LUZ_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    cur_y = avisos(scene, text_system, theme, x, w, cur_y, info);
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
