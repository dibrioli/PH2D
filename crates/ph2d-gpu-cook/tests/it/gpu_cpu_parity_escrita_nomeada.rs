//! ⭐ **PARIDADE CPU↔GPU DA ESCRITA POR NOME** (doc 131 D.14) — o canal `Custom` do
//! `motion.drive` pelo [`StreamOp::WriteNamed`]: a coluna do artista sob o alias do kernel e de
//! volta. Compara TODAS as colunas da saída do último drive, e cada degrau da escada da CPU.
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-gpu-cook --test it -- --ignored \
//!     --nocapture gpu_cpu_parity_escrita_nomeada
//! ```

use super::placa::try_headless_gpu;
use ph2d_gpu_cook::{CookClock, GpuCook, plan};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::gpu::{KernelResolver, StreamOp};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;
use std::collections::BTreeSet;

const ADD: f32 = 0.0;
const SET: f32 = 1.0;
const MUL: f32 = 2.0;
const CUSTOM: f32 = 9.0;
/// Relativo à magnitude: a mistura pela máscara (`b + (c − b)·f`) tem sítios de FMA.
const EPS: f32 = 1e-5;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_motion_drive::register(&mut reg).unwrap();
    ph2d_node_value_attribute::register(&mut reg).unwrap();
    reg
}

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16) {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: false,
    })
    .expect("fio");
}

/// Um drive `Custom`: o nome, o modo, e se a porta de valor está ligada.
type Passo = (&'static str, f32, bool);

/// `grid → [falloff] → drive… → output`, cada drive com o valor `Index · 0,5` (um `value.attribute`
/// sobre a grelha). Devolve o grafo, a grelha, o último drive e a saída.
fn cadeia(passos: &[Passo], falloff: bool) -> (Graph, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let grade = g.add_node("motion.grid");
    g.set_param(grade, "rows", 6.0);
    g.set_param(grade, "cols", 7.0);
    let attr = g.add_node("value.attribute");
    g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, "Index");
    liga(&mut g, grade, attr, 0);
    let mut cauda = grade;
    if falloff {
        let f = g.add_node("motion.falloff");
        g.set_param(f, "radius", 2.0);
        liga(&mut g, cauda, f, 0);
        cauda = f;
    }
    for &(nome, modo, com_valor) in passos {
        let d = g.add_node("motion.drive");
        g.set_param(d, "channel", CUSTOM);
        g.set_param(d, "mode", modo);
        g.set_param(d, "scale", 0.5);
        g.set_text_param(d, ph2d_node_motion_drive::DRIVE_COL_KEY, nome);
        liga(&mut g, cauda, d, 0);
        if com_valor {
            liga(&mut g, attr, d, 1);
        }
        cauda = d;
    }
    let out = g.add_node("motion.output");
    liga(&mut g, cauda, out, 0);
    (g, grade, cauda, out)
}

fn achatada(c: &Column) -> Vec<f32> {
    match c {
        Column::Scalar(v) => v.clone(),
        Column::Vec2(v) => v.concat(),
        Column::Vec3(v) => v.iter().flat_map(|x| [x[0], x[1], x[2], 0.0]).collect(),
        Column::Vec4(v) => v.concat(),
    }
}

fn nomes(s: &Stream) -> Vec<String> {
    s.columns().map(|(n, _)| n.clone()).collect()
}

/// ⭐ **Cada degrau da escada, CPU × placa, todas as colunas.** Com os controlos do lado da CPU: o
/// que escreve escreveu (a coluna nova existe / mudou) e o que recusa devolveu a entrada.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_named_write_matches_the_cpu_on_every_rung() {
    let Some(gpu) = try_headless_gpu() else {
        panic!("sem adapter — este gate mede o dispositivo");
    };
    let reg = registry();
    // (rótulo, passos, falloff, escreve?)
    let casos: [(&str, &[Passo], bool, bool); 9] = [
        ("nome novo", &[("heat", SET, true)], false, true),
        ("escalar existente", &[("Index", ADD, true)], false, true),
        (
            "escrita sobre escrita, com mascara",
            &[("heat", SET, true), ("heat", MUL, true)],
            true,
            true,
        ),
        (
            "o alias ocupado",
            &[("drive_named", SET, true), ("heat", ADD, true)],
            false,
            true,
        ),
        ("recusa: vazio", &[("", SET, true)], false, false),
        ("recusa: id", &[("id", SET, true)], false, false),
        ("recusa: sim_t", &[("sim_t", SET, true)], false, false),
        ("recusa: P e' Vec2", &[("P", SET, true)], false, false),
        ("recusa: sem valor", &[("heat", SET, false)], false, false),
    ];
    for (rotulo, passos, falloff, escreve) in casos {
        let (g, grade, ultimo, out) = cadeia(passos, falloff);
        g.validate(&reg).expect("bem tipada");
        let plano = plan(&g, &reg, &reg, out);
        assert!(plano.is_fully_gpu(), "{rotulo}: {:?}", plano.boundaries);
        let mut cook = Cook::new();
        let cpu: Stream = cook.cook(&g, &reg, ultimo, 0.0).expect("cpu")[0]
            .as_stream()
            .clone();
        let entrada: Stream = cook.cook(&g, &reg, grade, 0.0).expect("cpu")[0]
            .as_stream()
            .clone();
        let alvo = passos.last().expect("um passo").0;
        if escreve {
            assert!(
                cpu.get(alvo) != entrada.get(alvo),
                "{rotulo}: a CPU tem de ter escrito `{alvo}`"
            );
        } else {
            assert_eq!(
                nomes(&cpu),
                nomes(&entrada),
                "{rotulo}: a recusa devolve a entrada"
            );
        }
        let mut gc = GpuCook::new();
        gc.retain_streams_for_debug(true);
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plano,
            &[],
            CookClock::at(0.0),
            [0.25, 0.25, 0.75, 0.75],
            [0.4, 0.4],
            SinkStyle::PLAIN,
        )
        .expect("gpu cook");
        let mut disp: Vec<String> = gc.node_columns(ultimo).unwrap_or(&[]).to_vec();
        disp.sort_unstable();
        let mut cpu_nomes = nomes(&cpu);
        cpu_nomes.sort_unstable();
        assert_eq!(cpu_nomes, disp, "{rotulo}: o conjunto de colunas");
        let mut pior = 0.0f32;
        for (nome, col) in cpu.columns() {
            let a = achatada(col);
            let b = gc
                .read_column(&gpu, ultimo, nome)
                .unwrap_or_else(|| panic!("{rotulo}: `{nome}` nao voltou"));
            assert_eq!(a.len(), b.len(), "{rotulo}: `{nome}` comprimento");
            for (k, (x, y)) in a.iter().zip(&b).enumerate() {
                let d = (x - y).abs();
                assert!(
                    d <= EPS * x.abs().max(1.0),
                    "{rotulo}: `{nome}`[{k}] cpu {x} disp {y}"
                );
                pior = pior.max(d);
            }
        }
        eprintln!("{rotulo}: {} colunas, pior |Δ| {pior:e}", cpu_nomes.len());
    }
}

/// ⭐ **Toda coluna que o plano PERGUNTA é uma coluna da convenção** — senão uma escrita por nome
/// dela não entraria na forma provada e a pergunta responderia «ausente» sobre uma coluna presente.
/// Derivada do registo: as bindings que recusam ou pareiam por `id` (de cada variante) e a coluna
/// de cada segundo nascimento. A escrituração (`id`…) fica de fora: a escada nunca a escreve.
#[test]
fn every_column_the_plan_asks_is_a_convention_column() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut perguntas: BTreeSet<&'static str> = BTreeSet::new();
    for man in reg.manifests() {
        let Some(base) = reg.gpu_kernel(man.id) else {
            continue;
        };
        let hints = reg.param_ui(man.id).unwrap_or(&[]);
        let mut kernels = vec![base];
        for spec in man.params {
            let valores: Vec<f32> = match hints.iter().find(|h| h.param == spec.name) {
                Some(h) => match h.widget {
                    ph2d_node_registry::ParamWidget::Enum { labels } => {
                        (0..labels.len()).map(|i| i as f32).collect()
                    }
                    _ => vec![h.min, (h.min + h.max) * 0.5, h.max],
                },
                None => vec![spec.default],
            };
            for v in valores {
                let p = |n: &str| {
                    if n == spec.name {
                        v
                    } else {
                        man.params
                            .iter()
                            .find(|q| q.name == n)
                            .map_or(0.0, |q| q.default)
                    }
                };
                kernels.push(base.resolve(&p));
            }
        }
        for k in kernels {
            for b in k.bindings {
                if b.access.refuses() || b.access.is_gather_key() {
                    perguntas.insert(b.column);
                }
            }
        }
        if let Some(StreamOp::SourceRows { fired: Some(f), .. }) = reg.stream_op(man.id) {
            perguntas.insert(f.column);
        }
    }
    assert!(perguntas.len() >= 3, "a varredura achou {perguntas:?}");
    for c in &perguntas {
        if ph2d_nodegraph::attr::is_bookkeeping_column(c) {
            continue;
        }
        assert_eq!(
            ph2d_gpu_cook::plan::coluna_da_convencao(c),
            Some(*c),
            "`{c}` e' perguntada pelo plano e nao e' coluna da convencao"
        );
    }
    eprintln!("perguntas do plano: {perguntas:?}");
}
