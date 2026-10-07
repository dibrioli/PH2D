//! Os gates da cena `=6` — pela cena REAL (`nav_smoke::montar(…, 6)`), cada uso com o CONTROLO: a mesma
//! cena com todas as áreas a `Cost 1` e nenhuma proibida (o caminho corta a direito em todos).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

fn ao_rect(p: [f32; 2], (c, h): ([f32; 2], [f32; 2])) -> f32 {
    let dx = ((p[0] - c[0]).abs() - h[0]).max(0.0);
    let dy = ((p[1] - c[1]).abs() - h[1]).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

/// O que cada corredor fez: o mínimo do centro ao terreno, ao rio, ao canteiro e à BORDA da luz, e quem
/// chegou.
struct Desfecho {
    ao_terreno: f32,
    ao_rio: f32,
    ao_jardim: f32,
    a_luz: f32,
    chegaram: [bool; 4],
}

fn corre(controlo: bool) -> Desfecho {
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 6);
    assert_eq!(m.nivel, 6);
    let u = m.usos.expect("a cena =6 são os usos");
    assert_eq!(m.escolhido, u.terreno, "o terreno escolhido");
    if controlo {
        for e in [u.terreno, u.rio, u.jardim, u.luz] {
            if let Some(mut a) = sim.world_mut().get_mut::<NavCostArea>(e) {
                (a.cost, a.forbidden) = (1.0, false);
            }
        }
    }
    let mut bridge = PhysicsBridge::new();
    let mut d = Desfecho {
        ao_terreno: f32::MAX,
        ao_rio: f32::MAX,
        ao_jardim: f32::MAX,
        a_luz: f32::MAX,
        chegaram: [false; 4],
    };
    let lc = Usos::luz_centro();
    for t in 1..=1500u64 {
        bridge.dispatch(&mut sim, true, t);
        let p = |k: usize| {
            let tr = sim
                .world()
                .get::<Transform>(u.corredores[k])
                .expect("o corpo");
            [tr.translation.x, tr.translation.y]
        };
        d.ao_terreno = d.ao_terreno.min(ao_rect(p(0), Usos::terreno_rect()));
        d.ao_rio = d.ao_rio.min(ao_rect(p(1), Usos::rio_rect()));
        d.ao_jardim = d.ao_jardim.min(ao_rect(p(2), Usos::jardim_rect()));
        let q = p(3);
        d.a_luz = d
            .a_luz
            .min(((q[0] - lc[0]).hypot(q[1] - lc[1]) - RAIO_DA_LUZ).max(0.0));
        for (k, c) in d.chegaram.iter_mut().enumerate() {
            *c |= bridge
                .nav_agent(u.corredores[k])
                .is_some_and(|r| r.status == ph2d_nav::Status::Arrived);
        }
    }
    d
}

/// ⭐⭐⭐ **A cena CONTÉM os quatro fenómenos**, e o CONTROLO prova que são as áreas que os fazem: o CORPO
/// de cada corredor nunca entra na área dele (`2 cm` de folga); com tudo a `Cost 1`, o centro atravessa as
/// quatro.
#[test]
fn cada_uso_muda_o_caminho_e_o_controlo_corta_a_direito() {
    let (p, c) = (corre(false), corre(true));
    eprintln!(
        "produto: terreno {:.2} · rio {:.2} · canteiro {:.2} · luz {:.2} · chegaram {:?}",
        p.ao_terreno, p.ao_rio, p.ao_jardim, p.a_luz, p.chegaram
    );
    eprintln!(
        "controlo: terreno {:.2} · rio {:.2} · canteiro {:.2} · luz {:.2} · chegaram {:?}",
        c.ao_terreno, c.ao_rio, c.ao_jardim, c.a_luz, c.chegaram
    );
    assert_eq!(p.chegaram, [true; 4], "nem todos chegaram à bandeira");
    let folga = RAIO - 0.02;
    assert!(
        p.ao_terreno >= folga,
        "o corredor 1 pisou as pedras ({})",
        p.ao_terreno
    );
    assert!(p.ao_rio >= folga, "o corredor 2 molhou-se ({})", p.ao_rio);
    assert!(
        p.ao_jardim >= folga,
        "o corredor 3 pisou o canteiro ({})",
        p.ao_jardim
    );
    assert!(p.a_luz >= folga, "o corredor 4 entrou na luz ({})", p.a_luz);
    assert!(
        [c.ao_terreno, c.ao_rio, c.ao_jardim, c.a_luz] == [0.0; 4],
        "o CONTROLO não atravessa as quatro áreas — a cena não as mede"
    );
}

/// O roteiro nomeia as peças, e a shell abre a secção da área escolhida.
#[test]
fn a_cena_tem_as_pecas_que_o_roteiro_nomeia() {
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 6);
    assert_eq!(
        m.secao_do_roteiro(),
        ph2d_editor_core::ids::INSP_LIVE_NAV_COST_AREA_SECTION
    );
    let u = m.usos.expect("os usos");
    let area = |e| sim.world().get::<NavCostArea>(e).copied();
    assert_eq!(area(u.terreno).map(|a| a.cost), Some(CUSTO_TERRENO));
    assert_eq!(area(u.rio).map(|a| a.cost), Some(CUSTO_RIO));
    assert_eq!(area(u.jardim).map(|a| a.forbidden), Some(true));
    assert_eq!(area(u.luz).map(|a| a.cost), Some(CUSTO_LUZ));
    let mut q = sim.world_mut().try_query::<&Name>().expect("nomes");
    let n: Vec<String> = q.iter(sim.world()).map(|n| n.as_str().to_owned()).collect();
    for peca in [
        "Rough Ground",
        "Road",
        "River",
        "Bridge",
        "Garden",
        "Guard Light",
        "Runner Road",
        "Flag Light",
    ] {
        assert!(n.iter().any(|s| s == peca), "falta {peca}: {n:?}");
    }
}
