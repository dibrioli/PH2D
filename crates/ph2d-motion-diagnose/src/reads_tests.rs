//! Os gates do nome-que-não-resolve. Irmão por `#[path]`, logo módulo FILHO — o
//! `use super::*` alcança `projected_text_param`/`authored_name`.
//!
//! ⚠️ **A fixture não constrói um registry inteiro de propósito.** A regra é sobre
//! *quem DECLARA um `StreamOp::Project`*, e um nó de teste com essa declaração
//! prova a derivação com muito mais força do que o `value.attribute` real: com o nó
//! real, um gate que passasse não distinguiria *"a regra lê a declaração"* de *"a
//! regra conhece o nome do tipo"*. Há um gate que fecha essa metade explicitamente.

use super::*;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::GpuKernel;
use ph2d_nodegraph::graph::Edge;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const ANY: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// Um leitor-por-nome de brincadeira: declara o `StreamOp::Project` que a regra
/// procura, e mais nada.
static READER: NodeManifest = NodeManifest {
    id: NodeTypeId::of("diagnose.test.reader"),
    name: "diagnose.test.reader",
    inputs: &[PortSpec {
        name: "in",
        ty: ANY,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: ANY,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "mode",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// Um nó ORDINÁRIO com um text param — o CONTROLE que separa *"a regra lê a
/// declaração"* de *"a regra vê um text param e chuta"*.
static PLAIN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("diagnose.test.plain"),
    name: "diagnose.test.plain",
    inputs: &[PortSpec {
        name: "in",
        ty: ANY,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: ANY,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct Reader;
impl NodeOp for Reader {
    fn manifest(&self) -> &'static NodeManifest {
        &READER
    }
    fn eval(&self, _ctx: &mut EvalCtx<'_>) {}
}
struct Plain;
impl NodeOp for Plain {
    fn manifest(&self) -> &'static NodeManifest {
        &PLAIN
    }
    fn eval(&self, _ctx: &mut EvalCtx<'_>) {}
}
/// Uma FONTE, para a aresta de entrada ter de onde vir.
static SOURCE: NodeManifest = NodeManifest {
    id: NodeTypeId::of("diagnose.test.source"),
    name: "diagnose.test.source",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: ANY,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Source;
impl NodeOp for Source {
    fn manifest(&self) -> &'static NodeManifest {
        &SOURCE
    }
    fn eval(&self, _ctx: &mut EvalCtx<'_>) {}
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Source)).expect("source");
    reg.register(Box::new(Reader)).expect("reader");
    reg.register(Box::new(Plain)).expect("plain");
    reg.register_gpu_kernel(READER.id, GpuKernel::PASSTHROUGH);
    reg.register_stream_op(
        READER.id,
        StreamOp::Project {
            text_param: "attr",
            mode_param: "mode",
        },
    );
    reg
}

/// `src -> reader`, com o `attr` do leitor escrito como `name`.
fn wired(name: Option<&str>) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("diagnose.test.source");
    let rd = g.add_node("diagnose.test.reader");
    if let Some(n) = name {
        g.set_text_param(rd, "attr", n);
    }
    g.connect(Edge {
        from: (src, 0),
        to: (rd, 0),
        delayed: false,
    })
    .expect("edge");
    (g, src, rd)
}

/// O que a stream carrega, como o shell responderia.
fn carrying(cols: &'static [&'static str]) -> impl Fn(NodeId, u16) -> Option<Vec<String>> {
    move |_, _| Some(cols.iter().map(|s| (*s).to_string()).collect())
}

/// **Um nome que a stream de entrada NÃO carrega é reportado** — a regra inteira.
#[test]
fn a_name_the_incoming_stream_lacks_is_reported() {
    let reg = registry();
    let (g, _, rd) = wired(Some("velocty")); // o typo de `vel`
    let out = unresolved_reads(&g, &reg, &carrying(&["P", "vel", "age"]));
    assert_eq!(
        out,
        vec![UnresolvedRead {
            node: rd,
            column: "velocty".into()
        }]
    );
}

/// **E o CONTROLE: um nome que ela carrega é SILENCIOSO.**
///
/// ⚠️ Sem esta metade a regra podia estar a reportar todo leitor, e o gate acima
/// passaria igual.
#[test]
fn a_name_the_stream_carries_is_silent() {
    let reg = registry();
    let (g, _, _) = wired(Some("vel"));
    assert!(unresolved_reads(&g, &reg, &carrying(&["P", "vel", "age"])).is_empty());
}

/// **Os quatro silêncios**, cada um por um motivo diferente — e nenhum deles é
/// *"não sei ler isto"*.
#[test]
fn the_four_silences_are_silent() {
    let reg = registry();

    // 1. sem nome autorado — o nó está inacabado, não errado.
    let (g, _, _) = wired(None);
    assert!(
        unresolved_reads(&g, &reg, &carrying(&["P"])).is_empty(),
        "nome ausente"
    );

    // 2. nome em BRANCO — idem (o campo existe e está vazio).
    let (g, _, _) = wired(Some("   "));
    assert!(
        unresolved_reads(&g, &reg, &carrying(&["P"])).is_empty(),
        "nome em branco"
    );

    // 3. sem aresta de entrada — é o `MissingSource`, e reportar os dois nomearia a
    //    mesma causa duas vezes.
    let mut g = Graph::new();
    let rd = g.add_node("diagnose.test.reader");
    g.set_text_param(rd, "attr", "nao_existe");
    assert!(
        unresolved_reads(&g, &reg, &carrying(&["P"])).is_empty(),
        "sem fonte"
    );

    // 4. o chamador não SABE — `None` não é lista vazia, e é aqui que essa
    //    distinção deixa de ser prosa: com `Some(vec![])` este mesmo grafo seria
    //    reportado.
    let (g, _, _) = wired(Some("nao_existe"));
    assert!(
        unresolved_reads(&g, &reg, &|_, _| None).is_empty(),
        "stream desconhecida"
    );
    assert_eq!(
        unresolved_reads(&g, &reg, &|_, _| Some(Vec::new())).len(),
        1,
        "…e uma stream que SABIDAMENTE não tem colunas é reportada"
    );
}

/// **A regra é DERIVADA da declaração, não de uma lista de tipos** — um nó comum
/// com um text param chamado `attr` não é varrido.
///
/// ⚠️ É este gate que garante que um nó NOVO com a forma de projeção nasce coberto
/// e que um nó qualquer não é acusado por acidente de nome de param.
#[test]
fn the_rule_reads_the_declaration_not_the_type_name() {
    let reg = registry();
    let mut g = Graph::new();
    let src = g.add_node("diagnose.test.source");
    let plain = g.add_node("diagnose.test.plain");
    g.set_text_param(plain, "attr", "nao_existe");
    g.connect(Edge {
        from: (src, 0),
        to: (plain, 0),
        delayed: false,
    })
    .expect("edge");
    assert!(unresolved_reads(&g, &reg, &carrying(&["P"])).is_empty());

    // E a porta da derivação responde `Some` só para quem declarou.
    assert_eq!(projected_text_param(&reg, READER.id), Some("attr"));
    assert_eq!(projected_text_param(&reg, PLAIN.id), None);
}

/// **Uma aresta DELAYED não é a entrada** — ela carrega o estado do quadro anterior
/// de uma fonte com estado, e tomá-la pela entrada faria a regra medir a stream
/// errada (a mesma isenção `seeds_own_state` que o `MissingSource` já carrega).
#[test]
fn a_delayed_edge_is_not_the_input() {
    let reg = registry();
    let mut g = Graph::new();
    let src = g.add_node("diagnose.test.source");
    let rd = g.add_node("diagnose.test.reader");
    g.set_text_param(rd, "attr", "nao_existe");
    g.connect(Edge {
        from: (src, 0),
        to: (rd, 0),
        delayed: true,
    })
    .expect("edge");
    assert!(unresolved_reads(&g, &reg, &carrying(&["P"])).is_empty());
}

/// **O nome sobrevive APARADO, e é o que o artista escreveu** — a mensagem do badge
/// o cita, então um espaço à frente viraria uma citação que não casa com o campo.
#[test]
fn the_reported_name_is_the_authored_one_trimmed() {
    let reg = registry();
    let (g, _, rd) = wired(Some("  fantasma \n"));
    let out = unresolved_reads(&g, &reg, &carrying(&["P"]));
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].column, "fantasma");
    assert_eq!(out[0].node, rd);
    assert_eq!(authored_name(&g, rd, "attr"), Some("fantasma"));
}
