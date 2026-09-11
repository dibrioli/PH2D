//! **A FAMÍLIA TRANSFORM** — a cena `=69` (doc 89, folha 05: cinco células, cinco nós).
//!
//! Cinco pares. O mesmo grafo dos dois lados; só o controle novo muda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.move` | `Space = World` — o passo é do mundo | **`Local`** — cada um anda para a frente do próprio nariz |
//! | `motion.scale` | um peso para os dois eixos | **`Separate Y Mask`** — squash de um, stretch do vizinho |
//! | `motion.mirror` | o gêmeo COPIA a orientação | **`Flip Orientation`** — o gêmeo é o reflexo |
//! | `motion.kaleidoscope` | o `Index` recomeça em cada fatia | **`Reindex`** — uma lista só, uma rampa só |
//! | `motion.orbit` | a órbita move e não vira | **`Carry Rotation`** — o sprite vira com ela |
//!
//! ⚠️ **A ORDEM dos nós desta cena é load-bearing, e custou uma wave inteira aprender:**
//! o `motion.move` do layout **honra o `falloff`**, então uma banda que escreva um campo
//! antes dele sai deslocada por uma fração do que pediu (medido na cena `=66`: 5,6 → 4,6).
//! Aqui o layout move-se PRIMEIRO, e os campos do par 2 nascem centrados na posição já
//! final da banda — é por isso que [`band`] recebe `at` e o repassa aos campos.
//!
//! ⚠️ **As peças são BARRAS e não quadrados** (`motion.scale` com o link desligado): três
//! dos cinco pares falam de ORIENTAÇÃO, e um quadrado rodado é um quadrado.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as cinco linhas.
const GAP_X: f32 = 5.6;
const GAP_Y: f32 = 4.8;
/// O comprimento e a espessura da barra.
const BAR_X: f32 = 0.52;
const BAR_Y: f32 = 0.15;
/// O passo que o par 1 autora.
const STEP: f32 = 0.9;
/// A volta que o par 5 autora. ⚠️ **NÃO é múltiplo de `360/10`**: num anel de dez, uma
/// volta de 36° levaria cada peça ao lugar da vizinha e as duas bandas sairiam iguais.
const TURN: f32 = 60.0;
/// A inclinação das barras do par 3. ⚠️ **Escolhida para `180 − 2·LEAN` NÃO ser 0 nem 180**:
/// é essa a diferença entre o gêmeo copiado e o refletido, e uma barra é simétrica a 180°.
/// A `35°` a diferença é **110°** — o `\` contra o `/` que o olho lê de longe.
const LEAN: f32 = 35.0;
/// O deslocamento da linha de espelho do par 3, a partir do centroide da fileira (que tem
/// meia-largura `2,5·0,62 = 1,55`). Põe as duas fileiras lado a lado com uma folga de 0,8.
const MIRROR_OFF: f32 = 1.95;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Encadeia um nó novo no fim da corrente e devolve-o.
fn push(g: &mut Graph, head: NodeId, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    let _ = wire(g, head, 0, n, 0);
    n
}

/// Uma BARRA: o link do `motion.scale` desligado, para a orientação ser visível.
fn bar(g: &mut Graph, head: NodeId, ey: f32, x: f32) -> NodeId {
    push(
        g,
        head,
        "motion.scale",
        &[("uniform", 0.0), ("amount", BAR_X), ("amount_y", BAR_Y)],
        ey,
        x,
    )
}

/// Um ANEL de `n` peças voltadas para fora (`align`), já em barras e já no quadrante.
/// ⚠️ **É sempre a coroa FECHADA, e o leque saiu de propósito.** Num anel completo o
/// centroide coincide com o centro, então «radial» é uma leitura que o olho faz sem
/// referência nenhuma. Num LEQUE não: medido no par 3 da 1ª tentativa, depois do espelho o
/// centroide da figura fica a `1,03` do centro do anel, e nem as peças com a orientação
/// CERTA saíam radiais. *Um leque é um layout em que «para fora» deixa de ter um sítio.*
fn ring(g: &mut Graph, n: f32, radius: f32, at: [f32; 2], ey: f32) -> NodeId {
    let r = g.add_node("motion.distribute_radial");
    g.set_pos(r, Pos { x: 0.0, y: ey });
    g.set_param(r, "count", n);
    // ⚠️ **UMA coroa, explícita.** O default do `motion.distribute_radial` são três
    // (medido: raios `0,6 / 1,05 / 1,5` para `radius = 1,5`), e uma roseta de três
    // anéis lê-se mal quando o assunto é *para onde cada peça aponta*.
    g.set_param(r, "rings", 1.0);
    g.set_param(r, "radius", radius);
    g.set_param(r, "align", 1.0);
    let b = bar(g, r, ey, 140.0);
    push(
        g,
        b,
        "motion.move",
        &[("dx", at[0]), ("dy", at[1])],
        ey,
        280.0,
    )
}

/// Pinta a banda com uma cor sólida e fecha.
fn close(g: &mut Graph, head: NodeId, rgb: [f32; 3], ey: f32) -> Option<NodeId> {
    let t = push(
        g,
        head,
        "motion.tint",
        &[("r", rgb[0]), ("g", rgb[1]), ("b", rgb[2])],
        ey,
        900.0,
    );
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 1040.0, y: ey });
    wire(g, t, 0, out, 0)?;
    Some(out)
}

/// Monta UMA banda. `right` diz se é o lado do controle novo.
fn band(g: &mut Graph, row: usize, right: bool, ey: f32) -> Option<NodeId> {
    let at = [
        if right { GAP_X } else { -GAP_X },
        GAP_Y * 2.0 - row as f32 * GAP_Y,
    ];
    let on = f32::from(u8::from(right));
    match row {
        // **PAR 1** — o espaço do passo. Um anel voltado para fora: em World as doze
        // peças deslizam juntas; em Local cada uma anda para a frente e o anel ABRE.
        0 => {
            let head = ring(g, 12.0, 1.5, at, ey);
            let mv = push(
                g,
                head,
                "motion.move",
                &[("dx", STEP), ("space", on)],
                ey,
                420.0,
            );
            close(g, mv, [0.46, 0.72, 1.0], ey)
        }
        // **PAR 2** — as duas máscaras. O campo X cresce para a direita e o Y (invertido,
        // no canal `Falloff Y`) decresce; com o toggle os dois eixos deixam de partilhar
        // o peso. ⚠️ Os campos nascem CENTRADOS em `at`, porque o layout já se moveu.
        1 => {
            let grid = g.add_node("motion.grid");
            g.set_pos(grid, Pos { x: 0.0, y: ey });
            g.set_param(grid, "rows", 1.0);
            g.set_param(grid, "cols", 9.0);
            g.set_param(grid, "gap_x", 0.9);
            let fit = push(g, grid, "motion.scale", &[("amount", 0.3)], ey, 140.0);
            let mv = push(
                g,
                fit,
                "motion.move",
                &[("dx", at[0]), ("dy", at[1])],
                ey,
                280.0,
            );
            let fx = push(
                g,
                mv,
                "motion.falloff",
                &[
                    ("shape", 2.0),
                    ("curve", 0.0),
                    ("radius", 3.7),
                    ("center_x", at[0]),
                    ("center_y", at[1]),
                ],
                ey,
                420.0,
            );
            let fy = push(
                g,
                fx,
                "motion.falloff",
                &[
                    ("shape", 2.0),
                    ("curve", 0.0),
                    ("radius", 3.7),
                    ("center_x", at[0]),
                    ("center_y", at[1]),
                    ("invert", 1.0),
                    ("mask_channel", 1.0),
                ],
                ey,
                560.0,
            );
            let sc = push(
                g,
                fy,
                "motion.scale",
                &[
                    ("uniform", 0.0),
                    ("amount", 2.8),
                    ("amount_y", 2.8),
                    ("use_falloff_y", on),
                ],
                ey,
                700.0,
            );
            close(g, sc, [1.0, 0.74, 0.3], ey)
        }
        // **PAR 3** — o gêmeo do espelho: DUAS fileiras de barras inclinadas, lado a lado.
        //
        // ⚠️ **A 1ª tentativa foi um leque de raios, e o Enio não conseguiu lê-la.** A causa
        // está MEDIDA e é da FIGURA, não do nó: uma barra tem simetria de **180°**, então
        // «apontar para fora» e «apontar para dentro» desenham-se IGUAIS. No leque de ±65°,
        // quatro das catorze peças saíam com desvio `0` ou `180` graus do radial — invisíveis
        // — e as outras num meio-termo de ~135°. Pior: depois do espelho o «radial» passa a
        // ser medido a partir do CENTROIDE da figura, que não é o centro do anel, então nem
        // a metade CERTA era radial. *O gate media a coluna certa e a cena não a mostrava.*
        //
        // Aqui as duas metades diferem por `180 − 2·LEAN` = **110°**, que não é múltiplo de
        // 180 e por isso é visível numa barra: `\\\ ///` contra `/// ///`.
        //
        // ⚠️ O `Axis Offset` é obrigatório: o espelho reflete no CENTROIDE, e uma fileira
        // centrada nele mapeia sobre si mesma — os gêmeos nasceriam por cima dos originais.
        // O pré-deslocamento de `−MIRROR_OFF` devolve a união ao centro da banda.
        2 => {
            let grid = g.add_node("motion.grid");
            g.set_pos(grid, Pos { x: 0.0, y: ey });
            g.set_param(grid, "rows", 1.0);
            g.set_param(grid, "cols", 6.0);
            g.set_param(grid, "gap_x", 0.62);
            let b = bar(g, grid, ey, 140.0);
            let lean = push(g, b, "motion.rotate", &[("angle", LEAN)], ey, 280.0);
            let mv = push(
                g,
                lean,
                "motion.move",
                &[("dx", at[0] - MIRROR_OFF), ("dy", at[1])],
                ey,
                400.0,
            );
            let m = push(
                g,
                mv,
                "motion.mirror",
                &[("axis", 0.0), ("offset", MIRROR_OFF), ("flip_rot", on)],
                ey,
                520.0,
            );
            close(g, m, [0.62, 1.0, 0.66], ey)
        }
        // **PAR 4** — a renumeração. Seis fatias de seis, pintadas por um DEGRADÊ que lê
        // as colunas de identidade: sem o knob as seis fatias saem idênticas.
        3 => {
            let grid = g.add_node("motion.grid");
            g.set_pos(grid, Pos { x: 0.0, y: ey });
            g.set_param(grid, "rows", 1.0);
            g.set_param(grid, "cols", 6.0);
            g.set_param(grid, "gap_x", 0.34);
            let fit = push(g, grid, "motion.scale", &[("amount", 0.26)], ey, 140.0);
            let arm = push(g, fit, "motion.move", &[("dx", 1.15)], ey, 280.0);
            let mv = push(
                g,
                arm,
                "motion.move",
                &[("dx", at[0]), ("dy", at[1])],
                ey,
                420.0,
            );
            let k = push(
                g,
                mv,
                "motion.kaleidoscope",
                &[
                    ("segments", 6.0),
                    ("reflect", 0.0),
                    ("pivot_x", at[0]),
                    ("pivot_y", at[1]),
                    ("reindex", on),
                ],
                ey,
                560.0,
            );
            // Gradiente: escuro → claro ao longo da lista. É o único consumidor da
            // coluna `Index`, e é por isso que ele é o oráculo desta célula.
            let t = push(
                g,
                k,
                "motion.tint",
                &[
                    ("mode", 1.0),
                    ("r", 0.30),
                    ("g", 0.10),
                    ("b", 0.22),
                    ("r2", 1.0),
                    ("g2", 0.55),
                    ("b2", 0.80),
                ],
                ey,
                760.0,
            );
            let out = g.add_node("motion.output");
            g.set_pos(out, Pos { x: 1040.0, y: ey });
            wire(g, t, 0, out, 0)?;
            Some(out)
        }
        // **PAR 5** — a órbita que leva a orientação. Sem o knob os raios deixam de ser
        // raios (todos tortos pelo mesmo ângulo); com ele a estrela roda inteira.
        _ => {
            let head = ring(g, 10.0, 1.5, at, ey);
            let o = push(
                g,
                head,
                "motion.orbit",
                &[
                    ("pivot_x", at[0]),
                    ("pivot_y", at[1]),
                    ("angle", TURN),
                    ("speed", 0.0),
                    ("carry_rotation", on),
                ],
                ey,
                420.0,
            );
            close(g, o, [0.85, 0.78, 1.0], ey)
        }
    }
}

/// Monta a cena. Devolve os dez sinks, em pares.
pub(crate) fn build_transform_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(10);
    for row in 0..5 {
        for right in [false, true] {
            let ey = (row * 2 + usize::from(right)) as f32 * 240.0;
            sinks.push(band(g, row, right, ey)?);
        }
    }
    Some(sinks)
}

/// Os rótulos das dez bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "PASSO em World -- o anel inteiro desliza para o lado, rigido",
        "PASSO em Local -- cada peca anda para a frente do proprio nariz, e o anel ABRE",
        "ESCALA com uma mascara -- os nove crescem por igual da esquerda para a direita",
        "ESCALA com mascara separada em Y -- um fica ALTO e magro, o vizinho BAIXO e largo",
        "ESPELHO -- as DUAS fileiras inclinam para o mesmo lado: /// ///",
        "ESPELHO com Flip Orientation -- a copia e' o reflexo, e as duas fecham: \\\\\\ ///",
        "MANDALA -- as seis fatias saem IDENTICAS: o degrade recomeca em cada uma",
        "MANDALA com Reindex -- uma lista so', e o degrade da a volta inteira uma vez",
        "ORBITA -- a estrela roda e os raios NAO viram: ficam todos tortos pelo mesmo angulo",
        "ORBITA com Carry Rotation -- a estrela roda inteira e os raios seguem radiais",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (STEP, TURN)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_transform_tests.rs"]
mod tests;
