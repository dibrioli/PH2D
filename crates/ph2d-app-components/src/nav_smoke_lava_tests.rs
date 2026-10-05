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

/// ⭐⭐ (report do dono, 05/10) **Dois inimigos IGUAIS pelo mesmo portal não se prendem um ao outro.**
/// O dono ligou o *Avoid Harm* também no cinzento: os dois foram ao portal da esquerda e ficaram
/// parados, encostados, um de cada lado do ponto da entrada — o teletransporte disparava só no PONTO,
/// e para lá chegar cada um teria de se sobrepor ao outro (medido: `5` de `20` nascimentos presos para
/// sempre, `0,31` e `0,47 m` do ponto, a `0,70 m` um do outro = os dois raios). O cinzento a nascer em
/// `40` sítios à esquerda — entre eles `(-2, -1,5)`, o que ainda prende com meio raio de alcance; a
/// fixtura exige casos em que os DOIS saltam pelo portal.
#[test]
fn dois_inimigos_iguais_pelo_mesmo_portal_nao_se_prendem() {
    let mut pelos_dois = 0;
    for i in 0..8 {
        for j in 0..5 {
            let (x, y) = (-4.5 + 0.5 * i as f32, -1.5 + 0.5 * j as f32);
            let mut sim = SimWorld::new();
            let m = crate::nav_smoke::montar(sim.world_mut(), 4);
            let l = m.lava.expect("a lava");
            if let Some(mut a) = sim.world_mut().get_mut::<NavAgent>(l.cinzento) {
                a.avoid_harm = true;
            }
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(l.cinzento) {
                t.translation = Vec2::new(x, y);
            }
            let mut bridge = PhysicsBridge::new();
            let quem = [l.vermelho, l.cinzento];
            let mut antes = quem.map(|e| pos(&sim, e));
            let (mut saltou, mut chegou) = ([false; 2], [false; 2]);
            for t in 1..=900u64 {
                bridge.dispatch(&mut sim, true, t);
                for (k, &e) in quem.iter().enumerate() {
                    let p = pos(&sim, e);
                    saltou[k] |= (p[0] - antes[k][0]).abs() > 3.0;
                    antes[k] = p;
                    chegou[k] |= bridge
                        .nav_agent(e)
                        .is_some_and(|r| r.status == ph2d_nav::Status::Arrived);
                }
            }
            pelos_dois += usize::from(saltou == [true, true]);
            assert_eq!(
                chegou,
                [true, true],
                "o cinzento a nascer em ({x}, {y}): saltaram {saltou:?}"
            );
        }
    }
    assert!(
        pelos_dois >= 8,
        "a fixtura: os dois pelo portal em {pelos_dois} de 40"
    );
}

/// ⭐⭐ (report do dono, 05/10, 2.º) **Ninguém sai do portal DENTRO de outro corpo** — *«se fizerem o
/// teletransporte e caírem em cima do player, travam o player»*: o salto punha o corpo no ponto da saída
/// sem perguntar quem lá estava (medido: o vermelho aterrava a `2 cm` do centro do herói parado na
/// saída). Agora quem chega à entrada ESPERA nela enquanto a saída está ocupada. O herói parado na saída
/// `3 s` (os dois inimigos, com *Avoid Harm*, vão pelo portal); depois anda para cima e liberta-a.
#[test]
fn ninguem_sai_do_portal_dentro_de_outro_corpo() {
    use ph2d_physics_ecs::PlayerInput;
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 4);
    let l = m.lava.expect("a lava");
    if let Some(mut a) = sim.world_mut().get_mut::<NavAgent>(l.cinzento) {
        a.avoid_harm = true;
    }
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(l.cinzento) {
        t.translation = Vec2::new(-3.0, -1.0);
    }
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(l.heroi) {
        t.translation = Vec2::new(PORTAL_B[0], PORTAL_B[1]);
    }
    let mut bridge = PhysicsBridge::new();
    let quem = [l.vermelho, l.cinzento, l.heroi];
    let raio = [RAIO_PEQUENO, RAIO_PEQUENO, RAIO_HEROI];
    let mut antes = quem.map(|e| pos(&sim, e));
    let (mut saltos, mut a_espera, mut presos) = (0, 0, 0);
    for t in 1..=600u64 {
        if t > 180 {
            bridge.set_player_input(
                l.heroi,
                PlayerInput {
                    drive_y: 1.0,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, t);
        // Esperar na entrada não é estar PRESO (o sinal `On Stuck` calado).
        presos += bridge
            .nav_events()
            .iter()
            .filter(|ev| ev.kind == ph2d_nav::Event::Stuck && t <= 180)
            .count();
        let agora = quem.map(|e| pos(&sim, e));
        for k in 0..2 {
            if (agora[k][0] - antes[k][0]).abs() > 3.0 {
                saltos += 1;
                for o in 0..3 {
                    if o == k {
                        continue;
                    }
                    let d = ((agora[k][0] - agora[o][0]).powi(2)
                        + (agora[k][1] - agora[o][1]).powi(2))
                    .sqrt();
                    assert!(
                        d >= raio[k] + raio[o] - 0.01,
                        "no tique {t} o {k}.º saiu do portal a {d:.2} m do {o}.º"
                    );
                }
            }
            // À espera na entrada: perto do portal A, parado.
            let pa =
                ((agora[k][0] - PORTAL_A[0]).powi(2) + (agora[k][1] - PORTAL_A[1]).powi(2)).sqrt();
            let passo =
                ((agora[k][0] - antes[k][0]).powi(2) + (agora[k][1] - antes[k][1]).powi(2)).sqrt();
            a_espera += usize::from(t <= 180 && pa < 0.5 && passo < 1e-4);
        }
        antes = agora;
    }
    assert_eq!(
        saltos, 2,
        "os dois acabam por passar quando a saída fica livre"
    );
    assert_eq!(presos, 0, "à espera na entrada ninguém diz «preso»");
    assert!(
        a_espera >= 30,
        "a fixtura: alguém esperou na entrada com a saída ocupada ({a_espera} tiques)"
    );
}
