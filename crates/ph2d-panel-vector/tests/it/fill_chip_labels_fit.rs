//! **Cada rótulo do selector de preenchimento CABE no chip dele** — medido, não estimado.
//!
//! # Porque isto existe
//!
//! O `paint_segmented_button` diz de si próprio que é *"the single source of truth for grouped
//! **2–3-way** selectors"*. A fileira de tipo de preenchimento tinha **quatro** chips e o plano 33
//! acrescentou o **quinto** — e cada chip novo encolhe TODOS os outros, porque a largura é
//! `(inner_w − 4·gap) / 5`.
//!
//! ⚠️ **O painter não trunca**: um rótulo largo demais transborda o chip e encosta no vizinho. Isso
//! é exactamente o tipo de defeito que chega como foto, e é medível **aqui** — com o painter REAL
//! (a largura do chip sai do `painted_rect`) e o sistema de texto REAL (a largura do rótulo sai do
//! `prefix_width`). ⛔ Um valor escrito à mão de qualquer um dos dois mediria outra coisa.
//!
//! ⭐⭐ **E foi este gate que escolheu a CURA, não o rótulo.** A primeira leitura dele —
//! `Pattern = 41,37 px` num chip de `45,60`, folga **negativa** — ia levar a encurtar o rótulo para
//! `Tile`. A causa era outra: a fileira repartia a largura **à mão** em vez de usar o
//! `paint_segmented_group_adaptive`, que é a resposta canónica do design system a *"como um grupo
//! segmentado quebra"* e que o doc-comment de [`super`] já chamava de *"uma segunda regra de quebra
//! vivendo neste painel"*. Curada a fileira, os quatro chips antigos voltam aos `60,0 px` **de
//! antes** e o quinto ganha uma linha inteira. *Um rótulo que não cabe pode ser o sintoma de uma
//! fileira que não reflui.*

use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::{FillKind, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_text::TextSystem;
use ph2d_tokens::TypeToken;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// A folga mínima de cada lado. ⚠️ Não é estética: um rótulo colado à borda do chip lê-se como
/// texto cortado mesmo quando cabe, e a linha do chip vizinho fica a um pixel dele.
const SIDE_PAD: f32 = 3.0;

fn chip_rect(id: ph2d_a11y::NodeId) -> Option<Rect> {
    // A fileira só sobe com um preenchimento publicado — sem isto o painel não desenha a secção e
    // o teste mediria a ausência dela.
    state::set_current_fill(Some(FillKind::Solid), None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
}

#[test]
fn every_fill_kind_chip_label_fits_its_chip() {
    let mut text = TextSystem::without_system_fonts();
    let size = TypeToken::Sm.px();
    let chips = [
        (ids::VECTOR_FILL_KIND_SOLID, "Solid"),
        (ids::VECTOR_FILL_KIND_LINEAR, "Linear"),
        (ids::VECTOR_FILL_KIND_RADIAL, "Radial"),
        (ids::VECTOR_FILL_KIND_MULTI, "Multi"),
        (ids::VECTOR_FILL_KIND_PATTERN, "Pattern"),
    ];
    let mut tightest: Option<(&str, f32)> = None;
    let mut chip_w = 0.0f32;
    for (id, label) in chips {
        let rect = chip_rect(id).unwrap_or_else(|| panic!("o chip `{label}` nao foi pintado"));
        chip_w = rect.w;
        let w = text.prefix_width(label, size);
        let slack = rect.w - w - 2.0 * SIDE_PAD;
        println!(
            "chip `{label}`: rotulo {w:.2} px, chip {:.2} px, folga {slack:.2} px",
            rect.w
        );
        if tightest.is_none_or(|(_, s)| slack < s) {
            tightest = Some((label, slack));
        }
        assert!(
            slack >= 0.0,
            "o rotulo `{label}` mede {w:.2} px e o chip {:.2} px (folga pedida: {SIDE_PAD} de cada \
             lado). Encurte o rotulo ou reveja a fileira - o painter NAO trunca.",
            rect.w
        );
    }
    let (l, s) = tightest.expect("ha' chips");
    println!("mais apertado: `{l}` com {s:.2} px de folga");
    // ⚠️ **A restricao que ditava um rotulo curto DISSOLVEU-SE ao curar a fileira.** Com a
    // reparticao a' mao os cinco chips ficavam a 45,60 px e `Pattern` (41,37) nao cabia; com o
    // refluxo do `paint_segmented_group_adaptive` os quatro primeiros voltam aos 60,0 px de antes e
    // o quinto ganha uma linha inteira. Este numero e' o que reabre a decisao se a fileira crescer.
    let _ = chip_w;
}
