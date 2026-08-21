//! Os gates do **que um gesto faz à pose** — e do número que o painel mostra dela.
//!
//! ⚠️ Módulo-filho do arquivo de gates da cena: `use crate::field3d_scene::*` traz as fixtures (`a_world`, `names`,
//! `the_root`, `a_view`), que continuam a existir **uma vez**. Duas cópias delas divergiriam na
//! primeira mudança, e os dois arquivos passariam a medir cenas diferentes com o mesmo nome.

use super::*;

/// ⭐ **Um giro em torno de um eixo do MUNDO é em torno do eixo do mundo** — mesmo num filho cujo
/// pai está rodado.
///
/// ⚠️ A conta é a conjugação (`inv(R_pai) ⊗ Q ⊗ R_pai`), e sem o sanduíche um giro em torno do X do
/// mundo aplicado a um filho de pai rodado giraria em torno do X **do pai**. O eixo errado, e
/// ninguém diria que o culpado é o gizmo — diria que "a rotação está estranha".
#[test]
fn a_world_axis_spin_stays_on_the_world_axis_under_a_rotated_parent() {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let mut sim = a_world();
    let world = sim.world_mut();
    let doc = FieldDoc::new(
        vec![
            ph2d_field::Node {
                xform: Xform::at(0.3, 0.0, 0.0),
                kind: ph2d_field::NodeKind::Leaf(Primitive::Box {
                    half: [0.3, 0.1, 0.1],
                    round: 0.02,
                }),
                mods: Vec::new(),
            },
            ph2d_field::Node {
                xform: Xform::at(-0.3, 0.0, 0.0),
                kind: ph2d_field::NodeKind::Leaf(Primitive::Sphere { radius: 0.1 }),
                mods: Vec::new(),
            },
            ph2d_field::Node {
                // O pai roda um quarto de volta em torno de Z.
                xform: Xform {
                    translation: [0.0; 3],
                    rotation: [0.0, 0.0, s, s],
                    scale: 1.0,
                },
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![ph2d_field::NodeId(0), ph2d_field::NodeId(1)],
                },
                mods: Vec::new(),
            },
        ],
        ph2d_field::NodeId(2),
    )
    .expect("documento válido");
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    // Um ponto do nó, longe do centro dele, e o que ele faz sob um quarto de volta em torno do X
    // do MUNDO: `(0,1,0) → (0,0,1)`.
    let probe = [0.0f32, 0.5, 0.0];
    let before = ph2d_field_ecs::world_xform(world, child);
    ph2d_field_ecs::rotate_world(world, child, [1.0, 0.0, 0.0], std::f32::consts::FRAC_PI_2);
    let after = ph2d_field_ecs::world_xform(world, child);

    // A direção local `probe`, vista no mundo, tem de ter rodado em torno do X do mundo.
    let d0 = before.apply_dir(probe);
    let d1 = after.apply_dir(probe);
    let want = [d0[0], -d0[2], d0[1]];
    for k in 0..3 {
        assert!(
            (d1[k] - want[k]).abs() < 1e-5,
            "o giro saiu no eixo errado: {d0:?} -> {d1:?}, esperava {want:?}"
        );
    }
    // E o nó não SAIU do lugar: o pivô é o centro dele.
    for k in 0..3 {
        assert!(
            (after.translation[k] - before.translation[k]).abs() < 1e-6,
            "rodar não pode transladar: {:?} -> {:?}",
            before.translation,
            after.translation
        );
    }
}

/// ⭐ **Numa FOLHA, crescer é crescer as DIMENSÕES** — e a pose fica em 1.
///
/// ⚠️ As duas dariam a mesma forma, mas só uma delas é o número que o painel mostra: escalar a pose
/// deixaria uma caixa que mede 2 na tela e diz «1» no painel — duas verdades sobre o mesmo tamanho
/// visível, sem forma de o artista saber qual o gesto seguinte mexe.
///
/// ⛔ E o fator não-positivo é **recusado**: uma escala nula faria o campo deixar de ser uma
/// distância, e a invariante do módulo é *um nó que existe está válido*.
#[test]
fn scaling_a_leaf_grows_its_dimensions_and_leaves_the_pose_alone() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    let radius_of = |w: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity| {
        ph2d_field_ecs::dims_of(w, e)
            .iter()
            .find(|d| d.key == "field.dim.radius")
            .map(|d| d.value)
            .expect("um cilindro tem raio")
    };
    let before = radius_of(world, child);

    ph2d_field_ecs::scale_by(world, child, 1.5);
    ph2d_field_ecs::scale_by(world, child, 2.0);

    assert!(
        (radius_of(world, child) / before - 3.0).abs() < 1e-5,
        "1,5 x 2 = 3 sobre o RAIO, e deu {}",
        radius_of(world, child) / before
    );
    assert!(
        (world.get::<FieldPose>(child).expect("pose").xform.scale - 1.0).abs() < 1e-6,
        "a pose de uma folha não é onde o tamanho mora"
    );

    for bad in [0.0f32, -1.0, f32::NAN, f32::INFINITY] {
        ph2d_field_ecs::scale_by(world, child, bad);
        assert!(
            (radius_of(world, child) / before - 3.0).abs() < 1e-5,
            "o fator {bad} passou"
        );
    }
    assert!(
        ph2d_field_ecs::cook(world, root)
            .expect("não vazia")
            .is_ok()
    );
}

/// ⭐ **Numa OPERAÇÃO é a pose que escala** — ali ela não compete com nada, porque um grupo não tem
/// dimensões próprias.
#[test]
fn scaling_a_group_multiplies_its_pose() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");

    ph2d_field_ecs::scale_by(world, root, 1.5);
    ph2d_field_ecs::scale_by(world, root, 2.0);
    assert!(
        (world.get::<FieldPose>(root).expect("pose").xform.scale - 3.0).abs() < 1e-5,
        "1,5 x 2 = 3 na pose do grupo"
    );
    // E o painel mostra-a, porque ali ela é a única resposta.
    assert!(
        ph2d_field_ecs::params_of(world, root)
            .iter()
            .any(|(_, d)| d.key == "field.dim.scale"),
        "uma operação tem de mostrar a escala — é o único tamanho que ela tem"
    );
}

/// ⭐ **A ficha diz o que o MUNDO levou** — a lei que o `gizmo/readout.rs` da casa já escreveu, aqui
/// virada em assertiva.
///
/// ⚠️ O número que aparece durante um arrasto sai do `Grip::applied`. Isso só é honesto porque o que
/// o mundo recebe é exatamente esse valor — e "exatamente" é o tipo de afirmação que apodrece num
/// comentário. Se um dia a escrita recusar, limitar ou arredondar um pedido, a ficha passa a dizer
/// `0,503` enquanto a peça pousou em `0,500` — e este gate cai antes de alguém ver.
#[test]
fn the_readout_is_the_pose_the_world_took() {
    use crate::field3d_gizmo::Motion;

    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    let before = ph2d_field_ecs::world_xform(world, child);

    // Os três verbos, cada um com um total PRESO — que é o caso em que a ficha e o mundo mais
    // facilmente divergiriam.
    let moved = Motion::Translate([0.25, -0.1, 0.05]).snapped(0.05);
    let Motion::Translate(d) = moved else {
        panic!("translação")
    };
    ph2d_field_ecs::translate_world(world, child, d);
    let after = ph2d_field_ecs::world_xform(world, child);
    for k in 0..3 {
        assert!(
            (after.translation[k] - before.translation[k] - d[k]).abs() < 1e-5,
            "a ficha diz {d:?} e o mundo levou {:?}",
            [
                after.translation[0] - before.translation[0],
                after.translation[1] - before.translation[1],
                after.translation[2] - before.translation[2],
            ]
        );
    }

    let sized = Motion::Scale(1.47).snapped(0.05);
    let Motion::Scale(f) = sized else {
        panic!("escala")
    };
    // ⚠️ Numa FOLHA o tamanho mora nas dimensões, não na pose — então é ali que a ficha se confere.
    let radius = |w: &bevy_ecs::world::World| {
        ph2d_field_ecs::dims_of(w, child)
            .iter()
            .find(|d| d.key == "field.dim.radius")
            .map(|d| d.value)
            .expect("um cilindro tem raio")
    };
    let s0 = radius(world);
    ph2d_field_ecs::scale_by(world, child, f);
    let s1 = radius(world);
    assert!(
        (s1 / s0 - f).abs() < 1e-5,
        "a ficha diz x{f} e o mundo levou x{}",
        s1 / s0
    );

    let turned = Motion::Rotate {
        axis: [0.0, 0.0, 1.0],
        angle: 0.80,
    }
    .snapped(0.05);
    let Motion::Rotate { axis, angle } = turned else {
        panic!("rotação")
    };
    let r0 = ph2d_field_ecs::world_xform(world, child).rotation;
    ph2d_field_ecs::rotate_world(world, child, axis, angle);
    let r1 = ph2d_field_ecs::world_xform(world, child).rotation;
    // O ângulo entre duas orientações: `2·acos(|<q0, q1>|)`.
    let dot: f32 = (0..4).map(|k| r0[k] * r1[k]).sum();
    let swept = 2.0 * dot.abs().clamp(0.0, 1.0).acos();
    assert!(
        (swept - angle.abs()).abs() < 1e-4,
        "a ficha diz {}° e o mundo girou {}°",
        angle.to_degrees(),
        swept.to_degrees()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// A ROTAÇÃO NO PAINEL — e o piso que faltava a toda linha.
// ─────────────────────────────────────────────────────────────────────────────

/// O primeiro filho da peça — o objeto que os gates abaixo editam.
fn a_leaf(
    world: &mut bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
) -> bevy_ecs::entity::Entity {
    world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro")
}

/// ⭐ **Todo nó mostra posição e rotação, e só depois o que a forma mede.**
///
/// ⚠️ A ordem é a do Inspector de objeto de qualquer modelador, e ela não é decorativa: os dois
/// trios existem em **todo** nó, então uma peça inteira lê-se com o olho no mesmo sítio de linha
/// para linha. O gate mede a ordem **e** a ausência — uma folha não tem linha de escala, porque numa
/// forma o tamanho visível são as dimensões dela.
#[test]
fn every_node_shows_position_then_rotation_then_what_it_measures() {
    use ph2d_field::Param;
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let leaf = a_leaf(world, root);

    let params: Vec<Param> = ph2d_field_ecs::params_of(world, leaf)
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    assert_eq!(
        &params[..6],
        &[
            Param::Pos(0),
            Param::Pos(1),
            Param::Pos(2),
            Param::Rot(0),
            Param::Rot(1),
            Param::Rot(2),
        ],
        "a pose vem primeiro, e nesta ordem: {params:?}"
    );
    assert!(
        !params.contains(&Param::Scale),
        "uma FOLHA não tem linha de escala — o tamanho dela são as dimensões: {params:?}"
    );
    assert!(
        params[6..].iter().all(|p| matches!(p, Param::Dim(_))),
        "depois da pose só há dimensões: {params:?}"
    );
    // E a operação **tem** escala, senão o gate não distinguiria «não há escala» de «não há nós».
    let ops: Vec<Param> = ph2d_field_ecs::params_of(world, root)
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    assert!(
        ops.contains(&Param::Scale),
        "a operação perdeu a escala: {ops:?}"
    );
}

/// ⭐ **Digitar um ângulo roda o objeto, e o painel passa a mostrar o número escrito.**
///
/// ⚠️ O gate mede as duas metades porque elas são portas diferentes: a escrita vai por `set_param` e
/// a leitura por `params_of`, e uma conversão de graus que existisse só de um lado deixaria o painel
/// a mostrar radianos de uma peça corretamente rodada — 45 escritos, `0,785` mostrados.
#[test]
fn typing_an_angle_turns_the_node_and_the_panel_shows_it() {
    use ph2d_field::Param;
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let leaf = a_leaf(world, root);

    ph2d_field_ecs::set_param(world, leaf, Param::Rot(1), 45.0).expect("escreve o Y");

    let shown = ph2d_field_ecs::params_of(world, leaf)
        .into_iter()
        .find(|(p, _)| *p == Param::Rot(1))
        .map(|(_, d)| d.value)
        .expect("a linha do Y");
    assert!(
        (shown - 45.0).abs() < 0.01,
        "o painel mostra {shown}, e foi escrito 45"
    );

    // E a peça de facto virou: o X local aponta para outro sítio.
    let turned = world
        .get::<FieldPose>(leaf)
        .expect("pose")
        .xform
        .apply_dir([1.0, 0.0, 0.0]);
    assert!(
        turned[2].abs() > 0.5,
        "45° em torno do Y tinham de trazer o X para fora do plano: {turned:?}"
    );
}

/// ⭐ **Rodar com o gizmo move o número do painel** — uma verdade, duas superfícies.
///
/// ⚠️ É a costura que faz o painel valer alguma coisa durante um arrasto. O gizmo escreve o
/// quaternion por um eixo **arbitrário** (`rotate_world`), e o painel deriva os três ângulos dele:
/// se o painel guardasse um trio próprio, ele ficaria parado enquanto a peça gira — que é
/// exatamente a razão de a rotação **não** ser guardada como três números.
#[test]
fn rotating_with_the_gizmo_moves_the_number_in_the_panel() {
    use ph2d_field::Param;
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let leaf = a_leaf(world, root);

    let before = ph2d_field_ecs::params_of(world, leaf)
        .into_iter()
        .find(|(p, _)| *p == Param::Rot(2))
        .map(|(_, d)| d.value)
        .expect("a linha do Z");
    ph2d_field_ecs::rotate_world(world, leaf, [0.0, 0.0, 1.0], 30.0f32.to_radians());
    let after = ph2d_field_ecs::params_of(world, leaf)
        .into_iter()
        .find(|(p, _)| *p == Param::Rot(2))
        .map(|(_, d)| d.value)
        .expect("a linha do Z");

    assert!(
        (after - before - 30.0).abs() < 0.05,
        "o gizmo rodou 30° e o painel foi de {before} para {after}"
    );
}

/// ⭐ **Uma posição admite negativos; uma dimensão não; um ângulo é meia volta para cada lado.**
///
/// ⚠️ **É o gate da regressão da W13**, e ele mede a ponta de BAIXO — a que ninguém olha. Toda linha
/// nascia com piso zero, e por isso uma posição negativa era indigitável: o espelho do controle
/// reescrevia o número para `0` sem uma mensagem. A fixture é um cilindro de propósito, porque ele
/// tem os três tipos de faixa ao mesmo tempo e um gate por tipo separado não veria a diferença.
#[test]
fn a_position_admits_negatives_a_dimension_does_not_and_an_angle_is_half_a_turn() {
    use ph2d_field::{Bound, Param};
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let leaf = a_leaf(world, root);
    const VIEW: f32 = 2.5;

    let rows = crate::field3d_scene::panel::param_rows(world, Some(leaf), VIEW);
    let find = |want: Param| {
        rows.iter()
            .find(|r| r.param == want)
            .unwrap_or_else(|| panic!("a linha {want:?} não existe"))
    };

    let pos = find(Param::Pos(0));
    assert!(
        (pos.lo + VIEW).abs() < 1e-6 && (pos.bound.value() - VIEW).abs() < 1e-6,
        "uma posição é simétrica em torno da origem: {}..{}",
        pos.lo,
        pos.bound.value()
    );

    // ⚠️ **As três linhas de ângulo NÃO têm a mesma faixa**, e é a correção do bug reportado: num
    // XYZ Euler o ângulo do meio vive em [−90°, 90°] e os de fora em (−180°, 180°]. Dar 180 ao do
    // meio oferecia sítios que a leitura seguinte renomeava — e num arrasto isso é um ciclo de dois.
    for (axis, half) in [(0u8, 180.0f32), (1, 90.0), (2, 180.0)] {
        let rot = find(Param::Rot(axis));
        assert_eq!(
            (rot.lo, rot.bound),
            (-half, Bound::Wrap(half)),
            "o eixo {axis} tem de ir a ±{half}: a ponta é da REPRESENTAÇÃO, não da vista"
        );
    }

    // O raio do cilindro: piso em zero, teto da vista. E o filete: piso em zero, teto do DOCUMENTO.
    let radius = find(Param::Dim(0));
    assert_eq!(
        (radius.lo, radius.bound),
        (0.0, Bound::Soft(VIEW)),
        "uma dimensão positiva não desce abaixo de zero"
    );
    let fillet = rows
        .iter()
        .find(|r| r.key == "field.dim.round")
        .expect("o cilindro tem filete");
    assert!(
        fillet.lo == 0.0 && matches!(fillet.bound, Bound::Hard(_)),
        "o filete é a única ponta que o DOCUMENTO impõe: {}..{:?}",
        fillet.lo,
        fillet.bound
    );
}

/// ⭐ **Escrever o MESMO ângulo duas vezes não mexe a peça.**
///
/// ⚠️ **É o bug que o Enio reportou** (20/08: *"bug em rot y. Acima de 70 muda x e z e treme"*), e o
/// nome dele é este: um arrasto escreve o mesmo alvo **quadro após quadro**, então uma escrita que
/// não é ponto fixo vira um ciclo de dois — a peça alterna entre duas orientações enquanto o dedo
/// está parado. Medido antes da cura: `Y = 93,6` dava `(180, 86,4, 180)` na primeira escrita e
/// `(0, 86,4, 0)` na segunda, que **não é a mesma orientação**.
///
/// ⚠️ O gate varre os TRÊS eixos, nos TRÊS cilindros da cena 1 — um deles nasce **na trava de
/// cardan** (90° em torno do Y), que é o caso em que o trio deixa de ter três partes independentes e
/// onde uma cura ingénua reabre o mesmo ciclo por outro caminho.
#[test]
fn writing_the_same_angle_twice_does_not_move_the_part() {
    use ph2d_field::Param;
    for k in 0..3usize {
        for axis in 0..3u8 {
            let mut sim = a_world();
            let world = sim.world_mut();
            let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
            let e = world
                .get::<Children>(root)
                .expect("filhos")
                .iter()
                .copied()
                .nth(k)
                .expect("o cilindro");
            // ⚠️ A varredura passa **por cima** do quarto de volta de propósito: é ali que a versão
            // anterior partia, e um gate que parasse em 90 ficaria verde sobre o defeito.
            for step in 0..40 {
                let target = step as f32 * 9.0 - 180.0;
                ph2d_field_ecs::set_param(world, e, Param::Rot(axis), target).expect("escreve");
                let once = world.get::<FieldPose>(e).expect("pose").xform.rotation;
                ph2d_field_ecs::set_param(world, e, Param::Rot(axis), target).expect("de novo");
                let twice = world.get::<FieldPose>(e).expect("pose").xform.rotation;
                let same = (0..4)
                    .map(|i| (once[i] - twice[i]).abs())
                    .fold(0.0f32, f32::max);
                let flipped = (0..4)
                    .map(|i| (once[i] + twice[i]).abs())
                    .fold(0.0f32, f32::max);
                assert!(
                    same.min(flipped) < 1.0e-4,
                    "cilindro {k}, eixo {axis}, alvo {target}: a segunda escrita do MESMO valor \
                     mudou a peça ({once:?} -> {twice:?}) — num arrasto isto é um ciclo de dois"
                );
            }
        }
    }
}

/// ⭐ **Na trava de cardan a linha do Z chega ao painel como FACTO, não como controle.**
///
/// ⚠️ O gate mede as duas travessias — entra na trava e sai dela —, porque uma linha que ficasse
/// inerte para sempre depois de a peça passar por 90° seria um controle perdido em silêncio, e
/// nenhum gate de um estado só o apanharia.
#[test]
fn at_the_pole_the_third_angle_reaches_the_panel_as_a_fact() {
    use ph2d_field::Param;
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let leaf = a_leaf(world, root);
    const VIEW: f32 = 2.5;

    let live_of = |world: &bevy_ecs::world::World, axis: u8| -> bool {
        crate::field3d_scene::panel::param_rows(world, Some(leaf), VIEW)
            .into_iter()
            .find(|r| r.param == Param::Rot(axis))
            .map(|r| r.live)
            .expect("a linha existe")
    };

    // Fora da trava as três respondem.
    ph2d_field_ecs::set_param(world, leaf, Param::Rot(1), 45.0).expect("Y");
    for axis in 0..3u8 {
        assert!(
            live_of(world, axis),
            "fora da trava o eixo {axis} tem de responder"
        );
    }

    // Dentro dela, o Z é um facto — e só ele.
    ph2d_field_ecs::set_param(world, leaf, Param::Rot(1), 90.0).expect("Y no polo");
    assert!(!live_of(world, 2), "no polo o Z não é um controle");
    assert!(
        live_of(world, 0) && live_of(world, 1),
        "o X e o Y têm de continuar vivos — é pelo Y que se sai"
    );

    // E ao sair, ele volta: uma linha perdida em silêncio seria pior do que uma travada.
    ph2d_field_ecs::set_param(world, leaf, Param::Rot(1), 60.0).expect("sai do polo");
    assert!(live_of(world, 2), "sair da trava tem de devolver o Z");
}
