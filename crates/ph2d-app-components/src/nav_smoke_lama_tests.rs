//! Os gates da cena `=5` — pela cena REAL (`nav_smoke::montar(…, 5)`): os corredores da lama leve
//! atravessam-na, nenhum dos da pesada a atravessa, e o CONTROLO (a mesma cena com as duas a custo `1`)
//! cruza as duas; e o passo do roteiro (o `Cost` da pesada a `2` com eles a caminho).
//! ⚠️ «Atravessou» = o CENTRO esteve dentro do rectângulo; «não pisou» = o centro nunca chegou a menos de
//! um raio dele (o CORPO fora da lama, `2 cm` de folga). ⛔ Uma 1.ª versão descontava a quina do lado da
//! passagem («contornar a quina não é atravessar») — e foi na quina que o dono viu o R2 entrar (o centro
//! a `0,00` da lama): a régua escondia o defeito que devia apanhar.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// A distância do centro ao rectângulo da faixa (`0` = dentro).
fn a_faixa(p: [f32; 2], lado: f32) -> f32 {
    let (c, h) = Lama::faixa(lado);
    let dx = ((p[0] - c[0]).abs() - h[0]).max(0.0);
    let dy = ((p[1] - c[1]).abs() - h[1]).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

fn dentro(p: [f32; 2], lado: f32) -> bool {
    a_faixa(p, lado) == 0.0
}

/// O corpo entrou na faixa (o centro a menos de um raio dela, com `2 cm` de folga).
fn pisou_(p: [f32; 2], lado: f32) -> bool {
    a_faixa(p, lado) < RAIO_PEQUENO - 0.02
}

/// Por pista: quantos ATRAVESSARAM a faixa dela (o centro dentro), quantos a PISARAM (o corpo dentro), e
/// quantos chegaram à bandeira.
fn corre(custos: Option<f32>) -> ([usize; 2], [usize; 2], [usize; 2]) {
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 5);
    assert_eq!(m.nivel, 5);
    let l = m.lama.expect("a cena =5 é a lama");
    assert_eq!(m.escolhido, l.pesada, "a lama pesada escolhida");
    if let Some(c) = custos {
        for e in [l.leve, l.pesada] {
            if let Some(mut a) = sim.world_mut().get_mut::<NavCostArea>(e) {
                a.cost = c;
            }
        }
    }
    let mut bridge = PhysicsBridge::new();
    let pistas = [(l.na_leve, -1.0), (l.na_pesada, 1.0)];
    let (mut atravessou, mut pisou, mut chegou) =
        ([[false; 3]; 2], [[false; 3]; 2], [[false; 3]; 2]);
    for t in 1..=900u64 {
        bridge.dispatch(&mut sim, true, t);
        for (k, (quem, lado)) in pistas.iter().enumerate() {
            for (i, &e) in quem.iter().enumerate() {
                let tr = sim.world().get::<Transform>(e).expect("o corpo");
                let p = [tr.translation.x, tr.translation.y];
                atravessou[k][i] |= dentro(p, *lado);
                pisou[k][i] |= pisou_(p, *lado);
                chegou[k][i] |= bridge
                    .nav_agent(e)
                    .is_some_and(|r| r.status == ph2d_nav::Status::Arrived);
            }
        }
    }
    let conta = |v: [bool; 3]| v.iter().filter(|&&b| b).count();
    (
        [conta(atravessou[0]), conta(atravessou[1])],
        [conta(pisou[0]), conta(pisou[1])],
        [conta(chegou[0]), conta(chegou[1])],
    )
}

/// ⭐⭐⭐ **A cena CONTÉM o fenómeno** — e o CONTROLO prova que é a lama que o faz.
#[test]
fn a_leve_atravessa_se_a_pesada_contorna_se() {
    let (atravessaram, pisaram, chegaram) = corre(None);
    let (atravessaram_ctl, _, _) = corre(Some(1.0));
    eprintln!(
        "produto: atravessaram {atravessaram:?}, pisaram {pisaram:?}, chegaram {chegaram:?} · controlo: atravessaram {atravessaram_ctl:?}"
    );
    assert_eq!(atravessaram[0], 3, "nem todos atravessaram a lama leve");
    assert_eq!(pisaram[1], 0, "um corpo entrou na lama pesada");
    assert_eq!(chegaram, [3, 3], "nem todos chegaram à bandeira");
    assert!(
        atravessaram_ctl[0] >= 1 && atravessaram_ctl[1] >= 1,
        "o CONTROLO (sem custos) não cruza as duas faixas — a cena não mede a lama: {atravessaram_ctl:?}"
    );
}

/// O roteiro nomeia as peças.
#[test]
fn a_cena_tem_as_pecas_que_o_roteiro_nomeia() {
    let mut sim = SimWorld::new();
    let m = crate::nav_smoke::montar(sim.world_mut(), 5);
    // O passo do roteiro é o `Cost` da lama escolhida: a shell abre ESSA secção.
    assert_eq!(
        m.secao_do_roteiro(),
        ph2d_editor_core::ids::INSP_LIVE_NAV_COST_AREA_SECTION
    );
    let l = m.lama.expect("a lama");
    let custo = |e| sim.world().get::<NavCostArea>(e).map(|a| a.cost);
    assert_eq!(custo(l.leve), Some(CUSTO_LEVE));
    assert_eq!(custo(l.pesada), Some(CUSTO_PESADA));
    let mut q = sim.world_mut().try_query::<&Name>().expect("nomes");
    let n: Vec<String> = q.iter(sim.world()).map(|n| n.as_str().to_owned()).collect();
    for peca in [
        "Light Mud",
        "Heavy Mud",
        "Wall Middle",
        "Runner L1",
        "Runner R3",
        "Flag R3",
    ] {
        assert!(n.iter().any(|s| s == peca), "falta {peca}: {n:?}");
    }
}

/// ⭐⭐ **O passo do roteiro** — com os da direita a caminho da passagem, o `Cost` da lama pesada passa
/// a `2` (o dono pára o relógio, muda-o no Inspector e solta-o): eles viram e cortam A DIREITO pela lama,
/// e chegam todos. O CONTROLO é o gate de cima (sem a mudança, nenhum a pisa).
#[test]
fn com_o_cost_em_2_os_da_direita_cortam_pela_lama() {
    let mut sim = SimWorld::new();
    let l = crate::nav_smoke::montar(sim.world_mut(), 5)
        .lama
        .expect("a lama");
    let mut bridge = PhysicsBridge::new();
    let (mut pisaram, mut chegaram) = ([false; 3], [false; 3]);
    let mut antes = [0.0f32; 3];
    for t in 1..=900u64 {
        if t == 60 {
            for (i, &e) in l.na_pesada.iter().enumerate() {
                antes[i] = sim.world().get::<Transform>(e).expect("c").translation.y;
            }
            if let Some(mut a) = sim.world_mut().get_mut::<NavCostArea>(l.pesada) {
                a.cost = 2.0;
            }
        }
        bridge.dispatch(&mut sim, true, t);
        for (i, &e) in l.na_pesada.iter().enumerate() {
            let p = sim.world().get::<Transform>(e).expect("c").translation;
            pisaram[i] |= t > 60 && dentro([p.x, p.y], 1.0);
            chegaram[i] |= bridge
                .nav_agent(e)
                .is_some_and(|r| r.status == ph2d_nav::Status::Arrived);
        }
    }
    eprintln!(
        "Cost 2 no tique 60 (y deles então: {antes:?}): pisaram {pisaram:?}, chegaram {chegaram:?}"
    );
    assert!(
        antes.iter().all(|&y| y < LAMA_Y0),
        "a fixtura: no tique da mudança estão todos abaixo da lama"
    );
    assert!(
        pisaram.iter().any(|&b| b),
        "com o Cost em 2 ninguém cortou pela lama"
    );
    assert_eq!(chegaram, [true; 3], "nem todos chegaram à bandeira");
}

/// ⭐⭐ (report do dono, 06/10: *«o R2 tem uma movimentação bizarra, entrando na quina da lama»*) **Nenhum
/// corredor volta para trás atrás de um canto** — empurrado `2–5 cm` pelos vizinhos na passagem, o R2 não
/// acertava no passo do executor (`1,3 cm` a `0,8 m/s`) e passava `109` tiques a afastar-se do canto
/// seguinte, até o centro entrar na lama (`ph2d_nav::agent::ALCANCE_DO_CANTO`).
#[test]
fn nenhum_corredor_volta_atras_para_um_canto() {
    let mut sim = SimWorld::new();
    let l = crate::nav_smoke::montar(sim.world_mut(), 5)
        .lama
        .expect("a lama");
    let mut bridge = PhysicsBridge::new();
    let quem: Vec<Entity> = l.na_leve.iter().chain(&l.na_pesada).copied().collect();
    let pos = |sim: &SimWorld, e| {
        let p = sim
            .world()
            .get::<Transform>(e)
            .expect("o corpo")
            .translation;
        [f64::from(p.x), f64::from(p.y)]
    };
    let mut antes: Vec<[f64; 2]> = quem.iter().map(|&e| pos(&sim, e)).collect();
    let mut afasta = vec![0u32; quem.len()];
    for t in 1..=900u64 {
        bridge.dispatch(&mut sim, true, t);
        for (i, &e) in quem.iter().enumerate() {
            let p = pos(&sim, e);
            let rt = bridge.nav_agent(e).expect("o agente");
            if rt.status == ph2d_nav::Status::Moving
                && let Some(w) = rt.path.get(rt.next)
            {
                let d = |a: [f64; 2]| (a[0] - w[0]).hypot(a[1] - w[1]);
                afasta[i] += u32::from(d(p) > d(antes[i]) + 1e-4);
            }
            antes[i] = p;
        }
    }
    assert!(
        afasta.iter().all(|&n| n <= 2),
        "tiques a afastar-se do canto seguinte: {afasta:?}"
    );
}
