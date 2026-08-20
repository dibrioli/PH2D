//! `value.cursor` — the value-domain PRODUCER of the **world cursor**: where the mouse is,
//! as two plain values (Motion Nodes M2, the value domain — doc 12/80; doc 89 folha 08, a
//! célula do `followMouse`).
//!
//! **Por que um nó, e não um `follow_cursor` em cada nó que tem centro** (doc 89 folha 08 —
//! Cavalry põe `followMouse` no próprio *Falloff*, e o gizmo dele é arrastável na tela):
//!
//! - **A lei fica escrita UMA vez.** O centro existe em `motion.falloff`, `field.box` e
//!   `field.radial_sweep` — um toggle por nó seriam três cópias da mesma leitura, e duas
//!   implementações de uma lei é como elas divergem.
//! - **Serve TODO param, não os centros.** Um param dirigido (doc 58) chega a qualquer
//!   parâmetro de qualquer um dos 86 tipos de nó *"sem uma linha de mudança em nenhum
//!   deles"* — então isto liga o rato ao raio de um falloff, ao ângulo de um `motion.rotate`,
//!   ao `blend` de um mixer. Um toggle só saberia mover um centro.
//! - **Não abre um buraco CPU/GPU.** Os três nós de campo têm kernel WGSL que lê
//!   `params.center_x`; substituir o centro dentro do `eval` faria o device continuar a ler o
//!   número autorado e as duas metades divergiriam em silêncio. O plano de GPU **já recusa**
//!   um nó com param dirigido (`plan.rs`), então a rota composta cai para a CPU por um
//!   caminho que já existe e já é testado, em vez de um `applicable` novo em cada nó.
//!
//! **Duas saídas, `x` e `y`** — o cursor é UMA coisa com dois componentes, e o contrato do
//! motor diz *"call once per output port, in order"*. ⚠️ **É o primeiro nó do repo com duas
//! saídas**, e o gate `both_ports_cook_and_they_are_not_the_same_number` prova as duas em vez
//! de assumir a segunda. A alternativa (uma saída + um param `axis`) custaria **dois nós**
//! para seguir um centro 2D, todas as vezes, para sempre.
//!
//! **Cardinalidade segue a geometria** (o padrão do `value.lfo`/`value.time`): a porta `in` é
//! lida **só pela contagem** — ligada ⇒ um campo de comprimento N com o mesmo número em todas
//! as linhas, desligada ⇒ o global de comprimento 1, que é o caso comum e o que um param
//! dirigido lê. Nada da entrada passa através.
//!
//! ⚠️ **Cru de propósito: sem `scale`, sem `offset`.** *"Siga o rato, dois à direita"* é um
//! `value.math(Add)`, que já existe; pôr um offset aqui seria uma segunda porta para a mesma
//! pergunta, e a resposta ficaria em dois sítios.
//!
//! ⚠️ **Ausente, o canal lê ZERO e não a origem por acidente** — ver [`cursor_at`]. Ele é
//! publicado pelo editor a cada quadro (`external::CURSOR`); num cozimento de teste, ou num
//! host sem rato, ele simplesmente não está lá.
//!
//! `Effect::Pure`, como o `motion.look_at` que lê o mesmo canal: o memo do cozimento guarda
//! **quais externals um nó leu e em que revisão**, então um Pure que lê o cursor volta a
//! cozer quando ele mexe. CPU-only — o valor vem de um external, que um kernel não alcança.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The instance stream type — read for its count only (the optional `in` port).
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the per-instance scalar field on the `v` column (the local mirror every
/// leaf of this house writes; the shared vocabulary is the PORT, not a shared symbol).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.cursor"),
    name: "value.cursor",
    // Optional: connected → a length-N field; unconnected → the length-1 global.
    // Read for its count only; never passed through.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[
        PortSpec {
            name: "x",
            ty: VALUE,
        },
        PortSpec {
            name: "y",
            ty: VALUE,
        },
    ],
    // Pure: it reads an external, and the cook's memo tracks the externals a node read at
    // the revisions they have NOW — so this re-cooks exactly when the cursor moves. Same
    // effect the sibling reader of this channel (`motion.look_at`) declares.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    // CPU-only: the number comes from an EXTERNAL, and a WGSL kernel has no way to read
    // one — a kernel here would have to be fed the cursor as a param, which is a second
    // path to the same law.
    lowerings: &[LoweringKind::Cpu],
};

/// **Onde o rato está**, do canal que o editor publica a cada quadro.
///
/// ⚠️ **Ausente ⇒ `[0, 0]`, e a escolha é deliberada.** O irmão `motion.look_at` devolve
/// `None` aqui e cai para o ponto autorado — ele TEM um ponto autorado para onde cair. Este
/// nó não tem: a única outra resposta seria não emitir coluna nenhuma, e um param dirigido
/// por uma coluna ausente lê o default do param, o que faria o centro **saltar** de volta
/// para o valor antigo no primeiro quadro em que o editor não publicasse. Zero é uma posição
/// de mundo real (a origem), fica parado, e é o que um host sem rato deve mostrar.
fn cursor_at(ctx: &mut EvalCtx<'_>) -> [f32; 2] {
    let s = ctx.external(ph2d_nodegraph::external::CURSOR);
    let Some(Column::Vec2(p)) = s.get("P") else {
        return [0.0, 0.0];
    };
    if p.is_empty() {
        return [0.0, 0.0];
    }
    // A média, pela mesma razão do `motion.look_at`: o canal é um STREAM, e um publisher que
    // um dia publique mais de um ponto (dois dedos, dois cursores) tem de dar um número e não
    // o primeiro por acidente.
    #[expect(
        clippy::cast_precision_loss,
        reason = "uma contagem de pontos publicados; a média é uma posição de ecrã"
    )]
    let n = p.len() as f32;
    let sum = p
        .iter()
        .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
    [sum[0] / n, sum[1] / n]
}

struct ValueCursor;

impl NodeOp for ValueCursor {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let [x, y] = cursor_at(ctx);
        // A contagem segue a geometria: desligada, o global de comprimento 1.
        let n = ctx.input(0).count().max(1);
        // ⚠️ Duas emissões, na ORDEM das saídas do manifesto — o contrato do `emit` é
        // *"call once per output port, in order"*, e o cozimento reprova uma contagem
        // diferente de `outputs.len()`.
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(vec![x; n])));
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(vec![y; n])));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueCursor))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Cursor",
            // Utility grey: a value SOURCE, plumbing — não uma transformação visível.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
