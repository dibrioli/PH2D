//! A cena 26 com as melhorias do Ray Sensor (GEF 07 RS): o «Guard Eye» (um leque de 50°, só para o «Player») vê
//! o herói perto da beirada e não no começo; a «Turret» dá um tiro por SCRIPT (`ph2d.raycast`) a cada quadro, pela
//! ponte da física do quadro, e diz «turret_hit» quando o herói passa por baixo e «turret_miss» quando sai.

use ph2d_app_components::script_bridge::{ScriptDoors, frame};
use ph2d_app_game::smoke::ray;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, RaySensor};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let e = q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e);
    e.unwrap_or_else(|| panic!("{nome}"))
}

/// A cena 26 com o herói posto em `x` (de pé no chão).
fn com_o_heroi_em(x: f32) -> (SimWorld, ph2d_tags::TagTree, Entity) {
    let mut sim = SimWorld::new();
    let (_, tree) = crate::comp::montar_com_tags(&mut sim, ray::CENA);
    let heroi = por_nome(&mut sim, ray::HEROI);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(heroi) {
        t.translation = Vec2::new(x, ray::HEROI_EM[1]);
    }
    (sim, tree, heroi)
}

/// ⭐ **O leque do guarda** (RS-2): longe (no começo) não vê; perto da beirada vê o herói — só ele (a tag
/// «Player»), e o mais perto à cabeça. (Mutação: o leque com `cone = 0` ⇒ a linha passa ao lado ⇒ RED.)
#[test]
fn na_cena_26_o_guarda_ve_o_heroi_pelo_leque() {
    for (x, ve) in [(ray::HEROI_EM[0], false), (0.8, true)] {
        let (mut sim, tree, heroi) = com_o_heroi_em(x);
        let olho = por_nome(&mut sim, ray::RAIO_GUARDA);
        let r = sim
            .world()
            .get::<RaySensor>(olho)
            .expect("Guard Eye")
            .clone();
        assert!(
            (r.cone - ray::LEQUE_DO_GUARDA.1).abs() < 1e-6,
            "o leque autorado"
        );
        let mut b = PhysicsBridge::new();
        b.set_tag_tree(&tree);
        for t in 1..=3 {
            b.dispatch(&mut sim, true, t);
        }
        let vistos: Vec<Entity> = b.ray_sensor_hits_all(olho).iter().map(|h| h.body).collect();
        assert_eq!(
            vistos,
            if ve { vec![heroi] } else { vec![] },
            "herói em x = {x}"
        );
    }
}

/// ⭐⭐ **O tiro por script** (RS-8, no laço do produto): a ponte do script recebe a física do quadro; com o herói
/// debaixo da torreta o script emite «turret_hit», e ao tirá-lo de lá «turret_miss». Controlo: sem a física nas
/// portas o script pára com o erro (a porta diz que a cena não tem física). (Mutação: `physics: None` na shell ⇒
/// o smoke não acende; aqui, o controlo.)
#[test]
fn na_cena_26_a_torreta_atira_por_script() {
    let (mut sim, tree, heroi) = com_o_heroi_em(ray::TORRETA_EM[0]);
    let reg = crate::comp::registo();
    let (mut vec, mut mapa) = ph2d_app_components::instance_docs::empty_docs();
    let mut host = ph2d_script::ScriptHost::new().expect("vm");
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    let mut b = PhysicsBridge::new();
    b.set_tag_tree(&tree);
    let mut ouvidos: Vec<String> = Vec::new();
    for n in 1..=40u64 {
        if n == 20 {
            // O herói sai de baixo da torreta, para o começo do chão.
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(heroi) {
                t.translation = Vec2::new(ray::HEROI_EM[0], ray::HEROI_EM[1]);
            }
            b = PhysicsBridge::new();
            b.set_tag_tree(&tree);
        }
        b.dispatch(&mut sim, true, n);
        let mut portas = ScriptDoors {
            registry: &reg,
            tags: &tree,
            docs: ph2d_app_components::instance_docs::OwnedDocs {
                vec_scene: &mut vec,
                vec_entities: &mut mapa,
            },
            tick: n,
            images: ph2d_app_components::script_image::ImageLibrary::none(),
            physics: Some(&b),
        };
        let f = frame(
            &mut host,
            &mut sim,
            &mut drive,
            true,
            1,
            1.0 / 60.0,
            1.0 / 60.0,
            &[],
            &mut portas,
        );
        assert!(f.failed.is_empty(), "quadro {n}: {:?}", f.failed);
        ouvidos.extend(f.emitted.into_iter().map(|(_, s)| s));
        drive.settle();
    }
    assert_eq!(
        ouvidos,
        vec!["turret_hit".to_owned(), "turret_miss".to_owned()]
    );

    let (mut sim, tree, _) = com_o_heroi_em(ray::TORRETA_EM[0]);
    let mut host = ph2d_script::ScriptHost::new().expect("vm");
    let mut portas = ScriptDoors {
        registry: &reg,
        tags: &tree,
        docs: ph2d_app_components::instance_docs::OwnedDocs {
            vec_scene: &mut vec,
            vec_entities: &mut mapa,
        },
        tick: 1,
        images: ph2d_app_components::script_image::ImageLibrary::none(),
        physics: None,
    };
    let f = frame(
        &mut host,
        &mut sim,
        &mut drive,
        true,
        1,
        1.0 / 60.0,
        1.0 / 60.0,
        &[],
        &mut portas,
    );
    assert!(
        f.failed
            .iter()
            .any(|(_, d)| format!("{d:?}").contains("no physics")),
        "sem a física o script diz porquê: {:?}",
        f.failed
    );
}

/// ⭐⭐ **O filtro do script escolhe a forma pelo nome** (dono 15/11): do olho do guarda, a 12° do herói perto da beirada,
/// `ph2d.raycast(.., { shape = "cone", cone = 50° })` vê o herói; o mesmo `cone` SEM `shape` é a linha e não o vê (o
/// número da forma inactiva não conta); `shape = "line"` também não. (Mutação: o `lanca` ler o `cone` sem olhar a forma
/// ⇒ o sem-`shape` vê ⇒ RED.)
#[test]
fn o_filtro_do_script_escolhe_a_forma_pelo_nome() {
    use ph2d_script::SceneDoors;
    let (mut sim, tree, heroi) = com_o_heroi_em(0.8);
    let reg = crate::comp::registo();
    let (mut vec, mut mapa) = ph2d_app_components::instance_docs::empty_docs();
    let mut b = PhysicsBridge::new();
    b.set_tag_tree(&tree);
    b.dispatch(&mut sim, true, 1);
    let (angulo, cone, alcance) = ray::LEQUE_DO_GUARDA;
    // 12° ao lado do herói (que está no eixo do olho): dentro do leque de 50°, fora da linha.
    let r = (angulo - 12.0).to_radians();
    let de = [f64::from(ray::GUARDA_EM[0]), f64::from(ray::GUARDA_EM[1])];
    let ate = [
        de[0] + f64::from(alcance * r.cos()),
        de[1] + f64::from(alcance * r.sin()),
    ];
    let mut portas = ScriptDoors {
        registry: &reg,
        tags: &tree,
        docs: ph2d_app_components::instance_docs::OwnedDocs {
            vec_scene: &mut vec,
            vec_entities: &mut mapa,
        },
        tick: 1,
        images: ph2d_app_components::script_image::ImageLibrary::none(),
        physics: Some(&b),
    };
    for (forma, ve) in [(Some("cone"), true), (None, false), (Some("line"), false)] {
        let mut filtro = serde_json::Map::new();
        filtro.insert("only_see".into(), "only_tag".into());
        filtro.insert("tag".into(), ray::TAG_HEROI.into());
        filtro.insert("cone".into(), f64::from(cone.to_radians()).into());
        if let Some(f) = forma {
            filtro.insert("shape".into(), f.into());
        }
        let hits = portas
            .raycast(&mut sim, None, (de, ate), &filtro)
            .unwrap_or_else(|e| panic!("{forma:?}: {e}"));
        assert_eq!(
            hits.iter().any(|h| h.object == heroi),
            ve,
            "shape {forma:?}: {} acertos",
            hits.len()
        );
    }
}
