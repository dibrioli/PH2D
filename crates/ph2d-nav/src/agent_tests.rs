//! Os gates da condução — cada um é uma das queixas da pesquisa (Q4 · Q5 · Q6 · Q12) virada régua.

use super::*;
use crate::geom::dist;

/// Um quadrado `4 × 4` com um furo `2 × 2` no meio — o caminho de um canto ao oposto DOBRA.
fn anel() -> NavMesh {
    let v = vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [0.0, 4.0],
        [1.0, 1.0],
        [3.0, 1.0],
        [3.0, 3.0],
        [1.0, 3.0],
    ];
    let p = vec![
        vec![0, 1, 5, 4],
        vec![1, 2, 6, 5],
        vec![2, 3, 7, 6],
        vec![3, 0, 4, 7],
    ];
    NavMesh::from_polygons(v, p).expect("malha válida")
}

/// Duas ilhas: `[0,2]²` e `[3,5]×[0,2]`.
fn duas_ilhas() -> NavMesh {
    let v = vec![
        [0.0, 0.0],
        [2.0, 0.0],
        [2.0, 2.0],
        [0.0, 2.0],
        [3.0, 0.0],
        [5.0, 0.0],
        [5.0, 2.0],
        [3.0, 2.0],
    ];
    NavMesh::from_polygons(v, vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7]]).expect("malha válida")
}

const CFG: AgentConfig = AgentConfig {
    arrive_distance: 0.05,
    repath_distance: 0.25,
    stuck_after_s: 0.5,
    speed: 2.0,
    radius: 0.0,
};
const DT: f64 = 1.0 / 60.0;

/// Corre `n` tiques com o executor ideal (anda `speed·dt` na direcção pedida) e devolve a posição
/// final e os eventos, com o tique de cada um.
fn corre(
    mesh: &NavMesh,
    rt: &mut AgentRuntime,
    mut pos: V2,
    alvo: V2,
    n: usize,
) -> (V2, Vec<(usize, Event)>) {
    let mut s = Polyanya::new();
    let mut ev = Vec::new();
    for k in 0..n {
        let st = step(rt, Some(mesh), &mut s, pos, Some(alvo), &CFG, DT);
        if let Some(e) = st.event {
            ev.push((k, e));
        }
        pos = [
            pos[0] + st.dir[0] * CFG.speed * DT,
            pos[1] + st.dir[1] * CFG.speed * DT,
        ];
    }
    (pos, ev)
}

#[test]
fn o_primeiro_tique_ja_tem_caminho_e_dobra_o_canto() {
    // Q4: o caminho vazio do 1.º quadro.
    let m = anel();
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    let st = step(
        &mut rt,
        Some(&m),
        &mut s,
        [0.5, 0.5],
        Some([3.5, 3.5]),
        &CFG,
        DT,
    );
    assert_eq!(rt.searches, 1);
    assert_eq!(rt.status, Status::Moving);
    assert!(
        dist(st.dir, [0.0, 0.0]) > 0.99,
        "a direcção é unitária: {:?}",
        st.dir
    );
    assert!(rt.path.len() >= 3, "dá a volta ao furo: {:?}", rt.path);
}

#[test]
fn um_alvo_parado_e_procurado_uma_vez_e_chega_uma_vez() {
    // Q6 + Q12: uma procura em 600 tiques; `Arrived` sai UMA vez.
    let m = anel();
    let mut rt = AgentRuntime::default();
    let (pos, ev) = corre(&m, &mut rt, [0.5, 0.5], [3.5, 3.5], 600);
    assert_eq!(rt.searches, 1, "recalculou sem motivo");
    assert!(dist(pos, [3.5, 3.5]) <= CFG.arrive_distance + CFG.speed * DT);
    let chegadas: Vec<_> = ev.iter().filter(|(_, e)| *e == Event::Arrived).collect();
    assert_eq!(chegadas.len(), 1, "eventos: {ev:?}");
    assert!(
        ev.iter().all(|(_, e)| *e == Event::Arrived),
        "eventos: {ev:?}"
    );
    assert_eq!(rt.status, Status::Arrived);
}

#[test]
fn o_canto_e_alcancado_sem_orbitar() {
    // Q5: com o «alcançado» a `speed·dt`, o agente nunca anda para trás no caminho.
    let m = anel();
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    let mut pos = [0.5, 0.5];
    let mut ultimo = 0;
    for _ in 0..600 {
        let st = step(&mut rt, Some(&m), &mut s, pos, Some([3.5, 3.5]), &CFG, DT);
        assert!(rt.next >= ultimo, "o ponto em curso andou para trás");
        ultimo = rt.next;
        pos = [
            pos[0] + st.dir[0] * CFG.speed * DT,
            pos[1] + st.dir[1] * CFG.speed * DT,
        ];
    }
    // O tempo de chegada é o comprimento do caminho a `speed`, com uma folga de um passo por canto.
    assert_eq!(rt.status, Status::Arrived);
}

#[test]
fn o_alvo_que_anda_pouco_nao_recalcula_e_o_que_anda_muito_recalcula() {
    let m = anel();
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    step(
        &mut rt,
        Some(&m),
        &mut s,
        [0.5, 0.5],
        Some([3.5, 3.5]),
        &CFG,
        DT,
    );
    step(
        &mut rt,
        Some(&m),
        &mut s,
        [0.5, 0.5],
        Some([3.5, 3.4]),
        &CFG,
        DT,
    );
    assert_eq!(rt.searches, 1, "0,1 m abaixo da distância de recálculo");
    step(
        &mut rt,
        Some(&m),
        &mut s,
        [0.5, 0.5],
        Some([3.5, 3.0]),
        &CFG,
        DT,
    );
    assert_eq!(rt.searches, 2, "0,5 m acima da distância de recálculo");
}

#[test]
fn um_alvo_noutra_ilha_da_um_caminho_parcial_e_fala_uma_vez() {
    let m = duas_ilhas();
    let mut rt = AgentRuntime::default();
    let (pos, ev) = corre(&m, &mut rt, [0.5, 1.0], [4.0, 1.0], 240);
    assert_eq!(rt.status, Status::MovingPartial);
    assert!(rt.partial);
    assert!(
        dist(pos, [2.0, 1.0]) < 0.05,
        "parou na borda mais perto: {pos:?}"
    );
    assert_eq!(
        ev,
        vec![(0, Event::NoPath)],
        "a família «sem caminho» fala uma vez"
    );
    assert_eq!(rt.searches, 1);
}

#[test]
fn quem_nao_avanca_fica_preso_e_recalcula() {
    // O executor não anda (uma parede que a malha não conhece): o relógio enche.
    let m = anel();
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    let mut presos = Vec::new();
    for k in 0..120 {
        let st = step(
            &mut rt,
            Some(&m),
            &mut s,
            [0.5, 0.5],
            Some([3.5, 3.5]),
            &CFG,
            DT,
        );
        if st.event == Some(Event::Stuck) {
            presos.push(k);
        }
    }
    // A 60 Hz, `0,5 s` são 30 tiques; o 1.º tique planeia, o 2.º marca o melhor, depois 30 sem descer.
    assert!(!presos.is_empty(), "nunca ficou preso");
    assert!(
        presos[0] >= 29 && presos[0] <= 33,
        "preso no tique {}",
        presos[0]
    );
    assert_eq!(
        rt.searches as usize,
        1 + presos.len(),
        "cada «preso» recalcula uma vez"
    );
    assert_eq!(
        rt.status,
        Status::Moving,
        "preso é acontecimento, não estado"
    );
}

#[test]
fn sem_alvo_e_sem_malha_param_e_dizem_qual() {
    let m = anel();
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    let st = step(&mut rt, Some(&m), &mut s, [0.5, 0.5], None, &CFG, DT);
    assert_eq!(
        (st.dir, st.event, rt.status),
        ([0.0; 2], None, Status::Idle)
    );
    let st = step(
        &mut rt,
        None,
        &mut s,
        [0.5, 0.5],
        Some([1.0, 1.0]),
        &CFG,
        DT,
    );
    assert_eq!(
        (st.dir, st.event, rt.status),
        ([0.0; 2], Some(Event::NoPath), Status::NoPath)
    );
    let st = step(
        &mut rt,
        None,
        &mut s,
        [0.5, 0.5],
        Some([1.0, 1.0]),
        &CFG,
        DT,
    );
    assert_eq!(st.event, None, "o estado repetido não volta a falar");
    assert_eq!(rt.searches, 0, "sem malha não se procura nada");
}

#[test]
fn um_agente_empurrado_para_fora_da_malha_volta_pelo_ponto_mais_perto() {
    let m = anel();
    let mut rt = AgentRuntime::default();
    let mut s = Polyanya::new();
    // Dentro do furo — fora da malha.
    let st = step(
        &mut rt,
        Some(&m),
        &mut s,
        [2.0, 1.5],
        Some([0.5, 0.5]),
        &CFG,
        DT,
    );
    assert_eq!(rt.status, Status::Moving);
    assert!(
        st.dir[1] < -0.9,
        "volta para a borda de baixo do furo: {:?}",
        st.dir
    );
}

/// (W14) A estimativa da fila é o TRABALHO da última procura ([`crate::Stats::work`]): na lama, mais
/// do que os nós expandidos (as raízes de fronteira pesam o que custam); sem lama, os nós ao bit.
#[test]
fn a_ultima_procura_guarda_o_trabalho_e_nao_os_nos() {
    // Três faixas `[0,1] · [1,3] · [3,4] × [0,4]`, a do meio é a área `1`.
    let v = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [3.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [3.0, 4.0],
        [1.0, 4.0],
        [0.0, 4.0],
    ];
    let p = vec![vec![0, 1, 6, 7], vec![1, 2, 5, 6], vec![2, 3, 4, 5]];
    let m = NavMesh::from_polygons_with_areas(v, p, vec![0, 1, 0]).expect("malha válida");
    for (costs, lama) in [([1.0, 4.0], true), ([1.0, 1.0], false)] {
        let q = Query {
            costs: &costs,
            links: &[],
        };
        let (mut rt, mut s) = (AgentRuntime::default(), Polyanya::new());
        let antes = s.stats;
        step_with(
            &mut rt,
            Some(&m),
            &mut s,
            &q,
            [0.5, 0.5],
            Some([3.5, 3.5]),
            &CFG,
            DT,
        );
        let (nos, pend) = (
            s.stats.expanded - antes.expanded,
            s.stats.pending - antes.pending,
        );
        assert_eq!(rt.last_work, s.stats.work() - antes.work());
        if lama {
            assert!(pend > 0, "a fixtura refracta: {pend} raízes de fronteira");
            assert!(
                rt.last_work > nos,
                "trabalho {} contra {nos} nós",
                rt.last_work
            );
        } else {
            // O CONTROLO: a mesma malha com a área a `1` é a procura uniforme.
            assert_eq!((rt.last_work, pend), (nos, 0));
        }
    }
}

/// ⭐ (plano 30 §25, C2) **O alvo À VISTA** só quando a recta é mesmo o caminho mais curto: dentro da
/// malha, sem custo acima de `1` no caminho, nenhuma área barata ao alcance (W19: a cota, `cota.rs`) e
/// sem atalhos (um teletransporte podia). Três faixas com a do meio na área `1`.
#[test]
fn o_alvo_a_vista_e_so_a_recta_que_nenhum_caminho_bate() {
    let v = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [3.0, 0.0],
        [4.0, 0.0],
        [4.0, 4.0],
        [3.0, 4.0],
        [1.0, 4.0],
        [0.0, 4.0],
    ];
    let p = vec![vec![0, 1, 6, 7], vec![1, 2, 5, 6], vec![2, 3, 4, 5]];
    let m = NavMesh::from_polygons_with_areas(v, p, vec![0, 1, 0]).expect("malha válida");
    let q = |costs, links| Query { costs, links };
    let atalho = [crate::link::Link {
        id: 0,
        from: [0.5, 3.5],
        to: [3.5, 3.5],
        two_way: false,
        teleport: true,
        cost: 0.0,
    }];
    // A recta pela faixa da lama: à vista com a lama a `1`, não com ela a `4`.
    assert!(a_vista(&m, &q(&[1.0, 1.0], &[]), [0.5, 0.5], [3.5, 3.5]));
    assert!(!a_vista(&m, &q(&[1.0, 4.0], &[]), [0.5, 0.5], [3.5, 3.5]));
    // A recta dentro de uma faixa sem lama, com a lama a `4` noutra: à vista.
    assert!(a_vista(&m, &q(&[1.0, 4.0], &[]), [0.5, 0.5], [0.5, 3.5]));
    // (W19) Uma área mais BARATA perto (a ida e a volta até ela cabem no comprimento: `0,5 + 0,5 < 3`)
    // pode vencer a recta: não. Longe (`0,5 + 0,5 ≥ 1`), a recta vence: sim. Um atalho: nunca.
    assert!(!a_vista(&m, &q(&[1.0, 0.5], &[]), [0.5, 0.5], [0.5, 3.5]));
    assert!(a_vista(&m, &q(&[1.0, 0.5], &[]), [0.5, 0.5], [0.5, 1.5]));
    assert!(!a_vista(
        &m,
        &q(&[1.0, 1.0], &atalho),
        [0.5, 0.5],
        [0.5, 3.5]
    ));
    // Fora da malha (uma parede pelo meio): nunca.
    let a = anel();
    assert!(!a_vista(&a, &q(&[], &[]), [0.5, 0.5], [3.5, 3.5]));
    assert!(a_vista(&a, &q(&[], &[]), [0.5, 0.5], [3.5, 0.5]));
}
