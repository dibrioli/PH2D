//! **Para onde o Ray Sensor aponta e a forma do feixe** (GEF 07 RS): o «Angle», o «Turns with facing», o «Aim at»
//! (RS-3), o «Reach» | «Layer» e o «Thickness» | «Cone» (RS-1, RS-2) — irmão de [`super::ray`] pelo tecto de 600
//! linhas dos painéis.

use super::*;
use ph2d_editor_core::ray_edits::InspectorRayInfo;
use ph2d_i18n::tr;

/// Para onde ele aponta: o «Angle» e o «Turns with facing» — que só decidem sem alvo — e o «Aim at» (RS-3).
#[allow(clippy::too_many_arguments)]
pub(super) fn rumo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorRayInfo,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = y;
    let mirado = !i.aim_at.is_empty() && !i.aim_at_unknown && !i.aim_at_gone;
    if !mirado {
        cur_y = angulo(
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
    }
    let erro = if i.aim_at_unknown {
        (store.focus_id() != Some(crate::ids::INSP_RAY_AIM) && !i.aim_at.is_empty()).then_some((
            "panel.inspector.ray.up_to_nobody_called_x",
            i.aim_at.as_str(),
        ))
    } else {
        i.aim_at_gone
            .then_some(("panel.inspector.ray.aim_is_gone", ""))
    };
    super::conta_gotas_row::linha(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            TextKey::new("component.field.ray_sensor.16"),
            crate::ids::INSP_RAY_AIM,
            TextInput::new(crate::ids::INSP_RAY_AIM, "")
                .placeholder(tr("panel.inspector.ray.aim_none")),
        ),
        (crate::ids::INSP_RAY_AIM_PICK, i.aim_pick_armed),
        seccao,
        erro,
    )
}

/// O «Angle» (graus locais) e o «Turns with facing».
#[allow(clippy::too_many_arguments)]
fn angulo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorRayInfo,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = y;
    // GEF 07 (versão B): um ângulo no lugar de Direction X·Y; a origem é a posição do objeto.
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        TextKey::new("component.field.ray_sensor.0"),
        &[crate::ids::INSP_RAY_ANGLE],
        1.0, // LITERAL-PX-OK: passo de arrasto em GRAUS
        Some(ph2d_editor_core::widget::Unit::Degrees),
        seccao,
    );
    // Opção (dono 15/11): espelhar com o lado do personagem. O estado vem do SNAPSHOT.
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
            crate::ids::INSP_RAY_TURNS,
            TextKey::new("component.field.ray_sensor.3"),
            i.turns_with_facing,
        ),
        seccao,
    );
    cur_y
}

/// «Reach» | «Layer», e «Thickness» | «Cone» (RS-1, RS-2) — dois a dois em cartões.
#[allow(clippy::too_many_arguments)]
pub(super) fn alcance_e_feixe(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorRayInfo,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let cur_y = y;
    let [(xa, wa), (xb, wb)] = if crate::cards::active() {
        ph2d_editor_core::panel::cards::corpo::colunas_iguais(x, w)
    } else {
        [(x, w), (x, w)]
    };
    let depois_do_alcance = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        xa,
        wa,
        cur_y,
        TextKey::new("component.field.ray_sensor.1"),
        &[crate::ids::INSP_RAY_REACH],
        0.1, // LITERAL-PX-OK: passo de arrasto em METROS
        Some(ph2d_editor_core::widget::Unit::Meters),
        seccao,
    );
    let topo_da_camada = if crate::cards::active() {
        cur_y
    } else {
        depois_do_alcance
    };
    let mut t = super::physics_cartao::Tinta {
        scene,
        text: text_system,
        theme,
        hit: hit_index,
        store,
        x: xb,
        w: wb,
    };
    let camada = u8::try_from(i.layer)
        .ok()
        .filter(|&c| usize::from(c) < crate::ids::INSP_PHYS_LAYERS)
        .unwrap_or_default();
    let rotulo = TextKey::new("component.field.ray_sensor.2");
    let quem = &super::physics_colide::DO_RAIO;
    let depois_da_camada = super::camada_por_nome::linha_da_camada(
        &mut t,
        topo_da_camada,
        quem,
        camada,
        rotulo.into(),
        seccao,
    );
    let cur_y = depois_do_alcance.max(depois_da_camada);
    let depois_da_espessura = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        xa,
        wa,
        cur_y,
        TextKey::new("component.field.ray_sensor.14"),
        &[crate::ids::INSP_RAY_THICKNESS],
        0.05, // LITERAL-PX-OK: passo de arrasto em METROS
        Some(ph2d_editor_core::widget::Unit::Meters),
        seccao,
    );
    let topo_do_cone = if crate::cards::active() {
        cur_y
    } else {
        depois_da_espessura
    };
    let depois_do_cone = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        xb,
        wb,
        topo_do_cone,
        TextKey::new("component.field.ray_sensor.15"),
        &[crate::ids::INSP_RAY_CONE],
        1.0, // LITERAL-PX-OK: passo de arrasto em GRAUS
        Some(ph2d_editor_core::widget::Unit::Degrees),
        seccao,
    );
    depois_da_espessura.max(depois_do_cone)
}
