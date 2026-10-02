//! Os gates da cena `=2` — o que o dono vai ver tem de ACONTECER, e o controlo tem de o NÃO fazer.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

fn pos(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).expect("o corpo");
    [t.translation.x, t.translation.y]
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]) * (a[0] - b[0]) + (a[1] - b[1]) * (a[1] - b[1])).sqrt()
}

/// O desfecho de uma faixa: o tique em que todos chegaram, quantos chegaram no fim, e o par mais perto.
struct Faixa {
    todos: Option<u64>,
    no_fim: usize,
    min_par: f32,
}

fn corre(tiques: u64) -> (Faixa, Faixa) {
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 2);
    assert_eq!(m.nivel, 2);
    let p = m.porta.expect("a cena =2 é a porta");
    assert_eq!(m.escolhido, p.vermelhos[0], "o Red 1 escolhido");
    let mut bridge = PhysicsBridge::new();
    let mut faixas = [
        Faixa {
            todos: None,
            no_fim: 0,
            min_par: f32::INFINITY,
        },
        Faixa {
            todos: None,
            no_fim: 0,
            min_par: f32::INFINITY,
        },
    ];
    for t in 1..=tiques {
        bridge.dispatch(&mut sim, true, t);
        for (f, (grupo, alvos)) in faixas
            .iter_mut()
            .zip([(&p.vermelhos, &p.alvos[0]), (&p.cinzentos, &p.alvos[1])])
        {
            let ps: Vec<[f32; 2]> = grupo.iter().map(|&e| pos(&sim, e)).collect();
            for i in 0..8 {
                for j in i + 1..8 {
                    f.min_par = f.min_par.min(dist(ps[i], ps[j]));
                }
            }
            let chegados = ps
                .iter()
                .zip(alvos)
                .filter(|(p, a)| dist(**p, **a) < 0.1)
                .count();
            if chegados == 8 && f.todos.is_none() {
                f.todos = Some(t);
            }
            f.no_fim = chegados;
        }
    }
    let [a, b] = faixas;
    (a, b)
}

/// ⭐⭐⭐ **A cena CONTÉM o fenómeno** — os vermelhos cruzam-se na porta e chegam todos, sem se
/// invadirem; os cinzentos (o mesmo agente sem o desvio) entalam-se.
///
/// ⚠️ Sem a metade do controlo, uma porta larga demais passaria este gate e mostraria ao dono uma
/// faixa de cima que não precisa do desvio para nada.
#[test]
fn a_cena_contem_o_fenomeno() {
    let (cima, baixo) = corre(1200);
    eprintln!(
        "vermelhos: todos no tique {:?}, {} no fim, par mín {:.4} · cinzentos: todos {:?}, {} no fim, par mín {:.4}",
        cima.todos, cima.no_fim, cima.min_par, baixo.todos, baixo.no_fim, baixo.min_par
    );
    assert!(cima.todos.is_some(), "nem todos os vermelhos chegaram");
    assert!(
        cima.min_par >= 2.0 * RAIO - 1e-3,
        "vermelhos invadiram-se: {}",
        cima.min_par
    );
    assert!(
        baixo.todos.is_none(),
        "os cinzentos passaram — o controlo não mostra nada"
    );
    assert!(
        baixo.no_fim <= 4,
        "os cinzentos não se entalaram: {} chegaram",
        baixo.no_fim
    );
}
