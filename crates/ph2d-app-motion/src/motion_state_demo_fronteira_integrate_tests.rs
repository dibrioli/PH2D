//! **SONDAS e GATES — por que o `Integrate` da `=139` ficava no processador** (aberto 2 do handoff
//! PASSE_UNICO; doc 120 §8.3.3). Os gates: `um_condutor_na_placa_nunca_custa_o_laco` (sem placa) e
//! `a_galaxia_da_139_na_placa_e_a_da_cpu` (placa). O resto são sondas.
//!
//! Diagnóstico por ABLAÇÃO pela porta do produto: a mesma cena com um fio cortado de cada vez, e o
//! plano que o produto faz de cada uma (rota, fronteiras, o que a placa encena, os condutores na
//! placa, e o primeiro motivo por que cada fronteira não é estática — a regra do recuo,
//! `plan_uniao.rs`). Depois o quadro inteiro a ×1/×4/×11: `A` (processador, `gpu_enabled = false`),
//! `B` (hoje) e cada ablação pela rota dela, intercalados em blocos rodados, resumo = o MÍNIMO.
//!
//! ⛔ **Kill-criterion, declarado ANTES da corrida:** a cura (o laço da galáxia fica na placa quando a
//! fronteira temporal só o alcança por params DIRIGIDOS, já resolvidos por tique) só se constrói se a
//! ablação que põe o `Integrate` na placa cortar o quadro de `B` em **≥ 25 % E ≥ 100 µs a ×4 OU a
//! ×11**. Abaixo disso é recusa medida. A ×1 a escolha `A`/`B` é decisão de rota, não esta cura.
//!
//! `__GL_SHADER_DISK_CACHE=0 PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion \
//!     --profile smoke --lib sonda_a_fronteira_do_integrate -- --ignored --nocapture --test-threads=1`
//! (`PH2D_SONDA_ESCALAS`, `PH2D_SONDA_BLOCO`, `PH2D_SONDA_RODADAS`).

use super::carimbo_escala::{amplia, numero};
use super::medida::monta_demo;
use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::NodeId;
use std::time::{Duration, Instant};

const CENA: &str = "139";

fn do_tipo(m: &MotionState, tipo: &str) -> Vec<NodeId> {
    m.doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == tipo)
        .map(|n| n.id)
        .collect()
}

fn um(m: &MotionState, tipo: &str) -> NodeId {
    let v = do_tipo(m, tipo);
    assert_eq!(v.len(), 1, "a =139 tem um {tipo}");
    v[0]
}

/// As variantes: nome + o corte no grafo (ou no interruptor) que cada uma faz.
type Ajuste = fn(&mut MotionState);
const VARIANTES: &[(&str, Ajuste)] = &[
    ("hoje", |_| {}),
    ("sem_pre_do_envelope", |m| {
        let e = um(m, "pulse.adsr");
        m.doc.graph.disconnect(e, 1).expect("o envelope tem pre");
    }),
    ("sem_gatilho_no_envelope", |m| {
        let e = um(m, "pulse.adsr");
        m.doc.graph.disconnect(e, 0).expect("o envelope tem gatilho");
    }),
    ("sem_fio_no_redemoinho", |m| {
        let v = um(m, "force.vortex");
        m.doc
            .graph
            .undrive_param(v, "strength")
            .expect("o strength é dirigido");
    }),
    ("condutores_na_cpu", |m| m.condutores_na_placa = false),
];

fn monta(ajuste: Ajuste, k: f32) -> MotionState {
    let mut m = monta_demo(CENA).expect("a cena existe");
    ajuste(&mut m);
    amplia(&mut m, k);
    m
}

/// O primeiro motivo por que a cadeia de `n` não é estática (a lei do `boundary_is_static`).
fn porque_nao_e_estatica(m: &MotionState, n: NodeId) -> Option<String> {
    let g = &m.doc.graph;
    let tipo = |k: NodeId| g.node(k).map(|i| i.type_name.clone()).unwrap_or_default();
    let mut vistos = std::collections::BTreeSet::new();
    let mut pilha = vec![n];
    while let Some(k) = pilha.pop() {
        if !vistos.insert(k) {
            continue;
        }
        let Some(op) = g.node(k).and_then(|i| m.registry.resolve(i.type_id())) else {
            return Some(format!("{} sem op", tipo(k)));
        };
        let man = op.manifest();
        if man.effect != ph2d_nodegraph::effect::Effect::Pure {
            return Some(format!("{} efeito {:?}", tipo(k), man.effect));
        }
        if g.param_sources(k).is_some_and(|s| !s.is_empty()) {
            return Some(format!("{} tem param dirigido", tipo(k)));
        }
        for p in 0..man.inputs.len() {
            if let Some((src, _, pre)) = g.input_edge(k, p) {
                if pre {
                    return Some(format!("{} lê pre na porta {p}", tipo(k)));
                }
                pilha.push(src);
            }
        }
    }
    None
}

fn diz_o_plano(nome: &str, m: &mut MotionState, gpu: &ph2d_gpu::GpuContext) {
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    ph.play();
    for _ in 0..10 {
        ph.advance_ticks(1);
        crate::motion_bridge::quadro::o_quadro(
            m,
            Some(gpu),
            &ph,
            1.0 / 60.0,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
    }
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let plano = crate::motion_bridge::gpu::plano_do_produto(m, ph.time());
    let g = &m.doc.graph;
    let tipo = |k: NodeId| g.node(k).map(|i| i.type_name.clone()).unwrap_or_default();
    let mut na_placa: Vec<String> = plano.stages.iter().map(|s| tipo(s.node)).collect();
    na_placa.sort();
    na_placa.dedup();
    eprintln!(
        "[{nome}] rota «{}» · laço na placa {}\n  na placa {na_placa:?}",
        m.route_said.unwrap_or("—"),
        plano.drives_a_loop()
    );
    for (b, porta) in &plano.boundaries {
        eprintln!(
            "  fronteira {}#{porta}: {}",
            tipo(*b),
            porque_nao_e_estatica(m, *b).unwrap_or_else(|| "ESTÁTICA".into())
        );
    }
    for (consumidor, fios) in &plano.device_drivers {
        let fios: Vec<String> = fios
            .iter()
            .map(|(p, c)| format!("{p} ← {}", tipo(*c)))
            .collect();
        eprintln!("  condutor na placa: {} {fios:?}", tipo(*consumidor));
    }
}

/// A cena `n` como o produto a monta, com as saídas postas — sem quadro nenhum.
fn com_saidas(n: &str) -> Option<MotionState> {
    let mut m = MotionState::new();
    m.cpu_pedida = crate::motion_state::demo_router::cena_pede_a_cpu_em(Some(n));
    m.sinks = crate::motion_demo_legend::monta(n, &mut m.doc, &m.registry).0;
    (!m.sinks.is_empty()).then_some(m)
}

/// ⛔⛔ **GATE — um condutor na placa nunca custa o laço** (doc 120 §8.3.3). Em toda cena do
/// roteador, o plano do produto com os condutores na placa (a omissão) tem o laço na placa sempre
/// que o tem com eles na CPU; e a `=139` — a cena que o perdia pelo gatilho do envelope — tem a
/// galáxia encenada. Só o planeador: sem placa.
#[test]
fn um_condutor_na_placa_nunca_custa_o_laco() {
    let mut perdem = Vec::new();
    let mut com_laco = 0;
    for n in (1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL).map(|n| n.to_string()) {
        let laco = |condutores: bool| {
            let mut m = com_saidas(&n)?;
            m.condutores_na_placa = condutores;
            Some(crate::motion_bridge::gpu::plano_do_produto(&mut m, 0.5).drives_a_loop())
        };
        let (Some(hoje), Some(na_cpu)) = (laco(true), laco(false)) else {
            continue;
        };
        com_laco += usize::from(hoje);
        if na_cpu && !hoje {
            perdem.push(n);
        }
    }
    assert!(perdem.is_empty(), "o condutor na placa expulsa o laço em {perdem:?}");
    // O controlo: a varredura viu laços (senão passaria sem ler nada).
    assert!(com_laco >= 10, "só {com_laco} cenas com laço na placa");
    let mut m = com_saidas(CENA).expect("a =139 monta");
    let galaxia = um(&m, "motion.integrate");
    let plano = crate::motion_bridge::gpu::plano_do_produto(&mut m, 0.5);
    assert!(
        plano.stages.iter().any(|s| s.node == galaxia),
        "a galáxia da =139 saiu da placa: fronteiras {:?}",
        plano.boundaries
    );
}

/// A cadeia de `n` (fios de entrada e condutores) tem estado — um `pre` ou um efeito `Stateful`?
fn cadeia_com_estado(m: &MotionState, n: NodeId) -> Option<String> {
    let g = &m.doc.graph;
    let tipo = |k: NodeId| g.node(k).map(|i| i.type_name.clone()).unwrap_or_default();
    let mut vistos = std::collections::BTreeSet::new();
    let mut pilha = vec![n];
    while let Some(k) = pilha.pop() {
        if !vistos.insert(k) {
            continue;
        }
        let Some(op) = g.node(k).and_then(|i| m.registry.resolve(i.type_id())) else {
            continue;
        };
        let man = op.manifest();
        if man.effect == ph2d_nodegraph::effect::Effect::Stateful {
            return Some(format!("{} Stateful", tipo(k)));
        }
        for p in 0..man.inputs.len() {
            if let Some((src, _, pre)) = g.input_edge(k, p) {
                if pre {
                    return Some(format!("{} lê pre na porta {p}", tipo(k)));
                }
                pilha.push(src);
            }
        }
        for (s, _) in g.param_sources(k).into_iter().flat_map(|f| f.values()) {
            pilha.push(*s);
        }
    }
    None
}

/// **SONDA — os dois defeitos latentes da família** (doc 120 §8.3.3), na configuração de hoje:
/// (a) uma fronteira de porta ≠ 0 encenada (a placa recebe a porta 0 — `GpuSource::Boundary(n, _)`)
/// e (b) um param de um estágio dirigido pela CPU cuja cadeia tem estado e que a marcha não avança
/// (fora do cone das fronteiras e fora dos `condutores_na_marcha`).
#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_os_defeitos_latentes_da_entrega() {
    for n in (1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL).map(|n| n.to_string()) {
        let Some(mut m) = com_saidas(&n) else { continue };
        let plano = crate::motion_bridge::gpu::plano_do_produto(&mut m, 0.5);
        let g = &m.doc.graph;
        let tipo = |k: NodeId| g.node(k).map(|i| i.type_name.clone()).unwrap_or_default();
        for (b, porta) in plano.boundaries.iter().filter(|(_, p)| *p != 0) {
            eprintln!("={n} (a) fronteira {}#{porta} · laço {}", tipo(*b), plano.drives_a_loop());
        }
        let cone: std::collections::BTreeSet<NodeId> = plano
            .boundaries
            .iter()
            .flat_map(|(b, _)| ph2d_nodegraph::cook::upstream_cone(g, *b))
            .collect();
        for s in &plano.stages {
            let na_placa = plano.device_drivers.get(&s.node);
            for (p, (src, _)) in g.param_sources(s.node).into_iter().flat_map(|f| f.iter()) {
                if na_placa.is_some_and(|v| v.iter().any(|(q, _)| q == p))
                    || cone.contains(src)
                    || plano.condutores_na_marcha.contains(src)
                {
                    continue;
                }
                if let Some(porque) = cadeia_com_estado(&m, *src) {
                    eprintln!(
                        "={n} (b) {}.{p} ← {} ({porque}) · laço {}",
                        tipo(s.node),
                        tipo(*src),
                        plano.drives_a_loop()
                    );
                }
            }
        }
    }
    eprintln!("FIM");
}

/// O tecto da paridade de uma SIMULAÇÃO dirigida — o `EPS_SIM` do `gpu_cpu_parity_driven` do
/// `ph2d-gpu-cook` (o ε realimenta-se).
const EPS_SIM: f32 = 2e-3;

/// 2,5 s: a batida dispara de 1,5 em 1,5 s, e 40 tiques não a alcançavam.
const GALAXIA_TIQUES: u64 = 150;
/// Os tiques por quadro: um quadro lento deve VÁRIOS, e é aí que a marcha anda em passo com a placa.
const SALTOS: [u64; 6] = [1, 2, 7, 1, 4, 3];

/// As duas formas do plano: com as estrelas (a fronteira estática do `Shape`, rota híbrida) e sem
/// elas (a galáxia direto na saída: nenhuma fronteira, só o condutor com estado faz dela híbrida).
const FORMAS_DO_PLANO: [(&str, Ajuste); 2] = [
    ("com as estrelas", |_| {}),
    ("sem as estrelas", |m| {
        let (g, s) = (um(m, "motion.integrate"), um(m, "motion.output"));
        m.doc.graph.disconnect(s, 0).expect("a saída lê o carimbo");
        m.doc
            .graph
            .connect(ph2d_nodegraph::graph::Edge {
                from: (g, 0),
                to: (s, 0),
                delayed: false,
            })
            .expect("a galáxia liga à saída");
    }),
];

/// Os quadros do calendário: o tique em que cada um pousa.
fn quadros() -> Vec<u64> {
    let mut t = 0;
    SALTOS
        .iter()
        .cycle()
        .map_while(|s| {
            t += s;
            (t <= GALAXIA_TIQUES).then_some(t)
        })
        .collect()
}

/// ⭐ **GATE — a galáxia da `=139` na placa é a da CPU** (doc 120 §8.3.3): os quadros do
/// calendário pela porta do produto, o `P` do `Integrate` lido da placa contra o da CPU que publica
/// e marcha os MESMOS quadros, nas duas [`FORMAS_DO_PLANO`]. ⚠️ Os controlos: a placa encenou o
/// `Integrate` (senão não há coluna a ler), a força variou, e sem o fio do envelope a galáxia da CPU
/// é OUTRA (senão a paridade não provaria o número de cada tique).
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_galaxia_da_139_na_placa_e_a_da_cpu() {
    const DT: f64 = 1.0 / 60.0;
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter");
    let mut falhas = Vec::new();
    for (forma, ajuste) in FORMAS_DO_PLANO {
        let mut placa = com_saidas(CENA).expect("a =139 monta");
        ajuste(&mut placa);
        let galaxia = um(&placa, "motion.integrate");
        let vortex = um(&placa, "force.vortex");
        placa.gpu_cook.retain_streams_for_debug(true);
        let mut ph = ph2d_core::Playhead::new(DT);
        ph.play();
        let (mut forcas, mut antes) = (std::collections::BTreeSet::new(), 0);
        for t in quadros() {
            ph.advance_ticks(u32::try_from(t - antes).expect("um salto pequeno"));
            antes = t;
            crate::motion_externals::publish_all(&mut placa, ph.time());
            crate::motion_bridge::quadro::o_quadro(
                &mut placa,
                Some(&gpu),
                &ph,
                DT,
                crate::motion_bridge::quadro::MaoDoEditor::default(),
            );
            let d = crate::motion_bridge::gpu::valores_dirigidos(&mut placa, ph.time());
            if let Some(Some(v)) = d.get(&vortex).and_then(|m| m.get("strength")) {
                forcas.insert(v.to_bits());
            }
        }
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let rota = placa.route_said.unwrap_or("—");
        let Some(p_placa) = placa.gpu_cook.read_column_vec2(&gpu, galaxia, "P") else {
            falhas.push(format!("{forma}: o Integrate não coze na placa («{rota}»)"));
            continue;
        };
        let p_cpu = p_da_cpu(galaxia, ajuste, |_| {});
        let p_sem_fio = p_da_cpu(galaxia, ajuste, |m| {
            let v = um(m, "force.vortex");
            m.doc.graph.undrive_param(v, "strength");
        });
        assert_eq!(p_placa.len(), p_cpu.len());
        let (d, envelope) = (delta(&p_placa, &p_cpu), delta(&p_cpu, &p_sem_fio));
        eprintln!(
            "=139 {forma} · «{rota}» · {} pontos · {} forças · max|ΔP| placa×CPU {d:.3e} · CPU com×sem envelope {envelope:.3e}",
            p_cpu.len(),
            forcas.len()
        );
        if forcas.len() <= 2 || envelope <= 100.0 * EPS_SIM {
            falhas.push(format!("{forma}: o envelope não mexeu a galáxia ({envelope:e})"));
        }
        if d > EPS_SIM {
            falhas.push(format!("{forma}: a galáxia da placa afastou-se da CPU ({d:e})"));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

fn delta(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0f32, f32::max)
}

/// O `P` da galáxia da `=139` (com `forma` e `ajuste`) pela CPU, com os quadros do calendário: as
/// entradas publicadas uma vez por quadro e os tiques devidos marchados, como a rota da CPU.
fn p_da_cpu(galaxia: NodeId, forma: Ajuste, ajuste: impl FnOnce(&mut MotionState)) -> Vec<[f32; 2]> {
    let dt = 1.0 / 60.0;
    let mut cpu = com_saidas(CENA).expect("a =139 monta");
    forma(&mut cpu);
    ajuste(&mut cpu);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&cpu.doc.graph, &cpu.registry);
    cpu.pump.set_taps(&[galaxia]);
    for t in quadros() {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut cpu, t as f64 * dt);
        for tick in crate::motion_bridge::ticks_owed(cpu.pump.last_cooked_tick(), t) {
            cpu.pump.advance_or_scrub_scoped(
                &cpu.doc.graph,
                &cpu.registry,
                &cpu.sinks,
                tick,
                |k| k as f64 * dt,
                cpu.default_uv_rect,
                cpu.default_size,
                &scopes,
            );
        }
    }
    match cpu
        .pump
        .tap_streams()
        .iter()
        .find(|(k, _)| *k == galaxia)
        .and_then(|(_, s)| s.get("P"))
    {
        Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
        _ => panic!("a CPU não deu o P da galáxia"),
    }
}

/// O plano de uma cena (10 quadros pela porta do produto): `(laço na placa, nós encenados, rota)`.
fn plano_de(n: &str, condutores: bool, gpu: &ph2d_gpu::GpuContext) -> Option<(bool, usize, &'static str)> {
    let mut m = monta_demo(n)?;
    m.condutores_na_placa = condutores;
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    ph.play();
    for _ in 0..10 {
        ph.advance_ticks(1);
        crate::motion_bridge::quadro::o_quadro(
            &mut m,
            Some(gpu),
            &ph,
            1.0 / 60.0,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
    }
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let plano = crate::motion_bridge::gpu::plano_do_produto(&mut m, ph.time());
    Some((plano.drives_a_loop(), plano.stages.len(), m.route_said.unwrap_or("—")))
}

/// **A FAMÍLIA** (§0.10): toda cena do roteador com os condutores na placa (hoje) e na CPU — as
/// que mudam de plano, e em especial as que só têm o laço na placa com eles na CPU.
#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_o_condutor_que_expulsa_o_laco() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter");
    let mut mudam = 0;
    for n in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let n = n.to_string();
        let (Some(hoje), Some(cpu)) = (plano_de(&n, true, &gpu), plano_de(&n, false, &gpu)) else {
            continue;
        };
        if hoje.0 != cpu.0 || hoje.1 != cpu.1 || hoje.2 != cpu.2 {
            mudam += 1;
            let marca = if cpu.0 && !hoje.0 { "⛔ LAÇO EXPULSO" } else { "" };
            eprintln!(
                "={n} {marca}\n  hoje laço {} · {} estágios · «{}»\n  cpu  laço {} · {} estágios · «{}»",
                hoje.0, hoje.1, hoje.2, cpu.0, cpu.1, cpu.2
            );
        }
    }
    eprintln!("FAMÍLIA: {mudam} cenas mudam de plano");
}

#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_a_fronteira_do_integrate() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter");
    let (bloco, rodadas) = (
        numero("PH2D_SONDA_BLOCO", 20),
        numero("PH2D_SONDA_RODADAS", 7),
    );
    let escalas: Vec<f32> = std::env::var("PH2D_SONDA_ESCALAS")
        .unwrap_or_else(|_| "1,4,11".into())
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "FRONTEIRA DO INTEGRATE ={CENA} · placa «{}» · bloco {bloco} · {rodadas} rodadas · load {}",
        gpu.adapter.get_info().name,
        carga().trim()
    );
    for &(nome, ajuste) in VARIANTES {
        let mut m = monta(ajuste, 1.0);
        diz_o_plano(nome, &mut m, &gpu);
    }

    let dt = 1.0 / 60.0;
    // (quadro inteiro, marcha da bomba, dirigidos)
    let quadro = |m: &mut MotionState, ph: &mut ph2d_core::Playhead| -> [Duration; 3] {
        ph.advance_ticks(1);
        let t0 = Instant::now();
        crate::motion_bridge::quadro::o_quadro(
            m,
            Some(&gpu),
            ph,
            dt,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        [t0.elapsed(), m.fases.marcha, m.fases.dirigidos]
    };
    let us = |d: Duration| d.as_secs_f64() * 1e6;
    let mut linhas: Vec<(&str, Ajuste, bool)> = vec![("A·processador", |_| {}, false)];
    linhas.extend(VARIANTES.iter().map(|&(n, a)| (n, a, true)));
    for &k in &escalas {
        let mut ms: Vec<MotionState> = linhas
            .iter()
            .map(|&(_, a, placa)| {
                let mut m = monta(a, k);
                m.gpu_enabled = placa;
                m
            })
            .collect();
        let mut phs: Vec<ph2d_core::Playhead> = ms
            .iter()
            .map(|_| {
                let mut p = ph2d_core::Playhead::new(dt);
                p.play();
                p
            })
            .collect();
        for _ in 0..30 {
            for (m, p) in ms.iter_mut().zip(phs.iter_mut()) {
                quadro(m, p);
            }
        }
        let n = ms.len();
        let mut amostras: Vec<Vec<[Duration; 3]>> = vec![Vec::new(); n];
        for r in 0..rodadas {
            for i in (0..n).map(|j| (j + r) % n) {
                let mut soma = [Duration::ZERO; 3];
                for _ in 0..bloco {
                    let q = quadro(&mut ms[i], &mut phs[i]);
                    for c in 0..3 {
                        soma[c] += q[c];
                    }
                }
                amostras[i].push(soma.map(|d| d / bloco as u32));
            }
        }
        let resumo = |v: &[[Duration; 3]], c: usize| {
            let mut x: Vec<Duration> = v.iter().map(|a| a[c]).collect();
            x.sort();
            (us(x[0]), us(x[x.len() / 2]))
        };
        let (b_min, _) = resumo(&amostras[1], 0);
        for (i, &(nome, _, _)) in linhas.iter().enumerate() {
            let (q_min, q_med) = resumo(&amostras[i], 0);
            let (m_min, _) = resumo(&amostras[i], 1);
            let (d_min, _) = resumo(&amostras[i], 2);
            eprintln!(
                "={CENA} ×{k} {nome:<24} quadro min {q_min:>8.0} µs (med {q_med:>8.0}) · marcha {m_min:>7.0} · dirigidos {d_min:>5.0} · /B {:.3} · Δ {:+.0} µs · rota «{}»",
                q_min / b_min,
                q_min - b_min,
                ms[i].route_said.unwrap_or("—")
            );
        }
    }
    eprintln!("load no fim {}", carga().trim());
}
