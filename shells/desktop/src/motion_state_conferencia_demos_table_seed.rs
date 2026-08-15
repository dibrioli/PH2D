//! **A TABELA E A SEMENTE** (`PH2D_GPU_COOK_DEMO=44`) — a cena do **grupo D** da
//! conferência (doc 89, folha 15, a wave W-E): a lista que o artista DIGITA, sem
//! o teto de oito, e a semente que a identidade do nó decorrelaciona.
//!
//! ## As duas metades são a MESMA pergunta
//!
//! Um nó carrega um punhado de `f32` — o `NodeManifest` é f32-only por contrato
//! congelado (ADR-0039). As duas linhas deste grupo são o que **não cabe** ali:
//! uma LISTA de comprimento arbitrário, e a IDENTIDADE do próprio nó. Nenhuma
//! das duas é um param, e as duas chegam ao device por canais que já existiam
//! (a LUT de text param; um uniforme derivado do que o kernel pede).
//!
//! ## As duas leituras, e a segunda tem um CONTROLE embutido
//!
//! **Bandas 1-2 — A TABELA.** O mesmo `value.pattern` duas vezes. Em cima os
//! **oito slots** com `steps = 3`: um dente de serra que se repete dezasseis
//! vezes na fileira. Em baixo a **TABELA de doze**, digitada como texto: o mesmo
//! dente, quatro vezes mais LARGO. ⚠️ *Doze é o número que prova a wave* — acima
//! do teto de oito que nenhum recurso justificava.
//!
//! **Bandas 3-6 — A SEMENTE.** Quatro fileiras de ruído por-elemento, todas com
//! a **MESMA semente autorada** (`seed = 7`).
//!
//! - **3 e 4** têm o toggle DESLIGADO e são **IDÊNTICAS** — dois nós irmãos
//!   produzindo o mesmo campo. É o defeito, e ele é a metade mais importante da
//!   cena: sem ele à vista, *"ligado eles diferem"* seria verdade também num
//!   mundo em que eles nunca foram gêmeos.
//! - **5 e 6** têm o toggle LIGADO e **não** são idênticas.
//!
//! ⚠️ **O olho compara SILHUETA aqui, não altura** — por isso cada fileira tem a
//! própria linha de base, ao contrário da cena `=43`, cuja comparação era de
//! altura e por isso empilhava cadeias na MESMA base.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por fileira.
pub(crate) const COLS: f32 = 48.0;
/// Quantas fileiras a cena empilha.
pub(crate) const BANDS: usize = 6;
/// A distância vertical entre fileiras.
const BAND_GAP: f32 = 1.05;
/// Quanto o valor levanta a peça — o mesmo para TODAS, senão duas silhuetas
/// deixariam de ser comparáveis, que é a única coisa que as bandas 3-6 medem.
const VALUE_SCALE: f32 = 0.9;
/// O tamanho da peça.
const DOT: f32 = 0.19;

/// Quantos passos o caminho LEGADO cicla (dos oito slots).
pub(crate) const LEGACY_STEPS: f32 = 3.0;
/// Quantos passos a TABELA cicla. ⚠️ **Acima de oito de propósito** — é este
/// número que a wave destranca, e é ele que o olho conta na banda 2.
pub(crate) const TABLE_STEPS: usize = 12;
/// A semente que as quatro fileiras de ruído partilham.
const SHARED_SEED: f32 = 7.0;
/// O `mode` `Random` do `value.instance_field`.
const FIELD_RANDOM: f32 = 2.0;

/// A tabela que a banda 2 autora: uma rampa de [`TABLE_STEPS`] degraus.
///
/// Construída aqui em vez de escrita à mão para o número viver UMA vez — uma
/// string literal e uma const separadas divergiriam no dia em que alguém
/// mexesse numa delas, e a cena passaria a anunciar um período que não desenha.
pub(crate) fn table_text() -> String {
    (0..TABLE_STEPS)
        .map(|k| {
            #[expect(clippy::cast_precision_loss, reason = "TABLE_STEPS e' pequeno")]
            let t = k as f32 / (TABLE_STEPS - 1) as f32;
            format!("{t:.4} ")
        })
        .collect()
}

/// O que uma fileira desenha.
#[derive(Clone, Copy)]
enum Kind {
    /// O `value.pattern` pelos oito slots — o nó que sempre shipou.
    PatternLegacy,
    /// O `value.pattern` pela TABELA autorada.
    PatternTable,
    /// Ruído por-elemento; `unique` liga a decorrelação por identidade de nó.
    Random { unique: bool },
}

static LANES: [Kind; BANDS] = [
    Kind::PatternLegacy,
    Kind::PatternTable,
    Kind::Random { unique: false },
    Kind::Random { unique: false },
    Kind::Random { unique: true },
    Kind::Random { unique: true },
];

/// O que a cena anuncia — uma linha por fileira, na ordem em que estão na tela.
pub(crate) const BAND_LABELS: [&str; BANDS] = [
    "1 PATTERN  oito slots, steps=3   -- dente de serra FINO (16 repeticoes)",
    "2 PATTERN  TABELA de 12 (texto)  -- o mesmo dente, 4x mais LARGO",
    "3 RANDOM   seed 7, unique OFF    -- \\",
    "4 RANDOM   seed 7, unique OFF    -- /  as duas TEM de ser IDENTICAS (o defeito)",
    "5 RANDOM   seed 7, unique ON     -- \\",
    "6 RANDOM   seed 7, unique ON     -- /  e estas duas NAO podem ser",
];

/// `grid → scale → <cadeia de valor> → drive(Y) → transform → output`, uma vez
/// por fileira. Devolve os sinks.
pub(super) fn build_table_seed_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, kind) in LANES.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "BANDS e' pequeno")]
        let row = 100.0 + k as f32 * 210.0;
        // A fileira do topo é a PRIMEIRA da tabela — ler o gráfico de cima para
        // baixo tem de dar a mesma ordem que ler a tabela no código.
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
        Kind::PatternLegacy | Kind::PatternTable => {
            let vp = g.add_node("value.pattern");
            g.set_param(vp, "steps", LEGACY_STEPS);
            // Um dente de serra de três degraus, para o período curto ser
            // visível como FORMA e não só como ruído.
            g.set_param(vp, "v0", 0.0);
            g.set_param(vp, "v1", 0.5);
            g.set_param(vp, "v2", 1.0);
            if matches!(kind, Kind::PatternTable) {
                g.set_text_param(vp, ph2d_node_value_pattern::TABLE_KEY, table_text());
            }
            wire(g, geom, 0, vp, 0)?;
            vp
        }
        Kind::Random { unique } => {
            let f = g.add_node("value.instance_field");
            g.set_param(f, "mode", FIELD_RANDOM);
            g.set_param(f, "seed", SHARED_SEED);
            g.set_param(f, "unique_per_node", f32::from(u8::from(unique)));
            wire(g, geom, 0, f, 0)?;
            f
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
#[path = "motion_state_conferencia_demos_table_seed_tests.rs"]
mod tests;
