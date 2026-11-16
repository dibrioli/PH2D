//! A cena 27 (`PH2D_GAME_SMOKE=27`, ficha GEF 08): dois alvos com vida e Motion Input, a «Lamp» sem vida (o aviso) e
//! oito balas com Damage — o Play tira vida sozinho e o grafo `=134` lê-a pelos nomes «vida» e «boom».

use ph2d_app_game::smoke::motion_input as cena;
use ph2d_ecs::{Entity, MotionInput, MotionSource, Name, SimWorld};
use ph2d_physics_ecs::{
    BodyKind, Damage, Health, HealthNow, OnHit, PhysicsBridge, ProjectileMotion, RigidBody,
};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Vec<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .filter(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .collect()
}

fn um(sim: &mut SimWorld, nome: &str) -> Entity {
    let v = por_nome(sim, nome);
    assert_eq!(v.len(), 1, "{nome}");
    v[0]
}

/// O «Health Dummy» abre escolhido; cada objecto tem a fonte e o nome da tabela; só a «Lamp» não tem vida; cada
/// bala tem corpo Kinematic com forma, o Projectile Motion e o Damage que some ao acertar.
#[test]
fn a_cena_27_monta_os_alvos_a_lampada_e_as_balas() {
    let mut sim = SimWorld::new();
    let aberto = crate::comp::montar(&mut sim, cena::CENA);
    assert_eq!(um(&mut sim, cena::ALVO_VIDA), aberto);
    for (nome, _, fonte, entrada) in cena::OBJETOS {
        let e = um(&mut sim, nome);
        let w = sim.world();
        let mi = w.get::<MotionInput>(e).expect(nome);
        assert_eq!((mi.name.as_str(), mi.source), (entrada, fonte), "{nome}");
        let lampada = nome == cena::LAMPADA;
        assert_eq!(w.get::<Health>(e).is_none(), lampada, "{nome}: vida");
        assert_eq!(ph2d_physics_ecs::tem_forma(w, e), !lampada, "{nome}: forma");
    }
    let fontes: Vec<_> = cena::OBJETOS.iter().map(|o| (o.0, o.2)).collect();
    assert!(fontes.contains(&(cena::ALVO_GOLPE, MotionSource::Damage)));
    let balas = por_nome(&mut sim, "Bullet");
    assert_eq!(balas.len(), 8);
    let w = sim.world();
    for b in balas {
        assert_eq!(
            w.get::<ProjectileMotion>(b).map(|p| p.initial_speed),
            Some(cena::RAPIDEZ)
        );
        let kind = w.get::<RigidBody>(b).map(|r| r.kind);
        assert_eq!(kind, Some(BodyKind::Kinematic));
        assert!(ph2d_physics_ecs::tem_forma(w, b));
        let d = w.get::<Damage>(b).expect("Damage");
        assert_eq!((d.amount, d.on_hit), (cena::GOLPE, OnHit::Vanish));
    }
}

/// ⭐ **O Play tira vida sozinho**: aos 5 s as quatro balas de cada lado acertaram — os dois alvos ficam em 20 %,
/// vivos. (Mutação: a bala sem Damage, ou a rapidez a zero ⇒ a vida fica cheia e RED.)
#[test]
fn no_play_as_balas_tiram_a_vida_dos_dois_alvos() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, cena::CENA);
    let alvos = [
        um(&mut sim, cena::ALVO_VIDA),
        um(&mut sim, cena::ALVO_GOLPE),
    ];
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=300_u64 {
        bridge.dispatch(&mut sim, true, tick);
        // O dreno do Play (`mortes_anunciadas` sobre a cópia): a bala que acertou sai e abre caminho à seguinte.
        let gastas: Vec<Entity> = bridge.damage_spent().to_vec();
        for b in gastas {
            sim.world_mut().despawn(b);
        }
    }
    for a in alvos {
        let w = sim.world();
        let max = w.get::<Health>(a).expect("vida").max;
        let agora = w.get::<HealthNow>(a).expect("vida viva");
        let r = agora.pontos / f64::from(max);
        println!("[motion-input-27] {:?}: {r:.2}", w.get::<Name>(a));
        assert!(!agora.morta && (r - 0.2).abs() < 1e-3, "acaba em 20 %: {r}");
    }
}
