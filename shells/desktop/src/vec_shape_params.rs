//! O painel edita a forma VIVA — módulo irmão de [`crate::vec_shape_live`] (que cuida do
//! COZIMENTO e do nascimento da forma; separados pelo teto de 600 LOC, HR-18).
//!
//! O mesmo modelo do texto ([`crate::vec_text_object`]): uma config do painel tem um
//! **ALVO**. Aqui o alvo é a forma paramétrica SELECIONADA — os campos do painel deixam
//! de ser só o *default de desenho* e passam a editar a forma que está na tela, **mesmo
//! na ferramenta Select**. É o que fecha o ciclo Live Shape: desenhar um polígono de 5
//! lados e depois torná-lo de 7.
//!
//! **Tudo aqui é genérico sobre o catálogo.** Nenhuma função conhece estrela, seta ou
//! balão: elas falam `(ShapeKind, ShapeValues)` e consultam o descritor. Uma forma nova
//! entra no catálogo e ganha painel, semente, edição, undo e save sem tocar neste arquivo.
//!
//! Três peças, na ordem em que a shell as usa a cada frame:
//!
//! 1. [`panel_shape_target`] — quem é o alvo (a 1ª forma paramétrica da seleção).
//! 2. [`seed_shape_fields`] + [`ui_values_of`] — quando o alvo MUDA, os campos são
//!    semeados com os parâmetros dele e a tool os ADOTA (painel, tool e objeto passam a
//!    concordar; e a próxima forma desenhada os herda — modelo Figma).
//! 3. [`is_shape_field_id`] + [`apply_shape_field`] + [`edit_selected_shape`] — o
//!    `SetValue` de um campo muta o `VecShape` e RE-COZINHA a geometria in-place.

use ph2d_ecs::{Entity, SimWorld, VecShape};
use ph2d_editor::NodeId;
use ph2d_editor::interaction::WidgetStore;
use ph2d_tool_vector::shapes;
use ph2d_vec_scene::{MAX_SHAPE_FIELDS, ShapeKind, ShapeValues, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;
use crate::vec_shape_live::recook_into;

/// A forma VIVA paramétrica (não-texto) na seleção — o ALVO dos campos de forma do
/// painel. Espelho de [`crate::vec_text_object::panel_text_target`]: a primeira da
/// seleção manda. `None` = nada paramétrico selecionado (os campos voltam ao catálogo).
/// Um discriminante desconhecido (save de uma versão futura) resolve para `None` — vira
/// path cru, nunca pânico.
#[must_use]
pub(crate) fn panel_shape_target(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
) -> Option<(VecPathId, Entity, ShapeKind, ShapeValues)> {
    selection.iter().find_map(|id| {
        let &bits = map.get(id)?;
        let e = Entity::from_bits(bits);
        match sim.world().get::<VecShape>(e) {
            Some(VecShape::Param { kind, values, .. }) => {
                ShapeKind::from_u16(*kind).map(|k| (*id, e, k, *values))
            }
            _ => None,
        }
    })
}

/// `true` se `id` é um campo de parâmetro de forma (o que a shell captura para editar a
/// forma viva selecionada, além do default de desenho que a tool já atualiza).
#[must_use]
pub(crate) fn is_shape_field_id(id: NodeId) -> bool {
    shape_field_index(id).is_some()
}

/// O índice do parâmetro cujo id de campo é `id`.
#[must_use]
pub(crate) fn shape_field_index(id: NodeId) -> Option<usize> {
    (0..MAX_SHAPE_FIELDS).find(|&i| ph2d_editor::ids::vector_shape_field_id(i) == id)
}

/// Aplica a edição de um campo do painel no parâmetro correspondente da forma.
///
/// `v` chega na unidade de UI (px nos raios); a forma guarda MUNDO — a travessia é do
/// catálogo (`shapes::to_world`), num lugar só. `false` = o campo não existe nesta forma
/// (o painel nem o desenha, mas o caminho recusa por construção) e nada muda.
pub(crate) fn apply_shape_field(
    kind: ShapeKind,
    values: &mut ShapeValues,
    id: NodeId,
    v: f64,
    px_to_world: f64,
) -> bool {
    let Some(i) = shape_field_index(id) else {
        return false;
    };
    if i >= shapes::desc(kind).fields.len() {
        return false; // campo de outra forma
    }
    // Autora em UI (o clamp vale na unidade que o usuário vê), depois atravessa a fronteira.
    let mut ui = shapes::to_ui(kind, values, px_to_world);
    ui[i] = v;
    shapes::clamp(kind, &mut ui);
    *values = shapes::to_world(kind, &ui, px_to_world);
    true
}

/// Os valores de uma forma na unidade de UI (o que a tool adota e o painel mostra).
#[must_use]
pub(crate) fn ui_values_of(kind: ShapeKind, world: &ShapeValues, px_to_world: f64) -> ShapeValues {
    shapes::to_ui(kind, world, px_to_world)
}

/// Semente ONE-SHOT dos campos a partir do ALVO (chamada só quando o alvo MUDA): o store
/// é a fonte que o painel pinta, então sem isto os campos mostrariam o último valor
/// autorado em vez dos parâmetros da forma clicada. **Semear todo frame brigaria com o
/// arrasto** — a mesma armadilha da semente do texto.
///
/// Semeia também a FAIXA de cada caixa (`set_number_range`): as faixas são por-forma (3
/// lados · 500 px · 360°), então trocar de forma sem re-registrar deixaria a caixa
/// clampando na faixa da forma anterior.
pub(crate) fn seed_shape_fields(
    store: &mut WidgetStore,
    kind: ShapeKind,
    world: &ShapeValues,
    px_to_world: f64,
) {
    let ui = shapes::to_ui(kind, world, px_to_world);
    let d = shapes::desc(kind);
    for (i, v) in ui.iter().enumerate().take(MAX_SHAPE_FIELDS) {
        let id = ph2d_editor::ids::vector_shape_field_id(i);
        match d.fields.get(i) {
            Some(f) => {
                store.set_number_range(id, f.min, f.max, f.step);
                store.set_number_value(id, *v);
            }
            // Slot sem campo nesta forma: sem isto a faixa da forma ANTERIOR ficaria
            // valendo (e clamparia a próxima que usasse o slot).
            None => store.set_number_range(id, f64::MIN, f64::MAX, 1.0),
        }
    }
}

/// Edita os parâmetros da forma VIVA selecionada (via `f`) e RE-COZINHA in-place — id,
/// estilo e `Transform` preservados, então a forma muda **no lugar** (não pula, não perde
/// o pivô). `f` devolve `false` quando o campo não é dessa forma: aí nada é escrito.
/// `true` se editou. Espelho de [`crate::vec_text_object::edit_selected_text`].
pub(crate) fn edit_selected_shape(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    f: impl FnOnce(ShapeKind, &mut ShapeValues) -> bool,
) -> bool {
    let Some((id, entity, kind, mut values)) = panel_shape_target(sim, map, selection) else {
        return false;
    };
    if !f(kind, &mut values) {
        return false;
    }
    let Some(VecShape::Param { w, h, .. }) = sim.world().get::<VecShape>(entity).cloned() else {
        return false;
    };
    let shape = VecShape::Param {
        kind: kind.as_u16(),
        w,
        h,
        values,
    };
    if !recook_into(scene, id, &shape) {
        return false;
    }
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        e.insert(shape);
    }
    true
}

#[cfg(test)]
#[path = "vec_shape_params_tests.rs"]
mod tests;
