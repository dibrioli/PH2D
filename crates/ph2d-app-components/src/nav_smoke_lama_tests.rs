//! Os gates da cena `=5` — pela cena REAL (`nav_smoke::montar(…, 5)`): os corredores da lama leve
//! atravessam-na, nenhum dos da pesada a atravessa, e o CONTROLO (a mesma cena com as duas a custo `1`)
//! cruza as duas; e o passo do roteiro (o `Cost` da pesada a `2` com eles a caminho).
//! ⚠️ «Atravessou» = o CENTRO esteve dentro do rectângulo da faixa a mais de um raio da PONTA do lado
//! da passagem: na passagem os três empurram-se e um raspa a quina (medido: `1 cm` dentro, `x = 4,19`
//! contra a ponta em `4,20`) — contornar a quina não é atravessar; o CONTROLO cruza a `x ≈ 0,7–2,2`.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

fn dentro(p: [f32; 2], lado: f32) -> bool {
    let (c, h) = Lama::faixa(lado);
    let ponta = c[0].abs() + h[0] - RAIO_PEQUENO;
    (p[0] - c[0]).abs() <= h[0] && (p[1] - c[1]).abs() <= h[1] && p[0].abs() <= ponta
}

/// Quantos de cada pista estiveram dentro da faixa DELA, e quantos chegaram à bandeira.
fn corre(custos: Option<f32>) -> ([usize; 2], [usize; 2]) {
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
    let mut pisou = [[false; 3]; 2];
    let mut chegou = [[false; 3]; 2];
    for t in 1..=900u64 {
        bridge.dispatch(&mut sim, true, t);
        for (k, (quem, lado)) in pistas.iter().enumerate() {
            for (i, &e) in quem.iter().enumerate() {
                let tr = sim.world().get::<Transform>(e).expect("o corpo");
                pisou[k][i] |= dentro([tr.translation.x, tr.translation.y], *lado);
                chegou[k][i] |= bridge
                    .nav_agent(e)
                    .is_some_and(|r| r.status == ph2d_nav::Status::Arrived);
            }
        }
    }
    let conta = |v: [bool; 3]| v.iter().filter(|&&b| b).count();
    (
        [conta(pisou[0]), conta(pisou[1])],
        [conta(chegou[0]), conta(chegou[1])],
    )
}

/// ⭐⭐⭐ **A cena CONTÉM o fenómeno** — e o CONTROLO prova que é a lama que o faz.
#[test]
fn a_leve_atravessa_se_a_pesada_contorna_se() {
    let (pisaram, chegaram) = corre(None);
    let (pisaram_ctl, _) = corre(Some(1.0));
    eprintln!(
        "produto: pisaram {pisaram:?}, chegaram {chegaram:?} · controlo: pisaram {pisaram_ctl:?}"
    );
    assert!(
        pisaram[0] >= 1,
        "ninguém atravessou a lama leve: {pisaram:?}"
    );
    assert_eq!(pisaram[1], 0, "alguém atravessou a lama pesada");
    assert_eq!(chegaram, [3, 3], "nem todos chegaram à bandeira");
    assert!(
        pisaram_ctl[0] >= 1 && pisaram_ctl[1] >= 1,
        "o CONTROLO (sem custos) não cruza as duas faixas — a cena não mede a lama: {pisaram_ctl:?}"
    );
}

/// O roteiro nomeia as peças.
#[test]
fn a_cena_tem_as_pecas_que_o_roteiro_nomeia() {
    let mut sim = SimWorld::new();
    let l = crate::nav_smoke::montar(sim.world_mut(), 5)
        .lama
        .expect("a lama");
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
