//! **Para onde o Ray Sensor aponta e a forma do feixe** (GEF 07 RS): o «Angle», o «Turns with facing», o «Aim at»
//! (RS-3), o «Reach» | «Layer» e a «Shape» (Line · Thick · Cone, dono 15/11) com o número da forma activa — irmão de [`super::ray`] pelo tecto de 600
//! linhas dos painéis.

use super::*;
use ph2d_editor_core::ray_edits::{InspectorRayInfo, RayShapeKind};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
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

/// «Reach» | «Layer» (dois a dois em cartões), a «Shape» e, por baixo, SÓ o número da forma activa.
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
    let cur_y = linha_da_forma(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        (x, w, cur_y),
        i.shape,
        seccao,
    );
    // Só o número da forma activa: um campo que não decide nada seria um controlo morto.
    let (rotulo, id, passo, unidade) = match i.shape {
        RayShapeKind::Line => return cur_y,
        RayShapeKind::Thick => (
            "component.field.ray_sensor.14",
            crate::ids::INSP_RAY_THICKNESS,
            0.05, // LITERAL-PX-OK: passo de arrasto em METROS
            ph2d_editor_core::widget::Unit::Meters,
        ),
        RayShapeKind::Cone => (
            "component.field.ray_sensor.15",
            crate::ids::INSP_RAY_CONE,
            1.0, // LITERAL-PX-OK: passo de arrasto em GRAUS
            ph2d_editor_core::widget::Unit::Degrees,
        ),
    };
    super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        TextKey::new(rotulo),
        &[id],
        passo,
        Some(unidade),
        seccao,
    )
}

/// As opções da «Shape», na ordem do [`RayShapeKind::ALL`] (e dos ids).
pub(crate) fn opcoes_da_forma() -> Vec<DropdownOption<u8>> {
    const ROTULOS: [&str; 3] = [
        "panel.inspector.ray.shape_line",
        "panel.inspector.ray.shape_thick",
        "panel.inspector.ray.shape_cone",
    ];
    crate::ids::INSP_RAY_SHAPE_OPTS
        .iter()
        .zip(ROTULOS)
        .zip(0_u8..)
        .map(|((&id, k), n)| DropdownOption::new(id, n, tr(k)))
        .collect()
}

/// O índice de `forma` na lista.
pub(crate) fn indice_da_forma(forma: RayShapeKind) -> u8 {
    RayShapeKind::ALL
        .iter()
        .position(|&f| f == forma)
        .and_then(|n| u8::try_from(n).ok())
        .unwrap_or_default()
}

/// ⭐ **A linha «Shape»** (dono 15/11: *«thickness e cone são conflitantes»*): a lista suspensa Line · Thick · Cone,
/// com o popover diferido (o mesmo passe da lista da camada, nos dois modos) e o losango do canal.
#[allow(clippy::too_many_arguments)]
fn linha_da_forma(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    forma: RayShapeKind,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let id = crate::ids::INSP_RAY_SHAPE;
    let rotulo = TextKey::new("component.field.ray_sensor.18");
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ph2d_tokens::ROW_H_PX,
        tr("component.field.ray_sensor.18"),
        seccao,
    );
    hit_index.register(id, linha.control);
    let aberto = matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let sel = indice_da_forma(forma);
    let mut dd = Dropdown::new(id, "", opcoes_da_forma())
        .open(aberto)
        .visual(store.dropdown_visual(id));
    dd.select(sel);
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    if aberto {
        crate::state_popovers::set_pending_ray_shape_dd(Some((sel, linha.control)));
    }
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_campo(
        scene,
        theme,
        linha.dot,
        &[id],
        rotulo.into(),
    ) {
        hit_index.register(alvo, linha.dot);
    }
    linha.fim(ph2d_tokens::row_pitch_px())
}
