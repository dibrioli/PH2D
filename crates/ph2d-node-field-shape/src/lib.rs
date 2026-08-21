#![forbid(unsafe_code)]
//! `field.shape` — **a GEOMETRIA como campo**: a máscara `falloff` sai da distância de
//! cada elemento a uma forma ligada, em vez de a uma caixa ou a um raio (doc 89 folha
//! 08 / folha 10).
//!
//! É o *Shape Type → **Shape** (usa Input Shapes)* do Cavalry, com o *Path Mode*
//! (**Filled Path** / **Path Edges**) dele; no C4D é *"objeto qualquer arrastado
//! (mesh→distância) como camada de field"*; nos MOPs é o *Spline/Object Falloff* com a
//! sua *Area of Influence*.
//!
//! ⚠️ **O gap era dos DOIS lados da porta, e é por isso que ele não era redundância:**
//! nem o `motion.falloff` nem nenhum dos cinco `field.*` aceitava uma geometria como
//! fonte de máscara. Este é o nó que a aceita, e a fiação já estava provada — o
//! `field.combine` tem duas portas há waves.
//!
//! ## A lei
//!
//! Para cada elemento, `d` é a distância dele à **fronteira** do polígono formado pelos
//! pontos da porta `shape` (a aresta de fecho incluída), e `w = clamp(d / distance)`:
//!
//! - **Filled Path** (0) — dentro é `1` cheio; fora decai de `1` na borda a `0` a
//!   `distance` de afastamento. A forma é uma máscara SÓLIDA com penumbra por fora.
//! - **Path Edges** (1) — decai da borda para os **dois** lados. A forma é um CONTORNO,
//!   e o miolo dela fica tão vazio quanto o exterior.
//!
//! O resultado **multiplica** no `falloff` que já existir (o contrato MOPs que faz os
//! campos comporem), e toda outra coluna passa intacta. `Pure`.
//!
//! ⚠️ **A porta vazia é a IDENTIDADE.** Um nó recém-largado, sem forma ligada, escreve
//! `1` em todo elemento — o `falloff` sai byte-idêntico. *Um campo sem geometria não é
//! um campo vazio; é um campo que ainda não foi perguntado.*
//!
//! ⚠️ **Menos de três pontos degrada com graça, e de propósito:** dois pontos são um
//! segmento e um ponto é um ponto — a distância continua a fazer sentido, e o
//! `Filled Path` simplesmente não tem interior (nada é *dentro* de um segmento). Assim
//! uma forma a ser construída ao vivo nunca pisca para preto.
//!
//! ⚠️ **HR-5:** só `min`/`max`/`clamp`/`sqrt` e polinómios. O `sqrt` do IEEE-754 é
//! correctamente arredondado, então a máscara é a mesma em toda plataforma.
//!
//! ⚠️ **CPU-only, e o motivo tem NOME.** O canal que leva uma porta-template ao device
//! ([`ph2d_nodegraph::gpu::ColumnAccess::SourceRead`], ADR-0136) só existe emparelhado
//! com um `StreamOp::SourceRows` — um nó que MUDA a contagem lendo o template. Este
//! preserva a contagem, e para essa forma não há canal hoje. Escrever um seria
//! foundational, não um nó; a alternativa (declarar `SourceRows` sobre a porta 0 só
//! para ganhar o leitor) seria mentir ao plano sobre o que o nó faz.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// `mode = 0`: dentro é sólido, e a penumbra é só por fora.
const MODE_FILLED: i32 = 0;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.shape"),
    name: "field.shape",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // A GEOMETRIA. Os `P` desta porta são os vértices do polígono, na ordem em
        // que chegam; a aresta de fecho é implícita. Desligada ⇒ a identidade.
        PortSpec {
            name: "shape",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Filled Path · 1 Path Edges (o *Path Mode* do Cavalry).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // A largura da penumbra, em unidades de MUNDO — a *Area of Influence* dos
        // MOPs. `0` dá uma borda dura.
        ParamSpec {
            name: "distance",
            default: 1.0,
        },
        // A mesma família de 4 curvas do `motion.falloff`/`field.index_range`.
        ParamSpec {
            name: "curve",
            default: 2.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the SAME transcendental-free set as
/// `motion.falloff` (HR-5). `0` Linear · `1` Quad · `2` Smooth · `3` Smoother. Every
/// curve is monotone and endpoint-exact (`0→0`, `1→1`).
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,
        2 => s * s * (3.0 - 2.0 * s),
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0),
        _ => s,
    }
}

/// A distância QUADRADA de `p` ao segmento `a→b`.
///
/// ⚠️ Quadrada até ao fim de propósito: a raiz sai **uma vez**, sobre o mínimo, em vez
/// de uma por aresta. Numa forma de 64 vértices isso é 63 raízes a menos por elemento,
/// e a resposta é a mesma (a raiz é monótona).
fn dist2_to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
    let (wx, wy) = (p[0] - a[0], p[1] - a[1]);
    let len2 = vx * vx + vy * vy;
    // Segmento degenerado (dois vértices coincidentes): a projecção é o próprio `a`.
    let t = if len2 > 0.0 {
        ((wx * vx + wy * vy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (wx - t * vx, wy - t * vy);
    dx * dx + dy * dy
}

/// A distância de `p` à fronteira do polígono (a aresta de fecho incluída). Um polígono
/// VAZIO devolve `None` — é o sinal de *"nada ligado"*, e quem chama responde com a
/// identidade em vez de com um número.
fn boundary_distance(p: [f32; 2], poly: &[[f32; 2]]) -> Option<f32> {
    let n = poly.len();
    if n == 0 {
        return None;
    }
    if n == 1 {
        let (dx, dy) = (p[0] - poly[0][0], p[1] - poly[0][1]);
        return Some((dx * dx + dy * dy).sqrt());
    }
    let mut best = f32::INFINITY;
    for i in 0..n {
        // `(n − 1) → 0` é a aresta de FECHO: sem ela um polígono seria uma polilinha, e
        // o teste de interior abaixo (que fecha sempre) discordaria da distância.
        let j = if i + 1 == n { 0 } else { i + 1 };
        best = best.min(dist2_to_segment(p, poly[i], poly[j]));
    }
    Some(best.sqrt())
}

/// Teste de interior por paridade de cruzamentos (even-odd), o mesmo do
/// `ph2d-vec-scene` e de toda a literatura. Menos de três vértices não tem interior.
///
/// ⚠️ A comparação `(pi.y > y) != (pj.y > y)` é a forma canónica **porque ela conta
/// cada vértice exactamente uma vez**: um `>=` de um lado contaria duas vezes uma
/// aresta que toca a linha do raio, e o ponto sairia fora do próprio polígono.
fn inside_polygon(p: [f32; 2], poly: &[[f32; 2]]) -> bool {
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut c = false;
    let mut j = n - 1;
    for i in 0..n {
        let (pi, pj) = (poly[i], poly[j]);
        if (pi[1] > p[1]) != (pj[1] > p[1]) {
            let x = (pj[0] - pi[0]) * (p[1] - pi[1]) / (pj[1] - pi[1]) + pi[0];
            if p[0] < x {
                c = !c;
            }
        }
        j = i;
    }
    c
}

/// A máscara de um elemento. Ver os docs do módulo para os dois modos.
fn shape_mask(
    p: [f32; 2],
    poly: &[[f32; 2]],
    mode: i32,
    distance: f32,
    curve_kind: i32,
    invert: bool,
) -> f32 {
    let Some(d) = boundary_distance(p, poly) else {
        // Nada ligado: a identidade — e ela ignora o `invert`, porque inverter
        // "nenhum campo" continua a ser nenhum campo, não um campo cheio.
        return 1.0;
    };
    // `distance <= 0` é a borda dura: só quem está exactamente sobre ela recebe 1.
    let w = if distance > 0.0 {
        (d / distance).clamp(0.0, 1.0)
    } else {
        f32::from(d > 0.0)
    };
    let ramp = curve(curve_kind, 1.0 - w);
    let m = if mode == MODE_FILLED && inside_polygon(p, poly) {
        1.0
    } else {
        ramp
    };
    if invert { 1.0 - m } else { m }
}

/// Os vértices da porta-template, na ordem em que chegam.
fn poly_of(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

struct FieldShape;

impl NodeOp for FieldShape {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i32;
        let distance = ctx.param("distance");
        let curve_kind = ctx.param("curve").round() as i32;
        let invert = ctx.param("invert") >= 0.5;
        // ⚠️ O template é lido ANTES do input 0 e clonado: os dois `ctx.input` não
        // podem coexistir emprestados.
        let poly = poly_of(ctx.input(1));
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let pos: Option<&[[f32; 2]]> = match input.get("P") {
                Some(Column::Vec2(v)) => Some(v.as_slice()),
                _ => None,
            };
            let prev: Option<&[f32]> = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            let fall = par_build(n, |i| {
                let p = pos.and_then(|v| v.get(i).copied()).unwrap_or([0.0, 0.0]);
                let m = shape_mask(p, &poly, mode, distance, curve_kind, invert);
                let base = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                base * m
            });
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "falloff" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("falloff", Column::Scalar(fall));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via `ph2d-node-sync` codegen)
/// from `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FieldShape))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Shape Field",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // CPU-only (ver os docs do módulo): não há `ColumnBinding` de onde o diagnoser
    // pudesse derivar o papel deste nó, então ele é DECLARADO (ADR-0155). `Produces`
    // porque um `falloff` que ninguém consome é inerte — e é exactamente o defeito
    // silencioso que aquele ADR existe para nomear.
    reg.register_couplings(
        MANIFEST.id,
        &[
            ph2d_node_registry::Coupling::Produces("falloff"),
            ph2d_node_registry::Coupling::Requires("P"),
        ],
    );
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Path Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Filled Path", "Path Edges"],
        },
    },
    // ⚠️ A faixa é de MUNDO, como a do `field.box`: um campo cuja penumbra fosse uma
    // fracção não teria como ser comparado com a forma que o alimenta.
    ParamUiHint {
        param: "distance",
        label: "Distance",
        min: 0.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "curve",
        label: "Curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

/// A penumbra é um COMPRIMENTO de mundo, e a linha do painel diz isso (doc 88) — a
/// mesma unidade que o raio de um `field.box`, porque é a mesma grandeza.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "distance",
    unit: ParamUnit::Length,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
