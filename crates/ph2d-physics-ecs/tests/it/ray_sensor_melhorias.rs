//! ⭐⭐⭐ **As melhorias do Ray Sensor** (GEF 07 RS-1..RS-12, dono 15/11: «vamos às melhorias do nosso ray»).
//!
//! O que o Godot também faz é corrido contra ele ([`oraculo_godot_das_melhorias`], a fixture
//! `docs/GameEngineFuncional/componentes/oraculo_raio_rs/godot_rs.txt`, corrida por `corre.sh`); o que ele
//! não tem (o leque, o alvo, a lista pelo nome, a ordem) mede-se contra a lei escrita. Cada gate diz a
//! mutação que o sangra.

use crate::ray_sensor_versao_c::{caixa, com_tag, parede, raio};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, StableId, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, RaySensor, RayShape, RigidBody, SeeFilter,
};
use ph2d_tags::TagTree;

/// Uma caixa estática de meia largura `hx` e meia altura `hy` em `(x, y)`, com `StableId(id)`.
pub(crate) fn bloco(
    sim: &mut SimWorld,
    nome: &str,
    (x, y): (f32, f32),
    (hx, hy): (f32, f32),
    id: u64,
) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            StableId(id),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: hx,
                    half_y: hy,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ))
        .id()
}

/// A cena do oráculo, em metros: o Owner com o raio filho a olhar para baixo 3 m, o Floor, o Floor2 e o Side.
pub(crate) struct Cena {
    pub(crate) sim: SimWorld,
    pub(crate) dono: Entity,
    pub(crate) raio: Entity,
    pub(crate) floor: Entity,
    pub(crate) floor2: Entity,
    pub(crate) side: Entity,
}

pub(crate) fn cena(r: RaySensor) -> Cena {
    let mut sim = SimWorld::new();
    let floor = bloco(&mut sim, "Floor", (0.0, -1.1), (2.0, 0.1), 11);
    let floor2 = bloco(&mut sim, "Floor2", (0.0, -1.6), (2.0, 0.1), 12);
    let side = bloco(&mut sim, "Side", (0.25, -0.5), (0.05, 0.05), 13);
    let dono = bloco(&mut sim, "Owner", (0.0, 0.0), (0.1, 0.1), 10);
    let raio = raio(
        &mut sim,
        Vec2::ZERO,
        Some(dono),
        RaySensor {
            angle: -90.0,
            reach: 3.0,
            ..r
        },
    );
    Cena {
        sim,
        dono,
        raio,
        floor,
        floor2,
        side,
    }
}

pub(crate) fn nome(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<Name>(e)
        .map(|n| n.as_str().to_owned())
        .unwrap_or_default()
}

/// ⭐⭐ **RS-6 — desligar a ver emite On Exit no MESMO tique, e calado fica** (a regra do Unreal: desligar
/// fecha a aresta; sem ela uma luz acesa pelo On Enter ficava acesa para sempre). Religar volta a entrar.
///
/// **Mutação que sangra:** o laço saltar o desligado ANTES de tirar o que ele via (`velho.remove`).
#[test]
fn desligar_a_ver_emite_on_exit_e_cala() {
    let mut c = cena(RaySensor::default());
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut c.sim, true, 1);
    assert_eq!(b.ray_enters(), &[(c.raio, c.floor)]);
    assert!(b.ray_seeing(c.raio), "RS-5: está a ver");
    c.sim
        .world_mut()
        .get_mut::<RaySensor>(c.raio)
        .expect("r")
        .enabled = false;
    b.dispatch(&mut c.sim, true, 2);
    assert_eq!(
        b.ray_exits(),
        &[(c.raio, c.floor)],
        "On Exit no tique em que se desliga"
    );
    assert!(!b.ray_seeing(c.raio));
    b.dispatch(&mut c.sim, true, 3);
    assert!(
        b.ray_exits().is_empty() && b.ray_enters().is_empty(),
        "desligado cala"
    );
    c.sim
        .world_mut()
        .get_mut::<RaySensor>(c.raio)
        .expect("r")
        .enabled = true;
    b.dispatch(&mut c.sim, true, 4);
    assert_eq!(b.ray_enters(), &[(c.raio, c.floor)], "religado volta a ver");
}

/// ⭐⭐ **RS-1 com «See all hits»: o círculo vê o Side E os dois chãos**, por distância.
///
/// **Mutação que sangra:** o caminho «todos» do círculo usar a linha.
#[test]
fn o_circulo_ve_todos_os_acertos_por_distancia() {
    let mut c = cena(RaySensor {
        shape: RayShape::Thick,
        thickness: 0.6,
        all_hits: true,
        ..RaySensor::default()
    });
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut c.sim, true, 1);
    let todos: Vec<Entity> = b
        .ray_sensor_hits_all(c.raio)
        .iter()
        .map(|h| h.body)
        .collect();
    assert_eq!(todos, vec![c.side, c.floor, c.floor2]);
    let m = &b.ray_marks()[0];
    assert!(
        (m.feixe.largura - 0.6).abs() < 1.0e-6,
        "o desenho sabe a largura"
    );
}

/// ⭐⭐⭐ **A «Shape» escolhe qual número conta** (dono 15/11: *«thickness e cone são conflitantes»*). Na cena do
/// oráculo, com `thickness = 0,6` E `cone = 90` nos três: Line vê só o Floor (a linha passa ao lado do Side) e o
/// desenho é a linha; Thick vê o Side primeiro, um acerto só (o cone não conta), desenho de 0,6 m sem leque; Cone vê
/// o Side e o Floor por linhas de vista FINAS — o Side à distância do ponto mais perto dele (0,49 m; um círculo de
/// 0,3 m tocava-o a ~0,2 m) — e o desenho é o leque sem largura.
///
/// **Mutações que sangram:** o `lanca` usar `thickness` sem olhar a forma (Line vê o Side) · o leque com o
/// círculo da `thickness` (o Side mais perto) · o `com_feixe` ignorar a forma (o desenho com os dois números).
#[test]
fn a_forma_escolhe_qual_numero_conta() {
    for forma in [RayShape::Line, RayShape::Thick, RayShape::Cone] {
        let mut c = cena(RaySensor {
            shape: forma,
            thickness: 0.6,
            cone: 90.0,
            ..RaySensor::default()
        });
        let mut b = PhysicsBridge::new();
        b.dispatch(&mut c.sim, true, 1);
        let hits = b.ray_sensor_hits_all(c.raio).to_vec();
        let corpos: Vec<Entity> = hits.iter().map(|h| h.body).collect();
        let m = &b.ray_marks()[0];
        let (quer, largura, leque) = match forma {
            RayShape::Line => (vec![c.floor], 0.0, false),
            RayShape::Thick => (vec![c.side], 0.6, false),
            RayShape::Cone => (vec![c.side, c.floor], 0.0, true),
        };
        assert_eq!(corpos, quer, "{forma:?}");
        assert!(
            (m.feixe.largura - largura).abs() < 1.0e-6,
            "{forma:?}: o desenho com a largura {}",
            m.feixe.largura
        );
        assert_eq!(
            m.feixe.meio_cone > 0.0,
            leque,
            "{forma:?}: o leque desenhado"
        );
        if forma == RayShape::Cone {
            assert!(
                hits[0].distance > 0.45,
                "o leque olha por linhas finas, não pelo círculo: {}",
                hits[0].distance
            );
        }
    }
}

/// Um alvo com tag no plano, para o leque.
fn alvo(sim: &mut SimWorld, nome: &str, (x, y): (f32, f32), id: u64) -> Entity {
    bloco(sim, nome, (x, y), (0.2, 0.2), id)
}

/// ⭐⭐⭐ **RS-2 — o leque vê cada objecto DENTRO dele e não tapado, um acerto (e um On Enter) por objecto.**
/// Um guarda olha para a direita com 60°: vê o A (a 20°) e o B (a −15°), não vê o C (a 50°, fora) nem o
/// D (dentro, atrás de uma parede sem a tag, com «Walls block»). Sem «Walls block» o D aparece.
///
/// **Mutações que sangram:** o rumo sem o `clamp` (vê o C) · a linha de visão aceitar outro corpo
/// (vê o D atrás da parede) · o leque devolver só o primeiro.
#[test]
fn o_leque_ve_quem_esta_dentro_e_nao_tapado() {
    let mut tree = TagTree::new();
    let inimigo = tree.create("Enemy").expect("tag");
    for tapa in [true, false] {
        let mut sim = SimWorld::new();
        let a = alvo(
            &mut sim,
            "A",
            (
                3.0 * 20f32.to_radians().cos(),
                3.0 * 20f32.to_radians().sin(),
            ),
            1,
        );
        let bb = alvo(
            &mut sim,
            "B",
            (
                2.0 * (-15f32).to_radians().cos(),
                2.0 * (-15f32).to_radians().sin(),
            ),
            2,
        );
        let cc = alvo(
            &mut sim,
            "C",
            (
                3.0 * 50f32.to_radians().cos(),
                3.0 * 50f32.to_radians().sin(),
            ),
            3,
        );
        let d = alvo(&mut sim, "D", (4.0, 0.0), 4);
        for (e, id) in [(a, 1), (bb, 2), (cc, 3), (d, 4)] {
            com_tag(&mut sim, e, id, inimigo);
        }
        bloco(&mut sim, "Wall", (1.0, 0.0), (0.05, 0.2), 9);
        let olho = raio(
            &mut sim,
            Vec2::ZERO,
            None,
            RaySensor {
                angle: 0.0,
                reach: 6.0,
                shape: RayShape::Cone,
                cone: 60.0,
                only_see: SeeFilter::OnlyTag,
                see_tag: inimigo.0,
                walls_block: tapa,
                ..RaySensor::default()
            },
        );
        let mut b = PhysicsBridge::new();
        b.set_tag_tree(&tree);
        b.dispatch(&mut sim, true, 1);
        let mut vistos: Vec<Entity> = b.ray_sensor_hits_all(olho).iter().map(|h| h.body).collect();
        vistos.sort();
        let mut quer = if tapa { vec![a, bb] } else { vec![a, bb, d] };
        quer.sort();
        assert_eq!(vistos, quer, "walls_block = {tapa}");
        assert_eq!(b.ray_enters().len(), quer.len(), "um On Enter por objecto");
        assert_eq!(
            b.ray_sensor_hits().get(&olho).map(|h| h.body),
            Some(bb),
            "o mais perto à cabeça"
        );
        let m = &b.ray_marks()[0];
        assert!(
            (m.feixe.meio_cone - 30f32.to_radians()).abs() < 1.0e-5,
            "o desenho é a fatia"
        );
        assert_eq!(usize::from(m.feixe.n), quer.len(), "um tique por objecto");
    }
}

/// ⭐⭐ **RS-3 — «Aim at» aponta para o alvo na pose VIVA, no mesmo tique**: um corpo a cair é acertado em
/// todos os tiques, onde ele está AGORA. Controlo: sem alvo, o `angle` (para cima) não o vê. ⚠️ A
/// constraint «Look At» roda o objecto DEPOIS da física (um quadro atrasado) — medido no handoff.
///
/// **Mutação que sangra:** a mira ler o `Transform` do alvo (o do quadro anterior) em vez da pose do mundo.
#[test]
fn aim_at_acerta_o_alvo_na_pose_viva_no_mesmo_tique() {
    for mira in [true, false] {
        let mut sim = SimWorld::new();
        let alvo = sim
            .world_mut()
            .spawn((
                Name::new("Target"),
                StableId(77),
                RigidBody::default(),
                Collider {
                    shape: ColliderShape::Ball { radius: 0.1 },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(3.0, 4.0)),
            ))
            .id();
        let olho = raio(
            &mut sim,
            Vec2::ZERO,
            None,
            RaySensor {
                angle: 90.0,
                reach: 20.0,
                aim_at: if mira { 77 } else { 0 },
                ..RaySensor::default()
            },
        );
        let mut b = PhysicsBridge::new();
        // ⚠️ Em saltos de 20 tiques (um quadro lento): o `Transform` só é relido no FIM do dispatch, logo dentro
        // dele fica 20 tiques atrás do corpo — a mira que o lesse apontaria para onde o alvo ESTAVA.
        for t in (20..=120).step_by(20) {
            b.dispatch(&mut sim, true, t);
            let visto = b.ray_sensor_hits().get(&olho).map(|h| h.body);
            assert_eq!(visto, mira.then_some(alvo), "tique {t}, mira = {mira}");
        }
    }
}

/// ⭐⭐ **RS-12 — «Also ignore»: o objecto da lista e a família DEBAIXO dele são transparentes; escolher a
/// forma filha tira o corpo dela.** O «Only see» continua um modo só (a lista soma-se a ele).
///
/// **Mutações que sangram:** a lista só comparar o corpo (o filho da caixa listada é visto) · não subir da
/// forma ao corpo dono.
#[test]
fn also_ignore_tira_o_objecto_a_familia_e_o_corpo_da_forma() {
    let mut sim = SimWorld::new();
    let crate_ = caixa(&mut sim, "Crate", 2.0, 0.25, false);
    sim.world_mut().entity_mut(crate_).insert(StableId(5));
    let tampa = caixa(&mut sim, "Lid", -1.0, 0.25, false);
    sim.world_mut().entity_mut(tampa).insert(ChildOf(crate_));
    let longe = parede(&mut sim, "Far", 4.0);
    let veja = |sim: &mut SimWorld, lista: Vec<u64>| {
        let e = raio(
            sim,
            Vec2::new(-1.0, 0.0),
            None,
            RaySensor {
                angle: 0.0,
                reach: 20.0,
                also_ignore: lista,
                ..RaySensor::default()
            },
        );
        let mut b = PhysicsBridge::new();
        b.dispatch(sim, true, 1);
        let v = b.ray_sensor_hits().get(&e).map(|h| h.body);
        sim.world_mut().despawn(e);
        v
    };
    assert_eq!(
        veja(&mut sim, vec![]),
        Some(tampa),
        "controlo: a tampa (filha, em x = 1) à frente"
    );
    assert_eq!(
        veja(&mut sim, vec![5]),
        Some(longe),
        "a Crate e a tampa debaixo dela saem"
    );
    // A forma filha de um corpo (opção B), escolhida pelo conta-gotas: o corpo dela sai.
    let corpo = sim
        .world_mut()
        .spawn((
            Name::new("Body"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::from_translation(Vec2::new(1.0, 0.0)),
        ))
        .id();
    let forma = sim
        .world_mut()
        .spawn((
            Name::new("Collision Shape"),
            StableId(8),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.1,
                    half_y: 1.0,
                },
                ..Collider::default()
            },
            Transform::default(),
            ChildOf(corpo),
        ))
        .id();
    assert_eq!(
        veja(&mut sim, vec![5]),
        Some(corpo),
        "controlo: o Body à frente"
    );
    assert_eq!(
        veja(&mut sim, vec![5, 8]),
        Some(longe),
        "a forma escolhida tira o corpo dela"
    );
    let _ = forma;
}

/// ⭐⭐ **RS-4 — o acerto diz a FORMA acertada** (a «Collision Shape» filha), cujas tags são o «material»
/// dele. Um corpo de duas formas: a da frente tem a tag «Ice».
///
/// **Mutação que sangra:** `shape` = o corpo (as tags da forma perdem-se).
#[test]
fn o_acerto_diz_a_forma_que_tocou() {
    let mut tree = TagTree::new();
    let gelo = tree.create("Ice").expect("tag");
    let mut sim = SimWorld::new();
    let chao = sim
        .world_mut()
        .spawn((
            Name::new("Ground"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::from_translation(Vec2::new(0.0, -1.0)),
        ))
        .id();
    let forma = |sim: &mut SimWorld, nome: &str, x: f32| {
        sim.world_mut()
            .spawn((
                Name::new(nome),
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.5,
                        half_y: 0.1,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x, 0.0)),
                ChildOf(chao),
            ))
            .id()
    };
    let pedra = forma(&mut sim, "Stone", -1.0);
    let gelada = forma(&mut sim, "Ice Patch", 1.0);
    sim.world_mut()
        .entity_mut(gelada)
        .insert(ph2d_ecs::tags::Tags::from_ids([gelo]));
    for (x, quer) in [(-1.0, pedra), (1.0, gelada)] {
        let e = raio(
            &mut sim,
            Vec2::new(x, 0.0),
            None,
            RaySensor {
                reach: 3.0,
                ..RaySensor::default()
            },
        );
        let mut b = PhysicsBridge::new();
        b.dispatch(&mut sim, true, 1);
        let h = b.ray_sensor_hits().get(&e).copied().expect("o chão");
        assert_eq!(h.body, chao, "o objecto com o corpo");
        assert_eq!(h.shape, quer, "a forma que tocou, em x = {x}");
        sim.world_mut().despawn(e);
    }
}

/// ⭐⭐ **RS-10 — a ordem dos raios é a da identidade**: dois raios criados em ordem TROCADA (com os mesmos
/// `StableId`) dão os mesmos acertos e a mesma sequência de arestas.
///
/// **Mutação que sangra:** tirar o `sort_by_key` dos sensores (a ordem do arquétipo, a de criação).
#[test]
fn dois_raios_em_ordem_trocada_dao_as_mesmas_arestas() {
    let corre = |troca: bool| {
        let mut sim = SimWorld::new();
        bloco(&mut sim, "Wall", (3.0, 0.0), (0.2, 2.0), 1);
        bloco(&mut sim, "Floor", (0.0, -2.0), (5.0, 0.2), 2);
        let mut faz = |nome: &str, id: u64, angle: f32| {
            let e = raio(
                &mut sim,
                Vec2::ZERO,
                None,
                RaySensor {
                    angle,
                    reach: 10.0,
                    ..RaySensor::default()
                },
            );
            sim.world_mut()
                .entity_mut(e)
                .insert((StableId(id), Name::new(nome)));
        };
        if troca {
            faz("Down", 21, -90.0);
            faz("Front", 20, 0.0);
        } else {
            faz("Front", 20, 0.0);
            faz("Down", 21, -90.0);
        }
        let mut b = PhysicsBridge::new();
        b.dispatch(&mut sim, true, 1);
        b.ray_enters()
            .iter()
            .map(|&(olha, viu)| format!("{}→{}", nome(&sim, olha), nome(&sim, viu)))
            .collect::<Vec<_>>()
    };
    let a = corre(false);
    assert_eq!(a, corre(true), "a mesma sequência");
    assert_eq!(a.len(), 2);
}

/// ⭐⭐ **RS-8 — o tiro usa os MESMOS filtros do objecto**: com o filtro de um raio autorado, o
/// `PhysicsBridge::raycast` vê o mesmo que ele, opção a opção (Only tag + Walls block, a lista, o círculo).
///
/// **Mutação que sangra:** o `raycast` passar por outra consulta (sem o `lanca`).
#[test]
fn o_tiro_ve_o_mesmo_que_o_raio_com_o_mesmo_filtro() {
    let mut tree = TagTree::new();
    let inimigo = tree.create("Enemy").expect("tag");
    let filtros = [
        RaySensor::default(),
        RaySensor {
            only_see: SeeFilter::OnlyTag,
            see_tag: inimigo.0,
            ..RaySensor::default()
        },
        RaySensor {
            only_see: SeeFilter::OnlyTag,
            see_tag: inimigo.0,
            walls_block: false,
            ..RaySensor::default()
        },
        RaySensor {
            also_ignore: vec![1],
            ..RaySensor::default()
        },
        RaySensor {
            shape: RayShape::Thick,
            thickness: 0.8,
            all_hits: true,
            ..RaySensor::default()
        },
    ];
    for f in filtros {
        let mut sim = SimWorld::new();
        bloco(&mut sim, "Crate", (2.0, 0.0), (0.25, 0.25), 1);
        let e = bloco(&mut sim, "Enemy", (4.0, 0.0), (0.25, 0.25), 2);
        com_tag(&mut sim, e, 2, inimigo);
        bloco(&mut sim, "Post", (3.0, 0.45), (0.1, 0.1), 3);
        let olho = raio(
            &mut sim,
            Vec2::ZERO,
            None,
            RaySensor {
                angle: 0.0,
                reach: 8.0,
                ..f.clone()
            },
        );
        let mut b = PhysicsBridge::new();
        b.set_tag_tree(&tree);
        b.dispatch(&mut sim, true, 1);
        let do_raio: Vec<Entity> = if f.all_hits {
            b.ray_sensor_hits_all(olho).iter().map(|h| h.body).collect()
        } else {
            b.ray_sensor_hits()
                .get(&olho)
                .map(|h| h.body)
                .into_iter()
                .collect()
        };
        let tiro: Vec<Entity> = b
            .raycast(sim.world(), None, [0.0, 0.0], [8.0, 0.0], &f)
            .iter()
            .map(|h| h.body)
            .collect();
        assert_eq!(tiro, do_raio, "{f:?}");
    }
}
