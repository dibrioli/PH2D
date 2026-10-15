//! ⭐ **A RÉGUA DO ANEL contra a memória que ele RETÉM** (doc 132 §7, P2 — o irmão do
//! `tests/fita_memoria.rs`).
//!
//! O que o anel declara ao razão ([`ph2d_eval_motion::razao::dos_aneis`]) é o que o tecto compara; se declara menos do
//! que retém, o tecto é uma fração do que diz ser (medido antes da cura: `0 B` declarados por um
//! ponto de recuo leve, `~1,9 KB` ganhos pelo processo). O heap conta-se com o `dhat`: o retido é o
//! que LARGAR o anel devolve, `bytes + 16 · blocos` — o cabeçalho e o arredondamento do `malloc` —,
//! e o declarado tem de ficar a `±25 %` dele, com estados leves (um número por nó, um nó ou trinta),
//! pesados (milhares de elementos em várias colunas), com memória de nó, sem estado nenhum e com o
//! anel a afinar (acima do `MAX_ENTRIES`).
//!
//! Um único `#[test]`: o contador é global, e testes em paralelo no mesmo binário somariam-se.

use dhat::{HeapStats, Profiler};
use ph2d_eval_motion::{CheckpointRing, Razao};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx, Memo, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::sync::Arc;

#[global_allocator]
static ALOCADOR: dhat::Alloc = dhat::Alloc;

/// O que o `malloc` reserva para o que está vivo agora: o pedido e `16` por bloco.
fn vivos() -> isize {
    let h = HeapStats::get();
    isize::try_from(h.curr_bytes + 16 * h.curr_blocks).expect("cabe")
}

const INST: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

static ESTADO: NodeManifest = NodeManifest {
    id: NodeTypeId::of("anel.test.estado"),
    name: "anel.test.estado",
    inputs: &[PortSpec {
        name: "state",
        ty: INST,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST,
    }],
    effect: ph2d_nodegraph::effect::Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

static MEMORIA: NodeManifest = NodeManifest {
    id: NodeTypeId::of("anel.test.memoria"),
    name: "anel.test.memoria",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST,
    }],
    effect: ph2d_nodegraph::effect::Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// Um nó SEQUENCIAL (`pre` sobre si): `n` elementos em `colunas` colunas, todas novas a cada tique.
struct Estado {
    n: usize,
    colunas: usize,
}

const NOMES: [&str; 4] = ["P", "v", "Cd", "sim_t"];

impl NodeOp for Estado {
    fn manifest(&self) -> &'static NodeManifest {
        &ESTADO
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let antes = match ctx.input(0).get("P") {
            Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
            _ => 0.0,
        };
        let mut s = Stream::new(self.n);
        for nome in NOMES.iter().take(self.colunas) {
            s.set(*nome, Column::Vec2(vec![[antes + 1.0, 0.0]; self.n]));
        }
        ctx.emit(s);
    }
}

/// A memória de um nó: `n` números, declarados como o `Memo` manda (o que a cópia aloca).
#[derive(Clone)]
struct Numeros(Vec<f32>);
impl Memo for Numeros {
    fn approx_bytes(&self) -> usize {
        self.0.len() * size_of::<f32>()
    }
}

/// Um nó com MEMÓRIA (`n` números), sem `pre`.
struct ComMemoria {
    n: usize,
}

impl NodeOp for ComMemoria {
    fn manifest(&self) -> &'static NodeManifest {
        &MEMORIA
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mut m = ctx
            .take_memo::<Numeros>()
            .unwrap_or_else(|| Numeros(vec![0.0; self.n]));
        m.0.iter_mut().for_each(|x| *x += 1.0);
        let x = m.0[0];
        ctx.keep_memo(m);
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]])));
    }
}

struct Ops {
    estado: Estado,
    memoria: ComMemoria,
}

impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == ESTADO.id {
            Some(&self.estado)
        } else if ty == MEMORIA.id {
            Some(&self.memoria)
        } else {
            None
        }
    }
}

/// Um cenário: `nos` nós sequenciais de `n` elementos em `colunas` colunas (`nos == 0`: um nó sem
/// estado), um nó com memória de `memo` números se `memo > 0`, `t` tiques.
struct Cenario {
    nos: usize,
    n: usize,
    colunas: usize,
    memo: usize,
    t: u64,
}

/// O que um cenário mediu.
struct Medida {
    declarado: usize,
    retido: usize,
}

fn grafo(c: &Cenario) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let mut alvo = None;
    for _ in 0..c.nos {
        let e = g.add_node("anel.test.estado");
        g.connect(Edge {
            from: (e, 0),
            to: (e, 0),
            delayed: true,
        })
        .expect("liga");
        alvo.get_or_insert(e);
    }
    if c.memo > 0 {
        alvo = Some(g.add_node("anel.test.memoria"));
    } else if c.nos == 0 {
        // Sem `pre` e sem memória: o ponto de recuo não guarda estado nenhum.
        alvo = Some(g.add_node("anel.test.estado"));
    }
    (g, alvo.expect("um nó"))
}

fn mede(c: &Cenario) -> Medida {
    let ops = Ops {
        estado: Estado {
            n: c.n,
            colunas: c.colunas,
        },
        memoria: ComMemoria { n: c.memo.max(1) },
    };
    let (g, alvo) = grafo(c);
    let razao = Razao::new(usize::MAX);
    let mut cook = Cook::new();
    let mut anel = CheckpointRing::no_razao(Arc::clone(&razao));
    for t in 0..c.t {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        let playhead = t as f64 / 60.0;
        anel.regista(t, &cook);
        cook.cook(&g, &ops, alvo, playhead).expect("coze");
        cook.advance_tick(&g, &ops, playhead).expect("avança");
    }
    let declarado = razao.usado();
    let com = vivos();
    drop(anel);
    let retido = usize::try_from(com - vivos()).expect("o anel retém algo");
    assert_eq!(razao.usado(), 0, "o anel que morre devolve o que declarava");
    drop(cook);
    Medida { declarado, retido }
}

#[test]
fn o_anel_declara_a_memoria_que_retem() {
    let _perfil = Profiler::builder().testing().build();
    let c = |nos, n, colunas, memo, t| Cenario {
        nos,
        n,
        colunas,
        memo,
        t,
    };
    let cenarios = [
        ("leve: 1 nó, 1 número", c(1, 1, 1, 0, 1_000)),
        ("leve: 30 nós, 1 número", c(30, 1, 1, 0, 600)),
        ("leve: 1 nó, 3 colunas de 4", c(1, 4, 3, 0, 1_000)),
        ("sem estado", c(0, 1, 1, 0, 1_000)),
        ("a afinar: 1 nó, 3 000 tiques", c(1, 1, 1, 0, 3_000)),
        ("pesado: 4 096 × 3 colunas", c(1, 4_096, 3, 0, 200)),
        ("pesado: 65 536 × 2 colunas", c(1, 65_536, 2, 0, 40)),
        ("memória de 1 024 números", c(1, 1, 1, 1_024, 800)),
        ("memória de 16 números", c(0, 0, 0, 16, 800)),
    ];
    let mut falhas = Vec::new();
    for (nome, c) in &cenarios {
        let m = mede(c);
        #[expect(clippy::cast_precision_loss, reason = "uma razão")]
        let (d, r) = (m.declarado as f64, m.retido as f64);
        eprintln!(
            "anel {nome}: declara {} B · retém {} B ({:.2}×)",
            m.declarado,
            m.retido,
            r / d.max(1.0)
        );
        if (d - r).abs() > 0.25 * r {
            falhas.push(format!(
                "{nome}: declara {d} B e retém {r} B ({:.2}×)",
                r / d.max(1.0)
            ));
        }
    }
    assert!(
        falhas.is_empty(),
        "a régua do anel mente:\n{}",
        falhas.join("\n")
    );
}
