//! Gate da política das quinas editáveis: **geometria DERIVADA não tem quina que as
//! ferramentas de quina (Fillet / Chamfer) possam editar.**
//!
//! As ferramentas autoram um `corner_radius` DENTRO do vértice. Um host vivo que reescreve
//! `path.verts` a cada frame varre esse raio — o sintoma é o pior que existe: a quina
//! arredonda, funciona, e o trabalho some um frame depois, sem erro nenhum. O raio de uma
//! forma viva é um campo DELA (o painel); o por-vértice é para caminho DESENHADO.
//!
//! # E a política enumerava UM dos cinco (ADR-0132 §5)
//!
//! O guard perguntava `is_live_shape`, e a linha ganhou mais quatro objetos vivos desde
//! então. Hoje **quatro** hosts reescrevem `verts` (`vec_shape_live`, `connector_live`,
//! `morph_live`, `envelope_live`) e só o primeiro era recusado. A pergunta virou
//! [`super::has_derived_verts`], que o press das ferramentas de quina (`input_dispatch`)
//! consulta antes de agarrar. [[feedback_a_condition_that_enumerates_its_readers_rots]]
//!
//! Um gate por caso, de propósito: uma defesa em camadas precisa que a mutação de UMA
//! camada sangre sozinha. [[feedback_layered_defenses_need_per_layer_gates]]
//!
//! NOTA: esta política era a mesma da antiga ALÇA de raio do Node (removida na consolidação
//! Fillet/Chamfer). Os gates de handle-count (`view`) saíram com a alça; a política de
//! recusa que sobrevive é `has_derived_verts`, aqui verificada por-host.

use super::*;
use ph2d_ecs::{
    ChildOf, Entity, EnvelopeKind, Transform, VecConnector, VecEnvelope, VecEnvelopeChild,
    VecMorph, VecShape,
};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex};

/// Uma cena com um quadrado, a entidade dele, e o mapa path↔entidade. O `decorate` pendura
/// nessa entidade (ou numa nova, no caso do envelope) o que a torna geometria derivada.
fn square_with(
    decorate: impl FnOnce(&mut SimWorld, Entity),
) -> (SimWorld, VecEntityMap, VecPathId) {
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
    (sim, map, id)
}

/// Um caminho DESENHADO (sem receita, sem host vivo) NÃO é derivado — as ferramentas de
/// quina podem editá-lo. **É o controle positivo de todos os gates abaixo**: sem ele, "é
/// derivado" ficaria verde com a função a devolver `true` para tudo.
/// [[feedback_absence_gate_needs_a_presence_sibling]]
#[test]
fn a_drawn_path_is_not_derived_so_the_corner_tools_may_edit_it() {
    let (sim, map, id) = square_with(|_, _| {});
    assert!(
        !has_derived_verts(&sim, &map, id),
        "caminho desenhado é editável"
    );
}

/// **Uma FORMA VIVA é derivada.** O `vec_shape_live::recook_into` substitui `path.verts`
/// INTEIRO a cada mudança de parâmetro, e o `corner_radius` mora dentro do vértice. Um raio
/// autorado aqui sobreviveria até o usuário encostar num slider — e sumiria sem erro nenhum.
#[test]
fn a_live_shape_is_derived_because_the_recook_would_erase_the_radius() {
    let (sim, map, id) = square_with(|sim, e| {
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
}

/// **Um CONECTOR é derivado.** O `connector_live::recook` limpa e reescreve `p.verts` a cada
/// frame, a partir das pontas — o raio não sobreviveria a um único frame.
#[test]
fn a_connector_is_derived() {
    let (sim, map, id) = square_with(|sim, e| {
        sim.world_mut()
            .entity_mut(e)
            .insert(VecConnector::between(1, 2));
    });
    assert!(has_derived_verts(&sim, &map, id));
}

/// **Um MORPH é derivado.** `morph_live::recook` escreve `p.verts = cooked.verts` a cada
/// frame — e no morph isso é a feature (o `t` é animável), então o raio morreria já no frame
/// seguinte ao arrasto.
#[test]
fn a_morph_is_derived() {
    let (sim, map, id) = square_with(|sim, e| {
        sim.world_mut().entity_mut(e).insert(VecMorph::new(1, 2));
    });
    assert!(has_derived_verts(&sim, &map, id));
}

/// **Um FILHO DE ENVELOPE é derivado — e este é o caso que o ADR-0132 §5 reporta.**
///
/// O componente mora no CONTAINER, não no filho: `get::<VecEnvelope>(filho)` é `None`, e é
/// por isso que um guard que só olhasse a própria entidade deixaria a quina passar. A
/// pergunta sobe a cadeia de ancestrais — a mesma caminhada do `envelope_live::container_of`.
#[test]
fn an_envelope_child_is_derived() {
    let (sim, map, id) = square_with(|sim, e| {
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
}

/// **O SPINE de um blend NÃO é derivado — e isto não é um esquecimento.**
///
/// É o único host vivo cuja escrita é **condicional**: o `blend_live` só chama `write_spine`
/// enquanto `!spine_authored`, e a detecção de autoria compara o `verts` INTEIRO. Como o
/// `corner_radius` mora dentro do vértice, editar a quina **é** o gesto que marca
/// `spine_authored` — a partir daí ninguém reescreve, e o raio sobrevive.
///
/// Este gate existe para que ninguém "conserte" o spine para dentro da lista de recusa: a
/// diferença entre os cinco hosts é *escrita incondicional* vs *escrita até o artista
/// assumir*, e é essa a pergunta, não a contagem de componentes.
#[test]
fn a_blend_spine_is_not_derived_because_authoring_it_stops_the_rewrite() {
    let (sim, map, id) = square_with(|sim, e| {
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
}
