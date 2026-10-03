//! Os gates da cena `=4` — o vermelho não pisa a lava e salta pelo portal; o cinzento (o controlo)
//! atravessa a lava e perde vida. ⚠️ A régua de «pisou» é a DISTÂNCIA do centro ao rectângulo do rio
//! contra o raio (a quina recuada é redonda).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{HealthNow, PhysicsBridge};

fn pos(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).expect("o corpo");
    [t.translation.x, t.translation.y]
}

fn ao_rio(p: [f32; 2]) -> f32 {
    let (c, h) = Lava::rio();
    let dx = ((p[0] - c[0]).abs() - h[0]).max(0.0);
    let dy = ((p[1] - c[1]).abs() - h[1]).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

/// O que cada um fez em `tiques`: a menor distância ao rio, o maior passo num tique, se chegou ao
/// herói, e a vida no fim.
struct Desfecho {
    min_rio: f32,
    maior_passo: f32,
    chegou: bool,
    vida: f64,
}

fn corre(tiques: u64) -> (Desfecho, Desfecho) {
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 4);
    assert_eq!(m.nivel, 4);
    let l = m.lava.expect("a cena =4 é a lava");
    assert_eq!(m.escolhido, l.vermelho, "o vermelho escolhido");
    let mut bridge = PhysicsBridge::new();
    let mut d = [l.vermelho, l.cinzento].map(|e| {
        (
            e,
            pos(&sim, e),
            Desfecho {
                min_rio: f32::INFINITY,
                maior_passo: 0.0,
                chegou: false,
                vida: f64::from(VIDA),
            },
        )
    });
    for t in 1..=tiques {
        bridge.dispatch(&mut sim, true, t);
        for (e, antes, f) in &mut d {
            let p = pos(&sim, *e);
            f.min_rio = f.min_rio.min(ao_rio(p));
            let passo = ((p[0] - antes[0]).powi(2) + (p[1] - antes[1]).powi(2)).sqrt();
            f.maior_passo = f.maior_passo.max(passo);
            *antes = p;
            f.chegou |= bridge
                .nav_agent(*e)
                .is_some_and(|r| r.status == ph2d_nav::Status::Arrived);
            if let Some(v) = sim.world().get::<HealthNow>(*e) {
                f.vida = v.pontos;
            }
        }
    }
    let [(_, _, a), (_, _, b)] = d;
    (a, b)
}

/// ⭐⭐⭐ **A cena CONTÉM o fenómeno.**
#[test]
fn a_cena_contem_o_fenomeno() {
    let (verm, cinza) = corre(900);
    eprintln!(
        "vermelho: rio {:.3} m, passo {:.2} m, chegou {}, vida {} · cinzento: rio {:.3}, passo {:.2}, chegou {}, vida {}",
        verm.min_rio,
        verm.maior_passo,
        verm.chegou,
        verm.vida,
        cinza.min_rio,
        cinza.maior_passo,
        cinza.chegou,
        cinza.vida
    );
    assert!(
        verm.min_rio >= RAIO_PEQUENO - 0.02,
        "o vermelho chegou a {} m da lava",
        verm.min_rio
    );
    assert!(
        verm.maior_passo > 5.0,
        "o vermelho não saltou pelo portal (maior passo {})",
        verm.maior_passo
    );
    assert!(verm.chegou, "o vermelho não apanhou o herói");
    assert_eq!(verm.vida, f64::from(VIDA), "o vermelho perdeu vida");
    // O CONTROLO: atravessa a lava, não usa o portal, e a lava fere-o.
    assert!(
        cinza.min_rio < RAIO_PEQUENO * 0.5,
        "o cinzento não pisou a lava"
    );
    assert!(
        cinza.maior_passo < 1.0,
        "o cinzento saltou: {}",
        cinza.maior_passo
    );
    assert!(cinza.vida < f64::from(VIDA), "a lava não feriu o cinzento");
    assert!(cinza.chegou, "o cinzento não chegou ao herói");
}

/// ⭐ **Cada inimigo diz «caught you» UMA vez** — os dois encostam-se ao mesmo herói e o que chega
/// primeiro é empurrado pelo outro; medido antes da cura: o cinzento disparava 6 vezes em 15 s.
#[test]
fn cada_inimigo_diz_que_apanhou_uma_vez() {
    let mut sim = SimWorld::new();
    let l = crate::nav_smoke::montar(sim.world_mut(), 4)
        .lava
        .expect("a cena =4");
    let mut bridge = PhysicsBridge::new();
    let tree = ph2d_tags::TagTree::default();
    let mut ouvidos: Vec<Entity> = Vec::new();
    for t in 1..=900 {
        bridge.dispatch(&mut sim, true, t);
        ouvidos.extend(
            bridge
                .signal_events(&sim, &tree)
                .into_iter()
                .filter(|s| s.name == "caught you")
                .map(|s| s.source),
        );
    }
    let conta = |e: Entity| ouvidos.iter().filter(|&&x| x == e).count();
    assert_eq!(
        (conta(l.vermelho), conta(l.cinzento)),
        (1, 1),
        "{ouvidos:?}"
    );
}
