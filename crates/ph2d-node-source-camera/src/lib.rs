#![forbid(unsafe_code)]
//! `source.camera` — **a VISTA entra no grafo** (doc 89 folha 14 §3 item 2; ciclo 8, doc 113 §5).
//!
//! A conferência nomeou-o e mediu-o assim: *«Blender GN `Active Camera`/`Camera Info` … sem ela,
//! **orientar para a câmera**, **escalar com o zoom**, **distribuir na área visível** e o **culling**
//! são todos inexprimíveis. Exprimível? NÃO (nada publica a câmera). P1, custo quase zero.»* — e
//! ficou aberto porque era uma linha de PROSA da folha, não uma célula da tabela que o placar conta.
//!
//! ## Uma linha, três respostas
//!
//! O nó emite **UMA** instância, e ela É a vista:
//!
//! | coluna | o que é |
//! |---|---|
//! | `P` | o CENTRO da câmara, em unidades de mundo |
//! | `size` | a EXTENSÃO visível (largura × altura), em unidades de mundo |
//! | `zoom` | quantos pixels de ecrã vale uma unidade de mundo |
//!
//! ⚠️ **O `size` é a extensão de propósito, e não um nome próprio.** Ele é a coluna que a casa já
//! usa para *«que tamanho tem esta coisa»*, então a primeira coisa que se pode fazer com este nó —
//! ligá-lo ao `motion.output` — desenha **exactamente o rectângulo visível**, que é honesto e
//! imediatamente legível. Um nome próprio (`view_w`/`view_h`) obrigaria cada consumidor a aprender
//! outra palavra para a mesma pergunta.
//!
//! ⚠️ **O `zoom` é PX POR UNIDADE**, que é o sentido em que ele se usa: uma peça que queira medir
//! sempre os mesmos pixels no ecrã vale `pixels / zoom` unidades de mundo. O inverso seria igualmente
//! defensável e obrigaria a uma divisão em todo uso; a unidade escolhida é a que o consumidor lê.
//!
//! ## A porta é a do CURSOR, e isso é o desenho
//!
//! Um nó recebe params, entradas e o playhead — nada mais ([`ph2d_nodegraph::external`]). A câmara é
//! um valor do EDITOR que muda a cada quadro e não é documento, exactamente como o cursor: ela chega
//! pelo mesmo canal reservado (`$camera`, [`ph2d_nodegraph::external::CAMERA`]), publicada pela
//! mesma membrana que publica `$cursor`. ⚠️ **Sem shell não há vista**: um cook sem publicação lê o
//! external vazio ⇒ o nó emite **nada**, não adivinha e não falha — a lei do `source.object`.
//!
//! `Effect::Pure`: a saída é função do external, cuja revisão é o CONTEÚDO (a câmara parada não
//! invalida nada). `LoweringKind::Cpu` — uma linha por quadro atravessa a costura, e é uma linha.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// A coluna com os pixels de ecrã por unidade de mundo — ver o cabeçalho.
pub const ZOOM_COLUMN: &str = "zoom";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.camera"),
    name: "source.camera",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // ⚠️ **Zero params, e a ausência é a decisão:** a vista não se AUTORA aqui — ela é o que o
    // artista já fez com o rato. Um param neste cartão seria uma segunda resposta a *onde a câmara
    // está*, e as duas divergiriam no primeiro arrasto.
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct SourceCamera;

impl NodeOp for SourceCamera {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // A membrana publicou `(P, size, zoom)` sob a chave reservada. Clone é refcount; uma chave
        // sem publicação é o external vazio ⇒ stream vazio.
        let stream = ctx.external(ph2d_nodegraph::external::CAMERA).clone();
        ctx.emit(stream);
    }
}

/// Registra o nó no registry de runtime. Chamado (via codegen) do
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SourceCamera))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Camera",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::Graph;

    fn registry() -> NodeRegistry {
        let mut reg = NodeRegistry::new();
        register(&mut reg).expect("regista");
        reg
    }

    /// A vista publicada: centro `(3, −1)`, `16 × 9` unidades visíveis, `50` px por unidade.
    fn vista() -> Stream {
        Stream::new(1)
            .with("P", Column::Vec2(vec![[3.0, -1.0]]))
            .with("size", Column::Vec2(vec![[16.0, 9.0]]))
            .with(ZOOM_COLUMN, Column::Scalar(vec![50.0]))
    }

    /// ⭐⭐ **O QUE A MEMBRANA PUBLICA É O QUE O NÓ EMITE** — as três colunas, ao bit.
    #[test]
    fn the_node_emits_the_view_the_shell_published() {
        let reg = registry();
        let mut g = Graph::new();
        let n = g.add_node("source.camera");
        let mut cook = Cook::new();
        cook.set_external(ph2d_nodegraph::external::CAMERA.to_string(), vista());
        let out = cook.cook(&g, &reg, n, 0.0).expect("coze");
        let s = out[0].as_stream();
        assert_eq!(s.count(), 1, "a vista e' UMA instancia");
        assert_eq!(s.get("P"), vista().get("P"), "o centro");
        assert_eq!(s.get("size"), vista().get("size"), "a extensao visivel");
        assert_eq!(s.get(ZOOM_COLUMN), vista().get(ZOOM_COLUMN), "o zoom");
    }

    /// ⚠️ **SEM SHELL não há vista** — e o nó emite NADA, em vez de adivinhar uma câmara.
    ///
    /// ⛔ Sem esta metade, um default plausível (centro na origem, `10` unidades) faria um grafo
    /// cozido fora do app desenhar uma vista que ninguém escolheu — e o defeito lê-se como *«a
    /// câmara não me segue»*, que é a pergunta errada.
    #[test]
    fn without_a_published_view_it_emits_nothing() {
        let reg = registry();
        let mut g = Graph::new();
        let n = g.add_node("source.camera");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &reg, n, 0.0).expect("coze");
        assert_eq!(
            out[0].as_stream().count(),
            0,
            "sem publicacao, stream vazio"
        );
    }

    /// ⚠️ **A chave é a RESERVADA** — um objecto do artista chamado `$camera` é recusado na
    /// publicação (a lei do `$cursor`), e é isso que impede que ele VIRE a câmara.
    #[test]
    fn the_camera_key_lives_in_the_editors_namespace() {
        assert!(ph2d_nodegraph::external::is_reserved(
            ph2d_nodegraph::external::CAMERA
        ));
    }
}
