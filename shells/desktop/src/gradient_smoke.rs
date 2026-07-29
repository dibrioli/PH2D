//! **A cena pronta para o smoke do editor de GRADIENTE** (`PH2D_GRADIENT_SMOKE=1`, doc 85).
//!
//! O irmão de COR do `value_curve_smoke`: o `motion.color_ramp` Custom pinta uma fileira de
//! instâncias pela sua rampa, e o editor de gradiente (a barra + os stops arrastáveis + os
//! swatches OKLCH) fica no painel de params, já com o nó SELECIONADO.
//!
//! ```text
//! motion.grid → motion.scale → motion.color_ramp(Custom) → motion.output
//! ```
//!
//! O `t` da rampa fica DESconectado, então cada instância é colorida pelo índice
//! normalizado `i/(N−1)` — a rampa deitada ao longo da fileira: um SWEEP contínuo de cor.
//! Com a rampa vermelho→verde→azul, a fileira sai vermelha à esquerda, verde no meio, azul à
//! direita. Selecionado o `Color Ramp`, o painel mostra:
//!
//! - a **barra** de gradiente (a rampa desenhada) com **3 marcadores** de posição na base;
//! - um **swatch por stop** embaixo, cada um abre o picker OKLCH;
//! - **`+` / `−` / interp** no cabeçalho.
//!
//! TESTE: arraste um marcador → o stop anda e a fileira re-colore ao vivo; clique um swatch →
//! o picker abre naquela cor, escolha outra e o stop (e a fileira) muda; `+` insere um stop no
//! maior vão, `−` remove o selecionado. O nó cozinha **100% na GPU** (as 3 LUTs de canal do
//! doc 85), então o sweep é o mesmo com `PH2D_GPU_COOK=1` (default) e `=0`.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A rampa Custom da cena: vermelho → verde → azul, interp Linear (`RampInterp::to_u8`
/// Linear = 2). Cores primárias para o sweep ser inequívoco (uma parametrização que
/// nenhum preset expõe — é a prova de que a string Custom é lida).
const GRAD: &str = "g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1";

/// Monta a fileira `grid → scale → color_ramp(Custom) → output`. Devolve `(sink, hero)`:
/// o `motion.output` e o `motion.color_ramp` (o nó a avaliar/selecionar). O `t` fica
/// desconectado de propósito (a cor sai do índice — o sweep).
fn row(g: &mut Graph) -> (NodeId, NodeId) {
    let grid = g.add_node("motion.grid");
    let scale = g.add_node("motion.scale");
    let cr = g.add_node("motion.color_ramp");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(scale, "amount", 0.34);
    g.set_param(cr, "preset", 4.0); // Custom — lê a string abaixo
    g.set_text_param(cr, "ramp", GRAD.to_string());

    for (from, to, port) in [(grid, scale, 0u16), (scale, cr, 0), (cr, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .expect("gradient-smoke edge");
    }
    (out, cr)
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_GRADIENT_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `value_curve_smoke`. No-op sem a env.
    pub(crate) fn gradient_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sink, hero) = row(&mut gfx.motion.doc.graph);
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &[hero]);
        gfx.motion.sinks.push(sink);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // Seleciona o Color Ramp para o editor de gradiente estar na tela já no 1º frame.
        ph2d_panel_motion_graph::request_graph_selection(vec![hero.0]);
        eprintln!(
            "[gradient smoke] Uma fileira de 24 pontos colorida por um SWEEP \
             vermelho->verde->azul (a rampa deitada ao longo da fileira). O no 'Color Ramp' \
             ja esta selecionado e o painel de params (a direita) mostra o EDITOR DE \
             GRADIENTE: a barra, 3 marcadores de posicao na base, e um swatch por stop.\n  \
             TESTE: arraste um marcador -> o stop anda e a fileira re-colore ao vivo. Clique \
             um swatch -> o picker OKLCH abre naquela cor; escolha outra e o stop (e a \
             fileira) muda. '+' insere um stop no maior vao, '-' remove o selecionado. \
             (Roda igual com PH2D_GPU_COOK=1 e =0 -- as 3 LUTs de canal cozinham na GPU.)"
        );
    }
}
