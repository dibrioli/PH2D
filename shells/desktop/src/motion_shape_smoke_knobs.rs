//! **A CENA DOS KNOBS DE FORMA** (`PH2D_SHAPE_SMOKE=3`, doc 89 folha 14) — irmã de
//! `motion_shape_smoke.rs` pelo teto de 600 LOC do shell, e por RESPONSABILIDADE: o pai
//! prova que a forma CHEGA ao grafo e sobrevive a um deformer; aqui pergunta-se se ela se
//! **AJUSTA**.
//!
//! A folha dizia, sobre a pizza e a corda: *"a FORMA chegou, o CONTROLO não"* — as espécies
//! estavam no catálogo, mas corriam nas proporções canónicas da biblioteca e nenhum slider
//! as movia. E, sobre a caixa: `rounded_rect_corners(a, b, radii, smoothing)` existe há
//! muito, com raio POR CANTO e o *corner smoothing* do Figma, e o nó passava sempre
//! `[r, 0, 0, 0, 0]`.
//!
//! A cena põe **sete** formas lado a lado, cada uma exercendo um knob que até agora não
//! existia (a primeira é o CONTROLE: o círculo inteiro, com nada tocado). ⚠️ **Nenhuma delas é uma espécie NOVA** — é a mesma família do círculo e a mesma
//! caixa que já estavam ali, com números que o artista agora alcança.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Os índices de `ShapeKind` que a cena usa (ver `ph2d_node_motion_shape::ALL_KINDS`).
const CIRCLE: f32 = 0.0;
const SQUARE: f32 = 1.0;
const RECTANGLE: f32 = 3.0;
const PIE: f32 = 8.0;

/// O vão horizontal entre as formas da fileira.
const GAP: f32 = 2.4;
/// O raio de cada forma.
const R: f32 = 0.9;

/// Uma forma da fileira: o nó, os params que a definem, e o `motion.move` que a coloca.
/// Devolve a saída já posicionada.
fn placed(g: &mut Graph, slot: i32, kind: f32, knobs: &[(&str, f32)]) -> NodeId {
    let src = g.add_node("source.shape");
    let x = slot as f32 * 240.0;
    g.set_pos(src, Pos { x, y: -300.0 });
    g.set_param(src, "kind", kind);
    g.set_param(src, "size", R);
    for &(name, v) in knobs {
        g.set_param(src, name, v);
    }
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y: -180.0 });
    g.set_param(mv, "dx", (slot - 2) as f32 * GAP);
    g.connect(Edge {
        from: (src, 0),
        to: (mv, 0),
        delayed: false,
    })
    .expect("shape -> move");
    mv
}

/// Junta até quatro saídas num `motion.combine` posicionado em `y`.
fn merge(g: &mut Graph, ins: &[NodeId], y: f32) -> NodeId {
    let c = g.add_node("motion.combine");
    g.set_pos(c, Pos { x: 620.0, y });
    for (k, &n) in ins.iter().enumerate() {
        g.connect(Edge {
            from: (n, 0),
            to: (c, k as u16),
            delayed: false,
        })
        .expect("uma forma na juncao");
    }
    c
}

/// Monta a fileira e devolve o sink.
pub(crate) fn build_knob_row(g: &mut Graph) -> NodeId {
    // ⚠️ **Cada forma exerce UM knob novo, e as duas primeiras usam a MESMA espécie** — é
    // isso que mostra que a família do círculo é uma forma só: o que separa a rosquinha da
    // pizza são números, não uma entrada nova no catálogo.
    let shapes = [
        // 1 — o círculo INTEIRO, o controle: nenhum knob novo tocado.
        placed(g, 0, CIRCLE, &[]),
        // 2 — a ROSQUINHA: o mesmo círculo com miolo.
        placed(g, 1, CIRCLE, &[("inner", 0.55)]),
        // 3 — o ANEL PARCIAL: miolo + abertura + começo girado.
        placed(
            g,
            2,
            CIRCLE,
            &[("inner", 0.5), ("sweep", 220.0), ("start", 30.0)],
        ),
        // 4 — a PIZZA com fatia autorada (ela nascia num ângulo fixo).
        placed(g, 3, PIE, &[("sweep", 70.0)]),
        // 5 — o SQUIRCLE: canto redondo + a suavização do Figma.
        placed(g, 4, SQUARE, &[("corner", 0.55), ("smoothing", 0.85)]),
    ];
    let a = merge(g, &shapes[..4], -300.0);
    // A caixa de cantos DESIGUAIS entra na segunda junção, com a quinta forma.
    let uneven = placed(
        g,
        5,
        RECTANGLE,
        &[
            ("aspect", 0.6),
            ("corner", 0.25),
            ("corner_tr", 0.45),
            ("corner_bl", -0.2),
        ],
    );
    // 7 — o MESMO anel parcial do slot 3, agora com as QUATRO quinas arredondadas. É o par
    // que responde ao feedback do Enio (2026-08-19): a rosca cortada tinha quatro quinas
    // vivas e nenhum knob — e, medido, ela não tinha quina NENHUMA para o motor ver, porque
    // o handle do arco sobrava na ponta e curvava a borda radial.
    let round_ring = placed(
        g,
        6,
        CIRCLE,
        &[
            ("inner", 0.5),
            ("sweep", 220.0),
            ("start", 30.0),
            ("corner", 0.18),
        ],
    );
    let b = merge(g, &[a, shapes[4], uneven, round_ring], -60.0);
    let out = g.add_node("motion.output");
    g.set_pos(
        out,
        Pos {
            x: 860.0,
            y: -180.0,
        },
    );
    g.connect(Edge {
        from: (b, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("juncao -> output");
    out
}

impl crate::App {
    /// O corpo da cena `=3`, delegado de [`crate::App::motion_shape_smoke`] pelo braço `_`
    /// (o pai já avançou o `FRAME`). Só a combinação `(3, 3)` age.
    pub(super) fn motion_shape_smoke_knobs(&mut self, mode: u32, f: u32) {
        if (mode, f) != (3, 3) {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let out = build_knob_row(&mut gfx.motion.doc.graph);
        gfx.motion.sinks.push(out);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        eprintln!(
            "[shape smoke =3] SETE formas lado a lado, e nenhuma especie nova: (1) circulo \
             INTEIRO, o controle · (2) o MESMO circulo com Inner 0,55 = uma ROSQUINHA · (3) o \
             mesmo com Inner + Sweep 220 + Start 30 = um ANEL PARCIAL girado · (4) a PIZZA com \
             Sweep 70 (ela nascia num angulo fixo que nenhum slider movia) · (5) um SQUIRCLE \
             (Corner 0,55 + Smoothing 0,85 — o corner smoothing do Figma) · (6) uma caixa de \
             cantos DESIGUAIS (Corner 0,25, canto de cima-direita +0,45, o de baixo-esquerda \
             AFIADO por -0,2) · (7) O MESMO anel parcial do (3), agora com Corner 0,18: as \
             QUATRO quinas dele ARREDONDADAS. SE AS SETE PARECEREM A MESMA COISA, PARE. \
             ⚠️ COMPARE O (3) COM O (7): e' o par que responde ao seu smoke — a rosca cortada \
             nao tinha knob de quina, e a medicao achou o porque: o handle do arco sobrava na \
             ponta, a borda radial ABAULAVA (0,1865 de desvio num raio 1) e o motor de quinas \
             lia a ponta como curva CONTINUA, entao nao havia quina para arredondar. Agora a \
             borda e' reta ao bit e as quatro quinas existem. Clique qualquer forma: o painel \
             mostra SO os knobs daquela especie, e o Corner aparece em 36 das 43 (as 7 de fora \
             — circulo, elipse, coracao, pilula, cilindro, juncao, lua — nao tem quina nenhuma)."
        );
    }
}

#[cfg(test)]
#[path = "motion_shape_smoke_knobs_tests.rs"]
mod tests;
