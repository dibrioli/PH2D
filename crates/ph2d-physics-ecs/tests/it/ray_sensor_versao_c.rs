//! **O Ray Sensor, versão C** (dono 15/11, «abranger todas as possibilidades») — os gates das sete
//! opções novas. Cada um mede a opção LIGADA e DESLIGADA na mesma fixtura (o controlo), e diz no
//! doc-comment a mutação que o sangra.
//!
//! ⚠️ Os sensores do personagem (os cinco consumidores do `cast_ray`) NÃO passam por aqui: a porta
//! deles é a mesma de antes, medida por `the_caster_can_exclude_itself` e
//! `a_layer_the_caster_does_not_collide_with_is_invisible_to_it` (`ph2d-physics`, `world::cast`).

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, StableId, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, RaySensor, RaySignals, RigidBody, SeeFilter,
};
use ph2d_tags::TagTree;

pub(crate) fn caixa(sim: &mut SimWorld, nome: &str, x: f32, half_x: f32, sensor: bool) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x,
                    half_y: 1.0,
                },
                is_sensor: sensor,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ))
        .id()
}

pub(crate) fn parede(sim: &mut SimWorld, nome: &str, x: f32) -> Entity {
    caixa(sim, nome, x, 0.25, false)
}

/// Um corpo FILHO de `pai`, com forma própria, em `x` local.
fn parte(sim: &mut SimWorld, nome: &str, pai: Entity, x: f32) -> Entity {
    let e = caixa(sim, nome, x, 0.25, false);
    sim.world_mut().entity_mut(e).insert(ChildOf(pai));
    e
}

pub(crate) fn raio(sim: &mut SimWorld, em: Vec2, pai: Option<Entity>, r: RaySensor) -> Entity {
    let e = sim
        .world_mut()
        .spawn((Name::new("Ray Sensor"), Transform::from_translation(em), r))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

pub(crate) fn para(angle: f32) -> RaySensor {
    RaySensor {
        angle,
        reach: 20.0,
        ..RaySensor::default()
    }
}

pub(crate) fn ve(sim: &mut SimWorld, e: Entity, tree: &TagTree) -> Option<Entity> {
    let mut b = PhysicsBridge::new();
    b.set_tag_tree(tree);
    b.dispatch(sim, true, 1);
    b.ray_sensor_hits().get(&e).map(|h| h.body)
}

/// ⭐ **«Ignore my body» desligado: o raio vê o próprio dono.**
///
/// **Mutação que sangra:** passar sempre `raio.excluir` (ignorar a opção).
#[test]
fn versao_c_ignore_body_falso_ve_o_dono() {
    for ligada in [true, false] {
        let mut sim = SimWorld::new();
        let atras = parede(&mut sim, "Atras", -3.0);
        let heroi = caixa(&mut sim, "Heroi", 0.0, 0.5, false);
        let r = raio(
            &mut sim,
            Vec2::new(2.0, 0.0),
            Some(heroi),
            RaySensor {
                ignore_body: ligada,
                ..para(180.0)
            },
        );
        let visto = ve(&mut sim, r, &TagTree::new());
        assert_eq!(
            visto,
            Some(if ligada { atras } else { heroi }),
            "ignore_body = {ligada}"
        );
    }
}

/// ⭐⭐ **«Ignore my other parts»: a espada do herói (corpo próprio, filho) não é vista.**
///
/// **Mutação que sangra:** o predicado deixa passar a família (`raiz` sempre `None`).
#[test]
fn versao_c_ignore_parts_nao_ve_a_espada() {
    for ligada in [true, false] {
        let mut sim = SimWorld::new();
        let muro = parede(&mut sim, "Muro", 4.0);
        let heroi = caixa(&mut sim, "Heroi", 0.0, 0.5, false);
        let espada = parte(&mut sim, "Espada", heroi, 1.5);
        let r = raio(
            &mut sim,
            Vec2::new(0.75, 0.0),
            Some(heroi),
            RaySensor {
                ignore_parts: ligada,
                ..para(0.0)
            },
        );
        let visto = ve(&mut sim, r, &TagTree::new());
        assert_eq!(
            visto,
            Some(if ligada { muro } else { espada }),
            "ignore_parts = {ligada}"
        );
    }
}

/// ⭐⭐ **Um raio debaixo da ESPADA ignora o HERÓI** — a raiz da família é o corpo mais alto.
///
/// **Mutação que sangra:** `familia_raiz` devolver o corpo mais PERTO (o `owner_body`) em vez do
/// mais alto.
#[test]
fn versao_c_raio_debaixo_da_espada_ignora_o_heroi() {
    for ligada in [true, false] {
        let mut sim = SimWorld::new();
        let muro = parede(&mut sim, "Muro", -4.0);
        let heroi = caixa(&mut sim, "Heroi", 0.0, 0.5, false);
        let espada = parte(&mut sim, "Espada", heroi, 1.5);
        let r = raio(
            &mut sim,
            Vec2::new(-0.5, 0.0),
            Some(espada),
            RaySensor {
                ignore_parts: ligada,
                ..para(180.0)
            },
        );
        let visto = ve(&mut sim, r, &TagTree::new());
        assert_eq!(
            visto,
            Some(if ligada { muro } else { heroi }),
            "ignore_parts = {ligada}"
        );
    }
}

pub(crate) fn com_tag(sim: &mut SimWorld, e: Entity, id: u64, tag: ph2d_tags::TagId) {
    sim.world_mut()
        .entity_mut(e)
        .insert((StableId(id), ph2d_ecs::tags::Tags::from_ids([tag])));
}

/// ⭐⭐⭐ **«Only see» + «Walls block»**: uma parede SEM a tag tapa o alvo (linha de visão); sem
/// `walls_block` o raio atravessa-a e vê o alvo.
///
/// **Mutações que sangram:** ignorar `walls_block` no predicado · não filtrar depois do acerto
/// (vê a parede) · `OnlyTag` tratado como `Everything`.
#[test]
fn versao_c_only_tag_com_e_sem_paredes_que_tapam() {
    let mut tree = TagTree::new();
    let inimigo = tree.create("Inimigo").expect("tag");
    for tapa in [true, false] {
        let mut sim = SimWorld::new();
        let _parede = parede(&mut sim, "Parede", 2.0);
        let alvo = parede(&mut sim, "Alvo", 4.0);
        com_tag(&mut sim, alvo, 1, inimigo);
        let r = raio(
            &mut sim,
            Vec2::ZERO,
            None,
            RaySensor {
                only_see: SeeFilter::OnlyTag,
                see_tag: inimigo.0,
                walls_block: tapa,
                ..para(0.0)
            },
        );
        let visto = ve(&mut sim, r, &tree);
        assert_eq!(
            visto,
            if tapa { None } else { Some(alvo) },
            "walls_block = {tapa}"
        );
    }
}

/// ⭐⭐ **«All except tag»**: o amigo é transparente (sem `walls_block`) ou tapa (com); o
/// controlo `Everything` vê o amigo.
///
/// **Mutação que sangra:** `ExceptTag` com o sentido de `OnlyTag` (`pertence == so` trocado).
#[test]
fn versao_c_except_tag() {
    let mut tree = TagTree::new();
    let amigo = tree.create("Amigo").expect("tag");
    let casos = [
        (SeeFilter::ExceptTag, false, "alvo"),
        (SeeFilter::ExceptTag, true, "nada"),
        (SeeFilter::Everything, false, "amigo"),
    ];
    for (filtro, tapa, espera) in casos {
        let mut sim = SimWorld::new();
        let a = parede(&mut sim, "Amigo", 2.0);
        com_tag(&mut sim, a, 1, amigo);
        let alvo = parede(&mut sim, "Alvo", 4.0);
        let r = raio(
            &mut sim,
            Vec2::ZERO,
            None,
            RaySensor {
                only_see: filtro,
                see_tag: amigo.0,
                walls_block: tapa,
                ..para(0.0)
            },
        );
        let visto = ve(&mut sim, r, &tree);
        let certo = match espera {
            "alvo" => Some(alvo),
            "amigo" => Some(a),
            _ => None,
        };
        assert_eq!(visto, certo, "{filtro:?}, walls_block = {tapa}");
    }
}

/// ⭐⭐ **«Hit triggers»: o raio acerta um sensor**; desligado, atravessa-o (a lei de antes).
///
/// **Mutação que sangra:** `hit_sensors` ignorado na porta (`EXCLUDE_SENSORS` sempre).
#[test]
fn versao_c_hit_triggers_ve_um_sensor() {
    for ligada in [true, false] {
        let mut sim = SimWorld::new();
        let gatilho = caixa(&mut sim, "Gatilho", 2.0, 0.25, true);
        let muro = parede(&mut sim, "Muro", 4.0);
        let r = raio(
            &mut sim,
            Vec2::ZERO,
            None,
            RaySensor {
                hit_triggers: ligada,
                ..para(0.0)
            },
        );
        let visto = ve(&mut sim, r, &TagTree::new());
        assert_eq!(
            visto,
            Some(if ligada { gatilho } else { muro }),
            "hit_triggers = {ligada}"
        );
    }
}

/// ⭐⭐ **«Starts inside»**: desligado, a forma onde o raio nasce é ignorada (Godot
/// `hit_from_inside = false`); ligado, ela responde a `0`.
///
/// **Mutação que sangra:** o predicado `contains_point` ignorado · `hit_from_inside` invertido.
#[test]
fn versao_c_starts_inside() {
    for ligada in [true, false] {
        let mut sim = SimWorld::new();
        let casa = parede(&mut sim, "Casa", 2.0);
        let muro = parede(&mut sim, "Muro", 5.0);
        let r = raio(
            &mut sim,
            Vec2::new(2.0, 0.0),
            None,
            RaySensor {
                starts_inside: ligada,
                ..para(0.0)
            },
        );
        let mut b = PhysicsBridge::new();
        b.dispatch(&mut sim, true, 1);
        let h = b
            .ray_sensor_hits()
            .get(&r)
            .copied()
            .expect("ve alguma coisa");
        if ligada {
            assert_eq!(h.body, casa, "ligada: acha a forma onde nasce");
            assert!(h.distance.abs() < 1.0e-4, "a 0: {}", h.distance);
        } else {
            assert_eq!(h.body, muro, "desligada: a casa onde nasce e' ignorada");
            assert!((h.distance - 2.75).abs() < 1.0e-3, "d = {}", h.distance);
        }
    }
}

/// ⭐⭐⭐ **«See all hits»**: as duas paredes, por distância; On Enter/On Exit POR OBJECTO (encurtar
/// o alcance solta só a de longe). Controlo: desligado, a lista é vazia e o mais perto é o de antes.
///
/// **Mutações que sangram:** não ordenar · parar no primeiro · emitir a saída de quem continua na
/// lista · não emitir a entrada do segundo.
#[test]
fn versao_c_all_hits_duas_paredes_e_arestas_por_objecto() {
    let mut sim = SimWorld::new();
    // Inseridas LONGE primeiro: a ordem tem de ser a da distância, não a da inserção.
    let longe = parede(&mut sim, "Longe", 4.0);
    let perto = parede(&mut sim, "Perto", 2.0);
    let r = raio(
        &mut sim,
        Vec2::ZERO,
        None,
        RaySensor {
            all_hits: true,
            ..para(0.0)
        },
    );
    sim.world_mut().entity_mut(r).insert(RaySignals {
        on_enter: "vi".into(),
        on_exit: "perdi".into(),
    });
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, true, 1);
    let todos: Vec<Entity> = b.ray_sensor_hits_all(r).iter().map(|h| h.body).collect();
    assert_eq!(todos, vec![perto, longe], "as duas, por distancia");
    let d: Vec<f32> = b
        .ray_sensor_hits_all(r)
        .iter()
        .map(|h| h.distance)
        .collect();
    assert!(
        (d[0] - 1.75).abs() < 1.0e-3 && (d[1] - 3.75).abs() < 1.0e-3,
        "{d:?}"
    );
    assert_eq!(b.ray_sensor_hits().get(&r).map(|h| h.body), Some(perto));
    assert_eq!(b.ray_enters(), &[(r, perto), (r, longe)], "entra cada um");
    assert!(b.ray_exits().is_empty());

    // Encurta: a de longe sai, a de perto fica — e não sai nem volta a entrar.
    if let Some(mut s) = sim.world_mut().get_mut::<RaySensor>(r) {
        s.reach = 3.0;
    }
    b.dispatch(&mut sim, true, 2);
    assert_eq!(b.ray_exits(), &[(r, longe)], "sai so' a de longe");
    assert!(b.ray_enters().is_empty(), "a de perto continua: nada entra");

    // O CONTROLO: a opção desligada não enche a lista.
    let mut sim2 = SimWorld::new();
    parede(&mut sim2, "Longe", 4.0);
    let perto2 = parede(&mut sim2, "Perto", 2.0);
    let r2 = raio(&mut sim2, Vec2::ZERO, None, para(0.0));
    let mut b2 = PhysicsBridge::new();
    b2.dispatch(&mut sim2, true, 1);
    assert!(b2.ray_sensor_hits_all(r2).is_empty());
    assert_eq!(b2.ray_sensor_hits().get(&r2).map(|h| h.body), Some(perto2));
    assert_eq!(b2.ray_enters(), &[(r2, perto2)]);
}

/// A marca do raio `r` (o que o canvas desenha): `(alcance desenhado, primeiro acerto, os outros)`.
pub(crate) fn marca(b: &PhysicsBridge) -> (f32, Option<f32>, Vec<f32>) {
    let m = b.ray_marks().first().expect("a marca do raio");
    match m.shape {
        ph2d_physics_ecs::ProbeShape::Ray { reach, hit, .. } => {
            (reach, hit, m.more_hits.como_lista().to_vec())
        }
        _ => panic!("o raio sem marca de raio"),
    }
}

/// ⭐⭐ **O desenho de «See all hits»**: a linha vai ao alcance inteiro quando nada bloqueia, com um
/// tique em CADA acerto; com «Walls block» e uma parede sem a tag, a linha PÁRA nela. Controlo:
/// desligado, um tique só (o mais perto) e o alcance inteiro.
///
/// **Mutações que sangram:** a marca sem os acertos a mais · o fim no alcance mesmo com bloqueio.
#[test]
fn versao_c_all_hits_desenha_um_tique_por_acerto_ate_ao_bloqueio() {
    let perto_longe = |all_hits: bool| {
        let mut sim = SimWorld::new();
        parede(&mut sim, "Longe", 4.0);
        parede(&mut sim, "Perto", 2.0);
        let _ = raio(
            &mut sim,
            Vec2::ZERO,
            None,
            RaySensor {
                all_hits,
                ..para(0.0)
            },
        );
        let mut b = PhysicsBridge::new();
        b.dispatch(&mut sim, true, 1);
        marca(&b)
    };
    let (fim, hit, mais) = perto_longe(true);
    assert!(
        (fim - 20.0).abs() < 1.0e-4,
        "nada bloqueia: o alcance inteiro ({fim})"
    );
    assert!(hit.is_some_and(|h| (h - 1.75).abs() < 1.0e-3), "{hit:?}");
    assert_eq!(mais.len(), 1, "um tique a mais, o da de longe: {mais:?}");
    assert!((mais[0] - 3.75).abs() < 1.0e-3, "{mais:?}");
    let (fim, hit, mais) = perto_longe(false);
    assert!(
        (fim - 20.0).abs() < 1.0e-4 && mais.is_empty(),
        "controlo: {fim} {mais:?}"
    );
    assert!(hit.is_some_and(|h| (h - 1.75).abs() < 1.0e-3));

    // Com a tag e «Walls block»: a parede sem a tag tapa — a linha pára nela, sem acerto.
    let mut tree = TagTree::new();
    let inimigo = tree.create("Inimigo").expect("tag");
    let mut sim = SimWorld::new();
    parede(&mut sim, "Parede", 2.0);
    let alvo = parede(&mut sim, "Alvo", 4.0);
    com_tag(&mut sim, alvo, 1, inimigo);
    let _ = raio(
        &mut sim,
        Vec2::ZERO,
        None,
        RaySensor {
            only_see: SeeFilter::OnlyTag,
            see_tag: inimigo.0,
            all_hits: true,
            ..para(0.0)
        },
    );
    let mut b = PhysicsBridge::new();
    b.set_tag_tree(&tree);
    b.dispatch(&mut sim, true, 1);
    let (fim, hit, mais) = marca(&b);
    assert!((fim - 1.75).abs() < 1.0e-3, "a linha pára na parede: {fim}");
    assert_eq!((hit, mais.len()), (None, 0));
}
