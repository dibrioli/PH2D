//! **O QUE OS OBJECTOS DESTE SMOKE SÃO** — a arte e as entidades que cada modo carimba,
//! cortadas do despachante no teto de LOC (HR-18) pela costura que ele já tinha: o
//! `motion_object_smoke.rs` responde *qual modo liga o quê*, e este ficheiro *o que existe
//! na cena para ser ligado*.
//!
//! ⚠️ **É o QUARTO corte deste smoke por responsabilidade** — os modos `=7`, `=8` e `=9`
//! saíram cada um para o seu irmão quando trouxeram fiação própria; aqui sai o que os
//! modos PARTILHAM. A tolerância desce com o corte, como o gate manda.
//!
//! ⚠️ **A leitura que vale a pena levar daqui:** as três artes deste ficheiro não são
//! intercambiáveis, e a diferença é de ROTA DE DESENHO. Um `Sprite`/um Flip assado é um
//! quad TEXTURADO; uma forma vectorial é desenhada **viva** pelo passe do Vello desde o
//! ADR-0154 (`geometry_id`), e não tem textura nenhuma. Escolher a errada faz um smoke
//! passar mudo — foi o que aconteceu com a cena `=9` em 2026-08-25.

use super::{DEMO_TILE_KEY, OBJECT};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_vec_scene::{Paint, Rgba8, VecPath};

/// Modo `=1`: um sprite direto (entidade com `Name`, não precisa do `sync`).
pub(super) fn spawn_sprite(sim: &mut ph2d_ecs::SimWorld) {
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Sprite::atlas(DEMO_TILE_KEY, [0.8, 0.8], [1.0, 1.0, 1.0, 1.0]),
        Name::new(OBJECT),
    ));
}

/// Modo `=2`: uma estrela FILLED (a arte, inconfundível quando assada numa tile).
pub(super) fn star_shape() -> VecPath {
    let mut p = ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.45);
    p.fill = Some(Paint::solid(Rgba8::new(255, 170, 40, 255)));
    p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        Rgba8::new(60, 40, 10, 255),
        0.02,
    ));
    p
}

/// Modo `=2` frame 6: a entidade da forma já existe (o `sync` do frame a criou),
/// então ela ganha o **nome** que o `source.object` procura. A única forma da
/// cena é a nossa.
pub(super) fn name_vector_entity(
    sim: &mut ph2d_ecs::SimWorld,
    map: &crate::vec_entities::VecEntityMap,
) -> bool {
    name_vector_entity_as(sim, map, OBJECT)
}

/// Como o [`name_vector_entity`], mas com o nome dado — o `=9` põe a estrela ao lado de um
/// Flip, então ela não pode chamar-se `Object`.
pub(super) fn name_vector_entity_as(
    sim: &mut ph2d_ecs::SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    name: &str,
) -> bool {
    let Some((_, &bits)) = map.iter().next() else {
        return false;
    };
    let e = ph2d_ecs::Entity::from_bits(bits);
    match sim.world_mut().get_entity_mut(e) {
        Ok(mut ent) => {
            ent.insert(Name(name.to_string()));
            true
        }
        Err(_) => false,
    }
}

/// Modo `=3`: um objeto FLIP de 2 camadas (BG azul + FG laranja), empurrado no
/// `FlipDoc`. A ENTIDADE dele (com `Name` "Object") é criada pelo
/// `flip_entities::sync` — que copia o nome do objeto —, então o grafo o acha pelo
/// nome sem o smoke precisar nomear nada (≠ do vetor). A membrana compõe as DUAS
/// camadas no frame atual numa tile.
pub(super) fn spawn_flip_object(flip: &mut ph2d_flip::FlipDoc) {
    spawn_flip_object_named(flip, OBJECT);
}

/// Como `spawn_flip_object`, mas com um nome dado (o filho Flip de um grupo precisa
/// de um nome distinto do grupo, doc 86 §2 A4).
pub(super) fn spawn_flip_object_named(flip: &mut ph2d_flip::FlipDoc, name: &str) {
    use ph2d_flip::{Hold, KeyKind, Rgba};
    let oid = flip.push_object(name);
    let obj = flip.object_mut(oid).expect("objeto Flip recém-criado");
    obj.fps = 12.0;
    // BG: um retângulo azul preenchido (o campo).
    let bg = obj.add_layer("BG");
    if let Some(d) = obj.insert_frame(bg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("desenho BG")
            .strokes
            .push(flip_rect(
                Vec2::new(-0.9, -0.6),
                Vec2::new(0.9, 0.6),
                Rgba::new(0.2, 0.5, 0.95, 1.0),
            ));
    }
    // FG: um quadrado laranja menor por cima — a arte que torna a tile assada
    // INCONFUNDÍVEL (duas camadas compostas, não um quad chapado).
    let fg = obj.add_layer("FG");
    if let Some(d) = obj.insert_frame(fg, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("desenho FG")
            .strokes
            .push(flip_rect(
                Vec2::new(-0.35, -0.35),
                Vec2::new(0.35, 0.35),
                Rgba::new(0.98, 0.7, 0.15, 1.0),
            ));
    }
}

/// Um retângulo Flip FECHADO e PREENCHIDO (espelha `flip_demo::filled_rect`).
pub(super) fn flip_rect(min: Vec2, max: Vec2, color: ph2d_flip::Rgba) -> ph2d_flip::FlipStroke {
    use ph2d_flip::{Fill, FlipStroke, Point};
    let mut s = FlipStroke::new();
    for corner in [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)] {
        s.push_point(Point {
            pos: corner,
            width: 0.04,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hardness = 1.0;
    s.fill = Some(Fill {
        color,
        opacity: 1.0,
    });
    s
}

/// Um `Transform` LOCAL (relativo ao grupo) numa posição, resto identidade (A4).
pub(super) fn child_at(x: f32, y: f32) -> Transform {
    Transform {
        translation: Vec2::new(x, y),
        ..Transform::IDENTITY
    }
}

/// Acha a entidade-GRUPO pelo nome (`Name` + `GroupedChildren`), doc 86 §2 A4.
pub(super) fn find_group(sim: &mut ph2d_ecs::SimWorld, name: &str) -> Option<ph2d_ecs::Entity> {
    let mut q = sim
        .world_mut()
        .query_filtered::<(ph2d_ecs::Entity, &Name), ph2d_ecs::With<ph2d_ecs::GroupedChildren>>();
    let world = sim.world();
    q.iter(world).find(|(_, n)| n.0 == name).map(|(e, _)| e)
}
