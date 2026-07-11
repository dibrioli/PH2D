//! Previews do dropdown de fonte: o **nome de cada família desenhado na própria
//! fonte** (preview de estilo real, estilo Figma/Photoshop). Vive na shell porque é
//! aqui que os [`VariableFont`] são resolvidos ([`crate::vec_font`]); o painel só
//! recebe o [`FontPreview`] pronto e o desenha.
//!
//! Construção **lazy**: só chamada quando o dropdown abre pela 1ª vez (o painel pede
//! via `take_want_font_previews`), então o scan+parse das fontes do sistema é pago
//! uma vez, no open, nunca ao entrar no modo Text.
//!
//! O contorno sai em **espaço-em** (1 = `units_per_em`), **y-up** e com o avanço
//! acumulado em x — o painel escala e espelha (y-down de tela) por `Affine`.

use ph2d_panel_vector::FontPreview;
use ph2d_vector::{BezPath, Point};
use ph2d_vector_font::{GlyphOutline, PathCommand, VariableFont};

/// Avanço de um caractere sem glyph (raro): ~¼ em, só p/ o nome não colar.
const SPACE_ADVANCE_EM: f64 = 0.25;

/// Uma preview por família selecionável, na ordem canônica de
/// [`crate::vec_font::pickable_families`] — o índice publicado casa com o que a
/// shell aplica ao escolher a opção.
pub(crate) fn build_previews() -> Vec<FontPreview> {
    crate::vec_font::pickable_families()
        .into_iter()
        .map(|family| {
            let font = crate::vec_font::resolve(family.as_deref());
            let display = crate::vec_font::display_name(family.as_deref());
            let (outline, advance_em) = name_outline(&font, &display);
            FontPreview {
                family,
                display,
                outline,
                advance_em,
            }
        })
        .collect()
}

/// `text` desenhado nos glyphs de `font`, em espaço-em (y-up), na instância PADRÃO
/// da fonte (eixos vazios) — a preview mostra o estilo próprio, não o peso do
/// documento. Layout advance-only (sem shaping); basta p/ um rótulo. Devolve
/// `(path, avanço total em em)`.
fn name_outline(font: &VariableFont, text: &str) -> (BezPath, f64) {
    let upem = f64::from(font.units_per_em().max(1));
    let s = 1.0 / upem;
    let mut bp = BezPath::new();
    let mut penx = 0.0;
    for ch in text.chars() {
        let Some(gid) = font.glyph_for_char(ch) else {
            penx += SPACE_ADVANCE_EM;
            continue;
        };
        if let Ok(outline) = font.outline(gid, &[]) {
            append_glyph(&mut bp, &outline, penx, s);
        }
        penx += font.advance(gid, &[]).map_or(0.0, f64::from) * s;
    }
    (bp, penx)
}

/// Anexa um glyph a `bp`, transladado por `penx` (em) e escalado por `s`
/// (1/units_per_em). Sem flip-Y — o painel o faz no `Affine` de colocação.
fn append_glyph(bp: &mut BezPath, outline: &GlyphOutline, penx: f64, s: f64) {
    let pt = |x: f32, y: f32| Point::new(penx + f64::from(x) * s, f64::from(y) * s);
    for cmd in &outline.commands {
        match *cmd {
            PathCommand::MoveTo(p) => bp.move_to(pt(p.x, p.y)),
            PathCommand::LineTo(p) => bp.line_to(pt(p.x, p.y)),
            PathCommand::QuadTo { ctrl, to } => bp.quad_to(pt(ctrl.x, ctrl.y), pt(to.x, to.y)),
            PathCommand::CurveTo { c1, c2, to } => {
                bp.curve_to(pt(c1.x, c1.y), pt(c2.x, c2.y), pt(to.x, to.y))
            }
            PathCommand::Close => bp.close_path(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O nome da fonte embutida vira contorno real (o caminho glyph→BezPath) com
    /// avanço acumulado — o que o dropdown desenha em cada linha.
    #[test]
    fn bundled_name_outline_has_geometry() {
        let font = crate::vec_font::resolve(None);
        let (bp, advance_em) = name_outline(&font, "Inter");
        assert!(
            !bp.elements().is_empty(),
            "o nome deve produzir geometria de glyph"
        );
        assert!(advance_em > 0.0, "o avanco acumulado deve ser positivo");
    }

    /// Uma string vazia não gera geometria nem avanço (borda: sessão sem nome).
    #[test]
    fn empty_text_is_empty_outline() {
        let font = crate::vec_font::resolve(None);
        let (bp, advance_em) = name_outline(&font, "");
        assert!(bp.elements().is_empty());
        assert_eq!(advance_em, 0.0);
    }
}
