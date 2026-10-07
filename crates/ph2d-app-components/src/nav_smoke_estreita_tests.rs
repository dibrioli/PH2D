//! Os gates da cena `=7` — pela cena REAL (`nav_smoke::montar(…, 7)`), com o CONTROLO: as duas estradas a
//! `Cost 1` (o vermelho da esquerda corta a direito como o da direita).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::nav_edits::CostAreaQueixa;
use ph2d_physics_ecs::{NavCostArea, NavCostAreaNow, PhysicsBridge};

/// O que cada corredor fez: os tiques com o corpo INTEIRO na estrada, o maior desvio do `x` de partida,
/// e quem chegou.
struct Desfecho {
    na_estrada: [u32; 2],
    desvio: [f32; 2],
    chegaram: [bool; 2],
    sim: SimWorld,
    pecas: Estreita,
}

fn corre(controlo: bool) -> Desfecho {
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 7);
    assert_eq!(m.nivel, 7);
    assert_eq!(
        m.secao_do_roteiro(),
        ph2d_editor_core::ids::INSP_LIVE_NAV_COST_AREA_SECTION,
        "a shell abre a secção da área (a queixa)"
    );
    let pecas = {
        let mut q = sim
            .world_mut()
            .try_query::<(Entity, &Name)>()
            .expect("nomes");
        let mut por_nome = |n: &str| {
            q.iter(sim.world())
                .find(|(_, x)| x.as_str() == n)
                .map(|(e, _)| e)
                .expect(n)
        };
        Estreita {
            estreita: por_nome("Narrow Road"),
            larga: por_nome("Wide Road"),
            corredores: [por_nome("Runner Wide"), por_nome("Runner Narrow")],
        }
    };
    assert_eq!(m.escolhido, pecas.estreita, "a estrada estreita escolhida");
    if controlo {
        let mut q = sim.world_mut().query::<&mut NavCostArea>();
        for mut a in q.iter_mut(sim.world_mut()) {
            a.cost = 1.0;
        }
    }
    let mut bridge = PhysicsBridge::new();
    let mut d = Desfecho {
        na_estrada: [0; 2],
        desvio: [0.0; 2],
        chegaram: [false; 2],
        sim: SimWorld::new(),
        pecas,
    };
    let x_de = |k: usize| {
        let [x0, x1] = corredor(k);
        0.5 * (x0 + x1)
    };
    for t in 1..=1500u64 {
        bridge.dispatch(&mut sim, true, t);
        for k in 0..2 {
            let tr = sim
                .world()
                .get::<Transform>(d.pecas.corredores[k])
                .expect("o corpo");
            let q = [tr.translation.x, tr.translation.y];
            let fundo = |(c, h): ([f32; 2], [f32; 2])| {
                (h[0] - (q[0] - c[0]).abs()).min(h[1] - (q[1] - c[1]).abs())
            };
            d.na_estrada[k] += u32::from(
                estrada_rects(k, largura(k))
                    .into_iter()
                    .any(|r| fundo(r) >= RAIO - 0.02),
            );
            d.desvio[k] = d.desvio[k].max((q[0] - x_de(k)).abs());
            d.chegaram[k] |= bridge
                .nav_agent(d.pecas.corredores[k])
                .is_some_and(|r| r.status == ph2d_nav::Status::Arrived);
        }
    }
    d.sim = sim;
    d
}

/// ⭐⭐⭐ **A larga anda-se, a estreita não faz nada** — e o CONTROLO (as duas a `Cost 1`) prova que é a
/// estrada que leva o da esquerda: sem ela, ele vai a direito como o da direita.
#[test]
fn a_larga_anda_se_a_estreita_nao_faz_nada() {
    let (p, c) = (corre(false), corre(true));
    for (nome, d) in [("produto", &p), ("controlo", &c)] {
        eprintln!(
            "{nome}: na estrada {:?} tiques · desvio {:.2?} · chegaram {:?}",
            d.na_estrada, d.desvio, d.chegaram
        );
    }
    assert_eq!(p.chegaram, [true; 2], "nem todos chegaram à bandeira");
    assert!(
        p.na_estrada[0] >= 300 && p.desvio[0] > 2.0,
        "o da esquerda não seguiu a estrada larga ({} tiques, desvio {})",
        p.na_estrada[0],
        p.desvio[0]
    );
    assert!(
        p.na_estrada[1] == 0 && p.desvio[1] < 0.3,
        "o da direita foi à estrada estreita ({} tiques, desvio {})",
        p.na_estrada[1],
        p.desvio[1]
    );
    assert!(
        c.desvio[0] < 0.3 && c.chegaram[0],
        "o CONTROLO: sem o custo o da esquerda vai a direito ({})",
        c.desvio[0]
    );
}

/// ⭐⭐ **A estreita queixa-se no Inspector, a larga não** — o raio que a ponte publica chega ao
/// instantâneo da secção, e a queixa é a nova.
#[test]
fn a_estreita_queixa_se_e_a_larga_nao() {
    let d = corre(false);
    let w = d.sim.world();
    let raio = w
        .get::<NavCostAreaNow>(d.pecas.estreita)
        .map(|n| n.too_narrow_for);
    assert_eq!(raio, Some(RAIO), "a estreita leva o raio do corpo");
    assert!(
        w.get::<NavCostAreaNow>(d.pecas.larga).is_none(),
        "a larga cabe"
    );
    let queixa = |e: Entity| {
        crate::nav_inspector::build_info(w, e.to_bits(), 1, true)
            .and_then(|i| i.cost_area)
            .expect("a secção da área")
            .queixa()
    };
    assert_eq!(
        queixa(d.pecas.estreita),
        Some(CostAreaQueixa::MaisEstreitaQueOCorpo)
    );
    assert_eq!(queixa(d.pecas.larga), None);
}

/// As peças que o roteiro e o tutorial nomeiam.
#[test]
fn a_cena_tem_as_pecas_que_o_roteiro_nomeia() {
    let mut sim = SimWorld::new();
    let m = montar(sim.world_mut());
    let w = sim.world();
    for (e, nome) in [(m.estreita, "Narrow Road"), (m.larga, "Wide Road")] {
        assert_eq!(w.get::<Name>(e).map(Name::as_str), Some(nome));
        assert_eq!(w.get::<NavCostArea>(e).map(|a| a.cost), Some(CUSTO_ESTRADA));
    }
    assert!(
        ESTREITA < 2.0 * RAIO,
        "a estreita é mais estreita que o corpo"
    );
}
