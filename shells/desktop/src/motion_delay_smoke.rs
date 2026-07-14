//! **A cena pronta para o smoke do `motion.delay`** (`PH2D_MOTION_DELAY_SMOKE=1`, doc 63 §5).
//!
//! ## Por que esta cena existe (e o que ela corrige)
//!
//! Eu tinha posto o `motion.delay` no documento de boot dizendo que ele *"tirava o tremor da neve"*.
//! O Enio duvidou. **Ele estava certo, e eu estava errado — MEDIDO:**
//!
//! | | queda | desvio da aceleração (o "tremor") | excursão lateral |
//! |---|---|---|---|
//! | a neve, com o `gust` do demo | 85 ticks | **0,00024 = 0,1% de um floco** | **ZERO** |
//! | a neve, sem gust | 75 ticks | 0,00000 (parábola perfeita) | zero |
//!
//! **A neve não treme.** O `gust` do `force.wind` modula a **MAGNITUDE** de uma força que aponta
//! reto pra baixo — o floco cai em **linha reta**, só que mais rápido ou mais devagar. Não há
//! oscilação, não há deriva. (E a queda de 47% que eu tinha medido numa 3ª diferença era a ease
//! amaciando o **SPLASH**, não tremor nenhum: número certo, história errada.)
//!
//! Um suavizador precisa de algo **trêmulo** pra suavizar, e a simulação produz movimento **suave**.
//! Então o nó saiu do boot e ganhou a cena em que ele **é** o que ele diz ser:
//!
//! ```text
//! grid → move(cima)  → wiggle(f=8) ──────────────────→ scale → output   ← TREME
//! grid → move(baixo) → wiggle(f=8) → delay(Blend, 6) → scale → output   ← SEDOSO
//! ```
//!
//! **Medido, os dois:** o `motion.wiggle` a f=8 sacode **0,095 de mundo por tick — 53% da largura do
//! próprio objeto, a cada quadro**. Com a ease: **0,036 (20%)** — o tremor cai **61%** e a
//! **excursão sobrevive** (1,00 → 0,92). É essa a promessa: tira o tremor, **não** o movimento.
//!
//! As duas fileiras usam o MESMO `wiggle` (mesmos params, mesmas fileiras de ruído), então a única
//! diferença entre elas é o nó. Sem isso não seria uma comparação, seria uma ilustração.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// A frequência em que o wiggle deixa de ser um movimento e vira um TREMOR (medido: 53% da largura
/// do objeto por tick). Abaixo de ~4 ele é só uma flutuação lenta e não há o que suavizar.
const TWITCHY: f32 = 8.0;
/// A janela do polo simples. 6 ticks = 100 ms: o suficiente pra matar o tremor sem matar o gesto.
const EASE_TICKS: f32 = 6.0;
const WIGGLE_CH_Y: f32 = 1.0;
const DELAY_MODE_BLEND: f32 = 2.0;

/// Uma fileira: `grid → move → wiggle [→ delay] → scale → output`. Devolve o sink.
fn row(g: &mut Graph, y: f32, eased: bool, screen_row: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let wig = g.add_node("motion.wiggle");
    let dly = eased.then(|| g.add_node("motion.delay"));
    let scale = g.add_node("motion.scale");
    let out = g.add_node("motion.output");

    let mut chain: Vec<NodeId> = vec![grid, mv, wig];
    if let Some(d) = dly {
        chain.push(d);
    }
    chain.extend([scale, out]);
    for (i, n) in chain.iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: i as f32 * 190.0,
                y: screen_row,
            },
        );
    }
    for w in chain.windows(2) {
        g.connect(Edge {
            from: (w[0], 0),
            to: (w[1], 0),
            delayed: false,
        })
        .ok()?;
    }
    if let Some(d) = dly {
        // O self-loop `pre` do nó sequencial — o que o editor plumba ao soltar o card.
        g.connect(Edge {
            from: (d, 0),
            to: (d, 1),
            delayed: true,
        })
        .ok()?;
        g.set_param(d, "mode", DELAY_MODE_BLEND);
        g.set_param(d, "ticks", EASE_TICKS);
        g.set_label(d, "The Ease");
    }

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 6.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", y);
    g.set_param(wig, "channel", WIGGLE_CH_Y);
    g.set_param(wig, "amplitude", 0.5);
    g.set_param(wig, "frequency", TWITCHY);
    g.set_param(scale, "amount", 0.18);
    g.set_label(
        wig,
        if eased {
            "Wiggle (eased)"
        } else {
            "Wiggle (raw)"
        },
    );
    g.set_label(out, if eased { "SMOOTH" } else { "TWITCHY" });
    Some(out)
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_MOTION_DELAY_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn motion_delay_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // A de cima TREME; a de baixo é a mesma coisa com o nó no meio.
        let raw = row(g, 1.4, false, -460.0);
        let eased = row(g, -0.2, true, -300.0);
        gfx.motion.sinks.extend(raw.into_iter().chain(eased));
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}
