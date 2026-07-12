//! O texto como OBJETO VIVO (Live Shape) — o lado ECS do texto, módulo irmão da SESSÃO
//! ([`crate::vec_text`], que fica com a digitação e o re-cook do compound).
//!
//! Aqui mora tudo o que faz o texto ser um objeto que se lembra de si: o componente
//! `VecShape::Text` na entidade (params primitivos), o ALVO das configs do painel (a
//! sessão viva OU o objeto de texto SELECIONADO — as configs valem também na
//! ferramenta Select, enquanto o texto for texto), a edição/re-cook desse objeto, e o
//! **"Convert to Curves"** (explode num grupo de paths por-letra).
//!
//! Separado de `vec_text.rs` pelo teto de 600 LOC/arquivo (HR-18).

use ph2d_ecs::{Entity, SimWorld, Transform, VecShape, VecTextParams};
use ph2d_tool_vector::TextAlign;
use ph2d_vec_scene::{VecPathId, VecScene};
use ph2d_vector_font::AxisTag;

use crate::vec_entities::{VecEntityMap, group_entities};
use crate::vec_glyph::{TextLayout, text_to_compound_path, text_to_vec_paths};
use crate::vec_text::VecTextEdit;

fn text_params(edit: &VecTextEdit) -> VecTextParams {
    VecTextParams {
        text: edit.text.clone(),
        origin: edit.origin,
        family: edit.family.clone(),
        size: edit.size,
        weight: edit.weight,
        line_height: edit.line_height,
        tracking: edit.tracking,
        align: align_to_u8(edit.align),
        axes: edit
            .extra_axes
            .iter()
            .map(|(t, v)| (t.to_bytes(), *v))
            .collect(),
    }
}

pub(crate) fn align_to_u8(a: TextAlign) -> u8 {
    match a {
        TextAlign::Left => 0,
        TextAlign::Center => 1,
        TextAlign::Right => 2,
    }
}

/// Pendura/atualiza o `VecShape::Text` na entidade do compound do texto ativo, para o
/// objeto lembrar que é texto (re-cook, painel, Convert, save/undo). Chamado a cada
/// frame com sessão viva, DEPOIS do `sync` (a entidade já existe). Idempotente.
pub(crate) fn upsert_text_shape(sim: &mut SimWorld, map: &VecEntityMap, edit: &VecTextEdit) {
    let Some(id) = edit.id else { return };
    let Some(&bits) = map.get(&id) else { return };
    let params = text_params(edit);
    if let Ok(mut e) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) {
        e.insert(VecShape::Text(params));
        // A geometria é centrada no local 0 (pivô no centro); posiciona a baseline no
        // clique via `Transform = origin + center`. Só na sessão viva — depois o gizmo
        // é dono da pose (o modo Select não chama isto, então a translação congela).
        if let Some(mut t) = e.get_mut::<Transform>() {
            t.translation = ph2d_core::Vec2::new(
                (edit.origin[0] + edit.center[0]) as f32,
                (edit.origin[1] + edit.center[1]) as f32,
            );
        }
    }
}

/// `TextLayout` + a `location` de eixos ([wght] ++ extras) a partir dos parâmetros
/// primitivos — o inverso de [`text_params`], para re-cozinhar no Convert.
pub(crate) fn layout_of_params(p: &VecTextParams) -> TextLayout {
    TextLayout {
        size: p.size,
        line_height: p.line_height,
        tracking: p.tracking,
        align: match p.align {
            1 => TextAlign::Center,
            2 => TextAlign::Right,
            _ => TextAlign::Left,
        },
    }
}

pub(crate) fn axes_of_params(p: &VecTextParams) -> Vec<(AxisTag, f32)> {
    let mut axes = Vec::with_capacity(1 + p.axes.len());
    axes.push((AxisTag::WEIGHT, p.weight));
    axes.extend(p.axes.iter().map(|(b, v)| (AxisTag::new(*b), *v)));
    axes
}

/// O objeto de TEXTO na seleção (o 1º), se houver — `(path, entidade, params)`. É o
/// alvo das configs do painel quando NÃO há sessão ativa: o texto continua editável
/// enquanto for texto (não-curva), mesmo com a ferramenta Select.
#[must_use]
pub(crate) fn selected_text_object(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
) -> Option<(VecPathId, Entity, VecTextParams)> {
    selection.iter().find_map(|id| {
        let &bits = map.get(id)?;
        let e = Entity::from_bits(bits);
        match sim.world().get::<VecShape>(e) {
            Some(VecShape::Text(p)) => Some((*id, e, p.clone())),
            _ => None,
        }
    })
}

/// Re-cozinha um objeto de texto com `params` novos: grava o `VecShape` e regenera a
/// geometria (compound CENTRADO, in-place). O `Transform` **não é tocado** — a forma
/// re-cozinha em torno do próprio centro, preservando a pose (e o move do usuário).
pub(crate) fn recook_text_object(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    id: VecPathId,
    entity: Entity,
    params: VecTextParams,
) {
    let font = crate::vec_font::resolve(params.family.as_deref());
    let (fill, stroke) = scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .map_or((None, None), |p| (p.fill.clone(), p.stroke));
    let compound = text_to_compound_path(
        &font,
        &params.text,
        &layout_of_params(&params),
        &axes_of_params(&params),
        [0.0, 0.0],
        &fill,
        &stroke,
    )
    .map(|mut c| {
        let ctr = crate::vec_glyph::path_center(&c);
        crate::vec_glyph::offset_path(&mut c, [-ctr[0], -ctr[1]]);
        c
    });
    if let Some(mut np) = compound
        && let Some(p) = scene.path_mut(id)
    {
        np.id = id;
        *p = np;
    }
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        e.insert(VecShape::Text(params));
    }
}

/// Edita os parâmetros do objeto de texto SELECIONADO (via `f`) e re-cozinha. Usado
/// quando não há sessão ativa — é o que faz as configs do painel agirem sobre um texto
/// já finalizado, na ferramenta Select. `true` se havia um objeto de texto selecionado.
pub(crate) fn edit_selected_text(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    f: impl FnOnce(&mut VecTextParams),
) -> bool {
    let Some((id, e, mut params)) = selected_text_object(sim, map, selection) else {
        return false;
    };
    f(&mut params);
    recook_text_object(sim, scene, id, e, params);
    true
}

/// Troca a família do objeto de texto SELECIONADO e re-semeia os eixos extras (a fonte
/// nova tem os seus), re-cozinhando. `true` se havia um objeto de texto selecionado.
pub(crate) fn set_selected_text_font(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    family: Option<String>,
) -> bool {
    let axes: Vec<([u8; 4], f32)> = crate::vec_font::seed_extra_axes(family.as_deref())
        .into_iter()
        .map(|(t, v)| (t.to_bytes(), v))
        .collect();
    edit_selected_text(sim, scene, map, selection, |p| {
        p.family = family;
        p.axes = axes;
    })
}

/// O `TextAlign` de um `align` primitivo (`0/1/2`) — o inverso de `align_to_u8`.
#[must_use]
pub(crate) fn align_from_u8(a: u8) -> TextAlign {
    match a {
        1 => TextAlign::Center,
        2 => TextAlign::Right,
        _ => TextAlign::Left,
    }
}

/// O ALVO das configs de texto do painel: a sessão viva, senão o objeto de TEXTO
/// selecionado. É o que faz a seção Text aparecer e agir também na ferramenta Select —
/// o texto segue editável enquanto for texto (não-curva).
pub(crate) struct TextTarget {
    pub id: VecPathId,
    pub text: String,
    pub family: Option<String>,
    pub align: TextAlign,
    /// `[size, weight, line_height, tracking]` — a semente dos sliders do painel.
    pub sliders: [f64; 4],
    pub axes: Vec<(AxisTag, f32)>,
}

/// Resolve o alvo: a sessão ativa tem prioridade; sem ela, o objeto de texto
/// selecionado. `None` = nada de texto em foco (a seção some).
#[must_use]
pub(crate) fn panel_text_target(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
    edit: Option<&VecTextEdit>,
) -> Option<TextTarget> {
    if let Some(e) = edit {
        return Some(TextTarget {
            id: e.id.unwrap_or_default(),
            text: e.text.clone(),
            family: e.family.clone(),
            align: e.align,
            sliders: [e.size, f64::from(e.weight), e.line_height, e.tracking],
            axes: e.extra_axes.clone(),
        });
    }
    let (id, _, p) = selected_text_object(sim, map, selection)?;
    Some(TextTarget {
        id,
        text: p.text.clone(),
        family: p.family.clone(),
        align: align_from_u8(p.align),
        sliders: [p.size, f64::from(p.weight), p.line_height, p.tracking],
        axes: p.axes.iter().map(|(b, v)| (AxisTag::new(*b), *v)).collect(),
    })
}

/// "Convert to Curves": para cada objeto de TEXTO na seleção, re-cozinha os glyphs
/// como paths INDIVIDUAIS (no lugar/pose do objeto), agrupa-os (Ungroup depois separa
/// por letra) e descarta o compound + `VecShape`. Formas não-texto ficam intactas.
/// Devolve a nova seleção (os glyph-paths criados + os ids não convertidos).
pub(crate) fn convert_text_selection_to_curves(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    selection: &[VecPathId],
) -> Vec<VecPathId> {
    // (paths dos glyphs a criar por objeto de texto, id do compound a remover).
    let mut converted: Vec<(Vec<VecPathId>, VecPathId)> = Vec::new();
    let mut untouched: Vec<VecPathId> = Vec::new();
    for &id in selection {
        let Some(&bits) = map.get(&id) else {
            untouched.push(id);
            continue;
        };
        let entity = Entity::from_bits(bits);
        let Some(VecShape::Text(params)) = sim.world().get::<VecShape>(entity).cloned() else {
            untouched.push(id); // não é texto (Fase 1 só converte texto)
            continue;
        };
        // Estilo herdado do compound; glyphs re-cozidos por-letra.
        let (fill, stroke) = scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .map_or((None, None), |p| (p.fill.clone(), p.stroke));
        let font = crate::vec_font::resolve(params.family.as_deref());
        let layout = layout_of_params(&params);
        let axes = axes_of_params(&params);
        // Os glyphs são cozidos na BASELINE local e centrados pelo MESMO offset do
        // compound (para caírem na mesma pose), depois a pose do objeto (mundo) é
        // assada em cada um — assim ficam exatamente onde o texto estava.
        let center = text_to_compound_path(
            &font,
            &params.text,
            &layout,
            &axes,
            [0.0, 0.0],
            &fill,
            &stroke,
        )
        .map_or([0.0, 0.0], |c| crate::vec_glyph::path_center(&c));
        let glyphs = text_to_vec_paths(
            &font,
            &params.text,
            &layout,
            &axes,
            [0.0, 0.0],
            &fill,
            &stroke,
        );
        if glyphs.is_empty() {
            untouched.push(id);
            continue;
        }
        let xf = crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(
            sim, entity,
        ));
        let mut new_ids = Vec::new();
        for mut g in glyphs {
            crate::vec_glyph::offset_path(&mut g, [-center[0], -center[1]]);
            ph2d_vec_scene::bake_xform(&mut g, &xf);
            new_ids.push(scene.push_path(g));
        }
        converted.push((new_ids, id));
    }
    if converted.is_empty() {
        return untouched;
    }
    // Remove os compounds; reconcilia entidades (glyph-paths spawnam, compounds
    // despawnam). Assenta o pivô de CADA letra no seu centro (ADR-0112) ANTES de
    // agrupar — depois de agrupadas elas têm `ChildOf` e o settle as pula.
    for (_, compound) in &converted {
        scene.remove_path(*compound);
    }
    crate::vec_entities::sync(sim, scene, map);
    crate::vec_transform::settle_origins(sim, scene, map, &[]);
    let mut result = untouched;
    for (glyph_ids, _) in converted {
        let members: Vec<u64> = glyph_ids
            .iter()
            .filter_map(|p| map.get(p).copied())
            .collect();
        group_entities(sim, &members, "Text".to_owned());
        result.extend(glyph_ids);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec_text::regen_into;
    use ph2d_vec_scene::{Paint, Rgba8};

    fn black() -> Paint {
        Paint::solid(Rgba8::new(0, 0, 0, 255))
    }

    /// O texto vivo é UM objeto (compound + `VecShape::Text`); "Convert to Curves"
    /// explode num grupo de paths por-letra, some com o compound e re-seleciona os
    /// glyphs. É o coração do modelo Live Shapes para texto.
    #[test]
    fn convert_explodes_text_into_a_grouped_per_letter_set() {
        use ph2d_ecs::ChildOf;
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = VecEntityMap::new();
        let mut edit = Some(VecTextEdit {
            origin: [0.0, 0.0],
            size: 1.0,
            weight: 400.0,
            line_height: 1.2,
            tracking: 0.0,
            align: TextAlign::Left,
            extra_axes: Vec::new(),
            family: None,
            fill: Some(black()),
            stroke: None,
            text: "Hi".to_string(),
            id: None,
            center: [0.0, 0.0],
        });
        // Texto vivo: 1 compound + entidade + VecShape::Text.
        regen_into(&mut scene, edit.as_mut().unwrap());
        crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
        upsert_text_shape(&mut sim, &map, edit.as_ref().unwrap());
        let compound = edit.unwrap().id.expect("o compound do texto");
        assert_eq!(scene.paths().len(), 1, "texto vivo = UM objeto");

        // Converte em curvas.
        let new_sel = convert_text_selection_to_curves(&mut sim, &mut scene, &mut map, &[compound]);
        assert!(
            scene.paths().len() >= 2,
            "explodiu em vários glyph-paths (H, i)"
        );
        assert!(
            !scene.paths().iter().any(|p| p.id == compound),
            "o compound do texto sumiu"
        );
        assert_eq!(
            new_sel.len(),
            scene.paths().len(),
            "a seleção são os glyphs"
        );
        // Todo glyph-path virou filho de um grupo.
        let grouped = new_sel
            .iter()
            .filter(|id| {
                map.get(id)
                    .is_some_and(|&b| sim.world().get::<ChildOf>(Entity::from_bits(b)).is_some())
            })
            .count();
        assert_eq!(grouped, new_sel.len(), "todos os glyphs num grupo");
    }
}
