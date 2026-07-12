#![forbid(unsafe_code)]
//! `motion.expression` — the **formula escape-hatch**: evaluate a per-instance VEX-lite
//! expression into a value field (Motion Nodes M1, the value domain — doc 01 §1.7 /
//! doc 32). Where `value.math`/`value.map_range` are fixed ops, this is arbitrary: an
//! author types `sin(i*0.3 + t) * a` and gets a custom `v` field driving anything.
//!
//! **The formula is a TEXT param** (`expr`), stored in the graph's additive string
//! channel (`Graph::set_text_param`, read via `EvalCtx::text_param`) — the frozen
//! `NodeManifest` stays f32-only (ADR-0039), so this needed **no contract bump** (doc 32).
//! The text is parsed once per cook (sibling `parse.rs`, VEX-lite → the frozen
//! `ph2d_expr::Expr` IR) and evaluated per element with `ph2d_expr::eval`, which is
//! **HR-5-exempt** (presentation-side, per `ph2d-expr`'s own contract — the transcendentals
//! live there, not here). A parse error falls back to a zero field (the editor badges the
//! node; headless just zeroes). `Effect::Temporal` (the formula may read `t`).
//!
//! **Variables:** `i` (index), `n` (count), `t` (playhead seconds), `f` (`i/(n-1)`,
//! normalised), any **scalar column** of the input stream by name (e.g. `v`), and the
//! params `a`/`b`/`c`/`d`.

mod parse;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value-field output type (mirror of `value.instance_field`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";
/// The name of the formula text param (read via `EvalCtx::text_param`).
const EXPR_KEY: &str = "expr";

/// The static contract of this node type (ADR-0031). The formula is NOT a param here —
/// it is a **text** param (frozen `ParamSpec` is f32-only); these `a`/`b`/`c`/`d` are the
/// numeric coefficients the formula can reference.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.expression"),
    name: "motion.expression",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Reads `t`: a formula may be time-varying, so recompute each frame.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "a",
            default: 0.0,
        },
        ParamSpec {
            name: "b",
            default: 0.0,
        },
        ParamSpec {
            name: "c",
            default: 0.0,
        },
        ParamSpec {
            name: "d",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Per-element bindings the `ph2d_expr` evaluator reads from.
struct ElemBindings<'a> {
    stream: &'a Stream,
    i: usize,
    n: usize,
    t: f32,
    params: &'a [(&'static str, f32)],
}

impl ph2d_expr::Bindings for ElemBindings<'_> {
    fn attr(&self, name: &str) -> f32 {
        match name {
            "i" => self.i as f32,
            "n" => self.n as f32,
            "t" => self.t,
            "f" => {
                if self.n <= 1 {
                    0.0
                } else {
                    self.i as f32 / (self.n as f32 - 1.0)
                }
            }
            _ => {
                if let Some(Column::Scalar(v)) = self.stream.get(name) {
                    return v.get(self.i).copied().unwrap_or(0.0);
                }
                self.param(name)
            }
        }
    }

    fn param(&self, name: &str) -> f32 {
        self.params
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .unwrap_or(0.0)
    }
}

struct MotionExpression;

impl NodeOp for MotionExpression {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let t = ctx.playhead() as f32;
        let params = [
            ("a", ctx.param("a")),
            ("b", ctx.param("b")),
            ("c", ctx.param("c")),
            ("d", ctx.param("d")),
        ];
        // The formula lives in the text channel; unset → the constant 0.
        let formula = ctx.text_param(EXPR_KEY).unwrap_or("0").to_string();
        let parsed = parse::parse(&formula);
        let input = ctx.input(0);
        let n = input.count().max(1);
        let v: Vec<f32> = match &parsed {
            Ok(expr) => (0..n)
                .map(|i| {
                    let b = ElemBindings {
                        stream: input,
                        i,
                        n,
                        t,
                        params: &params,
                    };
                    ph2d_expr::eval(expr, &b)
                })
                .collect(),
            // Parse error → a zero field (the editor badges the node).
            Err(_) => vec![0.0; n],
        };
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionExpression))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Expression",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[coeff("a"), coeff("b"), coeff("c"), coeff("d")];

const fn coeff(param: &'static str) -> ParamUiHint {
    ParamUiHint {
        param,
        label: param,
        min: -10.0,
        max: 10.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.expression.test.src"),
        name: "motion.expression.test.src",
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
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // 4 elements carrying a `v` column [10,20,30,40].
            ctx.emit(
                Stream::new(4)
                    .with("P", Column::Vec2(vec![[0.0, 0.0]; 4]))
                    .with("v", Column::Scalar(vec![10.0, 20.0, 30.0, 40.0])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionExpression),
                _ => None,
            }
        }
    }

    /// Cook the expression with a `expr` text param, returning its `v` field.
    fn eval_formula(formula: &str, set_params: &[(&str, f32)]) -> Vec<f32> {
        let mut g = Graph::new();
        let src = g.add_node("motion.expression.test.src");
        let ex = g.add_node("motion.expression");
        g.set_text_param(ex, EXPR_KEY, formula);
        for (n, val) in set_params {
            g.set_param(ex, *n, *val);
        }
        g.connect(Edge {
            from: (src, 0),
            to: (ex, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, ex, 0.0).unwrap();
        match out[0].as_stream().get("v").unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("v"),
        }
    }

    /// The `i` index variable and arithmetic: `i * 2 + 1` → per-element ramp. This is the
    /// end-to-end proof of the TEXT-PARAM plumbing (set_text_param → EvalCtx::text_param).
    /// FALSIFIED if the formula were ignored (all zeros).
    #[test]
    fn the_index_formula_evaluates_per_element() {
        assert_eq!(eval_formula("i * 2 + 1", &[]), vec![1.0, 3.0, 5.0, 7.0]);
    }

    /// A named input column (`v`) and a coefficient param (`a`) are readable in the
    /// formula: `v + a` adds the param to the carried value field.
    #[test]
    fn reads_input_columns_and_params() {
        assert_eq!(
            eval_formula("v + a", &[("a", 5.0)]),
            vec![15.0, 25.0, 35.0, 45.0]
        );
    }

    /// A function + the `n` count: `n` broadcasts, `abs(-i)` folds. Also proves `select`.
    #[test]
    fn functions_and_select_work() {
        assert_eq!(eval_formula("n", &[]), vec![4.0, 4.0, 4.0, 4.0]);
        assert_eq!(eval_formula("abs(0 - i)", &[]), vec![0.0, 1.0, 2.0, 3.0]);
        // select(i > 1, 100, 0): 0,0,100,100.
        assert_eq!(
            eval_formula("select(i > 1, 100, 0)", &[]),
            vec![0.0, 0.0, 100.0, 100.0]
        );
    }

    /// A malformed formula falls back to a zero field (no panic, no stale data).
    #[test]
    fn a_parse_error_falls_back_to_zero() {
        assert_eq!(eval_formula("i * (", &[]), vec![0.0, 0.0, 0.0, 0.0]);
        assert_eq!(eval_formula("bogus(i)", &[]), vec![0.0, 0.0, 0.0, 0.0]);
    }

    /// Registers through the registry.
    #[test]
    fn registers() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
