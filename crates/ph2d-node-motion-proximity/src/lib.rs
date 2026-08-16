#![forbid(unsafe_code)]
//! `motion.proximity` — **a vizinhança vira um NÚMERO**: o *Neighbors* do MOPs e o
//! *POP Proximity* do Houdini (doc 89, folha 03 linhas 42 e 61). Lê a nuvem e escreve,
//! por elemento, **quantos discos ele toca** (`neighbours`) e **quão fundo é o pior
//! toque** (`overlap`). Não move nada, não apaga nada, não redimensiona nada.
//!
//! ## Por que ele existe: o número já era computado e jogado fora
//!
//! Duas vezes. O `motion.collide` conta os vizinhos de cada disco a cada varredura
//! (`contacts[i]`, e a penetração de cada par) e descarta os dois no fim; o
//! `motion.boids` mantém `neighbours` por agente e o usa só como divisor. Nenhum dos
//! **130 nós** do catálogo publica a vizinhança como coluna — medido, não suposto.
//!
//! ## O que ele DISSOLVE, e é a razão de ser um nó e não um `mode`
//!
//! O Push Apart Effector do C4D tem três modos — **Push · Scale · Hide** —, que a folha
//! 03 (linha 61) prescrevia como um param `mode` **dentro do `motion.collide`**, e que o
//! [doc 63 §2.2](../../../docs/Motion%20Nodes/63_pesquisa_industria_2026_e_plano_estado_da_arte.md)
//! propunha como um nó novo (`motion.push_apart`). Com a vizinhança publicada, **os três
//! são COMPOSIÇÃO** e as duas propostas caem:
//!
//! | modo do C4D | a cadeia, hoje |
//! |---|---|
//! | **Push** | `motion.collide` (é o que ele sempre foi) |
//! | **Scale** | `proximity → value.attribute(Overlap) → value.math(Subtract, 1−x) → motion.drive(Size, Multiply)` |
//! | **Hide** | `proximity → value.attribute(Overlap) → motion.drive(Falloff, Set) → motion.cull(Falloff, invert)` |
//!
//! ⚠️ **O Scale é EXATO, não aproximado.** Se o pior par está a `d = (1−o)·(r_i+r_j)`,
//! multiplicar os dois raios por `(1−o)` faz `r' = (1−o)(r_i+r_j) = d` — os discos passam
//! a apenas TOCAR-SE, que é literalmente o que o modo Scale do C4D promete. Um `mode`
//! dentro do solver teria de reimplementar essa multiplicação; aqui ela é o
//! `motion.drive(Size)` que já shipa.
//!
//! ⚠️ E o **Hide muda a CONTAGEM**, que nesta engine é a família do `StreamOp::Compact`
//! (ADR-0136, com lei de contagem própria). Enfiá-la num solver de posição alargaria o
//! `motion.collide` para *mover · redimensionar · apagar* — três verbos num nó — e cada
//! um pediria a sua metade do kernel de device. O `motion.cull` já é o nó que apaga.
//!
//! ## A LEI é a do `motion.collide`, verbatim — e é isso que os faz concordar
//!
//! O disco de `i` tem raio `r_i = radius · max(|size_i.x|, |size_i.y|)` (o `radius_scale`
//! do collide, que é o `pscale` do POP Interact: a coluna que o RENDERER desenha), e um
//! par **toca** quando `d < r_i + r_j`. Partilhar a lei não é zelo: é o que garante que
//! *o que este nó reporta como sobreposto é exactamente o que aquele empurraria* — duas
//! definições de "sobrepor" na mesma biblioteca divergiriam no primeiro tamanho não
//! uniforme, e o artista veria o Scale encolher um disco que o Push deixou em paz.
//!
//! - `neighbours[i]` = quantos discos `j ≠ i` tocam `i`.
//! - `overlap[i]` = `max_j (1 − d/(r_i+r_j))` sobre os que tocam, em `(0, 1]`; **0** quando
//!   nenhum toca. Coincidentes (`d ≈ 0`) dão **1**, o mesmo canto que o collide trata como
//!   penetração cheia.
//!
//! ⚠️ **A CONTAGEM é bit-exacta CPU×GPU; a FRACÇÃO não — e a segunda metade desta frase foi
//! escrita pela medição, não pelo raciocínio.** `max` é exacto em qualquer ordem e uma contagem
//! é inteira (um `f32` representa todo inteiro até 2²⁴, muito acima de qualquer contagem que a
//! engine dispare), então a ordem de visita não é observável — o irmão `motion.collide` não tem
//! isso porque **SOMA** correcções (ADR-0140 D4). Mas o número que entra no `max` sai de
//! `dot(d, d)`, que o WGSL pode **fundir num `fma`**: medido com entrada bit-exacta dos dois
//! lados, a fracção diverge **um ULP** (`5,96e-8`) e a contagem **zero**. Uma decisão sobrevive
//! a um ULP (só a fronteira exacta mudaria, medida zero); um valor aritmético não.
//!
//! ## Ele MEDE, então não lê `falloff`
//!
//! A espinha MOPs (*todo modificador é modulado por `falloff`*) vale para quem **modifica**.
//! Este nó reporta um facto sobre o layout, e uma medição mascarada MENTE: o
//! `value.attribute` a jusante não teria como saber que o número que recebeu foi
//! atenuado. É o precedente exacto do `motion.velocity` — que também mede e também não lê
//! a máscara. Quem quiser mascarar o EFEITO mascara o `motion.drive` que está no fim da
//! cadeia, onde a máscara pertence.
//!
//! ⚠️ **E ele não declara `Coupling::Produces`**, embora escreva duas colunas. O consumidor
//! canónico é o `value.attribute`, que lê a coluna que o **ARTISTA nomeou num text param** e
//! por isso não declara `Consumes` nenhum; declarar a produção marcaria como INERTE
//! exactamente a cadeia que este nó existe para servir — *a afirmação seria o próprio bug*,
//! a lição que o `MissingSource("P")` do Boids já pagou (ADR-0155). O diagnosticador DERIVA
//! `produces` da binding `Write` do kernel, e isso é inofensivo porque a análise de produtor
//! inerte filtra por `TRANSIENT_COLUMNS` (`accel`/`falloff`/`inv_mass`) — `neighbours` e
//! `overlap` **não entram nessa lista, de propósito**, pela mesma razão.
//!
//! `Effect::Pure` (função do input, sem estado), transcendental-free a menos de `sqrt`
//! (HR-5).

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod gpu;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The per-instance scale the RENDERER draws with (`lower_to_instances` reads it),
/// identity `[1, 1]` when absent — the same column `motion.collide` rides, spelled
/// locally by each reader (like `P` / `falloff`) rather than coupling the crates.
const SIZE_COL: &str = "size";

/// **Quantos discos este toca.** Uma contagem, não uma densidade normalizada: a
/// biblioteca normaliza (`value.map_range`), e um número já dividido por um máximo que
/// o nó escolheu seria uma decisão de apresentação assada no dado.
pub const NEIGHBOURS_COL: &str = "neighbours";

/// **Quão fundo é o PIOR toque**, em `[0, 1]` — `0` = livre, `1` = coincidente.
pub const OVERLAP_COL: &str = "overlap";

/// Abaixo disto um par é tratado como coincidente (a direção é indefinida) — o `EPS`
/// do `motion.collide`, ao dígito, porque a lei é a dele.
const EPS: f32 = 1e-9;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.proximity"),
    name: "motion.proximity",
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
        // O raio do disco — a MESMA grandeza do `radius` do `motion.collide`, e o mesmo
        // default, para que `proximity(r) → collide(r)` concorde sobre quem se toca.
        ParamSpec {
            name: "radius",
            default: 0.3,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **O raio de CADA disco, lido da coluna que o renderer desenha** — o verbatim do
/// `motion_collide::radius_scale`. `max(|x|, |y|)` porque o disco CONTÉM a arte; `abs`
/// porque uma extensão não tem sinal (uma instância espelhada tem o mesmo tamanho); e
/// não-finito lê como a identidade `1`, ou seja como se a coluna estivesse ausente.
fn radius_scale(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(SIZE_COL) {
        Some(Column::Vec2(v)) if v.len() == n => v
            .iter()
            .map(|e| {
                let m = e[0].abs().max(e[1].abs());
                if m.is_finite() { m } else { 1.0 }
            })
            .collect(),
        _ => vec![1.0; n],
    }
}

/// A varredura de pares: devolve `(neighbours, overlap)`.
///
/// ⚠️ **A contagem acumula num `f32` e isso é EXACTO** — todo inteiro até 2²⁴ tem
/// representação, e a contagem é limitada por `n`. Guardá-la num `u32` para converter no
/// fim daria o mesmo número e uma segunda representação do mesmo facto.
///
/// ⚠️ **O `overlap` é um `max`, nunca uma soma**, e é isso que o torna independente da
/// ordem de visita — a propriedade que o `motion.collide` teve de conquistar trocando
/// Gauss–Seidel por Jacobi médio, e que aqui vem de graça pela operação escolhida.
fn measure(p: &[[f32; 2]], radii: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let n = p.len();
    let mut neighbours = vec![0.0f32; n];
    let mut overlap = vec![0.0f32; n];
    for i in 0..n {
        for j in (i + 1)..n {
            // `r_i + r_j`, simétrica — e exactamente `2·radius` quando os tamanhos são
            // uniformes, já que `x + x` é exacto em IEEE-754.
            let min_dist = radii[i] + radii[j];
            if min_dist <= 0.0 {
                continue;
            }
            let dx = p[j][0] - p[i][0];
            let dy = p[j][1] - p[i][1];
            let d2 = dx * dx + dy * dy;
            if d2 >= min_dist * min_dist {
                continue;
            }
            // A fracção sobreposta: `1` quando coincidentes, `→0` quando apenas se tocam.
            let frac = if d2 > EPS {
                1.0 - d2.sqrt() / min_dist
            } else {
                1.0
            };
            neighbours[i] += 1.0;
            neighbours[j] += 1.0;
            overlap[i] = overlap[i].max(frac);
            overlap[j] = overlap[j].max(frac);
        }
    }
    (neighbours, overlap)
}

struct MotionProximity;

impl NodeOp for MotionProximity {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let base = ctx.param("radius");
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        // `base · s_i` — o raio global vezes a escala deste elemento. Sem coluna `size`
        // toda escala é `1` e `base * 1.0` é `base` exactamente.
        let radii: Vec<f32> = radius_scale(input, n).iter().map(|s| base * s).collect();
        let (neighbours, overlap) = measure(&p, &radii);
        let mut out = Stream::new(n);
        // Um segundo `proximity` na cadeia SUBSTITUI a resposta do primeiro; ele não a
        // funde. Duas medições da mesma pergunta com raios diferentes são duas perguntas,
        // e a última é a que o artista acabou de fazer.
        for (name, col) in input.columns() {
            if name != NEIGHBOURS_COL && name != OVERLAP_COL {
                out.set(name.clone(), col.clone());
            }
        }
        out.set(NEIGHBOURS_COL, Column::Scalar(neighbours));
        out.set(OVERLAP_COL, Column::Scalar(overlap));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionProximity))?;
    // GPU: a mesma grade de vizinhança do `motion.collide`, mas UM dispatch — não há
    // varredura a repetir, porque nada se move. `sweeps_param: None`.
    reg.register_gpu_kernel(MANIFEST.id, gpu::GPU_KERNEL);
    reg.register_grid(MANIFEST.id, gpu::GRID);
    reg.register_reduces(MANIFEST.id, gpu::REDUCES);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Proximity",
            // A categoria do `motion.velocity` — o irmão exacto em papel (mede a nuvem e
            // escreve o número numa coluna). O artista que achou um procura o outro no
            // mesmo lugar, e isso vale mais que a taxonomia: `Utility` descreveria
            // melhor *o que ele É* e escondê-lo-ia de quem o procura.
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "radius",
    label: "Radius",
    min: 0.0,
    max: 5.0,
    step: 0.01,
    widget: ParamWidget::Slider,
}];

/// **SEM `ParamHardMax`, e a ausência é MEDIDA** (CLAUDE.md §0 · doc 88 B2).
///
/// Sonda: `measure_proximity_cost`. O `radius` deste nó não tem um recurso atrás dele —
/// ele é ao mesmo tempo o raio do disco **e** a célula da grade, então crescê-lo aumenta
/// o alcance e a célula na MESMA proporção: o número de células varridas fica constante
/// (`reach = ceil((r_i + r_max)/r) = 2` com tamanhos uniformes, em qualquer raio). O que
/// de facto cresce é **quantos discos caem dentro do alcance**, e isso é uma propriedade
/// da CENA (a densidade que o artista construiu), limitada por `n` — a mesma fronteira
/// que o `motion.collide` tem, e pela qual ele também não declara teto de `radius`.
///
/// Um número digitável grande sobre uma nuvem esparsa é **barato e correcto**; sobre uma
/// nuvem densa é o O(n²) que a pergunta *"quem está perto de quem"* custa quando todos
/// estão perto de todos. Escrever um teto aqui seria capar a pergunta pelo caso pior de
/// outra cena.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "radius",
    unit: ParamUnit::Length,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
