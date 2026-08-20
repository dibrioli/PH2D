//! **A FAMÍLIA `fx.*`** — a cena `=70` (doc 89, folha 11: cinco células, três nós).
//!
//! ⚠️ **Esta cena NÃO é de pares, e a razão é arquitetural.** A folha 11 §0 tem uma
//! navalha: a família tem três nós e **duas arquiteturas**. O `fx.drop_shadow` e o
//! `fx.rgb_split` são FX de **STREAM** (duplicam linhas no cook, então dois deles
//! compõem e podem ficar lado a lado); o `fx.glow` é FX de **PASSE** — ele configura
//! um passe de render sobre a imagem inteira do Motion, e o `from_graph` lê **o
//! PRIMEIRO** nó do grafo. *Não existe um par de glows*: um segundo é inerte, e é
//! por isso que ele avisa (célula já fechada em 2026-08-19).
//!
//! Então: as sombras vêm em par, e o glow vem **em UM estado**, com os knobs
//! autorados para o artista arrastar ao vivo.
//!
//! | banda | o que mostra |
//! |---|---|
//! | 1 e 2 | `fx.drop_shadow` — a sombra DURA de sempre contra a **`Softness`** nova |
//! | 3 | a fonte emissiva do glow (`tint` > 1), com o **`Anamorphic`** ligado |
//! | 4 | UMA peça-vagalume (`tint` 40×) — o alvo do **`Clamp`** |
//!
//! ⚠️ **A banda 4 existe para o knob ter o que curar.** Um `Clamp` numa cena sem
//! estouro é um controle que não faz nada, e a lei da casa proíbe pintar isso.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// A maciez que a banda 2 autora, em unidades de mundo.
const SOFTNESS: f32 = 0.5;
/// A anamorfose que a banda 3 autora.
const STRETCH: f32 = 4.0;
/// O `tint` do vagalume — o valor que lava a tela sem um teto.
const FIREFLY: f32 = 40.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn push(g: &mut Graph, head: NodeId, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    let _ = wire(g, head, 0, n, 0);
    n
}

/// Uma grelha de `cols × rows` no quadrante `at`, já com a peça no tamanho pedido.
fn grid(g: &mut Graph, cols: f32, rows: f32, piece: f32, at: [f32; 2], ey: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_pos(n, Pos { x: 0.0, y: ey });
    g.set_param(n, "cols", cols);
    g.set_param(n, "rows", rows);
    g.set_param(n, "gap_x", 1.1);
    g.set_param(n, "gap_y", 1.1);
    let fit = push(g, n, "motion.scale", &[("amount", piece)], ey, 140.0);
    push(
        g,
        fit,
        "motion.move",
        &[("dx", at[0]), ("dy", at[1])],
        ey,
        280.0,
    )
}

fn out_of(g: &mut Graph, head: NodeId, ey: f32) -> Option<NodeId> {
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 1040.0, y: ey });
    wire(g, head, 0, out, 0)?;
    Some(out)
}

/// Monta a cena. Devolve os quatro sinks.
pub(crate) fn build_fx_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);

    // ── Bandas 1 e 2: a sombra dura contra a macia. O MESMO grafo dos dois lados.
    for (col, softness) in [(0usize, 0.0), (1, SOFTNESS)] {
        let ey = col as f32 * 240.0;
        let at = [if col == 0 { -5.0 } else { 5.0 }, 3.4];
        let src = grid(g, 3.0, 2.0, 0.62, at, ey);
        // Um cinza claro, para a sombra preta ler contra a peça.
        let tint = push(
            g,
            src,
            "motion.tint",
            &[("r", 0.88), ("g", 0.9), ("b", 0.94)],
            ey,
            420.0,
        );
        let sh = push(
            g,
            tint,
            "fx.drop_shadow",
            &[
                ("direction", 300.0),
                ("distance", 0.55),
                ("a", 0.55),
                ("softness", softness),
            ],
            ey,
            560.0,
        );
        sinks.push(out_of(g, sh, ey)?);
    }

    // ── Banda 3: a fonte do glow. `tint` acima de 1 é o que faz um pixel florescer
    // (o passe é HDR — doc 67), e sem isso o `Anamorphic` não teria o que esticar.
    let ey = 480.0;
    let src = grid(g, 4.0, 1.0, 0.42, [-4.6, -1.6], ey);
    let hot = push(
        g,
        src,
        "motion.tint",
        &[("r", 3.2), ("g", 2.4), ("b", 6.0)],
        ey,
        420.0,
    );
    // ⚠️ **UM só `fx.glow` em toda a cena** — ver o cabeçalho. Ele configura o passe
    // da imagem inteira, então também é ele que trata o vagalume da banda 4.
    let glow = push(
        g,
        hot,
        "fx.glow",
        &[
            ("threshold", 1.0),
            ("intensity", 1.6),
            ("radius", 2.0),
            ("stretch", STRETCH),
            ("angle", 0.0),
            ("clamp", 0.0),
        ],
        ey,
        560.0,
    );
    sinks.push(out_of(g, glow, ey)?);

    // ── Banda 4: o vagalume. UMA peça com `tint` absurdo — o caso que o `Clamp`
    // existe para curar, e que sem ele lava a cena inteira.
    let ey = 720.0;
    let one = grid(g, 1.0, 1.0, 0.34, [4.6, -1.6], ey);
    let ff = push(
        g,
        one,
        "motion.tint",
        &[("r", FIREFLY), ("g", FIREFLY * 0.8), ("b", FIREFLY * 0.6)],
        ey,
        420.0,
    );
    sinks.push(out_of(g, ff, ey)?);

    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "SOMBRA dura -- a borda do fantasma e' tao nitida quanto a da peca",
        "SOMBRA com Softness -- a mesma sombra com penumbra, e a MESMA densidade no miolo",
        "GLOW anamorfico -- o halo estica num eixo e aperta no outro",
        "VAGALUME -- uma peca 40x mais brilhante que o branco: o alvo do Clamp",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32, f32) {
    (SOFTNESS, STRETCH, FIREFLY)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_fx_tests.rs"]
mod tests;
