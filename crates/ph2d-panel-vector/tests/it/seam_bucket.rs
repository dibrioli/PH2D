//! ⭐⭐⭐ Seam do pill **Bucket** (plano 40) — ele está vivo sob o MOUSE, o clique chega ao bus, e
//! o modo que ele arma é o dele.
//!
//! ⚠️ **Escrito ANTES do smoke**, e a razão é o report de 2026-09-01: um verbo cujo botão não fala
//! com ninguém lê-se exactamente como um motor partido, e o `seam_weld.rs` só nasceu depois de o
//! Enio ter pago essa ambiguidade. *Um pill novo tem quatro sítios independentes (id · pintura ·
//! registo · allowlist), e três deles falham em silêncio.*

use ph2d_editor_core::tool::PanelEvent;




/// ⭐ **E o clique arma o modo do BALDE, não o do vizinho.**
///
/// ⚠️ A cadeia de guardas do `apply_panel_event` é escrita à mão, e um `id` copiado da linha de
/// cima compila: o pill do Balde acenderia o Trim, e o artista veria a ferramenta errada com o
/// nome certo.
#[test]
fn clicking_the_pill_arms_the_bucket_mode() {
    use ph2d_editor_core::tool::Tool;
    let mut tool = ph2d_tool_vector::VectorTool::default();
    tool.handle_panel_event(PanelEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_BUCKET));
    assert_eq!(
        tool.draw_config().mode,
        ph2d_tool_vector::DrawMode::Bucket,
        "o pill do Balde armou outro modo"
    );
}
