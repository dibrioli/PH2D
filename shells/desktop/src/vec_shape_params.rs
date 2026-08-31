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
//! 1. [`shape_field_target`] — de quem são os campos (a 1ª forma paramétrica da seleção, ou
//!    NINGUÉM quando o artista está armado para desenhar). A porta única; [`panel_shape_target`]
//!    é a metade dela que só sabe olhar a seleção.
//! 2. [`shape_seed_focus`] + [`seed_shape_fields`] + [`ui_values_of`] — quando o PAR
//!    `(alvo, tipo)` muda, os campos são semeados e a tool os ADOTA (painel, tool e objeto
//!    passam a concordar; e a próxima forma desenhada os herda — modelo Figma).
//! 3. [`is_shape_field_id`] + [`apply_shape_field`] + [`edit_selected_shape`] — o
//!    `SetValue` de um campo muta o `VecShape` e RE-COZINHA a geometria in-place.

use ph2d_ecs::{Entity, SimWorld, VecShape};
use ph2d_editor::NodeId;
use ph2d_editor::interaction::WidgetStore;
use ph2d_tool_vector::params::DrawMode;
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

/// ⭐⭐⭐ **DE QUEM SÃO OS CAMPOS DE FORMA NESTE FRAME** — a porta ÚNICA da pergunta, e a razão
/// de [`panel_shape_target`] não ser chamada por mais ninguém do produto.
///
/// ⚠️ **Um artista ARMADO não edita a selecção.** Clicar numa forma do catálogo põe a tool em
/// [`DrawMode::Shape`] (`VectorTool::set_shape` — escolher a forma sem armar o gesto seria um
/// clique morto): a partir daí o gesto seguinte DESENHA aquela forma, e é dela que os campos
/// falam. Enquanto o alvo vivo vencia, trocar de forma no catálogo deixava o painel a mostrar os
/// parâmetros da forma **anterior** até alguém desenhar — o report do Enio de 2026-08-31,
/// *"troco de Shape na tool Shape e as propriedades não trocam imediatamente"*.
///
/// ⚠️⚠️ **E o que o painel PINTA e o que a caixa ESCREVE têm de sair daqui os dois.** Com a
/// pintura a mostrar a Estrela armada e a escrita a alcançar o Polígono selecionado, digitar
/// *"Pontas = 9"* punha **9 lados** no polígono — os slots são por ÍNDICE, então o campo `0` de
/// uma forma cai no campo `0` da outra sem erro nenhum — *pintar por uma porta e escrever por
/// outra é o defeito, não a divergência de valores*.
///
/// ⚠️⚠️ **E o MODO sozinho não responde** — foi a 1.ª redacção desta cura, e ela matava o ciclo
/// Live Shape: *"desenhei uma estrela, deixa-me ajustar as pontas dela"* e *"armei o Polígono,
/// mostra-me o Polígono"* são os dois `DrawMode::Shape` com uma forma viva selecionada. O que os
/// separa é **qual gesto veio por último**, e é isso que o `armed` carrega
/// (`App::vec_shape_armed`: a tool publica o clique, a shell apaga o latch quando a selecção
/// muda — e desenhar selecciona a forma nova, então o ciclo volta sozinho).
///
/// ⛔ O modo **Moldura** NÃO entra: ali o gesto desenha um `RoundRect` e um `RoundRect`
/// selecionado é o mesmo objeto — o alvo vivo continua a mandar, como em Select.
#[must_use]
pub(crate) fn shape_field_target(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
    mode: DrawMode,
    armed: bool,
) -> Option<(VecPathId, Entity, ShapeKind, ShapeValues)> {
    if armed && mode == DrawMode::Shape {
        return None;
    }
    panel_shape_target(sim, map, selection)
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

/// **De quem são os campos de forma NESTE frame** — o par que decide a semente.
///
/// O ALVO manda quando existe (o artista está a editar uma forma viva); sem alvo mandam o
/// CATÁLOGO e os valores que a tool guarda para aquele tipo, que é o default do próximo desenho.
///
/// ⚠️ **É um PAR, e não só o alvo.** Dois estados diferentes — *"nada selecionado, catálogo em
/// Star"* e *"nada selecionado, catálogo em Polygon"* — têm o MESMO `None` de alvo, então uma memo
/// que guardasse só o id os trataria como o mesmo frame e nunca re-semearia. Foi exactamente esse
/// o defeito reportado.
#[must_use]
pub(crate) fn shape_seed_focus(
    target: Option<(VecPathId, ShapeKind)>,
    catalog: ShapeKind,
) -> (Option<VecPathId>, ShapeKind) {
    match target {
        Some((id, kind)) => (Some(id), kind),
        None => (None, catalog),
    }
}

/// Semente ONE-SHOT dos campos, em unidade de **UI** (a que a caixa mostra): o store é a fonte
/// que o painel pinta, então sem isto os campos mostrariam o último valor autorado em vez dos
/// parâmetros da forma em foco. **Semear todo frame brigaria com o arrasto** — a mesma armadilha
/// da semente do texto.
///
/// ⚠️ **Ela toma UI e não MUNDO**, e isso não é conveniência: o chamador já precisa dos valores
/// de UI para o `adopt_shape_values` da tool, então converter aqui dentro faria a MESMA conversão
/// duas vezes, em dois lugares — a forma exata de duas respostas que divergem no dia em que uma
/// unidade nova entra ([[feedback_derived_coordinate_seed_must_match_sample]]). Agora a conversão
/// é UMA, no chamador, e os dois consumidores leem o mesmo array.
///
/// ⚠️ **O gatilho é o PAR `(alvo, tipo)`** — ver [`shape_seed_focus`]. Com só o alvo, escolher
/// outra forma no CATÁLOGO com nada selecionado comparava `None == None` e a semente nunca
/// corria: os campos ficavam com os números da forma anterior (report do Enio, 2026-08-01).
///
/// Semeia também a FAIXA de cada caixa (`set_number_range`): as faixas são por-forma (3
/// lados · 500 px · 360°), então trocar de forma sem re-registrar deixaria a caixa
/// clampando na faixa da forma anterior.
pub(crate) fn seed_shape_fields(store: &mut WidgetStore, kind: ShapeKind, ui: &ShapeValues) {
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
    mode: DrawMode,
    armed: bool,
    f: impl FnOnce(ShapeKind, &mut ShapeValues) -> bool,
) -> bool {
    let Some((id, entity, kind, mut values)) = shape_field_target(sim, map, selection, mode, armed)
    else {
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
