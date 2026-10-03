//! ⭐⭐ **OS ATALHOS** (W7): o teleporte e a porta de um sentido, conduzidos pela lei do agente —
//! cada um com o CONTROLO ao lado (sem o atalho; o atalho caro demais; o sentido contrário).

use ph2d_nav::agent::{AgentConfig, AgentRuntime, Status, step_with};
use ph2d_nav::{Link, NavMesh, Polyanya, Query, V2};
use ph2d_navmesh::{Params, Shape, build};

use crate::cena::retangulo;

/// Duas salas `8 × 10` separadas por uma parede maciça (`x ∈ [8, 12]`): duas ilhas.
fn duas_salas() -> NavMesh {
    build(
        &retangulo(20.0, 10.0),
        &[Shape::Convex(vec![
            [8.0, -1.0],
            [12.0, -1.0],
            [12.0, 11.0],
            [8.0, 11.0],
        ])],
        &Params::default(),
    )
    .expect("constrói")
    .mesh
}

fn cfg() -> AgentConfig {
    AgentConfig {
        arrive_distance: 0.1,
        repath_distance: 0.5,
        stuck_after_s: 0.0,
        speed: 3.0,
    }
}

/// Anda o agente pela lei (o corpo vai exactamente para onde ela pede, e salta quando ela manda),
/// até chegar ou acabar o tempo. Devolve a posição, o estado, os atalhos atravessados e os saltos.
fn anda(m: &NavMesh, q: &Query<'_>, mut pos: V2, alvo: V2) -> (V2, Status, Vec<u32>, usize) {
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    let dt = 1.0 / 60.0;
    let mut cruzou = Vec::new();
    let mut saltos = 0;
    for _ in 0..3_000 {
        let st = step_with(&mut rt, Some(m), &mut s, q, pos, Some(alvo), &cfg(), dt);
        if let Some(l) = st.crossed {
            cruzou.push(l);
        }
        if let Some(p) = st.teleport {
            pos = p;
            saltos += 1;
            continue;
        }
        if rt.status == Status::Arrived || rt.status == Status::MovingPartial && st.dir == [0.0; 2]
        {
            break;
        }
        pos = [pos[0] + st.dir[0] * 3.0 * dt, pos[1] + st.dir[1] * 3.0 * dt];
    }
    (pos, rt.status, cruzou, saltos)
}

#[test]
fn o_teleporte_leva_a_outra_sala_e_sem_ele_e_parcial() {
    let m = duas_salas();
    let (a, z) = ([2.0, 5.0], [18.0, 5.0]);
    let tp = Link {
        id: 7,
        from: [6.0, 8.0],
        to: [14.0, 2.0],
        two_way: false,
        teleport: true,
        cost: 0.0,
    };
    let (fim, st, cruzou, saltos) = anda(
        &m,
        &Query {
            costs: &[],
            links: &[tp],
        },
        a,
        z,
    );
    assert_eq!(st, Status::Arrived, "fim {fim:?}");
    assert_eq!((cruzou, saltos), (vec![7], 1));
    // CONTROLO: sem o atalho a outra sala é outra ilha — o agente vai ao ponto mais perto e diz.
    let (_, st, cruzou, saltos) = anda(&m, &Query::default(), a, z);
    assert_eq!(st, Status::MovingPartial);
    assert!(cruzou.is_empty() && saltos == 0);
}

#[test]
fn a_porta_de_um_sentido_nao_volta() {
    let m = duas_salas();
    let porta = Link {
        id: 3,
        from: [7.5, 5.0],
        to: [12.5, 5.0],
        two_way: false,
        teleport: false,
        cost: 0.0,
    };
    let q = Query {
        costs: &[],
        links: &[porta],
    };
    // De A para B: anda a direito pela porta (sem saltar) e diz que atravessou.
    let (_, st, cruzou, saltos) = anda(&m, &q, [2.0, 5.0], [18.0, 5.0]);
    assert_eq!((st, cruzou, saltos), (Status::Arrived, vec![3], 0));
    // De B para A: um sentido só — parcial.
    let (_, st, cruzou, _) = anda(&m, &q, [18.0, 5.0], [2.0, 5.0]);
    assert_eq!(st, Status::MovingPartial);
    assert!(cruzou.is_empty());
    // CONTROLO: a mesma porta de DOIS sentidos volta.
    let dois = Link {
        two_way: true,
        ..porta
    };
    let (_, st, cruzou, _) = anda(
        &m,
        &Query {
            costs: &[],
            links: &[dois],
        },
        [18.0, 5.0],
        [2.0, 5.0],
    );
    assert_eq!((st, cruzou), (Status::Arrived, vec![3]));
}

#[test]
fn o_atalho_so_se_usa_quando_compensa() {
    // Uma sala só, com um teleporte de um canto ao outro: de graça encurta; caro, ninguém o usa.
    let m = build(&retangulo(20.0, 10.0), &[], &Params::default())
        .expect("constrói")
        .mesh;
    let (a, z) = ([1.0, 1.0], [19.0, 9.0]);
    let tp = Link {
        id: 1,
        from: [1.5, 1.5],
        to: [18.5, 8.5],
        two_way: false,
        teleport: true,
        cost: 0.0,
    };
    let (_, st, cruzou, _) = anda(
        &m,
        &Query {
            costs: &[],
            links: &[tp],
        },
        a,
        z,
    );
    assert_eq!((st, cruzou), (Status::Arrived, vec![1]));
    // CONTROLO: a atravessar custa mais que andar a sala toda (`√(18² + 8²) ≈ 19,7`).
    let caro = Link { cost: 25.0, ..tp };
    let (_, st, cruzou, _) = anda(
        &m,
        &Query {
            costs: &[],
            links: &[caro],
        },
        a,
        z,
    );
    assert_eq!(st, Status::Arrived);
    assert!(cruzou.is_empty(), "usou o atalho caro: {cruzou:?}");
}

#[test]
fn a_meio_da_porta_nao_se_replaneia_mesmo_com_o_alvo_a_andar() {
    // O alvo foge (mais que `repath_distance`) a cada tique em que o agente está DENTRO da porta: lá
    // ele está fora da malha, e replanear levava-o de volta à entrada — atravessaria outra vez.
    let m = duas_salas();
    let porta = Link {
        id: 3,
        from: [7.5, 5.0],
        to: [12.5, 5.0],
        two_way: false,
        teleport: false,
        cost: 0.0,
    };
    let q = Query {
        costs: &[],
        links: &[porta],
    };
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    let dt = 1.0 / 60.0;
    let mut pos = [2.0, 5.0];
    let mut alvo = [18.0, 5.0];
    let mut cruzou = Vec::new();
    let mut dentro = 0;
    for _ in 0..3_000 {
        if pos[0] > 8.0 && pos[0] < 12.0 {
            alvo[1] = if alvo[1] > 5.0 { 2.0 } else { 8.0 };
            dentro += 1;
        }
        let st = step_with(&mut rt, Some(&m), &mut s, &q, pos, Some(alvo), &cfg(), dt);
        cruzou.extend(st.crossed);
        if rt.status == Status::Arrived {
            break;
        }
        pos = [pos[0] + st.dir[0] * 3.0 * dt, pos[1] + st.dir[1] * 3.0 * dt];
    }
    assert!(
        dentro > 10,
        "a fixtura não tem o agente dentro da porta ({dentro} tiques)"
    );
    assert_eq!(rt.status, Status::Arrived);
    assert_eq!(cruzou, vec![3], "atravessou {cruzou:?}");
}

#[test]
fn a_chegada_anuncia_se_uma_vez_por_aproximacao() {
    // Chega; é EMPURRADO um pouco (para fora da chegada, dentro de chegada + recálculo) e volta —
    // calado; vai MESMO embora (2 m) e volta — anuncia outra vez (o controlo).
    let m = duas_salas();
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    let dt = 1.0 / 60.0;
    let alvo = [5.0, 5.0];
    let q = Query::default();
    let mut pos = [2.0, 5.0];
    let mut chegou = 0;
    let mut anda = |pos: &mut V2, rt: &mut AgentRuntime, chegou: &mut usize| {
        for _ in 0..600 {
            let st = step_with(rt, Some(&m), &mut s, &q, *pos, Some(alvo), &cfg(), dt);
            if st.event == Some(ph2d_nav::Event::Arrived) {
                *chegou += 1;
            }
            if rt.status == Status::Arrived {
                return;
            }
            *pos = [pos[0] + st.dir[0] * 3.0 * dt, pos[1] + st.dir[1] * 3.0 * dt];
        }
    };
    anda(&mut pos, &mut rt, &mut chegou);
    assert_eq!(chegou, 1);
    pos = [alvo[0] - 0.4, alvo[1]];
    anda(&mut pos, &mut rt, &mut chegou);
    assert_eq!(chegou, 1, "empurrado 0,4 m reanunciou");
    pos = [alvo[0] - 2.0, alvo[1]];
    anda(&mut pos, &mut rt, &mut chegou);
    assert_eq!(chegou, 2, "ido embora 2 m não reanunciou");
}
