//! ⭐ **A PONTE de estados de UI e o cozimento da booleana, no MESMO quadro** — irmão de [`super`]
//! pelo teto de 600 LOC, e o corte é por ASSUNTO: ali os gates que medem a **lei** do morfo de
//! verbo; aqui o único que atravessa a **costura inteira** (a ponte publica, o cozimento honra).
//!
//! ⚠️ Ele saiu de lá em 2026-08-26, quando a assinatura do `dispatch` mudou (os dois recados
//! passaram a viajar num `Cooked`) e o ficheiro, que estava **exactamente** no tecto, passou a
//! 601. Pago por EXTRACÇÃO, nunca por allowlist.

use super::*;

/// ⭐⭐⭐ **A COMPOSIÇÃO QUE O QUADRO CORRE** — a ponte dos estados publica, a booleana consome, e o
/// desenho fica entre as duas operações.
///
/// ⚠️ **Ele mora aqui de propósito, e nenhuma das duas metades sozinha o mostraria.** Os gates de
/// cima entregam recados escritos à mão a um cozimento; os da `ph2d-ui-state` provam que a
/// transição os publica. O que só existe na costura é *o recado chegar* — e esta casa já pagou
/// essa lição com vinte testes verdes sobre um `draw` cravado em `true`.
///
/// A fixture é a da cena `=74`: o grupo booleano pendurado num CHIP, que é o hospedeiro dos
/// estados — a única disposição em que o artista consegue selecionar um hospedeiro ÚNICO com uma
/// booleana dentro dele.
#[test]
fn the_frame_composes_the_bridge_and_the_cook() {
    use ph2d_anim::{Easing, EasingFamily, EasingMode};
    use ph2d_ui_state::{StateRole, StateSets};

    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let chip = scene.push_path(rectangle([-2.0, -2.0], [22.0, 22.0]));
    let outer = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let inner = scene.push_path(rectangle([6.0, 6.0], [14.0, 14.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&outer], map[&inner]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 0 });
    crate::vec_transform::reparent_keeping_world(&mut sim, g, Entity::from_bits(map[&chip]));

    // As duas poses, pela porta do PRODUTO — nunca escrevendo a tabela à mão.
    let mut states = StateSets::default();
    let rec = |sim: &mut SimWorld, scene: &mut VecScene, states: &mut StateSets, role| {
        crate::vec_ui_state_edit::apply(
            sim,
            scene,
            &map,
            &[chip],
            states,
            crate::vec_ui_state_edit::UiStateEdit::Record(role),
        );
    };
    rec(&mut sim, &mut scene, &mut states, StateRole::Default);
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 1 }); // Subtract no Hover
    rec(&mut sim, &mut scene, &mut states, StateRole::Hover);
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 0 }); // e a cena volta ao repouso

    // ⚠️ Curva LINEAR: com a de fábrica o `t` de meio caminho é deformado, e a barra passaria a
    // medir a curva em vez do desenho.
    states.set_easing(chip, Easing::new(EasingFamily::Linear, EasingMode::InOut));
    let (duration, _) = states.timing(chip);

    let mut machines = crate::render_loop::ui_state_bridge::UiMachines::new();
    crate::render_loop::ui_state_bridge::request(&mut machines, &states, chip, StateRole::Hover);
    let mut cooked = crate::render_loop::ui_state_bridge::Cooked::default();
    let animating = crate::render_loop::ui_state_bridge::dispatch(
        &mut machines,
        &mut states,
        &mut sim,
        &mut scene,
        &map,
        duration * 0.5,
        &mut cooked,
    );
    assert!(animating, "a ponte não pôs a máquina no ar");
    assert!(
        !cooked.bool_morphs.is_empty(),
        "a ponte não publicou recado nenhum: o buraco vai APARECER de uma vez no fim"
    );

    // ⚠️ A leitura é pelo id do OUTER, e não pelo primeiro caminho da cena: o portador do
    // resultado é a BASE do grupo, e aqui o primeiro caminho é o CHIP — que não é operando de
    // nada. Ler o índice 0 media uma entrada VAZIA e dizia *"o buraco é a peça inteira"*.
    let mut live = LiveGeometry::new();
    let m = &cooked.bool_morphs;
    BoolLive::default().recook(&scene, &sim, &map, &VecXforms::new(), m, &mut live);
    let drawn = live.get(&outer).cloned().unwrap_or_default();
    let hole = hole_of(&drawn);
    assert!(
        (hole - HOLE * 0.25).abs() < TOL,
        "a meio caminho o desenho mediu {hole:.2} de buraco, esperado {:.2} \
         (0 = o recado não chegou ao cozimento, {HOLE} = ele saltou)",
        HOLE * 0.25
    );
}
