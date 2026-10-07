//! Os gates da cena da navegação — o que o dono vai ver tem de estar lá, e tem de ACONTECER.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_nav::Status;
use ph2d_physics_ecs::PhysicsBridge;

fn monta() -> (SimWorld, Labirinto) {
    let mut sim = SimWorld::new();
    let m = montar(sim.world_mut(), 1);
    assert_eq!(m.nivel, 1);
    assert_eq!(Some(m.escolhido), m.labirinto.as_ref().map(|l| l.roxo));
    (sim, m.labirinto.expect("a cena =1 é o labirinto"))
}

fn nomes(sim: &SimWorld) -> Vec<String> {
    let mut q = sim.world().try_query::<&Name>().expect("nomes");
    q.iter(sim.world()).map(|n| n.as_str().to_owned()).collect()
}

fn pos(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("o corpo");
    (t.translation.x, t.translation.y)
}

/// ⚠️ **O roteiro nomeia as peças, e cada nome é uma afirmação de que a peça está na cena.**
#[test]
fn a_cena_tem_as_pecas_que_o_roteiro_nomeia() {
    let (sim, _) = monta();
    let n = nomes(&sim);
    for peca in [
        "Hero",
        "Chaser",
        "Big Chaser",
        "Control (homing)",
        "Wall Door",
        "Nav Region",
    ] {
        assert!(n.iter().any(|s| s == peca), "falta {peca}: {n:?}");
    }
}

/// O herói anda com as setas e os perseguidores NÃO — senão o teclado e a navegação falavam ao
/// mesmo tempo no mesmo mover.
#[test]
fn so_o_heroi_le_o_teclado() {
    let (sim, m) = monta();
    let w = sim.world();
    assert!(ph2d_physics_ecs::reads_the_keyboard(w, m.heroi));
    assert!(!ph2d_physics_ecs::reads_the_keyboard(w, m.vermelho));
    assert!(!ph2d_physics_ecs::reads_the_keyboard(w, m.roxo));
}

/// ⭐⭐⭐ **A cena CONTÉM o fenómeno** — com o herói parado (ninguém nas setas), corrida pela porta
/// do produto: o vermelho apanha-o pela porta estreita, o roxo pára antes dela, e o cinzento (o
/// controlo, em linha recta) não chega.
///
/// ⚠️ Sem esta metade, um labirinto em que o caminho recto já funcionasse passaria todos os gates
/// de NOMES e mostraria ao dono um vermelho que não precisa de navegação para nada.
#[test]
fn a_cena_contem_o_fenomeno() {
    let (mut sim, m) = monta();
    let mut bridge = PhysicsBridge::new();
    let mut roxo_max_x = f32::MIN;
    let mut vermelho_passou_a_porta = false;
    for t in 1..=900_u64 {
        bridge.dispatch(&mut sim, true, t);
        roxo_max_x = roxo_max_x.max(pos(&sim, m.roxo).0);
        let v = pos(&sim, m.vermelho);
        if v.0 > X_PORTA && v.1 < Y_CHAO + PORTA + 0.5 {
            vermelho_passou_a_porta = true;
        }
    }
    let st = |e| bridge.nav_agent(e).map(|a| a.status);
    assert_eq!(
        st(m.vermelho),
        Some(Status::Arrived),
        "o vermelho não apanhou o herói"
    );
    assert!(
        vermelho_passou_a_porta,
        "o vermelho chegou sem passar pela porta estreita"
    );
    assert!(
        roxo_max_x < X_PORTA,
        "o roxo passou uma porta mais estreita que ele (x = {roxo_max_x})"
    );
    assert_eq!(st(m.roxo), Some(Status::MovingPartial));
    let (hx, hy) = pos(&sim, m.heroi);
    let (cx, cy) = pos(&sim, m.controlo);
    assert!(
        (cx - hx).hypot(cy - hy) > 2.0,
        "o controlo em linha recta chegou ao herói — a cena não precisa de navegação"
    );
}

/// ⭐ **O `CENAS` é CONTADO do roteador** — cada nível que o [`montar`] serve devolve-se a si próprio, e
/// o seguinte cai no `=1` (a `=3` monta pela `montar_cena`, com o contexto vectorial). Um `CENAS` a
/// menos esconderia a última cena do censo do desenho e da família.
#[test]
fn o_cenas_conta_os_niveis_do_roteador() {
    for n in (1..=CENAS).filter(|&n| n != 3) {
        let mut sim = SimWorld::new();
        assert_eq!(montar(sim.world_mut(), n).nivel, n, "o nível {n}");
    }
    let mut sim = SimWorld::new();
    assert_eq!(montar(sim.world_mut(), CENAS + 1).nivel, 1, "há um nível depois de CENAS");
}
