//! **SONDA — o `par_build` no QUADRO do produto** (doc 98 §4): um núcleo × a pool, nas cenas em que um
//! nó sem kernel fica na CPU. Mesma porta da ponte do editor, placa esperada dentro da régua; método do
//! `MEDIR_VELOCIDADE.md`. A coluna `marcha` é o prefixo de CPU; o resto do quadro é a placa e a costura.
//!
//! ```text
//! PH2D_GPU=1 __GL_SHADER_DISK_CACHE=0 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib \
//!   --profile smoke -- --ignored --nocapture sonda_par_build_no_quadro
//! ```

use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DT: f64 = 1.0 / 60.0;
const AQUECE: usize = 20;
const BLOCO: usize = 10;

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16) {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: false,
    })
    .expect("as portas encaixam");
}

/// A grelha no tecto do slider, a mexer-se: sem movimento o memo da CPU responde.
fn grelha_viva(g: &mut Graph) -> (NodeId, NodeId) {
    let lado = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", lado);
    g.set_param(grid, "cols", lado);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "amplitude", 0.4);
    g.set_param(osc, "frequency", 0.7);
    g.set_param(osc, "phase_stagger", 0.01);
    liga(g, grid, osc, 0);
    (grid, osc)
}

fn saida(g: &mut Graph, de: NodeId) -> Vec<NodeId> {
    let out = g.add_node("motion.output");
    liga(g, de, out, 0);
    vec![out]
}

fn so_a_placa(m: &mut MotionState) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (_, osc) = grelha_viva(g);
    saida(g, osc)
}

fn randomize(m: &mut MotionState) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (_, osc) = grelha_viva(g);
    let r = g.add_node("motion.randomize");
    g.set_param(r, "channel", 2.0);
    g.set_param(r, "amount", 0.5);
    liga(g, osc, r, 0);
    saida(g, r)
}

fn expressao(m: &mut MotionState) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (_, osc) = grelha_viva(g);
    let (ex, ey) = (
        g.add_node("motion.expression"),
        g.add_node("motion.expression"),
    );
    g.set_text_param(ex, "expr", "x + sin(t + i * 0.01) * 0.1");
    g.set_text_param(ey, "expr", "y + cos(t + i * 0.01) * 0.1");
    let mp = g.add_node("motion.make_point");
    liga(g, osc, ex, 0);
    liga(g, osc, ey, 0);
    liga(g, osc, mp, 0);
    liga(g, ex, mp, 1);
    liga(g, ey, mp, 2);
    saida(g, mp)
}

fn pele(m: &mut MotionState) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (_, osc) = grelha_viva(g);
    let esq = |g: &mut Graph, curva: f32| {
        let s = g.add_node("rig.skeleton");
        g.set_param(s, "joints", 17.0);
        g.set_param(s, "angle", curva);
        let fk = g.add_node("rig.fk");
        liga(g, s, fk, 0);
        fk
    };
    let (rest, posed) = (esq(g, 0.0), esq(g, 8.0));
    let p = g.add_node("rig.skin_deformer");
    liga(g, osc, p, 0);
    liga(g, rest, p, 1);
    liga(g, posed, p, 2);
    saida(g, p)
}

fn step(m: &mut MotionState) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (_, osc) = grelha_viva(g);
    let s = g.add_node("motion.step");
    liga(g, osc, s, 0);
    saida(g, s)
}

fn morph(m: &mut MotionState) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (grid, osc) = grelha_viva(g);
    let o = g.add_node("motion.morph");
    liga(g, osc, o, 0);
    liga(g, grid, o, 1);
    saida(g, o)
}

/// A cena `=34` tal como sai de fábrica (`64² = 4 096`): quatro expressões abaixo do limiar.
fn formula_de_fabrica(m: &mut MotionState) -> Vec<NodeId> {
    crate::motion_state::conferencia_demos::build_formula_force_demo_document(
        &mut m.doc,
        &m.registry,
    )
    .expect("a cena")
}

/// A MESMA cena `=34` no tecto do slider da grelha (`181²`).
fn formula_no_tecto(m: &mut MotionState) -> Vec<NodeId> {
    let sinks = formula_de_fabrica(m);
    let lado = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
    let g = &mut m.doc.graph;
    let grid = g
        .nodes()
        .iter()
        .find(|i| i.type_name == "motion.grid")
        .map(|i| i.id)
        .expect("a grelha");
    g.set_param(grid, "rows", lado);
    g.set_param(grid, "cols", lado);
    sinks
}

type Monta = fn(&mut MotionState) -> Vec<NodeId>;
const CENAS: [(&str, Monta); 8] = [
    ("grelha+osc (toda na placa)", so_a_placa),
    ("randomize matiz 181²", randomize),
    ("2 expressoes+make_point 181²", expressao),
    ("pele 16 ossos 181²", pele),
    ("step 181²", step),
    ("morph 181²", morph),
    ("=34 de fabrica 64²", formula_de_fabrica),
    ("=34 no tecto 181²", formula_no_tecto),
];

struct Corrida {
    m: MotionState,
    playhead: ph2d_core::Playhead,
    marcha: f64,
}

impl Corrida {
    fn nova(gpu: &GpuContext, monta: Monta) -> Self {
        let mut m = MotionState::new();
        m.gpu_enabled = true;
        m.sinks = monta(&mut m);
        let mut c = Corrida {
            m,
            playhead: ph2d_core::Playhead::new(DT),
            marcha: 0.0,
        };
        for _ in 0..AQUECE {
            c.quadro(gpu);
        }
        c
    }

    fn quadro(&mut self, gpu: &GpuContext) -> f64 {
        let t = self.playhead.time() + DT;
        self.playhead.seek(t);
        let t0 = std::time::Instant::now();
        crate::motion_bridge::quadro::coze_o_quadro(&mut self.m, gpu, &self.playhead, DT);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        self.marcha += self.m.fases.marcha.as_secs_f64() * 1e3;
        t0.elapsed().as_secs_f64() * 1e3
    }
}

fn min_med(a: &mut [f64]) -> (f64, f64) {
    a.sort_by(f64::total_cmp);
    (a[0], a[a.len() / 2])
}

#[test]
#[ignore = "sonda: relógio, não um gate — `-- --ignored --nocapture`"]
fn sonda_par_build_no_quadro() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem placa — a sonda não corre");
        return;
    };
    let um = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("pool 1");
    let todos = rayon::ThreadPoolBuilder::new().build().expect("pool");
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "QUADRO placa «{}» · pool {} · load {}",
        gpu.adapter.get_info().name,
        todos.current_num_threads(),
        carga().trim()
    );
    // Duas corridas por cena: cada pool com o seu estado, os dois com os mesmos quadros.
    let so = std::env::var("PH2D_SONDA_CENAS").unwrap_or_default();
    let cenas: Vec<&(&str, Monta)> = CENAS
        .iter()
        .filter(|(nome, _)| so.is_empty() || so.split(',').any(|x| nome.contains(x)))
        .collect();
    let mut corridas: Vec<[Corrida; 2]> = cenas
        .iter()
        .map(|(_, monta)| {
            [
                um.install(|| Corrida::nova(&gpu, *monta)),
                todos.install(|| Corrida::nova(&gpu, *monta)),
            ]
        })
        .collect();
    for ((nome, _), c) in cenas.iter().copied().zip(&corridas) {
        eprintln!(
            "QUADRO rota {nome:<30} → {}",
            c[0].m.route_said.unwrap_or("— (CPU)")
        );
    }
    let rodadas: usize = std::env::var("PH2D_SONDA_RODADAS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);
    let n = corridas.len() * 2;
    let mut amostras = vec![Vec::new(); n];
    let mut marchas = vec![Vec::new(); n];
    for r in 0..rodadas {
        for k in 0..n {
            let i = (k + r) % n;
            let (cena, v) = (i / 2, i % 2);
            let c = &mut corridas[cena][v];
            c.marcha = 0.0;
            let pool = if v == 0 { &um } else { &todos };
            let ms: f64 = pool.install(|| (0..BLOCO).map(|_| c.quadro(&gpu)).sum());
            amostras[i].push(ms / BLOCO as f64);
            marchas[i].push(c.marcha / BLOCO as f64);
        }
    }
    for (k, (nome, _)) in cenas.iter().copied().enumerate() {
        let (s_min, s_med) = min_med(&mut amostras[k * 2]);
        let (p_min, p_med) = min_med(&mut amostras[k * 2 + 1]);
        let (m1, _) = min_med(&mut marchas[k * 2]);
        let (mt, _) = min_med(&mut marchas[k * 2 + 1]);
        eprintln!(
            "QUADRO {nome:<30} 1nucleo {s_min:>7.3} (med {s_med:>7.3}, marcha {m1:>6.3}) · pool {p_min:>7.3} (med {p_med:>7.3}, marcha {mt:>6.3}) · {:>5.2}× no quadro · ganho {:>6.3} ms = {:>5.1} % de 16,67",
            s_min / p_min,
            s_min - p_min,
            (s_min - p_min) / 16.67 * 100.0,
        );
    }
    eprintln!("QUADRO fim · load {}", carga().trim());
}
