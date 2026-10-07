//! ⭐⭐⭐ **O RELÓGIO DO CICLO 6** (doc 110 §14.1 (5)) — o que a sonda apagada no §6 fingia medir.
//!
//! ⛔ A anterior (`probe_the_price_of_driving_one_param`) corria o cozedor da CPU dos DOIS lados e
//! devolvia um `1,21×` plausível. Esta mede o PRODUTO: cada variante é um `MotionState` com a
//! cena montada, e cada quadro passa pela MESMA porta que a ponte do editor chama
//! ([`crate::motion_bridge::quadro::coze_o_quadro`]) — a placa ou a bomba, conforme o estado
//! decide —, com a espera pela placa dentro da régua (`poll` até acabar).
//!
//! O método é o do [MEDIR_VELOCIDADE](../../../docs/DevOps/MEDIR_VELOCIDADE.md): variantes no MESMO
//! processo, aquecidas fora da régua, intercaladas em BLOCOS por ordem RODADA, resumo = o MÍNIMO
//! das rodadas com a mediana ao lado. ⭐ **O controlo:** cada variante imprime a rota que a ponte
//! registou (`route_said`), e a metade «placa» tem de ler a rota da placa — senão a linha não conta.
//!
//! ```text
//! __GL_SHADER_DISK_CACHE=0 cargo test -p ph2d-app-motion --lib --profile smoke -- --ignored --nocapture sonda_o_relogio_do_ciclo_6
//! ```
//! `PH2D_SONDA_BLOCO=20` (quadros por bloco) · `PH2D_SONDA_RODADAS=7`.

use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DT: f64 = 1.0 / 60.0;
/// Quadros de aquecimento (pipelines, pool, memo) fora da régua.
const AQUECE: usize = 30;

fn numero(var: &str, omissao: usize) -> usize {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(omissao)
}

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16) {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: false,
    })
    .expect("as portas encaixam");
}

/// A grelha no tecto (`LADO_MAX_DE_GRELHA²`), a mexer-se — sem movimento o memo da CPU responde.
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

/// As cenas: um nome e quem a monta (devolve as saídas).
type Monta = fn(&mut MotionState) -> Vec<NodeId>;

/// A `=116` do produto: o pano no tecto e UM fio (`value.lfo → motion.scale.amount`).
fn pano_com_fio(m: &mut MotionState) -> Vec<NodeId> {
    crate::motion_demo_legend::monta("116", &mut m.doc, &m.registry).0
}

/// A MESMA `=116` sem o fio — o `amount` fica no valor que o fio daria no instante 0.
fn pano_sem_fio(m: &mut MotionState) -> Vec<NodeId> {
    let sinks = pano_com_fio(m);
    let fios: Vec<(NodeId, String)> = m
        .doc
        .graph
        .all_param_sources()
        .iter()
        .flat_map(|(n, ps)| ps.keys().map(move |p| (*n, p.clone())))
        .collect();
    assert_eq!(fios.len(), 1, "a `=116` tem UM fio");
    for (n, p) in fios {
        m.doc.graph.undrive_param(n, &p);
        m.doc.graph.set_param(n, &p, 1.0);
    }
    sinks
}

/// Um condutor CARO (item 2): o `amount` de um `motion.scale` dirigido pela MÉDIA do
/// comprimento de `P` sobre a grelha VIVA — a CPU tem de cozer a grelha inteira para o dar.
fn condutor_caro(m: &mut MotionState) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (_, osc) = grelha_viva(g);
    let attr = g.add_node("value.attribute");
    g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, "P");
    g.set_param(attr, "mode", 1.0); // o comprimento
    liga(g, osc, attr, 0);
    let media = g.add_node("value.reduce");
    g.set_param(media, "mode", 1.0); // Mean
    liga(g, attr, media, 0);
    let esc = g.add_node("motion.scale");
    liga(g, osc, esc, 0);
    g.drive_param(esc, "amount", (media, 0)).expect("o fio");
    let out = g.add_node("motion.output");
    liga(g, esc, out, 0);
    vec![out]
}

/// O controlo do condutor caro: o MESMO grafo com o condutor CONSTANTE.
fn condutor_constante(m: &mut MotionState) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (_, osc) = grelha_viva(g);
    let num = g.add_node("value.number");
    let esc = g.add_node("motion.scale");
    liga(g, osc, esc, 0);
    g.drive_param(esc, "amount", (num, 0)).expect("o fio");
    let out = g.add_node("motion.output");
    liga(g, esc, out, 0);
    vec![out]
}

/// O cursor (item 4) a mandar num `motion.drive` sobre a grelha viva — com a porta `in` ligada
/// (`N` cópias atravessam a costura) ou solta (`1`, difundido do outro lado).
fn cursor(m: &mut MotionState, com_in: bool) -> Vec<NodeId> {
    m.doc.graph = Graph::new();
    let g = &mut m.doc.graph;
    let (grid, osc) = grelha_viva(g);
    let cur = g.add_node("value.cursor");
    if com_in {
        liga(g, grid, cur, 0);
    }
    let drv = g.add_node("motion.drive");
    g.set_param(drv, "channel", 1.0);
    liga(g, osc, drv, 0);
    liga(g, cur, drv, 1);
    let out = g.add_node("motion.output");
    liga(g, drv, out, 0);
    vec![out]
}

fn cursor_com_in(m: &mut MotionState) -> Vec<NodeId> {
    cursor(m, true)
}

fn cursor_sem_in(m: &mut MotionState) -> Vec<NodeId> {
    cursor(m, false)
}

/// Uma variante: a cena, e os dois interruptores do estado (a placa · os fios na placa).
struct Variante {
    nome: &'static str,
    monta: Monta,
    placa: bool,
    fios_na_placa: bool,
    condutores_na_placa: bool,
}

const VARIANTES: &[Variante] = &[
    Variante {
        nome: "condutor caro · CPU (W1a)",
        monta: condutor_caro,
        placa: true,
        fios_na_placa: true,
        condutores_na_placa: false,
    },
    Variante {
        nome: "fio · placa",
        monta: pano_com_fio,
        placa: true,
        fios_na_placa: true,
        condutores_na_placa: true,
    },
    Variante {
        nome: "fio · fios=0",
        monta: pano_com_fio,
        placa: true,
        fios_na_placa: false,
        condutores_na_placa: true,
    },
    Variante {
        nome: "fio · CPU",
        monta: pano_com_fio,
        placa: false,
        fios_na_placa: true,
        condutores_na_placa: true,
    },
    Variante {
        nome: "sem fio · placa",
        monta: pano_sem_fio,
        placa: true,
        fios_na_placa: true,
        condutores_na_placa: true,
    },
    Variante {
        nome: "condutor caro · placa",
        monta: condutor_caro,
        placa: true,
        fios_na_placa: true,
        condutores_na_placa: true,
    },
    Variante {
        nome: "condutor const · placa",
        monta: condutor_constante,
        placa: true,
        fios_na_placa: true,
        condutores_na_placa: true,
    },
    Variante {
        nome: "cursor in=N · placa",
        monta: cursor_com_in,
        placa: true,
        fios_na_placa: true,
        condutores_na_placa: true,
    },
    Variante {
        nome: "cursor in=1 · placa",
        monta: cursor_sem_in,
        placa: true,
        fios_na_placa: true,
        condutores_na_placa: true,
    },
];

/// Um estado com a cena da variante, aquecido.
struct Corrida {
    m: MotionState,
    playhead: ph2d_core::Playhead,
}

impl Corrida {
    fn nova(gpu: &GpuContext, v: &Variante) -> Self {
        let mut m = MotionState::new();
        m.gpu_enabled = v.placa;
        m.driven_gpu = v.fios_na_placa;
        m.condutores_na_placa = v.condutores_na_placa;
        m.sinks = (v.monta)(&mut m);
        let mut c = Corrida {
            m,
            playhead: ph2d_core::Playhead::new(DT),
        };
        for _ in 0..AQUECE {
            c.quadro(gpu);
        }
        c
    }

    /// Um quadro do produto, com a espera pela placa dentro.
    fn quadro(&mut self, gpu: &GpuContext) -> f64 {
        let t = self.playhead.time() + DT;
        self.playhead.seek(t);
        let t0 = std::time::Instant::now();
        crate::motion_bridge::quadro::coze_o_quadro(&mut self.m, gpu, &self.playhead, DT);
        // `PH2D_SONDA_SEM_ESPERA=1`: só o lado da CPU (codificar e submeter), sem a placa.
        if std::env::var_os("PH2D_SONDA_SEM_ESPERA").is_none() {
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        ms
    }
}

#[test]
#[ignore = "sonda de relógio, não um gate — `-- --ignored --nocapture`"]
fn sonda_o_relogio_do_ciclo_6() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        eprintln!("sem placa — a sonda não corre");
        return;
    };
    let bloco = numero("PH2D_SONDA_BLOCO", 20);
    let rodadas = numero("PH2D_SONDA_RODADAS", 7);
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("loadavg (início): {}", carga().trim());
    let mut corridas: Vec<Corrida> = VARIANTES.iter().map(|v| Corrida::nova(&gpu, v)).collect();
    // ⭐ O CONTROLO: a rota que a ponte registou, por variante.
    for (v, c) in VARIANTES.iter().zip(&mut corridas) {
        let rota =
            c.m.route_said
                .unwrap_or("— (a ponte não roteou: CPU por omissão)");
        // E quem coze os condutores: a placa (`device_drivers`) ou a CPU (o mapa dos valores).
        let t = c.playhead.time();
        let na_cpu: usize = crate::motion_bridge::gpu::valores_dirigidos(&mut c.m, t)
            .values()
            .map(|m| m.len())
            .sum();
        let sinks = c.m.sinks.clone();
        let na_placa: usize = crate::motion_bridge::gpu::plano_do_produto_para(&mut c.m, &sinks, t)
            .device_drivers
            .values()
            .map(Vec::len)
            .sum();
        eprintln!(
            "  rota · {:<24} → {rota} · condutores: {na_placa} na placa, {na_cpu} na CPU",
            v.nome
        );
        if v.placa && v.fios_na_placa {
            assert!(
                rota.starts_with("device") || rota.starts_with("hybrid"),
                "{}: a metade «placa» não correu na placa ({rota}) — a linha não conta",
                v.nome
            );
        }
    }
    let mut amostras: Vec<Vec<f64>> = vec![Vec::new(); corridas.len()];
    for r in 0..rodadas {
        for k in 0..corridas.len() {
            let i = (k + r) % corridas.len();
            let ms: f64 = (0..bloco).map(|_| corridas[i].quadro(&gpu)).sum();
            amostras[i].push(ms / bloco as f64);
        }
    }
    eprintln!("loadavg (fim):    {}", carga().trim());
    eprintln!(
        "\n  {:<26} {:>9} {:>9}   (ms por quadro · {rodadas} rodadas × {bloco} quadros)",
        "variante", "mínimo", "mediana"
    );
    for (v, a) in VARIANTES.iter().zip(&mut amostras) {
        a.sort_by(f64::total_cmp);
        eprintln!("  {:<26} {:>9.3} {:>9.3}", v.nome, a[0], a[a.len() / 2]);
    }
}
