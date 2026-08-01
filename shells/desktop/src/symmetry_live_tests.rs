//! Gates da **SIMETRIA DE DESENHO** — arquivo irmão de `symmetry_live.rs`.
//!
//! As exigências do USUÁRIO que esta família existe para pinar (Enio, 2026-08-01):
//!
//! > *"A linha de simetria deve aparecer logo que se aperta o botão e não quando se inicia o
//! > desenho."*
//!
//! > *"A simetria funciona apenas para formas que serão desenhadas com a tool ligada e não deve
//! > fazer simetria de formas que já existem previamente."*
//!
//! > *"Com o botão checado pode-se fazer quantos desenhos desejar que a linha de simetria permanece
//! > no lugar."*
//!
//! > *"Uma vez que o desenho é feito, a referência para a linha passa a ser o próprio desenho … se
//! > o usuário mover o objeto no canvas a linha de simetria acompanha."*
//!
//! > *"Se a simetria for desmarcada antes do apply, as cópias somem mas não são destruídas."*
//!
//! ⚠️ **O oráculo NUNCA é "há duas formas desenhadas"** — isso é verdade em qualquer implementação
//! que reflicta alguma coisa em algum lugar. Os oráculos daqui são: (a) o desenho inteiro TRANSLADA
//! com a pose, ponto a ponto; (b) uma forma que ninguém desenhou continua **sem componente**; (c) o
//! eixo capturado, levado de volta ao MUNDO, coincide com o de sessão; (d) a curva autorada fica
//! byte-idêntica através do ciclo ligar→desligar; (e) uma forma já simétrica no próprio eixo
//! coincide com a própria cópia sob QUALQUER pose — a propriedade que só a ordem *reflectir → assar*
//! tem.

use super::*;
use crate::vec_entities::VecEntityMap;
use ph2d_vec_scene::VecVertex;
use ph2d_vec_scene::symmetry::{SymmetryKind, SymmetryStyle};

/// Um **meio-perfil aberto** com as pontas no eixo `x = 0`, posado. A fixture tem de conter o
/// fenômeno: é a forma para a qual a simetria existe (metade desenhada, metade derivada), e é a
/// única em que a fusão tem o que fazer.
fn half_profile_scene() -> (
    VecScene,
    ph2d_ecs::SimWorld,
    VecEntityMap,
    VecXforms,
    VecPathId,
    ph2d_ecs::Entity,
) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(half_profile());
    let e = spawn_for(&mut sim, &mut map, id, "Vase", [3.0, 1.0]);
    let xf = crate::vec_transform::build(&sim, &map);
    (scene, sim, map, xf, id, e)
}

fn half_profile() -> VecPath {
    VecPath {
        verts: [[0.0, -1.0], [0.8, -0.3], [0.5, 0.4], [0.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: false,
        ..VecPath::default()
    }
}

fn spawn_for(
    sim: &mut ph2d_ecs::SimWorld,
    map: &mut VecEntityMap,
    id: VecPathId,
    name: &str,
    at: [f32; 2],
) -> ph2d_ecs::Entity {
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(at[0], at[1]),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new(name),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    e
}

/// O estilo ARMADO com o default de espelho vertical — o que o painel produz ao ligar o botão.
fn armed() -> SymmetryStyle {
    SymmetryStyle {
        on: true,
        ..SymmetryStyle::default()
    }
}

/// As âncoras da geometria DESENHADA de `id` neste frame, na ordem de emissão.
fn drawn(live: &SymmetryLive, id: VecPathId) -> Vec<[f64; 2]> {
    live.live()
        .get(&id)
        .map(|paths| {
            paths
                .iter()
                .flat_map(|p| p.verts_all().map(|v| v.anchor))
                .collect()
        })
        .unwrap_or_default()
}

/// Move a entidade por `delta` e devolve as poses re-construídas.
fn nudge(
    sim: &mut ph2d_ecs::SimWorld,
    map: &VecEntityMap,
    e: ph2d_ecs::Entity,
    delta: [f32; 2],
) -> VecXforms {
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
        t.translation.x += delta[0];
        t.translation.y += delta[1];
    }
    crate::vec_transform::build(sim, map)
}

/// O eixo capturado por `id`, levado de volta ao MUNDO — o oráculo de *"capturou o eixo de sessão"*.
fn captured_axis_in_world(
    sim: &ph2d_ecs::SimWorld,
    map: &VecEntityMap,
    xforms: &VecXforms,
    id: VecPathId,
) -> [f64; 2] {
    let spec = spec_of(sim, map, id).expect("a forma tem simetria");
    ph2d_vec_scene::xform_of(xforms, id).apply(spec.center)
}

// ── R1: a linha existe antes de qualquer desenho ───────────────────────────────────────

/// **A linha aparece com a cena VAZIA.** *"Deve aparecer logo que se aperta o botão e não quando se
/// inicia o desenho."*
///
/// ⚠️ O oráculo é o eixo de SESSÃO ser desenhável sem nenhuma forma no mundo — é isso que separa
/// este modelo do anterior, em que a linha era propriedade de uma forma seleccionada e portanto não
/// podia existir antes de haver uma.
#[test]
fn the_line_exists_before_anything_is_drawn() {
    let ax = session_axis(armed(), [4.0, -2.0]);
    assert_eq!(ax.at, [4.0, -2.0], "a linha nasce onde foi semeada");
    assert!(
        ax.dir[0].abs() < 1e-9 && (ax.dir[1] - 1.0).abs() < 1e-9,
        "o espelho X é uma linha VERTICAL: {:?}",
        ax.dir
    );
    assert!(ax.segments.is_none(), "um espelho não é uma rosácea");

    let radial = session_axis(
        SymmetryStyle {
            kind: SymmetryKind::Radial,
            segments: 7,
            ..armed()
        },
        [0.0, 0.0],
    );
    assert_eq!(
        radial.segments,
        Some(7),
        "a rosácea de sessão desenha os raios que o estilo pede"
    );
}

// ── R2: só o que for DESENHADO com o modo ligado espelha ───────────────────────────────

/// **Uma forma que já existia NUNCA é adoptada.** *"Não deve fazer simetria de formas que já existem
/// previamente."*
///
/// ⚠️ Este é o gate que o modelo anterior não podia passar: lá armar operava sobre a SELECÇÃO, e
/// uma forma pré-existente seleccionada ganhava simetria no frame seguinte.
#[test]
fn a_shape_that_already_existed_is_never_adopted() {
    let (scene, mut sim, map, xf, id, _e) = half_profile_scene();
    let mut live = SymmetryLive::default();

    // Ninguém está em gesto: a lista de desenho está vazia.
    let armed_count = live.adopt(&mut sim, &map, &scene, &xf, armed(), [0.0, 0.0], &[]);

    assert_eq!(armed_count, 0, "nenhuma forma armada");
    assert!(
        spec_of(&sim, &map, id).is_none(),
        "a forma pré-existente continua SEM componente — o modo não a alcança"
    );
    live.recook(&scene, &sim, &map, &xf, true);
    assert!(
        drawn(&live, id).is_empty(),
        "e portanto não há cópia nenhuma desenhada sobre ela"
    );
}

/// **O que está em GESTO é adoptado, e captura o eixo de sessão.**
#[test]
fn what_is_being_drawn_captures_the_session_axis() {
    let (scene, mut sim, map, xf, id, _e) = half_profile_scene();
    let mut live = SymmetryLive::default();
    let origin = [5.0, 0.5];

    let n = live.adopt(&mut sim, &map, &scene, &xf, armed(), origin, &[id]);

    assert_eq!(n, 1, "a forma em gesto está armada");
    let world = captured_axis_in_world(&sim, &map, &xf, id);
    assert!(
        (world[0] - origin[0]).abs() < 1e-9 && (world[1] - origin[1]).abs() < 1e-9,
        "o eixo capturado, de volta ao mundo, É o de sessão: {world:?} vs {origin:?}"
    );
}

// ── R3: o eixo fica no lugar entre desenhos ────────────────────────────────────────────

/// **Vários desenhos, UMA linha.** *"Com o botão checado pode-se fazer quantos desenhos desejar que
/// a linha de simetria permanece no lugar."*
///
/// ⚠️ A fixture põe as duas formas em poses DIFERENTES de propósito: se o eixo fosse guardado em
/// mundo (ou re-semeado por forma) as duas ainda coincidiriam num teste de pose única, e o gate não
/// poderia falhar.
#[test]
fn the_axis_stays_put_across_several_drawings() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let a = scene.push_path(half_profile());
    spawn_for(&mut sim, &mut map, a, "A", [0.0, 0.0]);
    let b = scene.push_path(half_profile());
    spawn_for(&mut sim, &mut map, b, "B", [7.0, -3.0]);
    let xf = crate::vec_transform::build(&sim, &map);

    let mut live = SymmetryLive::default();
    let origin = [2.0, 1.0];
    // Primeiro desenho.
    live.adopt(&mut sim, &map, &scene, &xf, armed(), origin, &[a]);
    // Segundo desenho, mais tarde, com o MESMO eixo de sessão.
    live.adopt(&mut sim, &map, &scene, &xf, armed(), origin, &[b]);

    for (id, name) in [(a, "A"), (b, "B")] {
        let world = captured_axis_in_world(&sim, &map, &xf, id);
        assert!(
            (world[0] - origin[0]).abs() < 1e-9 && (world[1] - origin[1]).abs() < 1e-9,
            "{name} espelha na linha de sessão: {world:?} vs {origin:?}"
        );
    }
}

/// **A captura só SELA depois de o pivô assentar.**
///
/// ⚠️ O `settle_origins` translada a geometria e compensa no `Transform` no frame em que o gesto
/// acaba, e o faz DEPOIS de a forma sair da lista de gesto. Um eixo capturado só durante o gesto
/// ficaria deslocado exactamente por essa translação — e em SILÊNCIO, porque nada falha. A fixture
/// reproduz a sequência real: um frame em gesto, o frame seguinte com a pose já mexida e a forma
/// fora da lista.
#[test]
fn the_capture_seals_after_the_pivot_settles() {
    let (scene, mut sim, map, xf, id, e) = half_profile_scene();
    let mut live = SymmetryLive::default();
    let origin = [1.5, -0.5];

    // Frame N: em gesto.
    live.adopt(&mut sim, &map, &scene, &xf, armed(), origin, &[id]);
    // Frame N+1: o pivô assentou (a pose mudou) e o gesto acabou.
    let xf2 = nudge(&mut sim, &map, e, [2.0, -4.0]);
    live.adopt(&mut sim, &map, &scene, &xf2, armed(), origin, &[]);

    let world = captured_axis_in_world(&sim, &map, &xf2, id);
    assert!(
        (world[0] - origin[0]).abs() < 1e-9 && (world[1] - origin[1]).abs() < 1e-9,
        "depois do assentamento o eixo continua na linha de sessão: {world:?} vs {origin:?}"
    );

    // Frame N+2: nada mais re-deriva — daqui em diante o eixo é DA FORMA.
    let xf3 = nudge(&mut sim, &map, e, [6.0, 6.0]);
    live.adopt(&mut sim, &map, &scene, &xf3, armed(), origin, &[]);
    let moved = captured_axis_in_world(&sim, &map, &xf3, id);
    assert!(
        (moved[0] - (origin[0] + 6.0)).abs() < 1e-9 && (moved[1] - (origin[1] + 6.0)).abs() < 1e-9,
        "e a partir daí ele VIAJA com a forma: {moved:?}"
    );
}

/// **O estilo segue a ferramenta; o LUGAR fica com a forma.** Arrastar *Segments* actualiza a
/// sessão inteira sem teleportar eixo nenhum.
///
/// ⚠️ A fixture MOVE a forma antes de trocar o estilo, e é essa linha que dá dentes ao gate: com o
/// eixo de sessão parado (que é o que o produto faz enquanto o modo está ligado), uma
/// re-derivação indevida só é observável se a POSE tiver mudado — senão ela devolveria o mesmo
/// número e o gate não poderia falhar.
#[test]
fn the_style_follows_the_tool_but_the_place_stays_with_the_shape() {
    let (scene, mut sim, map, xf, id, e) = half_profile_scene();
    let mut live = SymmetryLive::default();
    let origin = [4.0, 4.0];
    live.adopt(&mut sim, &map, &scene, &xf, armed(), origin, &[id]);
    // O frame seguinte sela a captura (o pivô assentou; ver o gate irmão).
    live.adopt(&mut sim, &map, &scene, &xf, armed(), origin, &[]);
    let place = spec_of(&sim, &map, id).expect("armada").center;

    // O artista move a forma e SÓ ENTÃO troca para Radial 9, sem desenhar nada.
    let xf2 = nudge(&mut sim, &map, e, [8.0, -5.0]);
    live.adopt(
        &mut sim,
        &map,
        &scene,
        &xf2,
        SymmetryStyle {
            kind: SymmetryKind::Radial,
            segments: 9,
            ..armed()
        },
        origin,
        &[],
    );

    let now = spec_of(&sim, &map, id).expect("continua armada");
    assert_eq!(now.kind, SymmetryKind::Radial, "o estilo chegou");
    assert_eq!(now.segments, 9);
    assert_eq!(
        now.center, place,
        "e o LUGAR não se mexeu — o eixo é da forma desde que ela foi desenhada"
    );
}

#[path = "symmetry_cook_tests.rs"]
mod cook;
