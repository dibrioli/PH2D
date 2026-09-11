//! **A COMPARAÇÃO E O NOME QUE NÃO RESOLVE** (`PH2D_GPU_COOK_DEMO=45`) — a cena do
//! **grupo E** da conferência (doc 89, folha 15).
//!
//! ## As duas metades são a mesma frase, vista de dois lados
//!
//! *Perguntas que o grafo não sabia fazer.* Ele não sabia perguntar **"a é maior
//! que b?"** em um nó só — precisava de dois, e de quatro para *"a é quase igual a
//! b?"*. E ele não sabia **DIZER** que um nome de coluna não resolve: lia zeros no
//! comprimento certo, cozia sem erro, e a cena ficava parada.
//!
//! ## As três leituras
//!
//! **Bandas 1-2 — O OP É LIDO.** A MESMA rampa contra o MESMO limiar (`0,5`), uma
//! com `Greater` e outra com `Less`: dois degraus **complementares** (a de cima
//! sobe onde a de baixo desce). ⚠️ Se as duas desenharem o mesmo degrau, o kernel
//! não está a ler o `op`.
//!
//! **Bandas 3-4 — A TOLERÂNCIA É LIDA.** As duas em `Equal` contra o mesmo `0,5`, e
//! só o `epsilon` difere (`0,05` contra `0,20`): uma **BANDA ESTREITA** de peças
//! levantadas no meio da fileira, e outra bem mais larga (medido: **4 peças contra
//! 18**, numa fileira de 48). ⚠️ É o par
//! que um kernel cego a `params.epsilon` não consegue desenhar — ele daria duas
//! bandas idênticas.
//!
//! **Banda 5 — O NOME QUE NÃO RESOLVE.** Um `value.attribute` a ler **`velocty`**
//! (o typo de `vel`) de uma grade, que não carrega nem `vel` nem `velocty`.
//! ⚠️ **A fileira sai PLANA, e isso é o desenho CERTO** — a escada devolve zeros no
//! comprimento certo, que é exactamente a classe de erro que não produz erro. O que
//! a wave acrescenta não está no canvas: está no **painel de grafo**, onde o nó
//! ganha o badge ⚠, e clicar nele diz *"nothing upstream carries a column called
//! 'velocty'"*.
//!
//! ⚠️ **A banda 5 é o CONTROLE da 1-4 pelo outro lado:** ela prova que uma fileira
//! plana tem causa, e que a causa é agora **dizível**.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por fileira.
pub(crate) const COLS: f32 = 48.0;
/// Quantas fileiras a cena empilha.
pub(crate) const BANDS: usize = 5;
/// A distância vertical entre fileiras.
const BAND_GAP: f32 = 1.05;
/// Quanto a máscara levanta a peça — o mesmo para TODAS, senão duas silhuetas
/// deixariam de ser comparáveis.
const VALUE_SCALE: f32 = 0.7;
/// O tamanho da peça.
const DOT: f32 = 0.19;

/// O limiar que as quatro primeiras bandas comparam — o MEIO da rampa, para o
/// degrau cair no meio da fileira e as duas metades serem visíveis.
pub(crate) const THRESHOLD: f32 = 0.5;
/// As duas tolerâncias da banda 3 e da 4. ⚠️ Quatro vezes uma da outra **de
/// propósito**: uma razão pequena seria "mais ou menos igual" ao olho, e o par
/// existe para ser lido sem régua.
pub(crate) const EPS_NARROW: f32 = 0.05;
pub(crate) const EPS_WIDE: f32 = 0.20;

/// O `mode` `Ramp` do `value.instance_field` (`i/(n−1)` em `[0,1]`).
const FIELD_RAMP: f32 = 1.0;
/// Os índices do `op` do `value.math` que esta cena autora.
const OP_LESS: f32 = 8.0;
const OP_GREATER: f32 = 10.0;
const OP_EQUAL: f32 = 12.0;

/// O nome que a banda 5 pede — um typo de `vel`, e nem um nem outro está numa
/// grade. ⚠️ Um typo de um nome REAL é a forma do engano de verdade: um nome
/// inventado seria fácil de ver no painel sem badge nenhum.
pub(crate) const MISSING_NAME: &str = "velocty";

/// O que uma fileira desenha.
#[derive(Clone, Copy)]
enum Kind {
    /// A rampa comparada com o limiar por `op`, com a tolerância `eps`.
    Compare { op: f32, eps: f32 },
    /// Um `value.attribute` a ler um nome que a stream não tem.
    MissingColumn,
}

static LANES: [Kind; BANDS] = [
    Kind::Compare {
        op: OP_GREATER,
        eps: 0.0,
    },
    Kind::Compare {
        op: OP_LESS,
        eps: 0.0,
    },
    Kind::Compare {
        op: OP_EQUAL,
        eps: EPS_NARROW,
    },
    Kind::Compare {
        op: OP_EQUAL,
        eps: EPS_WIDE,
    },
    Kind::MissingColumn,
];

/// O que a cena anuncia — uma linha por fileira, na ordem em que estão na tela.
pub(crate) const BAND_LABELS: [&str; BANDS] = [
    "1 GREATER  rampa > 0.5          -- \\",
    "2 LESS     rampa < 0.5          -- /  degraus COMPLEMENTARES (o op e' lido)",
    "3 EQUAL    eps 0.05             -- \\",
    "4 EQUAL    eps 0.20             -- /  banda ESTREITA contra uma 4x mais larga",
    "5 ATTR     le' 'velocty' (typo) -- PLANA de proposito: o badge (!) e' a feature",
];

/// `grid → scale → <cadeia de valor> → drive(Y) → transform → output`, uma vez por
/// fileira. Devolve os sinks.
/// ⚠️ `pub(crate)` e não `pub(super)` como as irmãs: o gate do BADGE mora no
/// `render_loop` (onde o `inert_reaching_output` é alcançável), e sem esta cena ele
/// não teria como provar que a instrução do smoke — *abra o painel de grafo* — é
/// verdadeira para ESTE documento.
pub(crate) fn build_compare_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, kind) in LANES.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "BANDS e' pequeno")]
        let row = 100.0 + k as f32 * 210.0;
        // A fileira do topo é a PRIMEIRA da tabela — ler o gráfico de cima para
        // baixo tem de dar a mesma ordem que ler a lista no log.
        #[expect(clippy::cast_precision_loss, reason = "BANDS e' pequeno")]
        let y = (BANDS as f32 - 1.0) * 0.5 * BAND_GAP - k as f32 * BAND_GAP;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", 0.22);
        g.set_param(grid, "gap_y", 0.22);

        let dot = g.add_node("motion.scale");
        g.set_param(dot, "amount", DOT);

        let value = build_value(g, *kind, dot)?;

        let drive = g.add_node("motion.drive");
        g.set_param(drive, "channel", 1.0); // Y
        g.set_param(drive, "mode", 0.0); // Add
        g.set_param(drive, "scale", VALUE_SCALE);

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", y);
        let out = g.add_node("motion.output");

        for (i, n) in [grid, dot, drive, place, out].into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "poucos nos por fileira")]
            let x = 80.0 + i as f32 * 190.0;
            g.set_pos(n, Pos { x, y: row });
        }

        wire(g, grid, 0, dot, 0)?;
        wire(g, dot, 0, drive, 0)?;
        wire(g, value, 0, drive, 1)?;
        wire(g, drive, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Monta a cadeia de valor de uma fileira e devolve o nó terminal dela.
fn build_value(g: &mut Graph, kind: Kind, geom: NodeId) -> Option<NodeId> {
    Some(match kind {
        Kind::Compare { op, eps } => {
            let ramp = g.add_node("value.instance_field");
            g.set_param(ramp, "mode", FIELD_RAMP);
            wire(g, geom, 0, ramp, 0)?;

            // O limiar: um oscilador de amplitude ZERO é o campo constante de
            // comprimento 1 que o broadcast 1→N espalha — o idioma que os gates de
            // paridade da aritmética já usam.
            let thr = g.add_node("value.lfo");
            g.set_param(thr, "amplitude", 0.0);
            g.set_param(thr, "offset", THRESHOLD);

            let cmp = g.add_node("value.math");
            g.set_param(cmp, "op", op);
            g.set_param(cmp, "epsilon", eps);
            wire(g, ramp, 0, cmp, 0)?;
            wire(g, thr, 0, cmp, 1)?;
            cmp
        }
        Kind::MissingColumn => {
            let at = g.add_node("value.attribute");
            g.set_text_param(at, "attr", MISSING_NAME);
            wire(g, geom, 0, at, 0)?;
            at
        }
    })
}

/// Uma aresta. Função LIVRE e não closure: uma closure que captura `g` o empresta
/// até ao fim do escopo.
fn wire(g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16) -> Option<()> {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_compare_tests.rs"]
mod tests;
