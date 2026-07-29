//! **A cena pronta para o smoke do adapter automático** (`PH2D_ADAPTER_SMOKE=1`,
//! plan §1.1 item 2).
//!
//! O substrato é ESTRITO: um fio só conecta portas do mesmo tipo. Arrastar uma saída
//! de **stream** para uma entrada de **valor** é incompatível e o editor recusava —
//! agora ele **insere o nó-adapter** (`value.attribute`) no fio: `grid → attribute →
//! gain`, como UM passo de undo.
//!
//! A cena monta só DOIS nós, lado a lado, num grafo limpo:
//!
//! - **`motion.grid`** à esquerda — a saída é um **stream** `(Instances, Vec2)`.
//! - **`value.gain`** à direita — a entrada é um **valor** `(Instances, Scalar)`.
//!
//! Arraste da saída do `grid` para a entrada do `value.gain`. Os tipos não batem,
//! então o editor insere um `Attribute` entre eles (que lê uma coluna do stream como
//! escalar — o artista escolhe qual no painel) e mostra um toast. Um Ctrl+Z desfaz o
//! splice inteiro.

use ph2d_nodegraph::graph::Pos;

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_ADAPTER_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn adapter_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        // Um grafo LIMPO com só os dois nós a conectar (o boot document é a cena da
        // neve — cluttered para esta demonstração).
        gfx.motion.doc = ph2d_motion_doc::MotionDoc::new();
        gfx.motion.sinks.clear();
        let g = &mut gfx.motion.doc.graph;
        let grid = g.add_node("motion.grid"); // stream output
        let gain = g.add_node("value.gain"); // value input
        g.set_pos(grid, Pos { x: 140.0, y: 220.0 });
        g.set_pos(gain, Pos { x: 560.0, y: 220.0 });
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        eprintln!(
            "[adapter smoke] dois nós no grafo: 'motion.grid' (saída = STREAM) à \
             esquerda, 'value.gain' (entrada = VALOR) à direita. Arraste da SAÍDA do \
             grid para a ENTRADA do gain: os tipos não batem, então um adapter \
             'Attribute' é inserido AUTOMATICAMENTE no fio (grid -> attribute -> gain). \
             Ctrl+Z desfaz o splice inteiro. (Sem a env, ou entre portas do MESMO tipo, \
             nada muda.)"
        );
    }
}
