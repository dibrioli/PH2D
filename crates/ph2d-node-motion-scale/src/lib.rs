#![forbid(unsafe_code)]
//! `motion.scale` — a Motion **modifier**: multiplies the `size` (Vec2)
//! attribute by `amount`, eased per-instance by the multiplicative `falloff`
//! column (§1.2). The effective factor is `1 + (amount - 1) * falloff_i`, so an
//! instance with `falloff = 0` keeps its size and `falloff = 1` gets the full
//! `amount`. A stream without a `size` column starts from
//! [`SIZE_IDENTITY`](ph2d_nodegraph::attr::SIZE_IDENTITY) — the SAME unit scale
//! the lowering falls back to, which is what makes `amount = 1` a true no-op on
//! the render (doc 39). Every other column passes through unchanged (count
//! preserved). Pure.
//!
//! ## Os DOIS eixos (doc 88 §B3 — a varredura PRO da família TRANSFORM)
//!
//! `size` é uma coluna **Vec2** e este nó oferecia **um** número, então *squash &
//! stretch* — o primeiro dos doze princípios da animação — era **inexprimível** no
//! grafo: não havia nó nenhum capaz de esticar num eixo e não no outro.
//!
//! ⚠️ **O `amount` NÃO muda de significado, e isso é o desenho:** ele segue sendo o
//! fator UNIFORME. Fazê-lo virar "o fator X" e acrescentar um `amount_y` de default
//! `1` mudaria o comportamento de **todo grafo já autorado** (um `amount = 2` passaria
//! a esticar só a largura) — e a demo de boot sozinha põe treze `motion.scale`.
//! Em vez disso, o **`uniform`** (o *link* de corrente do AE / Cavalry / Figma) diz se
//! o segundo eixo é lido: ligado (o default) o `amount` vale para os dois, byte a byte
//! como antes; desligado, `amount` é X e `amount_y` é Y. O `ParamGate` esconde o
//! `amount_y` enquanto ele estiver travado — *um controle que não faz nada não é
//! pintado*.
//!
//! Params (read via `ctx.param`): `amount` (1.0) · `uniform` (1.0) · `amount_y` (1.0).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, SIZE_IDENTITY, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.scale"),
    name: "motion.scale",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "amount",
            default: 1.0,
        },
        // 1 = os dois eixos leem `amount` (o link ligado, e o comportamento que
        // sempre shipou); 0 = `amount` é X e `amount_y` é Y.
        ParamSpec {
            name: "uniform",
            default: 1.0,
        },
        ParamSpec {
            name: "amount_y",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Os dois fatores AUTORADOS (antes do falloff): `(x, y)`. Com o link ligado o
/// `amount` responde pelos dois — é a porta ÚNICA que o `eval` e o gate de paridade
/// perguntam, para o CPU e o WGSL não poderem discordar sobre o que "uniform" quer
/// dizer. `uniform >= 0.5` e não `!= 0.0`: o param chega como `f32` de um toggle, e
/// meio caminho é o limiar que todo widget booleano deste app usa.
fn authored_factors(amount: f32, uniform: f32, amount_y: f32) -> (f32, f32) {
    if uniform >= 0.5 {
        (amount, amount)
    } else {
        (amount, amount_y)
    }
}

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The falloff-eased scale factor: `1` at `falloff = 0`, `amount` at `1`.
fn eff_factor(amount: f32, falloff: f32) -> f32 {
    1.0 + (amount - 1.0) * falloff
}

/// GPU compute kernel (GPU/M5 Fase 2, ADR-0126): `size' = size · (1 + (amount −
/// 1)·falloff)`, the exact per-element map of the CPU `eval` in the same
/// mul/add order (parity within FMA ε; with `falloff` absent it is bit-exact —
/// no transcendentals). No `applicable`: a plain multiply covers the whole
/// param space. `ReadWrite` on `size` mirrors the CPU: a stream without a
/// `size` column starts each instance from the `SIZE_IDENTITY` (`[1, 1]`) the
/// lowering itself falls back to — the identity that makes `amount = 1` a true
/// render no-op — and the column is always written.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let sc_ay = select(params.amount_y, params.amount, params.uniform >= 0.5);\n\
        let sc_f = read_falloff(i);\n\
        let sc_fx = 1.0 + (params.amount - 1.0) * sc_f;\n\
        let sc_fy = 1.0 + (sc_ay - 1.0) * sc_f;\n\
        let sc_s = read_size(i);\n\
        write_size(i, vec2<f32>(sc_s.x * sc_fx, sc_s.y * sc_fy));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            // SIZE_IDENTITY = [1, 1]; only the first `dim` (Vec2) lanes are read.
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &["amount", "uniform", "amount_y"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct MotionScale;

impl NodeOp for MotionScale {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (ax, ay) = authored_factors(
            ctx.param("amount"),
            ctx.param("uniform"),
            ctx.param("amount_y"),
        );
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Base per-instance size (absent column → the identity the lowering
            // itself falls back to — see SIZE_IDENTITY: any other base would make
            // `amount = 1` resize the scene).
            let base: Vec<[f32; 2]> = match input.get("size") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => vec![SIZE_IDENTITY; n],
            };
            // Pure per-instance map → parallel above the threshold
            // (bit-identical, no reduction). GPU/M5 Fase 0.
            let scaled: Vec<[f32; 2]> = par_build(n, |i| {
                let w = falloff_at(input, i);
                let s = base.get(i).copied().unwrap_or(SIZE_IDENTITY);
                [s[0] * eff_factor(ax, w), s[1] * eff_factor(ay, w)]
            });
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "size" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("size", Column::Vec2(scaled));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionScale))?;
    // M1.R1 — UI metadata (a spatial modifier → blue transform, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Scale",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    // GPU/M5 Fase 2 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): o fator, o link, e o segundo eixo que ele destrava.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "amount",
        label: "Scale",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "uniform",
        label: "Uniform",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: "amount_y",
        label: "Scale Y",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

/// Com o link LIGADO o `amount_y` não é lido, então ele não é pintado — a mesma lei
/// que o `motion.look_at` e o `motion.oscillator` já aplicam aos params de um modo
/// que não está escolhido.
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: "amount_y",
    when: "uniform",
    values: &[0],
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances with size [2,2] and falloff [1, 0.5].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.scale.test.src"),
        name: "motion.scale.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]))
                    .with("size", Column::Vec2(vec![[2.0, 2.0], [2.0, 2.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.5])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionScale),
                _ => None,
            }
        }
    }

    #[test]
    fn scale_is_eased_by_falloff() {
        let mut g = Graph::new();
        let src = g.add_node("motion.scale.test.src");
        let sc = g.add_node("motion.scale");
        g.connect(Edge {
            from: (src, 0),
            to: (sc, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(sc, "amount", 3.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, sc, 0.0).unwrap();
        match out[0].as_stream().get("size").unwrap() {
            // i0 f=1: factor 3 → 2*3=6 ; i1 f=0.5: factor 1+2*0.5=2 → 2*2=4
            Column::Vec2(v) => assert_eq!(v, &vec![[6.0, 6.0], [4.0, 4.0]]),
            _ => panic!("size"),
        }
    }

    #[test]
    fn eff_factor_interpolates_one_to_amount() {
        assert_eq!(eff_factor(3.0, 0.0), 1.0); // no falloff → identity
        assert_eq!(eff_factor(3.0, 1.0), 3.0); // full falloff → amount
        assert_eq!(eff_factor(3.0, 0.5), 2.0); // half
    }

    /// Cozinha o nó com os params dados e devolve a coluna `size`.
    fn sizes(set: &[(&str, f32)]) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.scale.test.src");
        let sc = g.add_node("motion.scale");
        g.connect(Edge {
            from: (src, 0),
            to: (sc, 0),
            delayed: false,
        })
        .unwrap();
        for (k, v) in set {
            g.set_param(sc, *k, *v);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, sc, 0.0).unwrap();
        match out[0].as_stream().get("size").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("size"),
        }
    }

    /// **SQUASH & STRETCH — o eixo que faltava** (doc 88 §B3).
    ///
    /// ⚠️ Nasceu VERMELHO: antes desta wave `size` era uma coluna Vec2 escalada por UM
    /// número, então esticar em X sem esticar em Y não era *difícil* — era **inexprimível
    /// no grafo inteiro**. O oráculo é a RAZÃO de aspecto, não os valores: é ela que
    /// distingue "escalou" de "deformou", e ela não se move em nenhum fator uniforme.
    #[test]
    fn unlinking_the_axes_squashes_and_stretches() {
        let s = sizes(&[
            ("amount", 2.0),
            ("uniform", 0.0),
            ("amount_y", 0.5),
            // O falloff da fonte é [1, 0.5]; o elemento 0 recebe o fator cheio.
        ]);
        assert_eq!(
            s[0],
            [4.0, 1.0],
            "x dobra e y cai à metade no falloff cheio"
        );
        let aspect = s[0][0] / s[0][1];
        assert!(
            (aspect - 4.0).abs() < 1e-5,
            "a razão de aspecto tem de andar (4.0), e mediu {aspect} — \
             um fator uniforme a deixaria em 1.0 para sempre"
        );
    }

    /// **O LINK LIGADO É O MUNDO QUE JÁ SHIPAVA, AO BIT.**
    ///
    /// A regressão que importa: todo grafo autorado antes desta wave tem `uniform` no
    /// default, e a demo de boot sozinha põe treze `motion.scale`. O gate compara contra
    /// os MESMOS números que o `scale_is_eased_by_falloff` afirma desde sempre — e um
    /// `amount_y` gritante prova que ele nem é lido.
    #[test]
    fn the_linked_default_ignores_the_second_axis_entirely() {
        let base = sizes(&[("amount", 3.0)]);
        assert_eq!(base, vec![[6.0, 6.0], [4.0, 4.0]]);
        let noisy = sizes(&[("amount", 3.0), ("amount_y", 99.0)]);
        assert_eq!(
            noisy, base,
            "com o link ligado o `amount_y` não pode alcançar um único byte"
        );
    }

    /// **O falloff EASA OS DOIS EIXOS** — e não o primeiro só.
    ///
    /// O elemento 1 da fonte tem `falloff = 0.5`, então cada eixo caminha metade do
    /// caminho até o SEU fator: x de 1 até 2 (⇒ 1.5), y de 1 até 0.5 (⇒ 0.75).
    #[test]
    fn the_falloff_eases_both_axes() {
        let s = sizes(&[("amount", 2.0), ("uniform", 0.0), ("amount_y", 0.5)]);
        assert_eq!(s[1], [2.0 * 1.5, 2.0 * 0.75]);
    }

    /// O limiar do link é meio caminho, não `!= 0` — a porta única que o WGSL espelha.
    #[test]
    fn the_link_reads_as_a_toggle_at_half_way() {
        assert_eq!(authored_factors(2.0, 1.0, 0.5), (2.0, 2.0));
        assert_eq!(authored_factors(2.0, 0.0, 0.5), (2.0, 0.5));
        assert_eq!(authored_factors(2.0, 0.4, 0.5), (2.0, 0.5));
        assert_eq!(authored_factors(2.0, 0.5, 0.5), (2.0, 2.0));
    }
}
