//! ⭐⭐ **O CENSO DO COMPRIMENTO `1`** — todo consumidor de VALOR honra a regra `1→N`?
//! (doc 110 §11.4 · §14.1 item 4).
//!
//! A população é DERIVADA do registo: toda porta de entrada `Instances·Scalar·Frame` de todo tipo
//! de nó. Para cada uma, duas provas:
//!
//! - **CPU**: o mesmo número chega pela porta como campo de comprimento `1` (o `value.cursor` cru)
//!   e como `N` cópias (o mesmo cursor através de [`Difunde`]); as saídas do nó têm de sair iguais
//!   ao bit, quadro a quadro.
//! - **Placa** (estática): toda ligação de kernel — a base e as variantes — que lê essa porta
//!   difunde (`ReadBroadcast`), ou é a porta do despacho, ou recua (`RefuseIfPresent`).
//!
//! ⚠️ As portas `Clock::Event` (pulsos) ficam fora do âmbito e imprimem-se à parte.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{GpuAlgorithm, GpuKernel, KernelResolver, ReduceOp, StreamOp};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use ph2d_nodegraph::value::CookValue;

const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const LADO: f32 = 12.0;
const QUADROS: usize = 4;
/// Dois cursores × as duas saídas = quatro constantes, dos dois lados do `0,5` e do zero.
const CURSORES: [[f32; 2]; 2] = [[0.37, -1.25], [1.75, 0.6]];

/// Nó de teste: o valor da porta 0 (comprimento 1) em `N` cópias, `N` = contagem da porta 1.
static DIFUNDE: NodeManifest = NodeManifest {
    id: NodeTypeId::of("census.difunde"),
    name: "census.difunde",
    inputs: &[
        PortSpec {
            name: "v",
            ty: VALUE,
        },
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct Difunde;
impl NodeOp for Difunde {
    fn manifest(&self) -> &'static NodeManifest {
        &DIFUNDE
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let v = match ctx.input(0).get("v") {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let n = ctx.input(1).count().max(1);
        ctx.emit(Stream::new(n).with("v", Column::Scalar(vec![v; n])));
    }
}

struct Ops<'a>(&'a NodeRegistry);
impl OpResolver for Ops<'_> {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == DIFUNDE.id {
            Some(&Difunde)
        } else {
            self.0.resolve(ty)
        }
    }
}

fn liga(g: &mut Graph, de: (NodeId, u16), para: (NodeId, u16)) {
    g.connect(Edge {
        from: de,
        to: para,
        delayed: false,
    })
    .expect("fio");
}

fn porta(i: usize) -> u16 {
    u16::try_from(i).expect("porta cabe em u16")
}

/// Coze `man` com a porta `p` alimentada pelo cursor (`largo` ⇒ N cópias); as outras portas de
/// geometria levam a grelha e, com `outros`, as outras portas de VALOR levam um campo por elemento.
/// Devolve, por quadro, as saídas do nó.
fn coze(
    reg: &NodeRegistry,
    man: &NodeManifest,
    p: usize,
    largo: bool,
    outros: bool,
    cursor: [f32; 2],
    eixo: u16,
) -> Result<Vec<Vec<CookValue>>, String> {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", LADO);
    g.set_param(grid, "cols", LADO);
    let no = g.add_node(man.name);
    let cur = g.add_node("value.cursor");
    for (q, spec) in man.inputs.iter().enumerate() {
        if q == p {
            continue;
        }
        if INST_VEC2.connects_directly(spec.ty) {
            liga(&mut g, (grid, 0), (no, porta(q)));
        } else if outros && VALUE.connects_directly(spec.ty) {
            let campo = g.add_node("value.instance_field");
            liga(&mut g, (grid, 0), (campo, 0));
            liga(&mut g, (campo, 0), (no, porta(q)));
        }
    }
    if largo {
        let d = g.add_node(DIFUNDE.name);
        liga(&mut g, (cur, eixo), (d, 0));
        liga(&mut g, (grid, 0), (d, 1));
        liga(&mut g, (d, 0), (no, porta(p)));
    } else {
        liga(&mut g, (cur, eixo), (no, porta(p)));
    }
    let ops = Ops(reg);
    let mut cook = Cook::new();
    cook.set_external(
        ph2d_nodegraph::external::CURSOR,
        Stream::new(1).with("P", Column::Vec2(vec![cursor])),
    );
    let mut quadros = Vec::with_capacity(QUADROS);
    for k in 0..QUADROS {
        #[expect(clippy::cast_precision_loss, reason = "um indice de quadro")]
        let t = k as f64 / 24.0;
        let out = cook
            .cook(&g, &ops, no, t)
            .map_err(|e| format!("cook: {e:?}"))?;
        quadros.push(out.to_vec());
    }
    Ok(quadros)
}

/// Bit a bit pelo `Debug` (`-0.0`, `NaN` e o último ulp contam).
fn bits(v: &CookValue) -> String {
    format!("{v:?}")
}

/// A saída do lado `1` é a do lado `N` difundida: comprimento `1` contra `N`, e cada coluna
/// de `N` é a linha `0` repetida.
fn difundida(um: &CookValue, n: &CookValue) -> bool {
    let (a, b) = (um.as_stream(), n.as_stream());
    a.count() == 1
        && b.count() > 1
        && a.columns().count() == b.columns().count()
        && a.columns().all(|(k, col)| {
            let Some(outra) = b.get(k) else {
                return false;
            };
            let linha = |c: &Column, i: usize| match c {
                Column::Scalar(v) => format!("{:?}", v.get(i)),
                Column::Vec2(v) => format!("{:?}", v.get(i)),
                Column::Vec3(v) => format!("{:?}", v.get(i)),
                Column::Vec4(v) => format!("{:?}", v.get(i)),
            };
            outra.len() == b.count() && (0..b.count()).all(|i| linha(col, 0) == linha(outra, i))
        })
}

/// O que a CPU disse de uma porta.
#[derive(PartialEq, Eq)]
enum Cpu {
    /// Saídas iguais ao bit.
    Iguais,
    /// O nó PROPAGA o comprimento: sai `1` onde saía `N` cópias, e toda saída dele é VALOR —
    /// logo quem a consome está neste mesmo censo.
    Propaga,
}

/// O veredito da CPU para uma porta: todas as combinações (2 cursores × 2 eixos × outros
/// ligados/desligados) têm de sair iguais ao bit, ou propagar o `1` por saídas de VALOR.
fn cpu(reg: &NodeRegistry, man: &NodeManifest, p: usize) -> Result<Cpu, String> {
    let so_valor = man.outputs.iter().all(|o| o.ty == VALUE);
    let mut veredito = Cpu::Iguais;
    for outros in [true, false] {
        for cursor in CURSORES {
            for eixo in 0..2u16 {
                let caso = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (
                        coze(reg, man, p, false, outros, cursor, eixo),
                        coze(reg, man, p, true, outros, cursor, eixo),
                    )
                }));
                let (um, n) = caso.map_err(|_| "PANICO".to_string())?;
                let (um, n) = (um?, n?);
                for k in 0..QUADROS {
                    let iguais = um[k].len() == n[k].len()
                        && um[k].iter().zip(&n[k]).all(|(a, b)| bits(a) == bits(b));
                    if iguais {
                        continue;
                    }
                    let propaga = so_valor
                        && um[k].len() == n[k].len()
                        && um[k].iter().zip(&n[k]).all(|(a, b)| difundida(a, b));
                    if propaga {
                        veredito = Cpu::Propaga;
                        continue;
                    }
                    let corta = |v: &CookValue| bits(v).chars().take(160).collect::<String>();
                    return Err(format!(
                        "outros={outros} cursor={cursor:?} eixo={eixo} quadro={k}\n        1: {}\n        N: {}",
                        um[k].iter().map(corta).collect::<Vec<_>>().join(" | "),
                        n[k].iter().map(corta).collect::<Vec<_>>().join(" | ")
                    ));
                }
            }
        }
    }
    Ok(veredito)
}

/// O kernel base e todas as variantes alcançáveis varrendo UM param de cada vez (a varredura
/// do `every_registered_kernel_validates_across_the_whole_presence_space`).
fn kernels<'r>(reg: &'r NodeRegistry, man: &NodeManifest) -> Vec<&'r GpuKernel> {
    let Some(base) = reg.gpu_kernel(man.id) else {
        return Vec::new();
    };
    let mut vistos: Vec<&GpuKernel> = vec![base];
    let hints = reg.param_ui(man.id).unwrap_or(&[]);
    for spec in man.params {
        let hint = hints.iter().find(|h| h.param == spec.name);
        let mut valores: Vec<f32> = (0..16u8).map(f32::from).collect();
        valores.push(spec.default);
        if let Some(h) = hint {
            valores.extend([h.min, (h.min + h.max) * 0.5, h.max]);
        }
        for v in valores {
            let resolve = |name: &str| {
                if name == spec.name {
                    v
                } else {
                    man.param_default(name).unwrap_or(0.0)
                }
            };
            let k = base.resolve(&resolve);
            if !vistos.iter().any(|x| std::ptr::eq(*x, k)) {
                vistos.push(k);
            }
        }
    }
    // Os kernels que moram DENTRO de uma operação de corrente também lêem portas.
    match reg.stream_op(man.id) {
        Some(StreamOp::Compact { predicate, .. }) => vistos.push(predicate),
        Some(StreamOp::Carry {
            predicate,
            identity_kernel,
            ..
        }) => vistos.extend([predicate, identity_kernel]),
        _ => {}
    }
    vistos
}

/// O veredito da placa: `Ok(descrição)` ou `Err(descrição)`.
fn placa(reg: &NodeRegistry, man: &NodeManifest, p: usize) -> Result<String, String> {
    let ks = kernels(reg, man);
    if ks.is_empty() {
        return Ok("sem kernel".into());
    }
    let mut acessos = Vec::new();
    let mut falhas = Vec::new();
    for (i, k) in ks.iter().enumerate() {
        for b in k.bindings.iter().filter(|b| b.port == p) {
            let a = format!("v{i}:{}={:?}", b.column, b.access);
            // A porta 0 sem lei de contagem É o comprimento do despacho: `1` dentro ⇒ `1` fora,
            // o mesmo que a CPU propaga.
            let despacho = p == 0 && k.count_law.is_none();
            let bom = b.access.broadcasts() || b.access.refuses() || !b.access.reads() || despacho;
            if bom {
                acessos.push(a);
            } else {
                falhas.push(a);
            }
        }
    }
    for r in reg.reduces(man.id).iter().filter(|r| r.port == p) {
        let a = format!("reduce {}={:?}", r.name, r.op);
        if matches!(r.op, ReduceOp::Sum) {
            falhas.push(a);
        } else {
            acessos.push(a);
        }
    }
    // O Lloyd lê a porta `relax` na LINHA 0 (`algorithm_meta`) — difunde por construção.
    if let Some(GpuAlgorithm::LloydVoronoi { relax_port, .. }) = reg.algorithm(man.id)
        && *relax_port == p
    {
        acessos.push("lloyd: linha 0".into());
    }
    let n = ks.len();
    let txt = |v: &[String]| {
        if v.is_empty() {
            "nao le".to_string()
        } else {
            v.join(" ")
        }
    };
    if falhas.is_empty() {
        Ok(format!("{n}k {}", txt(&acessos)))
    } else {
        Err(format!("{n}k FALHA {}", falhas.join(" ")))
    }
}

/// ⭐⭐ **O CENSO** — toda porta de VALOR, provada na CPU e na placa.
#[test]
fn every_value_port_reads_a_length_one_field_as_its_n_copies() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registo");
    let mut mans: Vec<&'static NodeManifest> = reg.manifests().collect();
    mans.sort_unstable_by_key(|m| m.name);
    let (mut portas, mut ok_cpu, mut ok_placa, mut propagam) = (0usize, 0usize, 0usize, 0usize);
    let (mut pulsos, mut estados) = (Vec::new(), Vec::new());
    let mut falhas = Vec::new();
    eprintln!("\n  tipo · porta                         | CPU 1≡N | placa");
    for man in &mans {
        for (p, spec) in man.inputs.iter().enumerate() {
            let t = spec.ty;
            if t.domain != Domain::Instances || t.dim != Dim::Scalar {
                continue;
            }
            if t.clock != Clock::Frame {
                pulsos.push(format!("{}.{} ({:?})", man.name, spec.name, t.clock));
                continue;
            }
            // `state` é o laço `pre` do próprio nó (o editor liga-o ao soltar o nó), não um
            // consumidor de valor alheio.
            if spec.name == "state" {
                estados.push(format!("{}.{}", man.name, spec.name));
                continue;
            }
            portas += 1;
            let c = cpu(&reg, man, p);
            let d = placa(&reg, man, p);
            ok_cpu += usize::from(c.is_ok());
            ok_placa += usize::from(d.is_ok());
            propagam += usize::from(c.as_ref().is_ok_and(|v| *v == Cpu::Propaga));
            let rotulo = format!("{} · {}:{}", man.name, p, spec.name);
            eprintln!(
                "  {rotulo:<36} | {:<7} | {}",
                match &c {
                    Ok(Cpu::Iguais) => "iguais",
                    Ok(Cpu::Propaga) => "propaga",
                    Err(_) => "NAO",
                },
                d.as_ref().unwrap_or_else(|e| e)
            );
            if let Err(e) = &c {
                eprintln!("      {e}");
                falhas.push(format!("CPU {rotulo}"));
            }
            if d.is_err() {
                falhas.push(format!("placa {rotulo}"));
            }
        }
    }
    eprintln!(
        "\n  portas de VALOR: {portas} · CPU ok: {ok_cpu} (das quais propagam o 1: {propagam}) · placa ok: {ok_placa}\n  laço `state`: {} — {}\n  fora do âmbito (Event/outros relógios): {}\n  {}\n",
        estados.len(),
        estados.join(" · "),
        pulsos.len(),
        pulsos.join(" · ")
    );
    // O piso: um registo que mudasse de forma deixava o censo a varrer zero, verde.
    assert!(
        portas >= 30,
        "so' {portas} portas de VALOR — o registo mudou?"
    );
    // ⛔ A catraca: a lista dos que NÃO honram é exacta — um consumidor novo que não difunda
    // reprova, e uma cura também (para a lista encolher, nunca ficar a mentir).
    assert_eq!(
        falhas, NOMEADOS,
        "a lista dos consumidores que nao honram o comprimento 1 mudou (doc 110 §14.1 item 4)"
    );
}

/// Os que dão OUTRA resposta a `1` do que a `N` cópias (07/10, doc 110 §14.2 (4)). ⚠️ **Não são
/// defeitos: o comprimento É o significado** — um evento POR PEÇA, uma soma SOBRE o campo. É por
/// eles que a porta `in` do `value.cursor`/`value.table` não é morta, e que a cura das cópias é o
/// TRANSPORTE e não o comprimento.
const NOMEADOS: &[&str] = &[
    // Propagam o `1` para um PULSO: 1 linha disparada contra N (os pulsos estão fora do censo).
    "CPU pulse.compare · 0:value",
    "CPU pulse.on_change · 0:value",
    // Uma redução CONTA as linhas: `sum`/`count` de 1 contra N cópias.
    "CPU value.reduce · 0:in",
    "placa value.reduce · 0:in",
];
