//! Gate da política das alças de raio: **geometria DERIVADA não tem quina arrastável.**
//!
//! É o gate que me faltava e que quase deixou passar um bug de "funciona e depois
//! esquece": eu havia gateado o MODO (a alça é do Node) e achado que estava coberto. Mas
//! uma forma viva **selecionada dentro do modo Node** é outra coisa — e ali a alça
//! aparecia, funcionava, e o próximo arrasto de slider do painel varria o raio, porque o
//! `recook_into` reescreve `verts` inteiro. Funcionar e depois desfazer sozinho é pior que
//! não funcionar.
//!
//! # E a política enumerava UM dos cinco (ADR-0132 §5)
//!
//! O guard perguntava `is_live_shape`, e a linha ganhou mais quatro objetos vivos desde
//! então. Hoje **quatro** hosts reescrevem `verts` (`vec_shape_live`, `connector_live`,
//! `morph_live`, `envelope_live`) e só o primeiro era recusado — a alça aparecia sobre um
//! conector, sobre um morph e sobre um filho de envelope, e morria no frame seguinte, em
//! silêncio, exatamente como a doc do módulo descreve para a forma viva.
//! [[feedback_a_condition_that_enumerates_its_readers_rots]]
//!
//! Um gate por caso, de propósito: uma defesa em camadas precisa que a mutação de UMA
//! camada sangre sozinha. [[feedback_layered_defenses_need_per_layer_gates]]

use super::*;
use ph2d_ecs::{
    ChildOf, Entity, EnvelopeKind, Transform, VecConnector, VecEnvelope, VecEnvelopeChild,
    VecMorph, VecShape,
};
use ph2d_vec_scene::{VecPath, VecPathId, VecVertex};

/// Uma cena com um quadrado, a entidade dele, e o mapa path↔entidade. O `decorate` pendura
/// nessa entidade (ou numa nova, no caso do envelope) o que a torna geometria derivada.
fn square_with(
    decorate: impl FnOnce(&mut SimWorld, Entity),
) -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(Transform::IDENTITY).id();
    decorate(&mut sim, e);
    let mut map = VecEntityMap::default();
    map.insert(id, e.to_bits());
    (sim, scene, map, id)
}

/// Quantas alças de raio o produto oferece para `id` — a chamada REAL do frame.
fn handles(sim: &SimWorld, scene: &VecScene, map: &VecEntityMap, id: VecPathId) -> usize {
    view(
        sim,
        scene,
        map,
        Some(id),
        &ph2d_vec_scene::VecXforms::default(),
        0.01,
    )
    .len()
}

/// Um caminho DESENHADO (sem receita, sem host vivo) tem alça em cada quina — quatro, no
/// quadrado. **É o controle positivo de todos os gates abaixo**: sem ele, "não oferece
/// alça" ficaria verde com a função a devolver vazio para tudo.
/// [[feedback_absence_gate_needs_a_presence_sibling]]
#[test]
fn a_drawn_path_gets_a_radius_handle_on_every_corner() {
    let (sim, scene, map, id) = square_with(|_, _| {});
    assert!(!has_derived_verts(&sim, &map, id));
    assert_eq!(
        handles(&sim, &scene, &map, id),
        4,
        "as 4 quinas do quadrado"
    );
}

/// **Uma FORMA VIVA não tem nenhuma.**
///
/// Não é conservadorismo: o `vec_shape_live::recook_into` substitui `path.verts` INTEIRO a
/// cada mudança de parâmetro, e o `corner_radius` mora dentro do vértice. Um raio autorado
/// aqui sobreviveria até o usuário encostar num slider — e sumiria sem erro nenhum. O raio
/// de uma forma viva é um campo DELA (o painel); o por-vértice é para caminho desenhado.
#[test]
fn a_live_shape_has_no_radius_handles_because_the_recook_would_erase_them() {
    let (sim, scene, map, id) = square_with(|sim, e| {
        // A RECEITA: é isto que faz o `recook_into` reescrever a geometria toda vez.
        sim.world_mut().entity_mut(e).insert(VecShape::Param {
            kind: 0,
            w: 10.0,
            h: 10.0,
            values: [0.0; ph2d_ecs::MAX_SHAPE_VALUES],
        });
    });
    assert!(
        has_derived_verts(&sim, &map, id),
        "a receita está pendurada"
    );
    assert_eq!(handles(&sim, &scene, &map, id), 0);
}

/// **Um CONECTOR não tem nenhuma.** O `connector_live::recook` limpa e reescreve `p.verts`
/// a cada frame, a partir das pontas — o raio não sobreviveria a um único frame. (E o
/// conector já tem o raio dele: `VecConnector.corner_radius`, que é do cotovelo da rota.)
#[test]
fn a_connector_has_no_radius_handles() {
    let (sim, scene, map, id) = square_with(|sim, e| {
        sim.world_mut()
            .entity_mut(e)
            .insert(VecConnector::between(1, 2));
    });
    assert!(has_derived_verts(&sim, &map, id));
    assert_eq!(handles(&sim, &scene, &map, id), 0);
}

/// **Um MORPH não tem nenhuma.** `morph_live::recook` escreve `p.verts = cooked.verts` a
/// cada frame — e no morph isso é a feature (o `t` é animável), então o raio morreria já no
/// frame seguinte ao arrasto.
#[test]
fn a_morph_has_no_radius_handles() {
    let (sim, scene, map, id) = square_with(|sim, e| {
        sim.world_mut().entity_mut(e).insert(VecMorph::new(1, 2));
    });
    assert!(has_derived_verts(&sim, &map, id));
    assert_eq!(handles(&sim, &scene, &map, id), 0);
}

/// **Um FILHO DE ENVELOPE não tem nenhuma — e este é o caso que o ADR-0132 §5 reporta.**
///
/// O componente mora no CONTAINER, não no filho: `get::<VecEnvelope>(filho)` é `None`, e é
/// por isso que um guard que só olhasse a própria entidade deixaria a alça passar. A
/// pergunta tem de subir a cadeia de ancestrais — a mesma caminhada que o
/// `envelope_live::container_of` já é dono.
///
/// O `envelope_live::recook` reescreve `verts` de cada filho a cada frame a partir do
/// `source` congelado DENTRO do componente. Um raio autorado no filho sobrevive
/// exatamente **um** frame.
#[test]
fn an_envelope_child_has_no_radius_handles() {
    let (sim, scene, map, id) = square_with(|sim, e| {
        let container = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                VecEnvelope {
                    corners: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]],
                    edges: [[[0.0, 0.0]; 2]; 4],
                    kind: EnvelopeKind::Perspective,
                    warp: None,
                    pins: Vec::new(),
                    bend: 0.0,
                    children: vec![VecEnvelopeChild {
                        path: 0,
                        source: Vec::new(),
                    }],
                },
            ))
            .id();
        sim.world_mut().entity_mut(e).insert(ChildOf(container));
    });
    assert!(
        has_derived_verts(&sim, &map, id),
        "o filho vive sob um container de envelope"
    );
    assert_eq!(handles(&sim, &scene, &map, id), 0);
}

/// **O SPINE de um blend CONTINUA tendo alça — e isto não é um esquecimento.**
///
/// É o único host vivo cuja escrita é **condicional**: o `blend_live` só chama
/// `write_spine` enquanto `!spine_authored`, e a detecção de autoria compara o `verts`
/// INTEIRO contra o último auto-escrito. Como o `corner_radius` mora dentro do vértice,
/// arrastar a alça **é** o gesto que marca `spine_authored` — a partir daí ninguém
/// reescreve, e o raio sobrevive. Recusar aqui tiraria uma afordância que funciona.
///
/// Este gate existe para que ninguém "conserte" o spine para dentro da lista de recusa:
/// a diferença entre os cinco hosts é *escrita incondicional* vs *escrita até o artista
/// assumir*, e é essa a pergunta, não a contagem de componentes.
#[test]
fn a_blend_spine_keeps_its_radius_handles_because_authoring_one_stops_the_rewrite() {
    let (sim, scene, map, id) = square_with(|sim, e| {
        sim.world_mut().entity_mut(e).insert(ph2d_ecs::VecBlend {
            sources: vec![1, 2],
            steps: 3,
            spine_authored: false,
        });
    });
    assert!(
        !has_derived_verts(&sim, &map, id),
        "o spine se auto-autora: a escrita dele é condicional, não incondicional"
    );
    assert_eq!(handles(&sim, &scene, &map, id), 4);
}

/// Sem seleção, nenhuma alça (e sem varrer a cena inteira atrás delas).
#[test]
fn no_selection_no_handles() {
    let (sim, scene, map, _) = square_with(|_, _| {});
    assert!(
        view(
            &sim,
            &scene,
            &map,
            None,
            &ph2d_vec_scene::VecXforms::default(),
            0.01
        )
        .is_empty()
    );
}
