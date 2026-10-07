//! **O nascimento por PULSO do `sim.spawn` na placa** (doc 110 §14.1 (7)) — três cadeias, e em cada
//! tique a CORRENTE inteira do spawn comparada: a contagem, o CONJUNTO de colunas (uma a mais
//! também diverge — o `age` que o recém-nascido não herda) e cada valor. Ids, linhas e colunas
//! herdadas ao bit; o `vel` do empurrão dentro do ε do seno parabólico da casa (a placa pode
//! fundir `a·b + c`; o `motion.emitter` vive com o mesmo).

use super::gpu_stream_ops::try_headless_gpu;
use ph2d_gpu_cook::{CookClock, GpuCook, plan};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DT: f64 = 1.0 / 60.0;
/// O ε do `vel` (o seno parabólico, `|v| ≤ 1`, vezes `burst_speed`).
const EPS_VEL: f32 = 1e-5;
/// A metade de cima do espaço de ids — a dos nascidos do pulso (`PULSE_ID_BASE`).
const PULSE_ID_BASE: f32 = 8_388_608.0;

fn registry() -> NodeRegistry {
    let mut reg = super::gpu_stream_ops::registry();
    ph2d_node_pulse_beat::register(&mut reg).unwrap();
    reg
}

fn fio(g: &mut Graph, from: (NodeId, u16), to: (NodeId, u16), delayed: bool) {
    g.connect(Edge { from, to, delayed })
        .expect("fio bem formado");
}

/// `grid → beat → spawn.pulse` e `grid → spawn.template`. Devolve `(spawn, beat)`.
fn metronomo(g: &mut Graph, burst: f32, probability: f32, speed: f32) -> (NodeId, Option<NodeId>) {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 4.0);
    g.set_param(grid, "cols", 5.0);
    g.set_param(grid, "gap_x", 0.37);
    let bt = g.add_node("pulse.beat");
    g.set_param(bt, "period", 0.0537);
    g.set_param(bt, "phase_stagger", 0.0031);
    fio(g, (grid, 0), (bt, 0), false);
    fio(g, (bt, 0), (bt, 1), true);
    let sp = g.add_node("sim.spawn");
    g.set_param(sp, "rate", 37.0);
    g.set_param(sp, "burst", burst);
    g.set_param(sp, "probability", probability);
    g.set_param(sp, "burst_speed", speed);
    g.set_param(sp, "seed", 5.0);
    fio(g, (grid, 0), (sp, 0), false);
    fio(g, (bt, 0), (sp, 1), false);
    (sp, Some(bt))
}

/// Os FOGOS: `emitter → lifetime`, os mortos são o modelo e a morte é o pulso.
fn fogos(g: &mut Graph) -> (NodeId, Option<NodeId>) {
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", 300.0);
    g.set_param(em, "life", 60.0);
    let lt = g.add_node("sim.lifetime");
    g.set_param(lt, "life", 0.5);
    g.set_param(lt, "variance", 0.5);
    fio(g, (em, 0), (lt, 0), false);
    let sp = g.add_node("sim.spawn");
    g.set_param(sp, "rate", 0.0);
    g.set_param(sp, "burst", 3.0);
    g.set_param(sp, "probability", 0.6);
    g.set_param(sp, "burst_speed", 0.7);
    fio(g, (lt, 1), (sp, 0), false);
    fio(g, (lt, 2), (sp, 1), false);
    (sp, None)
}

/// Marcha as DUAS rotas `ticks` tiques e compara a corrente do `sp` em cada um. Devolve quantos
/// nasceram do pulso e quantos da taxa, somados (o controlo de não-vacuidade é de quem chama).
///
/// ⚠️ **O fio da navalha do METRÓNOMO não é do spawn.** O `pulse.beat` declara a divergência dele
/// (a placa conta o instante em `f32`): quando uma batida cai a um ULP de um tique, as rotas
/// discordam por um tique INTEIRO. O spawn não tem estado, logo cada tique é independente: onde o
/// pulso que as duas rotas RECEBERAM difere, o tique conta-se e salta-se — e o gate exige que
/// sejam raros e que os comparados tenham nascimentos do pulso.
fn paridade(
    rotulo: &str,
    monta: impl Fn(&mut Graph) -> (NodeId, Option<NodeId>),
    ticks: u64,
) -> (usize, usize) {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return (usize::MAX, usize::MAX);
    };
    let reg = registry();
    let mut g = Graph::new();
    let (sp, fonte) = monta(&mut g);
    g.validate(&reg).expect("bem tipado");
    let p = plan(&g, &reg, &reg, sp);
    assert!(p.is_fully_gpu(), "{rotulo}: fronteiras {:?}", p.boundaries);
    let mut cook = Cook::new();
    let mut gc = GpuCook::new();
    gc.retain_streams_for_debug(true);
    let (mut do_pulso, mut da_taxa, mut pior_vel, mut navalha) = (0usize, 0usize, 0.0f32, 0u64);
    for t in 0..=ticks {
        let playhead = t as f64 * DT;
        let cpu = cook.cook(&g, &reg, sp, playhead).expect("cpu");
        let cpu = cpu[0].as_stream().clone();
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &p,
            &[],
            CookClock {
                playhead,
                tick: p.drives_a_loop().then_some(t),
            },
            [0.25, 0.25, 0.75, 0.75],
            [0.4, 0.4],
            SinkStyle::PLAIN,
        )
        .expect("gpu");
        let mesma_entrada = fonte.is_none_or(|bt| {
            let c = cook.cook(&g, &reg, bt, playhead).expect("cpu beat");
            let c = match c[0].as_stream().get("pulse") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            };
            let d = gc.read_column(&gpu, bt, "pulse").unwrap_or_default();
            c.len() == d.len() && c.iter().zip(&d).all(|(a, b)| a.to_bits() == b.to_bits())
        });
        cook.advance_tick(&g, &reg, playhead).expect("cpu tick");
        if !mesma_entrada {
            navalha += 1;
            continue;
        }
        let n = cpu.count();
        let mut nomes_cpu: Vec<String> = cpu.columns().map(|(k, _)| k.clone()).collect();
        nomes_cpu.sort();
        if n == 0 {
            continue;
        }
        let mut nomes_dev = gc.debug_column_names(sp).unwrap_or_default();
        nomes_dev.sort();
        assert_eq!(
            nomes_cpu, nomes_dev,
            "{rotulo}: tique {t} -- o CONJUNTO de colunas"
        );
        for (nome, col) in cpu.columns() {
            match col {
                Column::Scalar(c) => {
                    let d = gc.read_column(&gpu, sp, nome).expect("escalar");
                    assert_eq!(d.len(), n, "{rotulo}: tique {t}, {nome}: contagem");
                    for (k, (a, b)) in c.iter().zip(&d).enumerate() {
                        assert!(
                            a.to_bits() == b.to_bits(),
                            "{rotulo}: tique {t}, {nome}[{k}]: {a} × {b}"
                        );
                    }
                    if nome == "id" {
                        do_pulso += c.iter().filter(|x| **x >= PULSE_ID_BASE).count();
                        da_taxa += c.iter().filter(|x| **x < PULSE_ID_BASE).count();
                    }
                }
                Column::Vec2(c) => {
                    let d = gc.read_column_vec2(&gpu, sp, nome).expect("vec2");
                    assert_eq!(d.len(), n, "{rotulo}: tique {t}, {nome}: contagem");
                    for (k, (a, b)) in c.iter().zip(&d).enumerate() {
                        for l in 0..2 {
                            if nome == "vel" {
                                let e = (a[l] - b[l]).abs();
                                pior_vel = pior_vel.max(e);
                                assert!(
                                    e <= EPS_VEL,
                                    "{rotulo}: tique {t}, vel[{k}]: {a:?} × {b:?}"
                                );
                            } else {
                                assert!(
                                    a[l].to_bits() == b[l].to_bits(),
                                    "{rotulo}: tique {t}, {nome}[{k}]: {a:?} × {b:?}"
                                );
                            }
                        }
                    }
                }
                Column::Vec4(c) => {
                    let d = gc.read_column_vec4(&gpu, sp, nome).expect("vec4");
                    assert!(
                        c.iter()
                            .flatten()
                            .zip(d.iter().flatten())
                            .all(|(a, b)| a.to_bits() == b.to_bits()),
                        "{rotulo}: tique {t}, {nome}"
                    );
                }
                Column::Vec3(_) => panic!("{rotulo}: nenhuma fixtura leva Vec3"),
            }
        }
    }
    eprintln!(
        "{rotulo}: {do_pulso} do pulso · {da_taxa} da taxa · pior |Δvel| {pior_vel:e} · \
         {navalha} tique(s) no fio da navalha do metrónomo"
    );
    assert!(
        navalha * 20 <= ticks,
        "{rotulo}: {navalha} tiques no fio da navalha — a fixtura vive nele"
    );
    (do_pulso, da_taxa)
}

/// ⭐ O metrónomo, uma irmã por disparo, sem sorteio nem empurrão.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_pulse_births_the_same_ids_at_the_same_rows() {
    let (pulso, taxa) = paridade("metrónomo", |g| metronomo(g, 1.0, 1.0, 0.0), 90);
    assert!(
        pulso > 0 && taxa > 0,
        "fixtura: nasceram {pulso} do pulso e {taxa} da taxa"
    );
}

/// ⭐⭐ Rajada de três, de cada dez pegam seis, e o empurrão — a busca de posto e o `vel`.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_pulse_burst_draws_and_kicks_the_same_survivors() {
    let (pulso, taxa) = paridade("rajada", |g| metronomo(g, 3.0, 0.6, 0.8), 90);
    assert!(
        pulso > 0 && taxa > 0,
        "fixtura: nasceram {pulso} do pulso e {taxa} da taxa"
    );
}

/// ⭐⭐⭐ **Os FOGOS** — o `died` é o modelo, o `pulse` da morte dispara: o complemento do
/// `Compact` (§14.1 (1)) e o nascimento por pulso juntos. O modelo traz `age`, e o recém-nascido
/// NÃO a herda (é o conjunto de colunas que o apanha).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_fireworks_are_born_on_the_device_as_on_the_cpu() {
    let (pulso, _) = paridade("fogos", fogos, 120);
    assert!(pulso > 0, "fixtura: ninguém nasceu de uma morte em 2 s");
}
