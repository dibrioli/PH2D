#![forbid(unsafe_code)]
//! `motion.bezier_warp` — o deformador de **fronteira CURVA** (doc 89, folha 04, P1).
//!
//! A referência é o *Bezier Warp* do After Effects: *"a closed Bezier curve along the
//! boundary… four segments. Each segment has three points (a vertex and two
//! tangents)"* — **4 vértices + 8 tangentes**. O interior sai por um **patch de
//! Coons** (ver [`coons`]), que interpola exactamente as quatro curvas.
//!
//! ## ⚠️ Por que ele é um NÓ e não um param do `motion.four_point_warp`
//!
//! O irmão é o *Corner Pin*: a homografia projectiva de Heckbert, que **mantém rectas
//! rectas** por construção. Este não pode ser um modo dele, e a razão é aritmética e
//! foi medida antes de uma linha ser escrita: com as tangentes nos terços a fronteira
//! deste vira um quadrilátero de lados rectos, e ali o Coons é o mapa **BILINEAR** —
//! que concorda com a homografia nos quatro CANTOS e **arqueia** as rectas interiores.
//! Os dois nós dão imagens diferentes para os mesmos quatro cantos, e é por isso que
//! o AE também os tem separados. Gate:
//! [`coons::tests::the_straight_edged_patch_is_bilinear_and_bends_the_interior_lines`].
//!
//! ## A superfície
//!
//! Como o irmão, tudo é **offset em unidades de mundo** a partir da caixa envolvente
//! do layout, e **tudo a zero é a identidade AO BIT** (as tangentes nascem nos terços,
//! onde a cúbica degenera na recta *por identidade polinomial*). A porta `warp` escala
//! todos os offsets de uma vez.
//!
//! `delta_i = coons(u_i, v_i) − P_i`, aplicado com a máscara `falloff` como todo
//! deformador desta família.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod coons;
mod params_ui;
use coons::Boundary;
use params_ui::{PARAM_GROUPS, PARAM_HINTS};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// O tipo da porta `warp` — espelho local do `VALUE` da família (crate-folha: o
/// vocabulário partilhado é a PORTA, nunca um símbolo importado).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Abaixo disto a caixa envolvente é degenerada (uma linha ou um ponto): não há
/// quadrado unitário para mapear, e o nó devolve o layout verbatim.
const EPS: f32 = 1e-6;

/// Os nomes dos oito offsets de CANTO, na ordem TL, TR, BR, BL — a mesma do irmão.
const CORNER_PARAMS: [[&str; 2]; 4] = [
    ["tl_dx", "tl_dy"],
    ["tr_dx", "tr_dy"],
    ["br_dx", "br_dy"],
    ["bl_dx", "bl_dy"],
];

/// Os nomes dos dezasseis offsets de TANGENTE, por lado (TOP, RIGHT, BOTTOM, LEFT) e
/// dentro do lado na direcção em que ele é percorrido.
const TANGENT_PARAMS: [[[&str; 2]; 2]; 4] = [
    [["top_a_dx", "top_a_dy"], ["top_b_dx", "top_b_dy"]],
    [["right_a_dx", "right_a_dy"], ["right_b_dx", "right_b_dy"]],
    [
        ["bottom_a_dx", "bottom_a_dy"],
        ["bottom_b_dx", "bottom_b_dy"],
    ],
    [["left_a_dx", "left_a_dy"], ["left_b_dx", "left_b_dy"]],
];

/// O manifesto: os 24 offsets, todos a zero.
///
/// ⚠️ **Vinte e quatro é a superfície da REFERÊNCIA, não inchaço** — o *Bezier Warp*
/// tem 12 pontos de controle, e cada ponto é `(x, y)`. O que os torna legíveis é o
/// agrupamento do painel ([`params_ui`]), não cortá-los: um lado sem as duas tangentes
/// não é uma cúbica.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.bezier_warp"),
    name: "motion.bezier_warp",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // A quantidade de warp, escalando TODO offset (animável). Desligada lê `1`.
        PortSpec {
            name: "warp",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "tl_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "tl_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "tr_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "tr_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "br_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "br_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "bl_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "bl_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "top_a_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "top_a_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "top_b_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "top_b_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "right_a_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "right_a_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "right_b_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "right_b_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "bottom_a_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "bottom_a_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "bottom_b_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "bottom_b_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "left_a_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "left_a_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "left_b_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "left_b_dy",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// A caixa envolvente do layout — o domínio do quadrado unitário.
fn bbox(p: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    let mut lo = [f32::MAX; 2];
    let mut hi = [f32::MIN; 2];
    for q in p {
        for k in 0..2 {
            lo[k] = lo[k].min(q[k]);
            hi[k] = hi[k].max(q[k]);
        }
    }
    (lo, hi)
}

/// A fronteira em coordenadas de MUNDO: o quadrado unitário mapeado na caixa, mais os
/// offsets escalados pelo `warp`.
///
/// ⚠️ Os offsets são somados **depois** do mapeamento, não antes: eles são unidades de
/// mundo (é o que um artista arrasta), e escalá-los pela caixa faria o mesmo número
/// significar coisas diferentes em layouts diferentes — a armadilha que a família
/// inteira evita.
fn boundary_in_world(lo: [f32; 2], hi: [f32; 2], off: &Offsets, warp: f32) -> Boundary {
    let unit = Boundary::unit();
    let to_world = |q: [f32; 2]| {
        [
            lo[0] + q[0] * (hi[0] - lo[0]),
            lo[1] + q[1] * (hi[1] - lo[1]),
        ]
    };
    let mut b = Boundary {
        corner: [[0.0; 2]; 4],
        tangent: [[[0.0; 2]; 2]; 4],
    };
    for c in 0..4 {
        let w = to_world(unit.corner[c]);
        b.corner[c] = [
            w[0] + warp * off.corner[c][0],
            w[1] + warp * off.corner[c][1],
        ];
    }
    for s in 0..4 {
        for t in 0..2 {
            let w = to_world(unit.tangent[s][t]);
            b.tangent[s][t] = [
                w[0] + warp * off.tangent[s][t][0],
                w[1] + warp * off.tangent[s][t][1],
            ];
        }
    }
    b
}

/// Os 24 offsets lidos do grafo.
struct Offsets {
    corner: [[f32; 2]; 4],
    tangent: [[[f32; 2]; 2]; 4],
}

impl Offsets {
    fn read(ctx: &mut EvalCtx<'_>) -> Self {
        let mut corner = [[0.0f32; 2]; 4];
        for (c, names) in CORNER_PARAMS.iter().enumerate() {
            corner[c] = [ctx.param(names[0]), ctx.param(names[1])];
        }
        let mut tangent = [[[0.0f32; 2]; 2]; 4];
        for (s, side) in TANGENT_PARAMS.iter().enumerate() {
            for (t, names) in side.iter().enumerate() {
                tangent[s][t] = [ctx.param(names[0]), ctx.param(names[1])];
            }
        }
        Self { corner, tangent }
    }

    /// `true` quando TODO offset é zero — o caminho da identidade.
    ///
    /// ⚠️ Ele existe para o nó recém-largado não pagar 24 leituras e um patch por
    /// elemento, e **não** para o resultado ser diferente: o patch neutro já é a
    /// identidade ao bit (gate `the_unit_boundary_gives_the_identity_patch`). É um
    /// atalho de custo sobre um resultado provado igual, nunca um segundo caminho.
    fn is_neutral(&self) -> bool {
        self.corner.iter().all(|c| c[0] == 0.0 && c[1] == 0.0)
            && self
                .tangent
                .iter()
                .all(|s| s.iter().all(|t| t[0] == 0.0 && t[1] == 0.0))
    }
}

fn positions(stream: &Stream) -> Vec<[f32; 2]> {
    match stream.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// O peso `falloff` do elemento `i` (ausente → `1.0`) — a convenção da família,
/// soletrada LOCALMENTE (crate-folha).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

struct MotionBezierWarp;

impl NodeOp for MotionBezierWarp {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let off = Offsets::read(ctx);
        // A porta `warp`: desligada é `1` (os offsets valem por inteiro).
        let warp = match ctx.input(1).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(1.0),
            _ => 1.0,
        };
        let out = {
            let input = ctx.input(0);
            let p = positions(input);
            if p.is_empty() || off.is_neutral() {
                input.clone()
            } else {
                let (lo, hi) = bbox(&p);
                let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
                if w < EPS || h < EPS {
                    // Uma linha ou um ponto não tem caixa 2D para deformar.
                    input.clone()
                } else {
                    let b = boundary_in_world(lo, hi, &off, warp);
                    let moved: Vec<[f32; 2]> = p
                        .iter()
                        .enumerate()
                        .map(|(i, q)| {
                            let (u, v) = ((q[0] - lo[0]) / w, (q[1] - lo[1]) / h);
                            let s = coons::coons(&b, u, v);
                            let f = falloff_at(input, i).clamp(0.0, 1.0);
                            [q[0] + (s[0] - q[0]) * f, q[1] + (s[1] - q[1]) * f]
                        })
                        .collect();
                    let mut out = Stream::new(p.len());
                    for (name, col) in input.columns() {
                        if name != "P" {
                            out.set(name.clone(), col.clone());
                        }
                    }
                    out.set("P", Column::Vec2(moved));
                    out
                }
            }
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionBezierWarp))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Bezier Warp",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    // ⚠️ **CPU-only: este nó lê `falloff` só no `eval`**, então o diagnoser não consegue
    // derivar o papel de uma `ColumnBinding` — ele TEM de ser declarado (ADR-0155). O
    // censo `every_cpu_only_falloff_reader_declares_it` apanhou exactamente esta linha em
    // falta no primeiro registo deste nó: *um nó novo não consegue saltar uma convenção
    // da casa em silêncio, e é essa a razão de o censo existir.*
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("falloff")],
    );
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
