//! **O que o Inspector mostra do MOTION INPUT** (ciclo 19 do Motion) — o objecto que publica uma
//! entrada para o grafo.
//!
//! A QUEIXA vem antes dos campos (porta [`InspectorMotionInputInfo::queixa`]); os nomes que o grafo
//! lê são chips que ESCREVEM o nome; e as linhas `Value`/`Color` só existem com a fonte que as lê
//! (uma linha que não mexe nada é um controlo morto).

use super::*;
use ph2d_editor_core::motion_input_edits::{
    InspectorMotionInputInfo, MOTION_INPUT_SOURCES, MotionInputQueixa,
};
use ph2d_editor_core::property_row::{Seccao, paint_choice_row};
use ph2d_i18n::{tr, tr_with};

/// Os rótulos das fontes, pela ordem de `MotionSource::ALL`.
const FONTES: [&str; MOTION_INPUT_SOURCES] = [
    "panel.inspector.motion_input.source.0",
    "panel.inspector.motion_input.source.1",
    "panel.inspector.motion_input.source.2",
    "panel.inspector.motion_input.source.3",
];

const _: () = assert!(crate::ids::INSP_MI_SOURCE.len() == MOTION_INPUT_SOURCES);

#[must_use]
const fn chave_da_queixa(q: MotionInputQueixa) -> &'static str {
    match q {
        MotionInputQueixa::NomeVazio => "panel.inspector.motion_input.empty_name",
        MotionInputQueixa::GrafoSemEntradas => "panel.inspector.motion_input.graph_has_no_input",
        MotionInputQueixa::NomeNaoLido => "panel.inspector.motion_input.name_not_read",
        MotionInputQueixa::SemVida => "panel.inspector.motion_input.no_health",
    }
}

/// Os chips dos nomes que o grafo lê (o aceso é o nome actual), e a linha do que não coube.
#[allow(clippy::too_many_arguments)]
fn nomes_do_grafo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorMotionInputInfo,
    seccao: Seccao,
) -> f32 {
    if i.nomes_do_grafo.is_empty() {
        return y;
    }
    let atual = i.name.trim();
    let chips: Vec<(&str, bool, NodeId)> = i
        .nomes_do_grafo
        .iter()
        .zip(crate::ids::INSP_MI_GRAFO)
        .map(|(n, id)| (n.as_str(), n == atual, id))
        .collect();
    let mut cur_y = paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.motion_input.graph_reads"),
        &chips,
        seccao,
    );
    let resto = i
        .nomes_do_grafo
        .len()
        .saturating_sub(crate::ids::INSP_MI_GRAFO.len());
    if resto > 0 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with(
                "panel.inspector.motion_input.more_names",
                &[("n", &resto.to_string())],
            ),
            ColorToken::Text3,
        );
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_motion_input_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorMotionInputInfo,
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
        ph2d_editor_core::ids::INSP_LIVE_MOTION_INPUT_SECTION,
        tr("component.motion_input.name"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    if let Some(q) = i.queixa() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_queixa(q)),
            ColorToken::Warn,
        );
    }
    let rotulos = [
        tr("component.field.motion_input_fields.1"),
        tr("component.field.motion_input_fields.2"),
        tr("component.field.motion_input_fields.3"),
        tr("component.field.motion_input_fields.4"),
        tr("panel.inspector.motion_input.graph_reads"),
    ];
    let seccao = Seccao::medida(text_system, 1, &rotulos);
    let nome = crate::ids::INSP_MI_NAME;
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        rotulos[0],
        nome,
        TextInput::new(nome, "").placeholder(tr("panel.inspector.motion_input.name_hint")),
        seccao,
    );
    cur_y = nomes_do_grafo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        i,
        seccao,
    );
    let fontes: Vec<(&str, bool, NodeId)> = crate::ids::INSP_MI_SOURCE
        .iter()
        .enumerate()
        .map(|(k, &id)| (tr(FONTES[k]), k == usize::from(i.source), id))
        .collect();
    cur_y = paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        rotulos[1],
        &fontes,
        seccao,
    );
    if i.le_valor() {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            rotulos[2],
            &[crate::ids::INSP_MI_VALUE],
            0.01, // LITERAL-PX-OK: passo de arrasto de uma fracção 0..1
            None,
            seccao,
        );
    }
    if i.le_cor() {
        cur_y = super::color_tint::bloco_de_cores(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            &[(crate::ids::INSP_MI_COLOR, rotulos[3], i.color)],
        );
    }
    fold.finish(store, scene, hit_index, cur_y)
}
