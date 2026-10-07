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

/// O que cada corredor fez: os tiques com o corpo INTEIRO na estrada e o `x` mais à esquerda do 1, o
/// mínimo do centro ao rio, ao canteiro e à BORDA da luz, e quem chegou.
struct Desfecho {
    na_estrada: u32,
    mais_a_esquerda: f32,
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
    assert_eq!(m.escolhido, u.estrada, "a estrada escolhida");
    if controlo {
        let estrada: Vec<Entity> = {
            let mut q = sim
                .world_mut()
                .try_query::<(Entity, &Name)>()
                .expect("nomes");
            q.iter(sim.world())
                .filter(|(_, n)| n.as_str().starts_with("Road"))
                .map(|(e, _)| e)
                .collect()
        };
        assert_eq!(estrada.len(), 3, "as três tiras da estrada");
        for e in estrada.into_iter().chain([u.rio, u.jardim, u.luz]) {
            if let Some(mut a) = sim.world_mut().get_mut::<NavCostArea>(e) {
                (a.cost, a.forbidden) = (1.0, false);
            }
        }
    }
    let mut bridge = PhysicsBridge::new();
    let mut d = Desfecho {
        na_estrada: 0,
        mais_a_esquerda: f32::MAX,
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
        let q0 = p(0);
        d.mais_a_esquerda = d.mais_a_esquerda.min(q0[0]);
        let fundo = |(c, h): ([f32; 2], [f32; 2])| {
            (h[0] - (q0[0] - c[0]).abs()).min(h[1] - (q0[1] - c[1]).abs())
        };
        d.na_estrada += u32::from(
            Usos::estrada_rects()
                .into_iter()
                .any(|r| fundo(r) >= RAIO - 0.02),
        );
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

/// ⭐⭐⭐ **A cena CONTÉM os quatro fenómenos**, e o CONTROLO prova que são as áreas que os fazem. O 1 anda
/// pela estrada com o corpo INTEIRO nela e vai à tira da esquerda; os outros três nunca põem o CORPO na
/// área deles (`2 cm` de folga). Com tudo a `Cost 1` o 1 corta a direito pelo meio e o centro dos outros
/// atravessa as três áreas.
#[test]
fn cada_uso_muda_o_caminho_e_o_controlo_corta_a_direito() {
    let (p, c) = (corre(false), corre(true));
    for (nome, d) in [("produto", &p), ("controlo", &c)] {
        eprintln!(
            "{nome}: estrada {} tiques (x mín {:.2}) · rio {:.2} · canteiro {:.2} · luz {:.2} · chegaram {:?}",
            d.na_estrada, d.mais_a_esquerda, d.ao_rio, d.ao_jardim, d.a_luz, d.chegaram
        );
    }
    assert_eq!(p.chegaram, [true; 4], "nem todos chegaram à bandeira");
    let [x0, _] = corredor(0);
    assert!(
        p.mais_a_esquerda < x0 + ESTRADA && p.na_estrada >= 300,
        "o corredor 1 não andou POR CIMA da estrada ({} tiques, x mín {})",
        p.na_estrada,
        p.mais_a_esquerda
    );
    let folga = RAIO - 0.02;
    assert!(p.ao_rio >= folga, "o corredor 2 molhou-se ({})", p.ao_rio);
    assert!(
        p.ao_jardim >= folga,
        "o corredor 3 pisou o canteiro ({})",
        p.ao_jardim
    );
    assert!(p.a_luz >= folga, "o corredor 4 entrou na luz ({})", p.a_luz);
    assert!(
        c.mais_a_esquerda > meio(0) - 0.1,
        "o CONTROLO foi pela estrada (x mín {})",
        c.mais_a_esquerda
    );
    assert!(
        [c.ao_rio, c.ao_jardim, c.a_luz] == [0.0; 3],
        "o CONTROLO não atravessa as três áreas — a cena não as mede"
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
    assert_eq!(area(u.estrada).map(|a| a.cost), Some(CUSTO_ESTRADA));
    assert_eq!(area(u.rio).map(|a| a.cost), Some(CUSTO_RIO));
    assert_eq!(area(u.jardim).map(|a| a.forbidden), Some(true));
    assert_eq!(area(u.luz).map(|a| a.cost), Some(CUSTO_LUZ));
    let mut q = sim.world_mut().try_query::<&Name>().expect("nomes");
    let n: Vec<String> = q.iter(sim.world()).map(|n| n.as_str().to_owned()).collect();
    for peca in [
        "Road",
        "Road Bottom",
        "Road Top",
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

/// ⭐ **O passo do roteiro** — só a `Road` escolhida (a tira comprida) a `Cost 1`: o corredor 1 corta a
/// direito pelo meio (a volta pela tira já não paga).
#[test]
fn com_a_road_escolhida_a_1_o_corredor_1_corta_a_direito() {
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 6);
    let u = m.usos.expect("os usos");
    if let Some(mut a) = sim.world_mut().get_mut::<NavCostArea>(m.escolhido) {
        a.cost = 1.0;
    }
    let mut bridge = PhysicsBridge::new();
    let mut mais_a_esquerda = f32::MAX;
    let mut chegou = false;
    for t in 1..=1500u64 {
        bridge.dispatch(&mut sim, true, t);
        let p = sim
            .world()
            .get::<Transform>(u.corredores[0])
            .expect("o corpo")
            .translation;
        mais_a_esquerda = mais_a_esquerda.min(p.x);
        chegou |= bridge
            .nav_agent(u.corredores[0])
            .is_some_and(|r| r.status == ph2d_nav::Status::Arrived);
    }
    assert!(chegou, "o corredor 1 não chegou");
    assert!(
        mais_a_esquerda > meio(0) - 0.1,
        "foi pela estrada mesmo com a Road a 1 (x mín {mais_a_esquerda})"
    );
}
