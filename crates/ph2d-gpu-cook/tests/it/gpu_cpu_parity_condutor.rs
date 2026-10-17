//! **O CONDUTOR NA PLACA** (doc 110 §14.1 (2)) — um fio cujo condutor a placa coze entrega o
//! número ao uniform do consumidor SEM passar pela CPU: o plano encena o condutor antes do
//! consumidor, e o sequenciador copia 4 bytes (`v[0]`). Aqui a CPU NÃO entrega valor nenhum
//! (`set_driven` nunca é chamado): se a placa concorda com a CPU, foi ela que leu o fio.

use super::placa::try_headless_gpu;
use ph2d_gpu_cook::{CookClock, DrivenParams, GpuSource, plan, plan_with_device_drivers};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const EPS: f32 = 1e-4;

/// A coluna `size` do sink nas duas rotas, num instante: `(cpu, placa)`.
type Par = (Vec<[f32; 2]>, Vec<[f32; 2]>);
const INSTANTES: [f64; 4] = [0.1, 0.37, 0.8, 1.3];

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_scale::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    ph2d_node_value_attribute::register(&mut reg).unwrap();
    ph2d_node_value_reduce::register(&mut reg).unwrap();
    ph2d_node_value_math::register(&mut reg).unwrap();
    reg
}

fn liga(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

/// `grid → scale → output`, o `amount` do `scale` dirigido por `condutor` (montado por quem chama).
fn cadeia(condutor: impl Fn(&mut Graph) -> NodeId) -> (Graph, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    let mv = g.add_node("motion.scale");
    g.set_param(mv, "amount", 0.25); // o override: o que um condutor VAZIO deixa
    let out = g.add_node("motion.output");
    liga(&mut g, grid, mv);
    liga(&mut g, mv, out);
    let c = condutor(&mut g);
    g.drive_param(mv, "amount", (c, 0)).unwrap();
    (g, out, mv, c)
}

fn lfo(amp: f32) -> impl Fn(&mut Graph) -> NodeId {
    move |g: &mut Graph| {
        let l = g.add_node("value.lfo");
        g.set_param(l, "amplitude", amp);
        l
    }
}

/// ⭐ **O condutor é estágio ANTES do consumidor**, e a porta antiga continua a lei antiga.
#[test]
fn the_device_driver_is_staged_before_its_consumer() {
    let reg = registry();
    let (g, out, mv, l) = cadeia(lfo(0.8));
    let p = plan_with_device_drivers(&g, &reg, &reg, &[out], &DrivenParams::new());
    assert!(p.is_fully_gpu(), "fronteiras: {:?}", p.boundaries);
    let pos = |n: NodeId| p.stages.iter().position(|s| s.node == n).expect("estágio");
    assert!(pos(l) < pos(mv), "o condutor coze antes do consumidor");
    assert_eq!(
        p.device_drivers.get(&mv),
        Some(&vec![("amount".to_string(), l)])
    );
    assert!(
        !p.stages
            .iter()
            .any(|s| s.inputs.contains(&GpuSource::Stage(l))),
        "o condutor não alimenta porta nenhuma: só o uniform"
    );
    // ⛔ A porta ANTIGA, sem mapa, é a lei de antes da W1a — ao bit.
    assert!(!plan(&g, &reg, &reg, out).is_fully_gpu());
    assert!(ph2d_gpu_cook::device_driven_params(&g, &reg, &reg).contains(&(mv, "amount".into())));
}

/// ⛔ **Um param que uma lei do hospedeiro lê NÃO vai pela placa** — o `rows` de uma grelha é a
/// CONTAGEM dela: a placa leria o fio e o hospedeiro o default, e as rotas desenhariam outra cena.
#[test]
fn a_param_a_host_law_reads_is_never_device_driven() {
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    liga(&mut g, grid, out);
    let l = lfo(3.0)(&mut g);
    g.drive_param(grid, "rows", (l, 0)).unwrap();
    assert!(ph2d_gpu_cook::device_driven_params(&g, &reg, &reg).is_empty());
    let p = plan_with_device_drivers(&g, &reg, &reg, &[out], &DrivenParams::new());
    assert!(!p.is_fully_gpu(), "sem o número da CPU a grelha recua");
    // E um kernel com VARIANTES (o `motion.move` escolhe-a por um param, no hospedeiro) também
    // fica de fora: o critério é estrutural, não uma lista de nós.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    liga(&mut g, grid, mv);
    let l = lfo(0.5)(&mut g);
    g.drive_param(mv, "dx", (l, 0)).unwrap();
    assert!(ph2d_gpu_cook::device_driven_params(&g, &reg, &reg).is_empty());
}

/// As duas rotas, sem a CPU entregar valor nenhum à placa: a coluna `P` do sink em cada instante.
fn nas_duas(g: &Graph, reg: &NodeRegistry, out: NodeId) -> Vec<Par> {
    let gpu = try_headless_gpu().expect("placa");
    let p = plan_with_device_drivers(g, reg, reg, &[out], &DrivenParams::new());
    assert!(p.is_fully_gpu(), "fronteiras: {:?}", p.boundaries);
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    INSTANTES
        .iter()
        .map(|&t| {
            let mut cook = Cook::new();
            let cpu = cook.cook(g, reg, out, t).expect("cpu");
            let cpu = match cpu[0].as_stream().get("size") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => panic!("sem size"),
            };
            gc.cook(
                &gpu,
                g,
                reg,
                reg,
                &p,
                &[],
                CookClock::at(t),
                [0.25, 0.25, 0.75, 0.75],
                [0.4, 0.4],
                SinkStyle::PLAIN,
            )
            .expect("gpu");
            (cpu, gc.read_column_vec2(&gpu, out, "size").expect("size"))
        })
        .collect()
}

fn pior(pares: &[Par]) -> f32 {
    pares
        .iter()
        .flat_map(|(a, b)| {
            assert_eq!(a.len(), b.len());
            a.iter()
                .zip(b)
                .map(|(x, y)| (x[0] - y[0]).abs().max((x[1] - y[1]).abs()))
        })
        .fold(0.0, f32::max)
}

/// ⭐⭐⭐ **A placa lê o fio que ELA coze**, e concorda com a CPU — com o controlo de que mudar
/// o condutor muda o campo (senão as duas rotas estariam a ignorar o fio da mesma maneira).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_device_driver_reaches_the_uniform_without_the_cpu() {
    if try_headless_gpu().is_none() {
        return;
    }
    let reg = registry();
    let (g, out, _, _) = cadeia(lfo(0.8));
    let a = nas_duas(&g, &reg, out);
    assert!(pior(&a) < EPS, "as rotas divergem em {}", pior(&a));
    let (g2, out2, _, _) = cadeia(lfo(0.2));
    let b = nas_duas(&g2, &reg, out2);
    let mudou = a
        .iter()
        .zip(&b)
        .flat_map(|((_, x), (_, y))| x.iter().zip(y).map(|(p, q)| (p[0] - q[0]).abs()))
        .fold(0.0, f32::max);
    assert!(
        mudou > 1e-2,
        "mudar o condutor tem de mudar o campo ({mudou})"
    );
}

/// ⭐⭐ **O condutor CARO** — a média do comprimento de `P` sobre uma grelha que se MEXE: o que
/// custava `0,155 ms` por quadro na CPU (doc 110 §14.2 (2)) corre agora na placa, inteiro.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn an_expensive_driver_reaches_the_uniform_without_the_cpu() {
    if try_headless_gpu().is_none() {
        return;
    }
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 12.0);
    g.set_param(grid, "cols", 12.0);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "amplitude", 0.4);
    g.set_param(osc, "frequency", 0.7);
    liga(&mut g, grid, osc);
    let attr = g.add_node("value.attribute");
    g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, "P");
    g.set_param(attr, "mode", 1.0);
    liga(&mut g, osc, attr);
    let media = g.add_node("value.reduce");
    g.set_param(media, "mode", 1.0);
    liga(&mut g, attr, media);
    let esc = g.add_node("motion.scale");
    liga(&mut g, osc, esc);
    g.drive_param(esc, "amount", (media, 0)).unwrap();
    let out = g.add_node("motion.output");
    liga(&mut g, esc, out);
    let p = plan_with_device_drivers(&g, &reg, &reg, &[out], &DrivenParams::new());
    assert_eq!(
        p.device_drivers.get(&esc),
        Some(&vec![("amount".to_string(), media)])
    );
    let pares = nas_duas(&g, &reg, out);
    assert!(pior(&pares) < EPS, "as rotas divergem em {}", pior(&pares));
}

/// ⭐ **Um condutor VAZIO deixa o override, nas duas rotas** — o `value.math` desligado não dá
/// número nenhum; a CPU cai no override (`0,25`), e a placa, sem nada a copiar, também.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn an_empty_device_driver_leaves_the_override_like_the_cpu() {
    if try_headless_gpu().is_none() {
        return;
    }
    let reg = registry();
    let (g, out, _, _) = cadeia(|g: &mut Graph| g.add_node("value.math"));
    let pares = nas_duas(&g, &reg, out);
    assert!(pior(&pares) < EPS, "as rotas divergem em {}", pior(&pares));
}

/// A galáxia da `=139`: `grid → integrate ⇢pre⇢ falloff → vortex → attractor → integrate.forces`,
/// com o `strength` do redemoinho dirigido por `map_range ← adsr` (laço próprio). `gatilho` liga
/// ao envelope um `value.input` de gatilho disparado por um LFO — uma fronteira TEMPORAL.
/// Devolve `(grafo, saída, integrate, vortex)`.
fn galaxia_com_envelope(gatilho: bool) -> (Graph, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let fio = |g: &mut Graph, a: (NodeId, u16), b: NodeId, porta: u16, pre: bool| {
        g.connect(Edge {
            from: a,
            to: (b, porta),
            delayed: pre,
        })
        .unwrap();
    };
    let grid = g.add_node("motion.grid");
    let ig = g.add_node("motion.integrate");
    let nucleo = g.add_node("motion.falloff");
    let vx = g.add_node("force.vortex");
    let ima = g.add_node("force.attractor");
    let out = g.add_node("motion.output");
    fio(&mut g, (grid, 0), ig, 0, false);
    fio(&mut g, (ig, 0), nucleo, 0, true);
    fio(&mut g, (nucleo, 0), vx, 0, false);
    fio(&mut g, (vx, 0), ima, 0, false);
    fio(&mut g, (ima, 0), ig, 1, false);
    fio(&mut g, (ig, 0), out, 0, false);
    let env = g.add_node("pulse.adsr");
    fio(&mut g, (env, 0), env, 1, true);
    let forca = g.add_node("value.map_range");
    fio(&mut g, (env, 0), forca, 0, false);
    g.drive_param(vx, "strength", (forca, 0)).unwrap();
    if gatilho {
        let quadrado = g.add_node("value.lfo");
        g.set_param(quadrado, "wave", 2.0);
        let batida = g.add_node("value.input");
        g.set_param(batida, "kind", 2.0);
        g.drive_param(batida, "fire", (quadrado, 0)).unwrap();
        fio(&mut g, (batida, 5), env, 0, false);
    }
    (g, out, ig, vx)
}

/// ⛔⛔ **Um condutor cuja cadeia deixa uma fronteira TEMPORAL não vai para a placa** (doc 120
/// §8.3.3): ela fazia o recuo expulsar o laço INTEIRO da `=139` — o `Integrate` da galáxia na CPU
/// por causa do gatilho do envelope, `~5×` o quadro a ×11. A CPU dá o número por tique, e o laço
/// fica na placa. ⚠️ O controlo: sem o gatilho a mesma cadeia continua condutor na placa.
#[test]
fn a_temporal_driver_chain_stays_on_the_cpu_and_keeps_the_loop_on_the_device() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let forca = |vx: NodeId| -> DrivenParams {
        [(vx, [("strength".to_string(), Some(0.8))].into())].into()
    };
    // O controlo: sem fronteira, o condutor coze na placa e o laço também.
    let (g, out, ig, vx) = galaxia_com_envelope(false);
    assert!(ph2d_gpu_cook::device_driven_params(&g, &reg, &reg).contains(&(vx, "strength".into())));
    let p = plan_with_device_drivers(&g, &reg, &reg, &[out], &DrivenParams::new());
    assert!(
        p.drives_a_loop() && p.device_drivers.contains_key(&vx),
        "{p:?}"
    );
    // Com o gatilho: o número vem da CPU, e o `Integrate` continua estágio.
    let (g, out, ig2, vx) = galaxia_com_envelope(true);
    assert_eq!(ig, ig2);
    assert!(
        !ph2d_gpu_cook::device_driven_params(&g, &reg, &reg).contains(&(vx, "strength".into())),
        "a CPU tem de cozer o condutor que a placa recusa"
    );
    let p = plan_with_device_drivers(&g, &reg, &reg, &[out], &forca(vx));
    assert!(p.device_drivers.is_empty(), "{:?}", p.device_drivers);
    assert!(
        p.drives_a_loop(),
        "o laço saiu da placa: fronteiras {:?}",
        p.boundaries
    );
    assert!(p.stages.iter().any(|s| s.node == ig));
    assert!(p.boundaries.is_empty(), "{:?}", p.boundaries);
    // ⛔ E o condutor COM ESTADO (o envelope lê o seu `pre`) vai na lista que a marcha avança: cozido
    // à parte, o estado dele não andava e a galáxia girava sem a batida.
    let (src, _) = *g.param_sources(vx).unwrap().get("strength").unwrap();
    assert_eq!(p.condutores_na_marcha, vec![src]);
}

/// ⛔ **Uma fronteira de porta ≠ 0 recua quem a lê** — a entrega da bomba é a saída `0` de cada nó,
/// e a porta `5` (o pulso) de um gatilho chegaria à placa como a coluna da porta `0`.
#[test]
fn a_side_port_boundary_recedes_its_reader() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut g = Graph::new();
    let batida = g.add_node("value.input");
    g.set_param(batida, "kind", 2.0);
    let env = g.add_node("pulse.adsr");
    let out = g.add_node("motion.output");
    for (a, b, porta) in [((batida, 5), env, 0), ((env, 0), out, 0)] {
        g.connect(Edge {
            from: a,
            to: (b, porta),
            delayed: false,
        })
        .unwrap();
    }
    let p = plan_with_device_drivers(&g, &reg, &reg, &[out], &DrivenParams::new());
    assert!(!p.stages.iter().any(|s| s.node == env), "{p:?}");
    assert!(
        p.boundaries.iter().all(|(_, porta)| *porta == 0),
        "{:?}",
        p.boundaries
    );
    // O controlo: pela porta `0` o envelope encena-se, com a fronteira na porta `0`.
    g.disconnect(env, 0).unwrap();
    g.connect(Edge {
        from: (batida, 0),
        to: (env, 0),
        delayed: false,
    })
    .unwrap();
    let p = plan_with_device_drivers(&g, &reg, &reg, &[out], &DrivenParams::new());
    assert!(p.stages.iter().any(|s| s.node == env), "{p:?}");
}
