//! **O QUE UM CAMPO DE VALOR NÃO SABIA DIZER** — a cena `=96` (doc 89, folha 15).
//!
//! Quatro pares, um por célula. ⚠️ **Esta cena é ESTÁTICA** — não precisa de Play.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `value.wrap` | uma faixa para todos | **a faixa como CAMPO** — cada peça dobra na sua |
//! | `value.smooth` | `Centered` | **`Left Half`** — o filtro deixa de ler o futuro |
//! | `value.median` | `tolerance = 0` | **a tolerância** — o pico cai, a ondulação fica |
//! | `value.attribute` | `Size` (a magnitude) | **`Size X`** — o EIXO |
//!
//! ## ⚠️ A LEI QUE ESTA CENA HERDA: **posicionar é UPSTREAM da máscara**
//!
//! Está paga pela cena `=73` (Enio, 2026-08-21: *"tudo misturado e bagunçado"*), e é a
//! razão de [`place`] correr **imediatamente a seguir à fonte**: todo comportamento desta
//! biblioteca é mascarado pelo `falloff`, então um deslocamento de colocação posto DEPOIS
//! do campo vira `dx · falloff_i` e a banda estica-se por cima das vizinhas. ⛔ Não mova
//! aquela chamada para o fim da cadeia.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O centro de cada coluna — a da esquerda é sempre *como era*.
const COL_X: f32 = 3.1;
/// O centro de cada linha, de cima para baixo.
const ROW_Y: [f32; 4] = [4.3, 1.5, -1.3, -4.1];
/// A fileira de peças que toda banda partilha.
const PIECES: f32 = 24.0;
const GAP: f32 = 0.23;
/// O tamanho de repouso — o `drive(Size, Set)` de cada linha escreve por cima.
const PIECE: f32 = 0.16;

/// Os canais do `motion.drive` que esta cena usa (a escada é a do `ParamWidget::Enum`
/// daquele nó, e ⚠️ **`Size X`/`Size Y` são `10`/`11` porque foram APENDADOS** — quem os
/// contar de cabeça pela ordem visual do painel erra por seis).
const DRIVE_SIZE: f32 = 3.0;
const DRIVE_ROTATION: f32 = 2.0;
const DRIVE_SIZE_X: f32 = 10.0;
const DRIVE_SIZE_Y: f32 = 11.0;
/// O modo `Set` do `motion.drive`.
const SET: f32 = 1.0;

/// Os modos do `value.instance_field`.
const FIELD_RAMP: f32 = 1.0;
const FIELD_RANDOM: f32 = 2.0;

/// A cor de cada linha.
const LIT: [[f32; 3]; 4] = [
    [0.52, 0.76, 1.0],
    [1.0, 0.78, 0.4],
    [0.66, 1.0, 0.72],
    [0.95, 0.6, 0.78],
];

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn node(g: &mut Graph, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    n
}

fn push(g: &mut Graph, head: NodeId, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = node(g, kind, ps, ey, x);
    let _ = wire(g, head, 0, n, 0);
    n
}

/// **PÔR A BANDA NO QUADRANTE DELA — e isto corre ANTES de qualquer campo.**
///
/// Ver o aviso no topo do módulo: chamado aqui, logo a seguir à fonte, não existe coluna
/// `falloff` — a máscara ausente lê `1.0` e a colocação é rígida.
fn place(g: &mut Graph, head: NodeId, at: [f32; 2], ey: f32) -> NodeId {
    push(
        g,
        head,
        "motion.transform",
        &[("offset_x", at[0]), ("offset_y", at[1])],
        ey,
        200.0,
    )
}

/// A semente de uma banda: a fileira de peças, **já posicionada**.
fn seed(g: &mut Graph, k: usize, right: bool) -> (NodeId, f32) {
    let ey = (k * 2 + usize::from(right)) as f32 * 240.0;
    let n = node(
        g,
        "motion.grid",
        &[
            ("rows", 1.0),
            ("cols", PIECES),
            ("gap_x", GAP),
            ("gap_y", GAP),
        ],
        ey,
        80.0,
    );
    let at = [if right { COL_X } else { -COL_X }, ROW_Y[k]];
    let placed = place(g, n, at, ey);
    let sized = push(g, placed, "motion.scale", &[("amount", PIECE)], ey, 260.0);
    (sized, ey)
}

/// Uma rampa `0..1` sobre a fileira, escalada para `[lo, hi]`.
///
/// ⚠️ **O `value.instance_field` LÊ O FLUXO** — ele deriva um número por instância, logo
/// precisa de saber quantas há. Sem o fio de entrada a cena não valida, e foi assim que a
/// 1.ª versão desta cena não montou.
fn ramp(g: &mut Graph, from: NodeId, ey: f32, x: f32, lo: f32, hi: f32) -> NodeId {
    let f = node(g, "value.instance_field", &[("mode", FIELD_RAMP)], ey, x);
    let _ = wire(g, from, 0, f, 0);
    push(
        g,
        f,
        "value.map_range",
        &[
            ("in_lo", 0.0),
            ("in_hi", 1.0),
            ("out_lo", lo),
            ("out_hi", hi),
        ],
        ey,
        x + 90.0,
    )
}

/// Liga um campo de valor a um canal do `motion.drive` — o campo entra na porta **1**.
fn drive(g: &mut Graph, head: NodeId, field: NodeId, ch: f32, ey: f32, x: f32) -> Option<NodeId> {
    let d = node(
        g,
        "motion.drive",
        &[("channel", ch), ("mode", SET), ("scale", 1.0)],
        ey,
        x,
    );
    wire(g, head, 0, d, 0)?;
    wire(g, field, 0, d, 1)?;
    Some(d)
}

/// Pinta e fecha a banda.
fn finish(g: &mut Graph, head: NodeId, k: usize, ey: f32) -> Option<NodeId> {
    let t = node(
        g,
        "motion.tint",
        &[("r", LIT[k][0]), ("g", LIT[k][1]), ("b", LIT[k][2])],
        ey,
        900.0,
    );
    wire(g, head, 0, t, 0)?;
    let out = node(g, "motion.output", &[], ey, 1000.0);
    wire(g, t, 0, out, 0)?;
    Some(out)
}

/// **PAR 1 — a FAIXA como campo.** A rampa `0..4` dobra em `[0,1]`: à esquerda a mesma
/// faixa para toda a fileira (quatro dentes iguais); à direita a faixa CRESCE ao longo da
/// banda, e o dente estica com ela.
fn row_wrap(g: &mut Graph, right: bool) -> Option<NodeId> {
    let (head, ey) = seed(g, 0, right);
    let src = ramp(g, head, ey, 340.0, 0.0, 4.0);
    let w = node(
        g,
        "value.wrap",
        &[("lo", 0.0), ("hi", 1.0), ("mode", 1.0)],
        ey,
        540.0,
    );
    wire(g, src, 0, w, 0)?;
    if right {
        // A porta `hi` (índice 2) — a faixa deixa de ser um número e passa a ser um campo.
        let hi = ramp(g, head, ey + 90.0, 340.0, 0.3, 1.6);
        wire(g, hi, 0, w, 2)?;
    }
    let sized = push(
        g,
        w,
        "value.map_range",
        &[("out_lo", 0.06), ("out_hi", 0.3)],
        ey,
        660.0,
    );
    let d = drive(g, head, sized, DRIVE_SIZE, ey, 780.0)?;
    finish(g, d, 0, ey)
}

/// **PAR 2 — a janela CAUSAL.** Um degrau a meio da fileira, suavizado: à esquerda o
/// filtro centrado começa a crescer **antes** do degrau (ele lê o futuro); à direita a
/// meia-janela só olha para trás, e nada se mexe antes do acontecimento.
fn row_smooth(g: &mut Graph, right: bool) -> Option<NodeId> {
    let (head, ey) = seed(g, 1, right);
    let src = ramp(g, head, ey, 340.0, 0.0, 1.0);
    let step = push(
        g,
        src,
        "value.step",
        &[("threshold", 0.5), ("width", 0.0)],
        ey,
        540.0,
    );
    let sm = push(
        g,
        step,
        "value.smooth",
        &[
            ("radius", 4.0),
            ("weight", 0.0),
            ("window", if right { 1.0 } else { 0.0 }),
        ],
        ey,
        640.0,
    );
    let sized = push(
        g,
        sm,
        "value.map_range",
        &[("out_lo", 0.06), ("out_hi", 0.3)],
        ey,
        740.0,
    );
    let d = drive(g, head, sized, DRIVE_SIZE, ey, 840.0)?;
    finish(g, d, 1, ey)
}

/// **PAR 3 — a TOLERÂNCIA do de-spike.** Uma ondulação fina com picos: à esquerda a
/// mediana reescreve TUDO e a fileira sai lisa; à direita só o que passa da barra cai, e a
/// textura sobrevive.
fn row_median(g: &mut Graph, right: bool) -> Option<NodeId> {
    let (head, ey) = seed(g, 2, right);
    let noise = node(
        g,
        "value.instance_field",
        &[("mode", FIELD_RANDOM), ("seed", 7.0)],
        ey,
        340.0,
    );
    wire(g, head, 0, noise, 0)?;
    let ripple = push(
        g,
        noise,
        "value.map_range",
        // ⚠️ A ondulação tem de ser LARGA o bastante para a tolerância a poder distinguir do
        // pico: a 1.ª fixtura dava-lhe `0,14` de amplitude e os dois lados saíam a `0,017` um
        // do outro — a régua media produto correcto sobre um sinal que não continha o caso.
        &[("out_lo", 0.30), ("out_hi", 0.70)],
        ey,
        440.0,
    );
    let spike = push(
        g,
        noise,
        "value.step",
        &[("threshold", 0.86), ("width", 0.0)],
        ey + 90.0,
        440.0,
    );
    let spike = push(
        g,
        spike,
        "value.map_range",
        &[("out_lo", 0.0), ("out_hi", 0.9)],
        ey + 90.0,
        540.0,
    );
    let sum = node(g, "value.math", &[("op", 0.0)], ey, 640.0);
    wire(g, ripple, 0, sum, 0)?;
    wire(g, spike, 0, sum, 1)?;
    let med = push(
        g,
        sum,
        "value.median",
        // A tolerância fica ACIMA da ondulação (`0,4` de amplitude) e ABAIXO do pico (`0,9`).
        &[
            ("radius", 2.0),
            ("tolerance", if right { 0.5 } else { 0.0 }),
        ],
        ey,
        740.0,
    );
    let sized = push(
        g,
        med,
        "value.map_range",
        &[
            ("in_lo", 0.3),
            ("in_hi", 1.4),
            ("out_lo", 0.05),
            ("out_hi", 0.32),
        ],
        ey,
        840.0,
    );
    let d = drive(g, head, sized, DRIVE_SIZE, ey, 900.0)?;
    finish(g, d, 2, ey)
}

/// **PAR 4 — o EIXO contra a MAGNITUDE.** As peças recebem uma largura que cresce e uma
/// altura que encolhe, então a MAGNITUDE do tamanho é quase constante ao longo da banda.
/// À esquerda lê-se `Size` e a rotação mal se mexe; à direita lê-se `Size X` e ela varre.
fn row_lanes(g: &mut Graph, right: bool) -> Option<NodeId> {
    let (head, ey) = seed(g, 3, right);
    let wide = ramp(g, head, ey, 340.0, 0.5, 2.2);
    let tall = ramp(g, head, ey + 90.0, 340.0, 2.2, 0.5);
    let a = drive(g, head, wide, DRIVE_SIZE_X, ey, 540.0)?;
    let b = drive(g, a, tall, DRIVE_SIZE_Y, ey, 620.0)?;
    // A leitura de volta: a magnitude (esquerda) contra o eixo (direita).
    let read = node(
        g,
        "value.attribute",
        &[(
            "mode",
            if right {
                f64::from(ph2d_node_value_attribute::MODE_COMPONENT_BASE) as f32
            } else {
                f64::from(ph2d_node_value_attribute::MODE_LENGTH) as f32
            },
        )],
        ey,
        700.0,
    );
    g.set_text_param(read, ph2d_node_value_attribute::ATTR_KEY, "size");
    wire(g, b, 0, read, 0)?;
    let deg = push(
        g,
        read,
        "value.map_range",
        &[
            ("in_lo", 0.5),
            ("in_hi", 2.4),
            ("out_lo", 0.0),
            ("out_hi", 80.0),
        ],
        ey,
        800.0,
    );
    let d = drive(g, b, deg, DRIVE_ROTATION, ey, 880.0)?;
    finish(g, d, 3, ey)
}

/// Monta a cena. Devolve os oito sinks, em pares.
pub(crate) fn build_value_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(8);
    for right in [false, true] {
        sinks.push(row_wrap(g, right)?);
    }
    for right in [false, true] {
        sinks.push(row_smooth(g, right)?);
    }
    for right in [false, true] {
        sinks.push(row_median(g, right)?);
    }
    for right in [false, true] {
        sinks.push(row_lanes(g, right)?);
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das oito bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "FAIXA unica -- quatro dentes iguais",
        "FAIXA como CAMPO -- o dente estica ao longo da banda",
        "Centered -- o filtro cresce ANTES do degrau",
        "Left Half -- nada se mexe antes do acontecimento",
        "tolerancia 0 -- a mediana reescreve tudo, e a fileira sai lisa",
        "TOLERANCIA -- o pico cai e a ondulacao fica",
        "Size (a magnitude) -- ela quase nao muda, e a rotacao nao varre",
        "Size X (o EIXO) -- a rotacao varre a banda",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let (row, col) = (k / 2, k % 2);
            let at = [if col == 0 { -COL_X } else { COL_X }, ROW_Y[row] + 0.75];
            crate::motion_demo_legend::Caption::new(at, short_of(label))
        })
        .collect()
}

/// A ficha curta: o que está ANTES do primeiro `--`.
fn short_of(label: &'static str) -> &'static str {
    match label.find(" --") {
        Some(i) => &label[..i],
        None => label,
    }
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_value_tests.rs"]
mod tests;
