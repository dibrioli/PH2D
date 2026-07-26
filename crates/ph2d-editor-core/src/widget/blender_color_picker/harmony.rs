//! **Color Harmonies** — conjuntos de cores LIGADAS à base pelo matiz.
//!
//! O *Color Harmonies* do Corel / o *Adobe Color*: a cor base mais parceiras em offsets FIXOS de
//! matiz na roda. As parceiras são DERIVADAS, nunca guardadas — então mover a base gira todas,
//! preservando o espaçamento relativo (a propriedade "linked" que o Illustrator/Figma/Affinity não
//! têm).
//!
//! A rotação é na roda **HSV** (a roda que o artista vê): um complementar fica DIAMETRALMENTE
//! OPOSTO, uma tríade nos cantos do triângulo — exatamente onde o olho os espera. (OKLCH-hue foi
//! considerado e rejeitado: offsets perceptuais não pousam em ângulos reconhecíveis da roda, e um
//! complementar que não é oposto lê como quebrado. A SAÍDA continua [`ColorValue`] com o oklch
//! sincronizado, então consumidores perceptuais não perdem nada — o pin está no gate.)
//!
//! Esta é a porta ÚNICA de *"quais são as cores ligadas a esta?"* — o painel a desenha e o
//! dispatch a consome pela MESMA [`partners`]; duas cópias divergiriam no 1º ajuste de offset.

use super::channels::{hsv_to_rgba8, rgba_to_hsv};
use super::state::BlenderColorPicker;
use super::sub_ids::BlenderSubIds;
use crate::interaction::HitIndex;
use crate::paint::{fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect};
use crate::widget::{RadioGroup, RadioOption, RadioOrientation, paint_radio_group_with_labels};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ColorValue, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O esquema de harmonia selecionado. `None` = a seção mostra só o seletor, sem parceiras.
///
/// View-state (como [`super::ChannelMode`]): não é serde — não viaja no projeto (as palettes
/// persistem à parte, num `~/.ph2d/palettes.txt`), então não move `PROJECT_SCHEMA`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Harmony {
    /// Sem harmonia — a base sozinha. É o default (a seção não intromete no picker que todo
    /// módulo já usa; o artista escolhe um esquema para vê-la).
    #[default]
    None,
    /// A cor oposta (+180°).
    Complementary,
    /// As duas vizinhas (±30°).
    Analogous,
    /// O triângulo equilátero (+120°, +240°).
    Triad,
    /// A base + as duas ao lado do complementar (+150°, +210°).
    SplitComplementary,
    /// O quadrado (+90°, +180°, +270°).
    Tetrad,
    /// Mesmo matiz, valores diferentes (a única que NÃO gira o matiz).
    Monochromatic,
}

impl Harmony {
    /// Todos os esquemas, na ordem do seletor. `None` primeiro (o default, "desligado").
    pub const ALL: [Harmony; 7] = [
        Harmony::None,
        Harmony::Complementary,
        Harmony::Analogous,
        Harmony::Triad,
        Harmony::SplitComplementary,
        Harmony::Tetrad,
        Harmony::Monochromatic,
    ];

    /// O maior número de cores (base + parceiras) que um esquema produz — dimensiona a tira de
    /// swatches e o bundle de ids. Tetrad e Monochromatic empatam em 4.
    pub const MAX_COLORS: usize = 4;

    /// O rótulo curto do seletor. Literal em inglês, como o resto do picker (`RGB`/`HSV`/`Red`…) —
    /// o app é inglês-only e o widget inteiro é literal; 6 chaves i18n no meio dele seriam pior.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Harmony::None => "Off",
            Harmony::Complementary => "Comp",
            Harmony::Analogous => "Anlg",
            Harmony::Triad => "Triad",
            Harmony::SplitComplementary => "Split",
            Harmony::Tetrad => "Tetra",
            Harmony::Monochromatic => "Mono",
        }
    }

    /// A chave estável do seletor (para o [`crate::widget::RadioOption`]); nunca localizada.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Harmony::None => "off",
            Harmony::Complementary => "comp",
            Harmony::Analogous => "anlg",
            Harmony::Triad => "triad",
            Harmony::SplitComplementary => "split",
            Harmony::Tetrad => "tetra",
            Harmony::Monochromatic => "mono",
        }
    }

    /// Os offsets de matiz (em GRAUS) das parceiras, sem a base (que é sempre a 1ª cor). Vazio para
    /// `None` (só a base) e para `Monochromatic` (que não gira matiz — ver [`partners`]).
    fn hue_offsets(self) -> &'static [f32] {
        match self {
            Harmony::None | Harmony::Monochromatic => &[],
            Harmony::Complementary => &[180.0],
            Harmony::Analogous => &[-30.0, 30.0],
            Harmony::Triad => &[120.0, 240.0],
            Harmony::SplitComplementary => &[150.0, 210.0],
            Harmony::Tetrad => &[90.0, 180.0, 270.0],
        }
    }
}

/// **As cores ligadas a `base` sob `scheme`** — a base é SEMPRE a 1ª. Porta única.
///
/// Rotações de matiz na roda HSV, preservando saturação e valor (`Monochromatic` faz o inverso:
/// preserva matiz e satura, varia o valor). O alfa da base sobrevive em todas. Uma base acromática
/// (cinza, `s≈0`) devolve cópias iguais — a rotação de matiz de um cinza é identidade visual; é o
/// caso degenerado que o gate evita usando uma base saturada.
#[must_use]
pub fn partners(base: ColorValue, scheme: Harmony) -> Vec<ColorValue> {
    let (h, s, v, a) = rgba_to_hsv(base.rgba);
    let mut out = vec![base];
    if scheme == Harmony::Monochromatic {
        // Mesmo matiz, uma escada de valor: mais escuro, mais escuro, e um mais claro/lavado.
        // Cobre a faixa tonal que uma paleta monocromática quer, ancorada no valor da base.
        for &(vmul, smul) in &[(0.66_f32, 1.0_f32), (0.33, 1.0), (1.0, 0.5)] {
            let rgba = hsv_to_rgba8(h, (s * smul).min(1.0), (v * vmul).min(1.0), a);
            out.push(ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]));
        }
        return out;
    }
    for &off in scheme.hue_offsets() {
        let h2 = (h + off / 360.0).rem_euclid(1.0);
        let rgba = hsv_to_rgba8(h2, s, v, a);
        out.push(ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]));
    }
    out
}

/// Altura do seletor de esquema (uma linha de segmentos).
pub const HARMONY_SEL_H: f32 = 24.0;
/// Altura da tira de swatches das parceiras.
pub const HARMONY_SWATCH_H: f32 = 22.0;

/// **Pinta a seção Color Harmonies** — o seletor de 7 esquemas + (quando um esquema está ativo) a
/// tira de swatches das parceiras derivadas + um botão "+" (add ao palette). Registra as hit-rects
/// de tudo que desenha. Devolve a ALTURA consumida (o layout avança `y` por ela).
///
/// As parceiras saem da MESMA [`partners`] que o dispatch consome (o clique num swatch pede a ela de
/// novo) — uma porta, então o que o artista vê é o que o clique pega.
#[allow(clippy::too_many_arguments)]
pub fn paint_harmony_section(
    cp: &BlenderColorPicker,
    rect: Rect,
    ids: &BlenderSubIds,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let sel_rect = Rect::new(rect.x, rect.y, rect.w, HARMONY_SEL_H);
    let group = RadioGroup::new(
        NodeId(0),
        "Harmony",
        Harmony::ALL
            .iter()
            .map(|h| RadioOption::new(NodeId(0), h.key(), h.label()))
            .collect(),
    )
    .orientation(RadioOrientation::Segmented)
    .selected(cp.harmony.key());
    paint_radio_group_with_labels(&group, sel_rect, scene, text_system, theme);
    #[allow(clippy::cast_precision_loss)]
    let seg_w = sel_rect.w / Harmony::ALL.len() as f32;
    for (i, id) in ids.harmony_schemes.iter().enumerate() {
        if id.0 != 0 {
            #[allow(clippy::cast_precision_loss)]
            let x = sel_rect.x + seg_w * i as f32;
            hit_index.register(*id, Rect::new(x, sel_rect.y, seg_w, sel_rect.h));
        }
    }
    let mut used = HARMONY_SEL_H;

    // Parceiras só quando um esquema está ativo (Off = a base sozinha, sem tira).
    if cp.harmony != Harmony::None {
        let sw_y = rect.y + HARMONY_SEL_H + Spacing::Xs.px();
        let colors = partners(cp.value, cp.harmony);
        let radius = Radius::Sm.px();
        let gap = 4.0_f32;
        // Reserva um "+" ao fim (add ao palette); as swatches dividem o resto.
        let add_w = HARMONY_SWATCH_H;
        let strip_w = (rect.w - add_w - gap).max(0.0);
        #[allow(clippy::cast_precision_loss)]
        let n = colors.len().max(1) as f32;
        let cell = ((strip_w - gap * (n - 1.0)) / n).max(1.0);
        for (i, c) in colors.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let sx = rect.x + (cell + gap) * i as f32;
            let sr = Rect::new(sx, sw_y, cell, HARMONY_SWATCH_H);
            let [r, g, b, a] = c.rgba;
            fill_rounded_rect(
                scene,
                sr,
                radius,
                ph2d_vector::Color::from_rgba8(r, g, b, a),
            );
            stroke_rounded_rect(scene, sr, radius, 1.0, resolve(ColorToken::Border, theme));
            if let Some(id) = ids.harmony_swatches.get(i).filter(|id| id.0 != 0) {
                hit_index.register(*id, sr);
            }
        }
        let add_rect = Rect::new(rect.x + rect.w - add_w, sw_y, add_w, HARMONY_SWATCH_H);
        fill_rounded_rect(scene, add_rect, radius, resolve(ColorToken::Bg2, theme));
        stroke_rounded_rect(
            scene,
            add_rect,
            radius,
            1.0,
            resolve(ColorToken::Border, theme),
        );
        paint_text_centered(
            text_system,
            scene,
            "+",
            add_rect,
            TypeToken::Sm.px(),
            resolve(ColorToken::Text2, theme),
        );
        if ids.harmony_add.0 != 0 {
            hit_index.register(ids.harmony_add, add_rect);
        }
        used += Spacing::Xs.px() + HARMONY_SWATCH_H;
    }
    used
}

#[cfg(test)]
#[path = "harmony_tests.rs"]
mod tests;
