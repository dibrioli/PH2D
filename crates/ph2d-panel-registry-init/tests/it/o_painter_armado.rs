//! ⭐⭐⭐ **O PAINTER COM UM PADRÃO NA MÃO — o ponto cego que o report de 2026-09-21 revelou.**
//!
//! # ⛔⛔ O buraco
//!
//! O dono mandou uma foto: a secção `SHAPE ▸ Texture`, com a textura **Wood** escolhida, pintava
//! quatro fileiras assim —
//!
//! ```text
//! paint_brush.pattern_param.contrast     0.500
//! paint_brush.pattern_param.turbulence   0.500
//! ```
//!
//! — ou seja, a **chave** de tradução no lugar do nome. ⭐ A tabela sabia traduzi-las; o que
//! faltava era o pintor chamar o `tr`, e o laço que monta aquelas fileiras estava escrito **três
//! vezes** (duas traduziam).
//!
//! ⛔⛔⛔ **E a varredura de rótulos NÃO o via, com o gate dela verde.** O estado de FÁBRICA do
//! Painter tem a textura em `None`, e ali aquelas fileiras **não são pintadas**: elas só existem
//! depois de o artista escolher um padrão. *Um painel cujas fileiras dependem de uma escolha é
//! medido vazio exactamente na parte que o artista usa* — a mesma cegueira que o
//! [`super::o_inspector_armado`] existe para fechar, no painel ao lado.
//!
//! # ⭐ O que esta fixtura é
//!
//! O instantâneo de pincel que o próprio painel usa antes de a shell publicar um
//! ([`ph2d_panel_painter_layers::FALLBACK_BRUSH`]), com as **três** famílias de padrão armadas:
//! a da SILHUETA (`shape_kind`), a do GRÃO (`texture_kind`) e a do PAPEL (`paper_kind`).
//!
//! ⚠️ **O padrão é o `Wood` de propósito** — é o da foto do dono, e é um dos que tem params
//! próprios (`Turbulence` · `Rings`) além dos dois comuns. Um padrão sem params extra mediria
//! menos rótulos e a fixtura não conteria o fenómeno.

use ph2d_painter_brush::TextureKind;

/// O `Wood` da foto do dono.
fn wood() -> u8 {
    TextureKind::Wood.to_u8()
}

/// Publica um pincel com as três famílias de padrão armadas.
pub fn arma() {
    let brush = ph2d_tool_painter::BrushSettings {
        shape_kind: wood(),
        texture_kind: wood(),
        paper_kind: wood(),
        ..ph2d_panel_painter_layers::FALLBACK_BRUSH
    };
    ph2d_panel_painter_layers::state::set_current_brush(Some(brush));
}

/// ⚠️ O estado que uma fixtura deixa para trás é o estado que a régua seguinte mede — estas
/// portas são `thread_local`.
pub fn desarma() {
    ph2d_panel_painter_layers::state::set_current_brush(None);
}
