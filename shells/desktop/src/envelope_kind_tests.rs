//! Os gates da **Fatia D do ADR-0129**: a gaiola tem DOIS gestos — `Perspective` (homografia,
//! lados retos) e `Mesh` (patch de Coons, lados que dobram).
//!
//! Eles NÃO são o mesmo mapa com um knob: com os lados retos o Coons é *bilinear* e a homografia é
//! *projetiva*, e as duas só coincidem com a gaiola em REPOUSO. O que estes gates fixam é
//! exatamente essa fronteira — trocar em repouso é seguro, e fora dele o gesto escolhido manda de
//! verdade (senão o chip do painel seria decorativo). A matemática dos dois mapas está gateada na
//! crate `ph2d-vec-envelope`; aqui é o comportamento do HOST.

use super::*;
/// Troca o gesto do container (o que os chips do painel fazem).
fn set_kind(sim: &mut SimWorld, bits: u64, kind: EnvelopeKind) {
    crate::envelope_gesture::set_kind(sim, bits, kind);
}

/// Dobra o lado de BAIXO da gaiola para fora — o gesto que só o Mesh oferece.
fn bend_bottom(sim: &mut SimWorld, bits: u64) {
    let mut env = sim
        .world_mut()
        .get_mut::<VecEnvelope>(Entity::from_bits(bits))
        .expect("VecEnvelope");
    let span = env.corners[1][0] - env.corners[0][0];
    let drop = env.corners[3][1] - env.corners[0][1];
    env.edges[0][0] = [
        env.corners[0][0] + span / 3.0,
        env.corners[0][1] - drop * 0.4,
    ];
    env.edges[0][1] = [
        env.corners[0][0] + span * 2.0 / 3.0,
        env.corners[0][1] - drop * 0.4,
    ];
}

/// A soma das âncoras — uma assinatura barata da geometria, sensível a qualquer deslocamento.
fn signature(p: &VecPath) -> [f64; 2] {
    p.verts_all()
        .fold([0.0, 0.0], |a, v| [a[0] + v.anchor[0], a[1] + v.anchor[1]])
}

/// **TROCAR DE GESTO NUMA GAIOLA EM REPOUSO NÃO MOVE A ARTE.** Em repouso a homografia e o patch de
/// Coons são ambos a identidade, então o chip é seguro de tocar antes de deformar.
///
/// A metade **presença** é a asserção de que a forma continua CHEIA: um recook que apagasse tudo
/// também deixaria as duas "iguais".
#[test]
fn switching_the_cage_gesture_at_rest_does_not_move_the_art() {
    let shape = ellipse([5.0, 5.0], 3.0);
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![shape]);
    let before = frame(&mut sim, &mut scene, ids[0]);
    assert!(
        before.verts.len() >= 4,
        "a forma nasceu vazia: fixture morto"
    );

    set_kind(&mut sim, container, EnvelopeKind::Mesh);
    let after = frame(&mut sim, &mut scene, ids[0]);

    let (a, b) = (signature(&before), signature(&after));
    assert!(
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
        "trocar o gesto em repouso moveu a arte: {a:?} -> {b:?}"
    );
}

/// **O LADO DOBRADO SÓ DEFORMA NO MESH** — e no Mesh ele deforma DE VERDADE.
///
/// Este é o par que prova que o chip não é decorativo. Se o `recook` ignorasse `env.kind` e usasse
/// sempre a homografia, a 2ª metade falharia; se ele usasse sempre o Coons, a 1ª falharia. Uma
/// mutação em qualquer um dos dois ramos sangra aqui.
#[test]
fn a_bent_side_only_deforms_in_the_mesh_gesture() {
    let shape = ellipse([5.0, 5.0], 3.0);
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![shape]);
    let rest = signature(&frame(&mut sim, &mut scene, ids[0]));

    // Perspective: os lados existem no componente mas o mapa NÃO os lê.
    bend_bottom(&mut sim, container);
    let persp = signature(&frame(&mut sim, &mut scene, ids[0]));
    assert!(
        (rest[0] - persp[0]).abs() < 1e-9 && (rest[1] - persp[1]).abs() < 1e-9,
        "o gesto Perspective leu os lados curvos: {rest:?} -> {persp:?}"
    );

    // Mesh: o MESMO lado dobrado agora manda.
    set_kind(&mut sim, container, EnvelopeKind::Mesh);
    let mesh = signature(&frame(&mut sim, &mut scene, ids[0]));
    let moved = (rest[0] - mesh[0]).hypot(rest[1] - mesh[1]);
    assert!(
        moved > 1.0,
        "o gesto Mesh nao deformou pelo lado dobrado (deslocamento {moved:.3e})"
    );
}

/// **SAIR DO MESH ENDIREITA OS LADOS GUARDADOS.** Em Perspective os lados *são* retos — deixar os
/// controles dobrados no componente faria a volta ao Mesh ressuscitar uma gaiola que o mapa nunca
/// aplicou, e as alças apareceriam fora dos lados desenhados.
#[test]
fn leaving_the_mesh_gesture_straightens_the_stored_sides() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    set_kind(&mut sim, container, EnvelopeKind::Mesh);
    bend_bottom(&mut sim, container);
    assert_ne!(
        env_of(&sim, container).edges,
        ph2d_vec_envelope::rest_edges(&env_of(&sim, container).corners),
        "fixture morto: o lado nao ficou dobrado"
    );

    set_kind(&mut sim, container, EnvelopeKind::Perspective);
    let env = env_of(&sim, container);
    assert_eq!(
        env.edges,
        ph2d_vec_envelope::rest_edges(&env.corners),
        "voltar ao Perspective deixou lados curvos guardados"
    );
}

/// **A ALÇA DE LADO SÓ É AGARRÁVEL NO MESH.** O mesmo cursor, sobre o mesmo ponto: em Perspective o
/// pen fica com o clique (a alça não existe), no Mesh o envelope o toma.
///
/// Sem o 2º ramo o gate ficaria verde num `press` que nunca pega nada.
#[test]
fn a_side_handle_is_only_grabbable_in_the_mesh_gesture() {
    let (mut sim, _scene, _map, container, _ids) = envelope_over(vec![ellipse([5.0, 5.0], 3.0)]);
    let on_handle = env_of(&sim, container).edges[0][0];
    let mut drag = None;
    // Escala de zoom que torna o raio da bolinha PEQUENO em unidades de mundo (0,3): o controle
    // fica exatamente sob o cursor e o canto vizinho, a ~2 unidades, fora de alcance. Com o raio
    // do fixture ingênuo (1 px/unidade = 6 unidades) o canto engoliria o controle e o gate mediria
    // outra coisa — [[feedback_test_with_product_numbers_not_convenient_ones]].
    const PX_TO_WORLD: f64 = 0.05;

    assert!(
        !crate::envelope_gesture::press(&sim, Some(container), on_handle, PX_TO_WORLD, &mut drag),
        "o Perspective ofereceu uma alca de lado que o mapa dele ignora"
    );
    set_kind(&mut sim, container, EnvelopeKind::Mesh);
    assert!(
        crate::envelope_gesture::press(&sim, Some(container), on_handle, PX_TO_WORLD, &mut drag),
        "o Mesh nao pegou a alca de lado sob o cursor"
    );
    assert_eq!(
        drag.map(|(_, h)| h),
        Some(ph2d_vec_envelope::edge_handle_index(0, 0)),
        "pegou a alca errada"
    );
}
