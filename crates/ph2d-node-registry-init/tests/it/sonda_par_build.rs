//! **SONDA — o cook de CPU dos nós por-elemento: um núcleo × todos, no mesmo processo** (doc 98 §4).
//!
//! As variantes são a pool do rayon em execução (`num_threads(1)` × a omissão) em volta do MESMO
//! `Cook`, intercaladas por ordem rodada; o resumo é o mínimo e a mediana ao lado
//! (`docs/DevOps/MEDIR_VELOCIDADE.md`). `N = 1024` fica abaixo do `PAR_THRESHOLD`: é o controlo A/A.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-node-registry-init --test it --profile smoke \
//!   -- --ignored --nocapture sonda_par_build
//! ```
//! `PH2D_SONDA_NOS=motion.wave,rig.skin_deformer` filtra; `PH2D_SONDA_NS=8192,32768` troca os `N`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::KernelResolver;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::time::Instant;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

macro_rules! fonte_manifest {
    ($ident:ident, $nome:literal, $ty:expr) => {
        static $ident: NodeManifest = NodeManifest {
            id: NodeTypeId::of($nome),
            name: $nome,
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: $ty,
            }],
            effect: Effect::Temporal,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
    };
}
fonte_manifest!(F_A, "sonda.a", INST_VEC2);
fonte_manifest!(F_B, "sonda.b", INST_VEC2);
fonte_manifest!(F_V, "sonda.v", VALUE);
fonte_manifest!(F_REST, "sonda.rest", INST_VEC2);
fonte_manifest!(F_POSED, "sonda.posed", INST_VEC2);

/// Uma fonte que relê a mesma corrente em cada instante: invalida o memo a jusante e não custa nada
/// (as colunas são `Arc`).
struct Fonte(&'static NodeManifest, Stream);
impl NodeOp for Fonte {
    fn manifest(&self) -> &'static NodeManifest {
        self.0
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(self.1.clone());
    }
}

/// Uma corrente típica de cena: posição, identidade, tamanho, rotação e cor.
fn corrente(n: usize, fase: f32) -> Stream {
    let lado = (n as f32).sqrt().ceil();
    let p = (0..n)
        .map(|i| {
            let (x, y) = ((i as f32) % lado, ((i as f32) / lado).floor());
            [x * 0.37 + fase, y * 0.41 - fase]
        })
        .collect();
    let tint = (0..n)
        .map(|i| {
            let f = (i % 251) as f32 / 251.0;
            [f, 1.0 - f, 0.5 + f * 0.25, 1.0]
        })
        .collect();
    Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("Index", Column::Scalar((0..n).map(|i| i as f32).collect()))
        .with("Count", Column::Scalar(vec![n as f32; n]))
        .with(
            "size",
            Column::Vec2((0..n).map(|i| [1.0 + (i % 7) as f32 * 0.1, 1.0]).collect()),
        )
        .with(
            "rot",
            Column::Scalar((0..n).map(|i| (i % 360) as f32 * 0.5).collect()),
        )
        .with("tint", Column::Vec4(tint))
}

/// Uma corrente de valores `v`, por elemento.
fn valores(n: usize) -> Stream {
    Stream::new(n).with(
        "v",
        Column::Scalar((0..n).map(|i| (i % 97) as f32 / 97.0).collect()),
    )
}

/// Um esqueleto em cadeia de `ossos + 1` juntas, repouso ou posado (rotação `giro` por osso).
fn esqueleto(ossos: usize, giro: f32) -> Stream {
    let m = ossos + 1;
    let p = (0..m)
        .map(|i| {
            let k = i as f32;
            [k * 3.0, k * k * giro * 0.01]
        })
        .collect();
    Stream::new(m)
        .with("P", Column::Vec2(p))
        .with(
            "parent",
            Column::Scalar((0..m).map(|i| i as f32 - 1.0).collect()),
        )
        .with(
            "wrot",
            Column::Scalar((0..m).map(|i| i as f32 * giro).collect()),
        )
}

pub(crate) fn registo(n: usize) -> NodeRegistry {
    let mut r = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut r).expect("registo");
    let fontes: [(&'static NodeManifest, Stream); 5] = [
        (&F_A, corrente(n, 0.0)),
        (&F_B, corrente(n, 1.5)),
        (&F_V, valores(n)),
        (&F_REST, esqueleto(16, 0.0)),
        (&F_POSED, esqueleto(16, 7.0)),
    ];
    for (m, s) in fontes {
        r.register(Box::new(Fonte(m, s))).expect("fonte");
    }
    r
}

/// De onde vem uma porta.
#[derive(Clone, Copy)]
pub(crate) enum De {
    A,
    B,
    V,
    Rest,
    Posed,
    /// A saída do próprio nó, no tique anterior (o laço de estado).
    Laco,
}

/// Um nó sob a sonda: as portas ligadas, os params e o texto.
pub(crate) struct Caso {
    pub rotulo: &'static str,
    pub tipo: &'static str,
    pub portas: &'static [(u16, De)],
    pub params: &'static [(&'static str, f32)],
    pub texto: Option<(&'static str, &'static str)>,
    /// O `N` é o tamanho da grelha do próprio nó (`rows`/`cols`) ou um `count`.
    pub lado: Option<fn(&mut Graph, NodeId, usize)>,
}

fn lado_rc(g: &mut Graph, id: NodeId, n: usize) {
    let l = (n as f32).sqrt().ceil();
    g.set_param(id, "rows", l);
    g.set_param(id, "cols", l);
}
fn conta(g: &mut Graph, id: NodeId, n: usize) {
    g.set_param(id, "count", n as f32);
}

const fn caso(
    rotulo: &'static str,
    tipo: &'static str,
    portas: &'static [(u16, De)],
    params: &'static [(&'static str, f32)],
) -> Caso {
    Caso {
        rotulo,
        tipo,
        portas,
        params,
        texto: None,
        lado: None,
    }
}

/// Todos os candidatos, com params fora do ponto neutro.
pub(crate) fn casos() -> Vec<Caso> {
    vec![
        caso(
            "make_point polar",
            "motion.make_point",
            &[(0, De::A), (1, De::V), (2, De::V)],
            &[("mode", 1.0)],
        ),
        caso(
            "randomize posicao",
            "motion.randomize",
            &[(0, De::A)],
            &[("channel", 6.0), ("amount", 0.5)],
        ),
        caso(
            "randomize matiz",
            "motion.randomize",
            &[(0, De::A)],
            &[("channel", 2.0), ("amount", 0.5)],
        ),
        caso(
            "velocity",
            "motion.velocity",
            &[(0, De::A), (1, De::Laco)],
            &[("smooth", 2.0)],
        ),
        caso(
            "step",
            "motion.step",
            &[(0, De::A), (2, De::Laco)],
            &[("step", 0.5)],
        ),
        caso(
            "delay",
            "motion.delay",
            &[(0, De::A), (1, De::Laco)],
            &[("ticks", 8.0)],
        ),
        caso(
            "morph",
            "motion.morph",
            &[(0, De::A), (1, De::B), (2, De::V)],
            &[],
        ),
        caso(
            "mixer",
            "motion.mixer",
            &[(0, De::A), (1, De::B), (4, De::V)],
            &[("weight_1", 0.5)],
        ),
        caso(
            "sort radial",
            "motion.sort",
            &[(0, De::A)],
            &[("key", 0.0), ("center_x", 3.0)],
        ),
        caso(
            "spline_wrap",
            "motion.spline_wrap",
            &[(0, De::A), (1, De::V)],
            &[
                ("p0x", -3.0),
                ("p1x", -1.0),
                ("p1y", 2.0),
                ("p2x", 1.0),
                ("p2y", -2.0),
                ("p3x", 3.0),
            ],
        ),
        Caso {
            rotulo: "expression",
            tipo: "motion.expression",
            portas: &[(0, De::A)],
            params: &[("a", 0.7)],
            texto: Some(("expr", "sin(i * 0.01 + t) * a + x")),
            lado: None,
        },
        caso(
            "skin_deformer 16 ossos",
            "rig.skin_deformer",
            &[(0, De::A), (1, De::Rest), (2, De::Posed)],
            &[("falloff", 3.0)],
        ),
        Caso {
            rotulo: "wave",
            tipo: "motion.wave",
            portas: &[(1, De::Laco)],
            params: &[("speed", 0.2), ("damping", 0.01)],
            texto: None,
            lado: Some(lado_rc),
        },
        Caso {
            rotulo: "scatter",
            tipo: "motion.scatter",
            portas: &[],
            params: &[],
            texto: None,
            lado: Some(conta),
        },
        Caso {
            rotulo: "distribute_curve",
            tipo: "motion.distribute_curve",
            portas: &[],
            params: &[],
            texto: None,
            lado: Some(conta),
        },
        caso(
            "clone x8 (tem kernel)",
            "motion.clone",
            &[(0, De::A)],
            &[("count", 8.0)],
        ),
        caso(
            "mirror (tem kernel)",
            "motion.mirror",
            &[(0, De::A)],
            &[("offset", 1.0)],
        ),
        caso(
            "trail (tem kernel)",
            "motion.trail",
            &[(0, De::A), (1, De::Laco)],
            &[("length", 8.0)],
        ),
    ]
}

/// O grafo do caso a `n` linhas; devolve o nó medido.
pub(crate) fn monta(c: &Caso, n: usize) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let no = g.add_node(c.tipo);
    for &(porta, de) in c.portas {
        let (from, delayed) = match de {
            De::A => (g.add_node("sonda.a"), false),
            De::B => (g.add_node("sonda.b"), false),
            De::V => (g.add_node("sonda.v"), false),
            De::Rest => (g.add_node("sonda.rest"), false),
            De::Posed => (g.add_node("sonda.posed"), false),
            De::Laco => (no, true),
        };
        g.connect(Edge {
            from: (from, 0),
            to: (no, porta),
            delayed,
        })
        .expect("liga");
    }
    for &(p, v) in c.params {
        g.set_param(no, p, v);
    }
    if let Some((k, v)) = c.texto {
        g.set_text_param(no, k, v);
    }
    if let Some(f) = c.lado {
        f(&mut g, no, n);
    }
    (g, no)
}

/// Um tique: coze e avança. O `seed` varia para um gerador `Pure` sem entradas não cair no memo.
pub(crate) fn tique(
    cook: &mut Cook,
    g: &mut Graph,
    no: NodeId,
    r: &NodeRegistry,
    t: &mut f64,
) -> usize {
    *t += 1.0 / 60.0;
    if g.edges().is_empty() {
        g.set_param(no, "seed", ((*t * 60.0).round() as u32 % 1000) as f32 + 1.0);
    }
    let linhas = cook.cook(g, r, no, *t).expect("cook")[0]
        .as_stream()
        .count();
    cook.advance_tick(g, r, *t).expect("tick");
    linhas
}

/// FNV-1a sobre os bits de todas as colunas, por ordem determinística.
pub(crate) fn impressao(s: &Stream) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut come = |w: u32| {
        for b in w.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    come(s.count() as u32);
    for (nome, col) in s.columns() {
        for b in nome.bytes() {
            come(u32::from(b));
        }
        match col {
            Column::Scalar(v) => v.iter().for_each(|x| come(x.to_bits())),
            Column::Vec2(v) => v.iter().flatten().for_each(|x| come(x.to_bits())),
            Column::Vec3(v) => v.iter().flatten().for_each(|x| come(x.to_bits())),
            Column::Vec4(v) => v.iter().flatten().for_each(|x| come(x.to_bits())),
        }
    }
    h
}

fn min_med(v: &[f64]) -> (f64, f64) {
    let mut o = v.to_vec();
    o.sort_by(f64::total_cmp);
    (o[0], o[o.len() / 2])
}

fn lista(var: &str) -> Option<Vec<String>> {
    std::env::var(var)
        .ok()
        .filter(|v| !v.is_empty())
        .map(|v| v.split(',').map(str::to_owned).collect())
}

#[test]
#[ignore = "censo: lê-se com --nocapture"]
fn sonda_par_build_censo() {
    let r = registo(1);
    let mut sem = Vec::new();
    let mut com = 0;
    for m in r.manifests().filter(|m| !m.name.starts_with("sonda.")) {
        if r.gpu_kernel(m.id).is_some() {
            com += 1;
        } else {
            sem.push(m.name);
        }
    }
    sem.sort_unstable();
    eprintln!("CENSO tipos com kernel {com} · sem kernel {}", sem.len());
    eprintln!("CENSO sem kernel: {}", sem.join(" "));
    let par = |t: &str| r.manifests().any(|m| m.name == t);
    for c in casos() {
        let id = NodeTypeId::of(c.tipo);
        eprintln!(
            "CENSO {:<26} existe={} kernel={}",
            c.rotulo,
            par(c.tipo),
            r.gpu_kernel(id).is_some()
        );
    }
}

#[test]
#[ignore = "sonda: relógio"]
fn sonda_par_build() {
    let ns: Vec<usize> = lista("PH2D_SONDA_NS").map_or_else(
        // `32 768` é o tecto por nó (`MAX_INSTANCIAS_POR_NO`): nenhum gerador passa dele.
        || vec![1024, 8192, 32768],
        |v| v.iter().map(|x| x.parse().expect("N")).collect(),
    );
    let so = lista("PH2D_SONDA_NOS");
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let um = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("pool 1");
    let todos = rayon::ThreadPoolBuilder::new().build().expect("pool");
    const RODADAS: usize = 9;
    const BLOCO: usize = 4;
    eprintln!(
        "SONDA pool {} · {RODADAS} rodadas × {BLOCO} tiques · load {}",
        todos.current_num_threads(),
        carga().trim()
    );
    for c in casos() {
        if so
            .as_ref()
            .is_some_and(|f| !f.iter().any(|x| x == c.rotulo || x == c.tipo))
        {
            continue;
        }
        for &n in &ns {
            let reg = registo(n);
            let (mut g1, no1) = monta(&c, n);
            let (mut gt, not) = monta(&c, n);
            let (mut c1, mut ct) = (Cook::new(), Cook::new());
            let (mut t1, mut tt) = (0.0, 0.0);
            let mut linhas = 0;
            for _ in 0..3 {
                um.install(|| tique(&mut c1, &mut g1, no1, &reg, &mut t1));
                linhas = todos.install(|| tique(&mut ct, &mut gt, not, &reg, &mut tt));
            }
            // Mesmo número de tiques dos dois lados: estado igual, logo saída igual ao bit.
            let (mut a1, mut at) = (Vec::new(), Vec::new());
            for r in 0..RODADAS {
                let mut um_bloco = |serial: bool| {
                    let i = Instant::now();
                    if serial {
                        um.install(|| {
                            for _ in 0..BLOCO {
                                tique(&mut c1, &mut g1, no1, &reg, &mut t1);
                            }
                        });
                        a1.push(i.elapsed().as_secs_f64() * 1e3 / BLOCO as f64);
                    } else {
                        todos.install(|| {
                            for _ in 0..BLOCO {
                                tique(&mut ct, &mut gt, not, &reg, &mut tt);
                            }
                        });
                        at.push(i.elapsed().as_secs_f64() * 1e3 / BLOCO as f64);
                    }
                };
                um_bloco(r % 2 == 0);
                um_bloco(r % 2 == 1);
            }
            let (s_min, s_med) = min_med(&a1);
            let (p_min, p_med) = min_med(&at);
            let igual = {
                let a = impressao(c1.peek(no1).expect("cozido")[0].as_stream());
                let b = impressao(ct.peek(not).expect("cozido")[0].as_stream());
                if a == b {
                    format!("bit={a:016x}")
                } else {
                    format!("DIFERE {a:016x} {b:016x}")
                }
            };
            eprintln!(
                "SONDA {:<24} N={n:<7} linhas={linhas:<8} 1nucleo min {s_min:>9.3} med {s_med:>9.3} · todos min {p_min:>9.3} med {p_med:>9.3} · {:>5.2}× · {:>6.1} % do quadro · {igual}",
                c.rotulo,
                s_min / p_min,
                s_min / 16.67 * 100.0,
            );
        }
    }
    eprintln!("SONDA fim · load {}", carga().trim());
}
