//! **EM TORNO DE QUÊ** (`PH2D_GPU_COOK_DEMO=111`) — a cena de smoke do ciclo 3, W1–W3
//! ([doc 106](../../docs/Motion%20Nodes/106_ciclo_3_transformes_e_deformadores.md)).
//!
//! ## O que se vê
//!
//! Um pano **longe do centro do mundo**, a torcer-se. Ele abre no modo `Point` com o ponto em
//! `(0, 0)` — ou seja, o eixo da torção está **fora do pano**, e o pano é atirado à volta de um
//! ponto onde não há nada. É o defeito, encenado.
//!
//! Trocar o **Pivot** do cartão *Twist* para **Centroid** põe o eixo no meio do próprio pano, e
//! ele passa a torcer-se sobre si — sem que ninguém tenha digitado um número.
//!
//! ## ⚠️ Por que o pano está deslocado, e por que isso é a cena inteira
//!
//! Com o pano na origem os três modos dão a MESMA imagem (o centroide de uma grelha centrada
//! **é** a origem), e a cena não ensinaria nada — a mesma armadilha que fez a primeira redacção
//! de dois gates de paridade desta wave ficar verde sobre um kernel a que faltava o braço do
//! centroide inteiro. *Uma cena que só mostra o caso em que as três respostas coincidem é uma
//! cena que ensina que a escolha não importa.*
//!
//! ## O que ela está de facto a demonstrar
//!
//! 1. **O pivô é uma porta só** ([`ph2d_nodegraph::pivot`]): o mesmo controlo, com os mesmos
//!    três valores e os mesmos rótulos, no `Twist`, no `Bend`, no `Kaleidoscope` e no
//!    `Transform`. Antes desta wave havia **seis** vocabulários no grupo.
//! 2. **O `Centroid` corre no DISPOSITIVO** — é um par de somas do canal
//!    `reduce → broadcast → map`, não uma volta pela CPU. E no `Twist` a redução do raio **lê**
//!    essas somas, que é uma capacidade que o substrato não tinha até esta wave.
//! 3. **O espelho também está no dispositivo** (W2): ele duplica a contagem e a cadeia
//!    continua reivindicada — antes, um espelho no meio da cadeia derrubava-a inteira.
//! 4. **O cisalhamento existe** (W3): os dois knobs `Skew X`/`Skew Y` no cartão `Transform`.
//!
//! ⚠️ **Se o pano se torcer sobre si próprio já no arranque**, o `pivot_mode` não está a ser
//! lido — o defeito que o gate `the_pivot_mode_decides_and_a_stale_typed_point_does_not_leak`
//! mediu a divergir `1,369` unidades de mundo entre a CPU e o dispositivo.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quão longe da origem o pano nasce, em unidades de mundo. ⚠️ **Não é um número escolhido:**
/// é `1,5 ×` a largura do pano (`LADO × gap`), o mínimo para o eixo da torção cair **fora** dele
/// com folga que se vê a olho.
const DESLOCAMENTO: f32 = 450.0;
/// 300 × 300 = 90 000 — grande o suficiente para o pano ler como pano e para o medidor de
/// dispositivo dizer alguma coisa, pequeno o suficiente para a cena abrir depressa.
const LADO: f32 = 300.0;

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    vec![
        Caption::new([0.0, 0.0], "a origem do mundo"),
        Caption::new(
            [DESLOCAMENTO, LADO * 0.6],
            "Twist ▸ Pivot: Point → Centroid",
        ),
    ]
}

/// `grid → move → mirror → twist → transform → output`, com o pano fora da origem.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", LADO);
    g.set_param(grid, "cols", LADO);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);

    // O pano sai da origem — ver o cabeçalho: é isto que faz a escolha do pivô ter imagem.
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", DESLOCAMENTO);
    g.set_param(mv, "dy", 0.0);

    // O espelho (W2): a contagem duplica e a cadeia CONTINUA no dispositivo.
    let mirror = g.add_node("motion.mirror");
    g.set_param(mirror, "axis", 1.0); // horizontal — o par fica um sobre o outro

    // A torção, no modo que a cena existe para ensinar: `Point` em `(0,0)`, que é a origem do
    // MUNDO e está fora do pano.
    let twist = g.add_node("motion.twist");
    g.set_param(twist, "angle", 150.0);
    g.set_param(twist, ph2d_nodegraph::pivot::PARAM, 1.0);
    g.set_param(twist, "pivot_x", 0.0);
    g.set_param(twist, "pivot_y", 0.0);

    // O afim, com o cisalhamento em ZERO — o knob que o artista arrasta no passo 4.
    let xform = g.add_node("motion.transform");
    g.set_param(xform, ph2d_nodegraph::pivot::PARAM, 2.0); // Centroid: o skew dobra no meio

    // Um LFO bipolar: a torção vai e volta, então o eixo errado vê-se nos dois sentidos.
    let wind = g.add_node("value.lfo");
    g.set_param(wind, "period", 8.0);
    g.set_param(wind, "amplitude", 1.0);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, mv, mirror, twist, xform, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                #[expect(clippy::cast_precision_loss, reason = "um indice de cartao")]
                x: 60.0 + i as f32 * 185.0,
                y: 140.0,
            },
        );
    }
    g.set_pos(wind, Pos { x: 430.0, y: 330.0 });

    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, mirror, 0),
        (mirror, twist, 0),
        (wind, twist, 1),
        (twist, xform, 0),
        (xform, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }

    g.validate(reg).ok()?;
    Some(vec![out])
}
