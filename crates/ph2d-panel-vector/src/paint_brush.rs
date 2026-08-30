//! ⭐⭐⭐ A secção **BRUSH** do painel Vector (plano 36, W4) — módulo irmão de [`super`] pelo teto de
//! LOC, e o corte é por MODELO.
//!
//! ⚠️⚠️ **Secção PRÓPRIA, e não mais um alvo da família do padrão.** Os knobs são OUTROS: um pincel
//! tem **avanço** e **escala relativa**; um padrão tem reticulado, fase e modo de repetição. Metade
//! dos de cada um ficaria morta na outra — que é exactamente o defeito que a wave F do plano 35
//! curou ao separar as duas secções do padrão.
//!
//! # Ela só sobe quando o traço TEM pincel
//!
//! `current_brush()` devolve `None` para todo traço que não é um — e então o cabeçalho nem aparece.

use super::*;

/// A faixa do **Spacing**: `<1` sobrepõe, `1` encaixa borda-a-borda, `>1` deixa vão.
pub(crate) const BRUSH_SPACING_MAX: f64 = 4.0; // LITERAL-PX-OK: domínio do documento
/// A faixa do **Size** (multiplicador da altura derivada da largura do traço).
pub(crate) const BRUSH_SCALE_MAX: f64 = 8.0; // LITERAL-PX-OK: domínio do documento
/// A faixa do **Offset**, em unidades de mundo — BIPOLAR (`0` ao centro do curso).
pub(crate) const BRUSH_OFFSET_MAX: f64 = 4.0; // LITERAL-PX-OK: domínio do documento
/// A faixa da **Rotation**, em graus.
pub(crate) const BRUSH_ROTATION_MAX: f64 = 360.0; // LITERAL-PX-OK: domínio do documento

/// O que o painel mostra de um pincel — espelho panel-local, pela mesma razão que o [`FillKind`] o é.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BrushRow {
    /// A arte já está escolhida? `false` = o traço é um pincel **sem** arte, e a linha pinta a cor
    /// de recurso — o estado em que ele nasce até o gesto de duas mãos se completar.
    pub has_art: bool,
    pub spacing: f64,
    pub scale: f64,
    pub offset: f64,
    pub rotation_deg: f64,
    pub flip: bool,
}

impl BodyCtx<'_> {
    /// Secção **BRUSH** — a lei da arte que percorre o contorno.
    pub(crate) fn brush_section(&mut self, y: f32) -> f32 {
        let Some(b) = state::current_brush() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_BRUSH,
            tr("panel.vector.section.brush"),
            y,
        );
        if collapsed {
            return y;
        }
        // ⭐⭐ **A ARTE, e o rótulo DIZ o estado.**
        //
        // ⚠️ Um pincel sem arte pinta a cor de recurso, e o artista não tem como saber porquê se o
        // botão disser sempre a mesma coisa. *Um controlo que não distingue "escolher" de "trocar"
        // manda o artista adivinhar em que estado está.*
        y = self.action_button(
            ids::VECTOR_BRUSH_PICK_SHAPE,
            if b.has_art {
                "Change Shape..."
            } else {
                "Pick Shape..."
            },
            y,
        );

        for (label, sid, nid, valor, faixa, fmt) in [
            (
                "Size",
                ids::VECTOR_BRUSH_SCALE,
                ids::VECTOR_BRUSH_SCALE_NUM,
                b.scale,
                BRUSH_SCALE_MAX,
                2,
            ),
            (
                "Spacing",
                ids::VECTOR_BRUSH_SPACING,
                ids::VECTOR_BRUSH_SPACING_NUM,
                b.spacing,
                BRUSH_SPACING_MAX,
                2,
            ),
            (
                "Rotation",
                ids::VECTOR_BRUSH_ROTATION,
                ids::VECTOR_BRUSH_ROTATION_NUM,
                b.rotation_deg,
                BRUSH_ROTATION_MAX,
                0,
            ),
        ] {
            let v = self.live_number(nid, valor);
            let track = self.live_track(sid, unipolar(valor, faixa));
            let txt = if fmt == 0 {
                format!("{v:.0}")
            } else {
                format!("{v:.2}")
            };
            y = self.slider_row(label, sid, nid, track, v, &txt, y);
        }

        // O **Offset** é BIPOLAR: negativo põe a arte do outro lado da linha.
        let off = self.live_number(ids::VECTOR_BRUSH_OFFSET_NUM, b.offset);
        let track = self.live_track(
            ids::VECTOR_BRUSH_OFFSET,
            bipolar(b.offset, BRUSH_OFFSET_MAX),
        );
        y = self.slider_row(
            "Offset",
            ids::VECTOR_BRUSH_OFFSET,
            ids::VECTOR_BRUSH_OFFSET_NUM,
            track,
            off,
            &format!("{off:.2}"),
            y,
        );

        // ⚠️ Um **checkbox**, e não um `segmented`, pela lei da casa: virar a arte é uma PROPRIEDADE
        // que o pincel tem ou não tem, não uma escolha entre modos nomeados.
        self.checkbox_row(
            ids::VECTOR_BRUSH_FLIP,
            tr("panel.vector.brush.flip"),
            b.flip,
            y,
        )
    }
}

/// O track `0..1` de um valor unipolar. ⚠️ O MESMO mapa que o `event` e o `populate` usam.
#[must_use]
pub(crate) fn unipolar(v: f64, max: f64) -> f32 {
    ((v / max).clamp(0.0, 1.0)) as f32
}

/// O track `0..1` de um valor BIPOLAR (`0.5` = zero).
#[must_use]
pub(crate) fn bipolar(v: f64, max: f64) -> f32 {
    (((v + max) / (2.0 * max)).clamp(0.0, 1.0)) as f32
}
