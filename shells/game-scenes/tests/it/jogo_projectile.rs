//! A cena 25 (`PH2D_GAME_SMOKE=25`, ficha GEF 06): cinco balas montadas pelas peças do produto, cada uma com UM
//! campo do Projectile Motion mudado — o Play mostra o que esse campo faz.

use ph2d_app_game::smoke::projectile;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, PhysicsBridge, ProjectileMotion, RigidBody};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let e = q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e);
    e.unwrap_or_else(|| panic!("{nome}"))
}

fn onde(sim: &SimWorld, e: Entity) -> Vec2 {
    sim.world().get::<Transform>(e).expect("pose").translation
}

/// O míssil abre selecionado, e cada bala tem corpo Kinematic com forma e o Projectile Motion.
#[test]
fn a_cena_25_monta_as_cinco_balas_pelas_pecas() {
    let mut sim = SimWorld::new();
    let missil = crate::comp::montar(&mut sim, projectile::CENA);
    assert_eq!(por_nome(&mut sim, projectile::MISSIL), missil);
    for (nome, _, _, _) in projectile::BALAS {
        let e = por_nome(&mut sim, nome);
        let w = sim.world();
        let pm = w.get::<ProjectileMotion>(e).expect(nome);
        assert_eq!(pm.initial_speed, projectile::RAPIDEZ, "{nome}");
        let kind = w.get::<RigidBody>(e).map(|r| r.kind);
        assert_eq!(kind, Some(BodyKind::Kinematic), "{nome}");
        assert!(ph2d_physics_ecs::tem_forma(w, e), "{nome} tem forma");
    }
}

/// ⭐ **O Play ensina o que diz**: depois de 3 s de relógio, o Missile (lançado para BAIXO) deu a volta e acabou no
/// Target, Straight parou na parede, Bouncy voltou para trás, Arc caiu longe do Target, Short Range parou a ~5 m.
/// (Mutação: o míssil sem o tecto da rapidez ⇒ o Pull vira poço, ele passa ao lado do Target e reprova.)
#[test]
fn no_play_cada_bala_faz_o_que_o_seu_campo_diz() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, projectile::CENA);
    let balas: Vec<(&str, Entity, Vec2)> = projectile::BALAS
        .iter()
        .map(|(nome, at, _, _)| (*nome, por_nome(&mut sim, nome), Vec2::new(at[0], at[1])))
        .collect();
    let mut bridge = PhysicsBridge::new();
    let mut max_x = [f32::MIN; 5];
    for tick in 1..=180_u64 {
        bridge.dispatch(&mut sim, true, tick);
        for (i, (_, e, _)) in balas.iter().enumerate() {
            max_x[i] = max_x[i].max(onde(&sim, *e).x);
        }
    }
    let acabou: Vec<Entity> = bridge.projectiles_finished().collect();
    for (i, (nome, e, a)) in balas.iter().enumerate() {
        let p = onde(&sim, *e);
        println!(
            "[projectile-25] {nome}: de ({:.2}, {:.2}) a ({:.2}, {:.2}), x máx {:.2}, acabou {}",
            a.x,
            a.y,
            p.x,
            p.y,
            max_x[i],
            acabou.contains(e)
        );
    }
    let bala = |nome: &str| {
        let i = balas.iter().position(|(n, _, _)| *n == nome).expect(nome);
        (balas[i].1, balas[i].2, i)
    };
    let alvo = Vec2::new(projectile::ALVO_EM[0], projectile::ALVO_EM[1]);
    let (missil, _, _) = bala(projectile::MISSIL);
    let d = (onde(&sim, missil) - alvo).length();
    println!("[projectile-25] Missile acaba a {d:.2} m do Target");
    assert!(
        acabou.contains(&missil) && d < 0.7,
        "o Missile acerta o Target: a {d} m"
    );
    let parede = 5.5 - 0.3;
    let (reta, _, _) = bala("Straight");
    assert!(acabou.contains(&reta), "Straight acaba na parede");
    assert!(onde(&sim, reta).x < parede, "Straight fica antes da parede");
    let (bouncy, _, ib) = bala("Bouncy");
    assert!(
        max_x[ib] > 0.0 && onde(&sim, bouncy).x < max_x[ib] - 1.0,
        "Bouncy bateu e voltou: x máx {} e agora {}",
        max_x[ib],
        onde(&sim, bouncy).x
    );
    let (arco, a, _) = bala("Arc");
    assert!(onde(&sim, arco).y < a.y - 1.0, "Arc caiu");
    let longe = (onde(&sim, arco) - alvo).length();
    assert!(longe > 2.0, "o Arc não acerta o Target: a {longe} m");
    let (curta, a, _) = bala("Short Range");
    assert!(acabou.contains(&curta), "Short Range acabou");
    let x = onde(&sim, curta).x;
    assert!(
        (x - (a.x + 5.0)).abs() < 0.5,
        "Short Range parou a 5 m: x = {x}"
    );
}
