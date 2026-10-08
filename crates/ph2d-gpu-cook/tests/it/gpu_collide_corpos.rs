//! ⭐ **O filtro do CORPO no colisor, nas duas rotas** (doc 124 §0.2, Bug #15).
//!
//! Um par do MESMO corpo (`body` igual e não nulo) cujo REPOUSO (`rest`) já o põe a menos de
//! `r_i + r_j` não colide — o filtro de auto-colisão do FleX. A fixtura tem DOIS corpos dobrados
//! um sobre o outro e linhas sem corpo (`body = 0`) no meio, de modo que as três perguntas — o par
//! filtrado, o par do mesmo corpo longe no repouso, o par de corpos diferentes — acontecem todas.
//!
//! A fonte é um nó de TESTE só de CPU: ela é a fronteira do plano e a mesma corrente entra nas duas
//! rotas (`boundary_streams`), logo o que se compara é SÓ o colisor.
//!
//! `#[ignore]`: precisa de adaptador.

use super::placa::try_headless_gpu;
use ph2d_gpu_cook::{CookClock, GpuCook, plan};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use ph2d_render::SinkStyle;

const RAIO: f32 = 0.28;
const LADO: usize = 4;
const PASSO: f32 = 0.30;

static FIXTURA: NodeManifest = NodeManifest {
    id: NodeTypeId::of("teste.dois_corpos"),
    name: "teste.dois_corpos",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame),
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // `1` apaga a identidade dos corpos (`body = 0`): o CONTROLO de que o filtro muda a resposta.
    params: &[ParamSpec {
        name: "sem_corpo",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// Dois corpos `4 × 4` de passo `0,30` no repouso, ESMAGADOS a `0,6` em `x` (os pares a dois
/// passos ficam a `0,36 < 0,56` — não ligados e sobrepostos), o segundo deslocado por cima do
/// primeiro; e quatro linhas sem corpo entre eles.
fn dois_corpos() -> Stream {
    let (mut p, mut rest, mut body) = (Vec::new(), Vec::new(), Vec::new());
    for (b, [ox, oy]) in [(1.0f32, [0.0f32, 0.0f32]), (2.0, [0.41, 0.13])] {
        for r in 0..LADO {
            for c in 0..LADO {
                let q = [c as f32 * PASSO, r as f32 * PASSO];
                rest.push(q);
                p.push([ox + q[0] * 0.6, oy + q[1] * 0.85]);
                body.push(b);
            }
        }
    }
    for k in 0..4 {
        p.push([0.2 + 0.17 * k as f32, 0.31]);
        rest.push([0.0, 0.0]);
        body.push(0.0);
    }
    Stream::new(p.len())
        .with("P", Column::Vec2(p))
        .with("rest", Column::Vec2(rest))
        .with("body", Column::Scalar(body))
}

struct DoisCorpos;
impl NodeOp for DoisCorpos {
    fn manifest(&self) -> &'static NodeManifest {
        &FIXTURA
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mut s = dois_corpos();
        if ctx.param("sem_corpo") >= 0.5 {
            s.set("body", Column::Scalar(vec![0.0; s.count()]));
        }
        ctx.emit(s);
    }
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(DoisCorpos)).unwrap();
    ph2d_node_motion_collide::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg
}

fn grafo(iteracoes: f32) -> (Graph, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let fx = g.add_node("teste.dois_corpos");
    let col = g.add_node("motion.collide");
    g.set_param(col, "radius", RAIO);
    g.set_param(col, "iterations", iteracoes);
    let out = g.add_node("motion.output");
    for (a, b) in [(fx, col), (col, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    (g, fx, col, out)
}

/// O pior par NÃO filtrado à saída — a régua da lei, sobre a pose que a rota devolve.
fn pior_nao_filtrado(p: &[[f32; 2]], s: &Stream) -> f32 {
    let (Some(Column::Vec2(rest)), Some(Column::Scalar(body))) = (s.get("rest"), s.get("body"))
    else {
        panic!("a fixtura declara rest/body")
    };
    let mut pior = f32::MAX;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let dr = [rest[j][0] - rest[i][0], rest[j][1] - rest[i][1]];
            let filtrado = body[i] != 0.0 && body[i] == body[j] && dr[0].hypot(dr[1]) < 2.0 * RAIO;
            if !filtrado {
                pior = pior.min((p[j][0] - p[i][0]).hypot(p[j][1] - p[i][1]));
            }
        }
    }
    pior
}

#[test]
#[ignore = "precisa de adaptador"]
fn the_body_filter_packs_the_same_on_both_routes() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adaptador — gpu_collide_corpos saltado");
        return;
    };
    let reg = registry();
    let fonte = dois_corpos();
    for (iteracoes, eps) in [(1.0f32, 1e-5f32), (8.0, 2e-3)] {
        let (g, fx, col, out) = grafo(iteracoes);
        let cpu = match Cook::new().cook(&g, &reg, col, 0.0).expect("cpu")[0]
            .as_stream()
            .get("P")
        {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("P"),
        };
        let p = plan(&g, &reg, &reg, out);
        assert_eq!(p.boundaries, vec![(fx, 0)], "a fixtura e' a fronteira");
        let mut gc = GpuCook::new();
        gc.retain_streams_for_debug(true);
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &p,
            &[(fx, &fonte)],
            CookClock {
                playhead: 0.0,
                tick: Some(0),
            },
            [0.25, 0.25, 0.75, 0.75],
            [0.4, 0.4],
            SinkStyle::PLAIN,
        )
        .expect("gpu");
        let dev = gc.read_column_vec2(&gpu, col, "P").expect("P do colisor");
        assert_eq!(cpu.len(), dev.len());
        let mut max = 0.0f32;
        for (c, d) in cpu.iter().zip(&dev) {
            max = max.max((c[0] - d[0]).abs()).max((c[1] - d[1]).abs());
        }
        // ⚠️ O CONTROLO: o filtro tem de MUDAR a resposta — sem a identidade dos corpos a CPU dá
        // outra pose, senão a paridade seria a do colisor antigo com outro nome.
        let mut g_sem = g.clone();
        g_sem.set_param(fx, "sem_corpo", 1.0);
        let sem = match Cook::new().cook(&g_sem, &reg, col, 0.0).expect("cpu")[0]
            .as_stream()
            .get("P")
        {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("P"),
        };
        let mudou = cpu
            .iter()
            .zip(&sem)
            .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
            .fold(0.0f32, f32::max);
        assert!(
            mudou > 1e-2,
            "CONTROLO: o filtro nao mudou a pose ({mudou})"
        );
        eprintln!(
            "corpos {iteracoes} varreduras: max |Δ| = {max:e} · pior nao-filtrado cpu {:.4} gpu {:.4}",
            pior_nao_filtrado(&cpu, &fonte),
            pior_nao_filtrado(&dev, &fonte)
        );
        assert!(max <= eps, "{iteracoes} varreduras: |Δ| {max} > {eps}");
    }
}
