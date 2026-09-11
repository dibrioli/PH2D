//! **A ARITMÉTICA DO VALOR** (`PH2D_GPU_COOK_DEMO=41`) — a cena do **grupo A** da
//! conferência (doc 89, folha 15): os cinco nós irmãos que respondem *"que número
//! sai deste número?"*, com os modos que faltavam a cada um.
//!
//! Irmão de `motion_state_conferencia_demos` (o pai bate no teto de LOC da shell).
//!
//! ## A cena é um GRÁFICO, e é por isso que ela se julga sozinha
//!
//! Cada fileira é uma linha de `{cols}` peças cuja **posição Y é o valor** — então
//! a fileira desenha o PERFIL da função. Um dente de serra é um dente de serra, uma
//! escada é uma escada, e um S é um S: não é preciso saber o que o nó faz para ver
//! se ele o fez.
//!
//! ⚠️ **Nenhuma fileira está sozinha.** Cada modo NOVO tem, imediatamente acima ou
//! abaixo, o modo VIZINHO do mesmo nó sobre a MESMA entrada — porque a pergunta
//! que um smoke responde aqui não é *"apareceu alguma coisa?"* e sim *"apareceu
//! coisa DIFERENTE?"*. Um kernel que ignorasse o parâmetro de modo desenharia dois
//! perfis idênticos, e é exactamente essa a falha que um `if/else if` de WGSL
//! produz quando o ramo novo não é alcançado.
//!
//! ## As três leituras que valem
//!
//! - **Módulo** (fileiras 1-2): a entrada é ASSINADA, e os dois dentes de serra
//!   diferem só na metade ESQUERDA — o truncado mergulha abaixo do eixo, o
//!   aterrado nunca. Numa rampa `[0,1]` os dois seriam idênticos, e a cena não
//!   provaria nada.
//! - **Quantize** (fileiras 3-4): duas escadas iguais **excepto no meio**. O
//!   `Truncate` tem um degrau de largura DUPLA em cima da origem — os dois lados
//!   colapsam para zero —, e essa é a assinatura visual de "para zero".
//! - **A rampa** (fileiras 5-7): reta · escada · S. A do meio e a de baixo saem da
//!   MESMA rampa do topo, que é o controle.
//!
//! ## O que a cena NÃO prova
//!
//! Ela não distingue a quíntica da cúbica pelo olho — as duas são um S, e a
//! diferença máxima entre elas é 5% do alcance (o gate de unidade a mede em
//! `0,053`). Quem separa isso é `smoother_agrees_at_the_ends_and_differs_in_between`
//! e o sweep de paridade `gpu_cpu_parity_arith`. A cena responde à pergunta que só
//! o olho responde: *cada modo desenha a forma que o nome dele promete?*

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças por fileira — a resolução do gráfico.
pub(crate) const COLS: f32 = 48.0;
/// Quantas fileiras a cena empilha.
pub(crate) const ROWS: usize = 10;
/// A distância vertical entre fileiras, em unidades de mundo. A amplitude de cada
/// perfil é escolhida (coluna `scale` da tabela) para caber DENTRO dela: dois
/// perfis que se cruzam deixariam de ser dois gráficos.
const ROW_GAP: f32 = 1.15;

/// Que nó a fileira exercita, e em que modo.
#[derive(Clone, Copy)]
enum Kind {
    /// `value.math` com um divisor constante — `op` 6 (truncado) ou 7 (aterrado).
    Modulo { op: f32 },
    /// `value.quantize` — `mode` 1 (Floor) ou 3 (Truncate).
    Quantize { mode: f32 },
    /// `value.map_range` — `interpolation` 0/1/3.
    Map { interp: f32 },
    /// `value.step` — `mode` 1 (Smooth) ou 2 (Smoother).
    Step { mode: f32 },
    /// `value.mix` com `b` constante — `blend` 8 (Overlay).
    Mix { blend: f32 },
}

struct Row {
    label: &'static str,
    kind: Kind,
    /// O alcance para o qual a rampa `[0,1]` é esticada ANTES do nó. `None` ⇒ a
    /// rampa entra crua (os modos que só falam de `[0,1]`).
    signed: Option<(f32, f32)>,
    /// Quanto o valor levanta a fileira. Por-fileira e não global: os alcances de
    /// saída diferem (um módulo de divisor `0,75` percorre 1,5; uma máscara
    /// percorre 1), e um número único deixaria metade dos perfis achatados.
    scale: f32,
}

static ROWS_TABLE: &[Row] = &[
    Row {
        label: "math Modulo (assinado) -- dente de serra que CRUZA o eixo",
        kind: Kind::Modulo { op: 6.0 },
        signed: Some((-2.0, 2.0)),
        scale: 0.55,
    },
    Row {
        label: "math Floored Modulo (assinado) -- o mesmo dente, sempre ACIMA",
        kind: Kind::Modulo { op: 7.0 },
        signed: Some((-2.0, 2.0)),
        scale: 0.55,
    },
    Row {
        label: "quantize Floor (assinado) -- escada de degraus iguais",
        kind: Kind::Quantize { mode: 1.0 },
        signed: Some((-1.1, 1.1)),
        scale: 0.38,
    },
    Row {
        label: "quantize Truncate (assinado) -- degrau DUPLO sobre a origem",
        kind: Kind::Quantize { mode: 3.0 },
        signed: Some((-1.1, 1.1)),
        scale: 0.38,
    },
    Row {
        label: "map_range Linear -- a rampa reta (o CONTROLE das duas abaixo)",
        kind: Kind::Map { interp: 0.0 },
        signed: None,
        scale: 0.8,
    },
    Row {
        label: "map_range Stepped -- a MESMA rampa em 6 niveis",
        kind: Kind::Map { interp: 1.0 },
        signed: None,
        scale: 0.8,
    },
    Row {
        label: "map_range Smoother -- a MESMA rampa como um S",
        kind: Kind::Map { interp: 3.0 },
        signed: None,
        scale: 0.8,
    },
    Row {
        label: "step Smooth -- a mascara com banda cubica",
        kind: Kind::Step { mode: 1.0 },
        signed: None,
        scale: 0.8,
    },
    Row {
        label: "step Smoother -- a mesma banda, quintica de Perlin",
        kind: Kind::Step { mode: 2.0 },
        signed: None,
        scale: 0.8,
    },
    Row {
        label: "mix Overlay -- a rampa contra uma constante, tonal",
        kind: Kind::Mix { blend: 8.0 },
        signed: None,
        scale: 0.8,
    },
];

/// `grid → scale → [rampa → (estica) → <nó> → drive(Y)] → transform → output`,
/// dez vezes. Devolve os DEZ sinks.
pub(crate) fn build_arith_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::Pos;
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let lane = 100.0 + k as f32 * 210.0;
        // A fileira do topo é a PRIMEIRA da tabela — ler o gráfico de cima para
        // baixo tem de dar a mesma ordem que ler a tabela no código.
        let y = (ROWS as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", 0.22);
        g.set_param(grid, "gap_y", 0.22);

        // Peças pequenas: o que se lê é a CURVA que elas traçam, não cada uma.
        let dot = g.add_node("motion.scale");
        g.set_param(dot, "amount", 0.30);

        let ramp = g.add_node("value.instance_field");
        g.set_param(ramp, "mode", 1.0); // Ramp: i/(N-1) em [0,1]

        let value = build_value(g, row, grid, ramp)?;

        let drive = g.add_node("motion.drive");
        g.set_param(drive, "channel", 1.0); // Y
        g.set_param(drive, "mode", 0.0); // Add
        g.set_param(drive, "scale", row.scale);

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", y);
        let out = g.add_node("motion.output");

        for (i, n) in [grid, dot, drive, place, out].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 190.0,
                    y: lane,
                },
            );
        }

        wire(g, grid, 0, dot, 0)?;
        wire(g, dot, 0, ramp, 0)?;
        wire(g, dot, 0, drive, 0)?;
        wire(g, value, 0, drive, 1)?;
        wire(g, drive, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Monta a cadeia de VALOR de uma fileira e devolve o nó terminal dela.
fn build_value(
    g: &mut ph2d_nodegraph::graph::Graph,
    row: &Row,
    grid: NodeId,
    ramp: NodeId,
) -> Option<NodeId> {
    // ⚠️ O esticamento usa `value.map_range` com a interpolação no DEFAULT
    // (`Linear`) — aqui ele é PLUMBING, e é o mesmo nó que as fileiras 5-7
    // exercitam como sujeito. Um `map_range` de plumbing com interpolação
    // autorada mudaria a entrada dos módulos sem ninguém ver.
    let src = match row.signed {
        Some((lo, hi)) => {
            let mr = g.add_node("value.map_range");
            g.set_param(mr, "out_lo", lo);
            g.set_param(mr, "out_hi", hi);
            wire(g, ramp, 0, mr, 0)?;
            mr
        }
        None => ramp,
    };

    Some(match row.kind {
        Kind::Modulo { op } => {
            let m = g.add_node("value.math");
            g.set_param(m, "op", op);
            wire(g, src, 0, m, 0)?;
            // O divisor: um campo de comprimento 1 que o broadcast espalha. `0,75`
            // dá quatro dentes sobre `[-2, 2]` — poucos o bastante para se contar
            // a olho, muitos o bastante para a forma ser inequívoca.
            let d = constant(g, grid, 0.75)?;
            wire(g, d, 0, m, 1)?;
            m
        }
        Kind::Quantize { mode } => {
            let q = g.add_node("value.quantize");
            g.set_param(q, "step", 0.25);
            g.set_param(q, "mode", mode);
            wire(g, src, 0, q, 0)?;
            q
        }
        Kind::Map { interp } => {
            let mr = g.add_node("value.map_range");
            g.set_param(mr, "interpolation", interp);
            g.set_param(mr, "steps", 5.0);
            wire(g, src, 0, mr, 0)?;
            mr
        }
        Kind::Step { mode } => {
            let s = g.add_node("value.step");
            g.set_param(s, "threshold", 0.5);
            // A banda cobre a rampa inteira: uma banda estreita desenharia um
            // degrau quase duro, e as duas curvaturas ficariam invisíveis.
            g.set_param(s, "width", 1.0);
            g.set_param(s, "mode", mode);
            wire(g, src, 0, s, 0)?;
            s
        }
        Kind::Mix { blend } => {
            let m = g.add_node("value.mix");
            // `factor = 1` para o modo aparecer INTEIRO. Com o default de 0,5 a
            // fileira desenharia o modo diluído a meio caminho da rampa, e a
            // forma seria a média de duas coisas.
            g.set_param(m, "factor", 1.0);
            g.set_param(m, "blend", blend);
            wire(g, src, 0, m, 0)?;
            let b = constant(g, grid, 0.6)?;
            wire(g, b, 0, m, 1)?;
            m
        }
    })
}

/// Um campo CONSTANTE de comprimento 1 — o oscilador de amplitude zero, que o
/// broadcast `1→N` espalha por toda a fileira.
fn constant(g: &mut ph2d_nodegraph::graph::Graph, grid: NodeId, v: f32) -> Option<NodeId> {
    let k = g.add_node("value.lfo");
    g.set_param(k, "amplitude", 0.0);
    g.set_param(k, "offset", v);
    wire(g, grid, 0, k, 0)?;
    Some(k)
}

/// Uma aresta. Função LIVRE e não closure, pela mesma razão do irmão da direção:
/// uma closure que captura `g` o empresta até ao fim do escopo.
fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    a: NodeId,
    ap: u16,
    b: NodeId,
    bp: u16,
) -> Option<()> {
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

/// O que a cena anuncia — as fileiras, na ordem em que estão na tela.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_arith_tests.rs"]
mod tests;
