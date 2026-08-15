//! [`SectionHeader`] — single-line uppercase header for inspector
//! sections.
//!
//! Layout (left → right): optional collapse chevron · accent dot ·
//! label uppercase · optional count chip on the far right.
//! Used by the editor Inspector to break a body into "Params (12)",
//! "Advanced (7)", "Inputs (24)" etc.

mod fold;
use fold::{paint_chevron, plate_color};

use crate::paint::{fill_rounded_rect, paint_text_centered, paint_text_title, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, ICON_BTN_SIZE_PX, Radius, SECTION_GAP_PX, Spacing, Theme, TypeToken,
};
use ph2d_vector::{Affine, Brush, Circle, Color as VelloColor, Fill, Point, VectorScene};

#[derive(Clone, Debug)]
pub struct SectionHeader {
    pub id: NodeId,
    pub label: String,
    pub count: Option<u32>,
    /// When `Some(open)`, paints a chevron that flips on open/closed.
    /// `None` means the section is non-collapsible.
    pub collapsible: Option<bool>,
    /// O `t` VIVO da dobra; `None` = o binário de sempre. Ver [`SectionHeader::open_t`].
    pub open_t: Option<f32>,
    /// Right-edge color circle (RGBA bytes). When `Some`, replaces
    /// the count chip — the user can click it to open the global
    /// color picker for the section. When `None`, falls back to
    /// the count chip (or nothing).
    pub color: Option<[u8; 4]>,
}

impl SectionHeader {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            count: None,
            collapsible: None,
            open_t: None,
            color: None,
        }
    }

    pub fn count(mut self, n: u32) -> Self {
        self.count = Some(n);
        self
    }

    /// **Quanto desta secção está ABERTO** (`0.0` fechada … `1.0` aberta) — o `t` VIVO que o
    /// chevron veste, vindo de [`crate::interaction::WidgetStore::section_open_live`].
    ///
    /// ⚠️ **Omitir é o NEUTRO, não o zero:** sem esta chamada o chevron cai no binário de sempre
    /// (`is_open()` escolhe entre os dois glifos), **byte a byte** como antes desta wave. É isso
    /// que torna a migração dos ~34 cabeçalhos segura de fazer aos poucos — um sítio esquecido
    /// fica discreto, nunca meio-animado.
    pub fn open_t(mut self, t: f32) -> Self {
        self.open_t = Some(t);
        self
    }

    pub fn collapsible(mut self, open: bool) -> Self {
        self.collapsible = Some(open);
        self
    }

    pub fn color(mut self, rgba: [u8; 4]) -> Self {
        self.color = Some(rgba);
        self
    }

    pub fn is_open(&self) -> bool {
        self.collapsible.unwrap_or(true)
    }

    /// O `t` que o chevron veste: o VIVO quando o chamador o passou, senão o **binário de sempre**.
    #[must_use]
    pub fn fold_t(&self) -> f32 {
        self.open_t
            .unwrap_or(if self.is_open() { 1.0 } else { 0.0 })
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::Label)
            .label(&self.label)
            .bounds(x, y, w, h);
        if self.collapsible.is_some() {
            builder = builder.focusable(true).action(Action::Click);
        }
        builder.build()
    }
}

/// Canonical section-header chrome (Inspector + every collapsible
/// panel uses this). Layout (left → right): collapse chevron + label
/// in UPPERCASE + optional count pill on the far right. The previous
/// accent-dot ornament was retired — every section is collapsible by
/// design, so the chevron itself is the only "anchor" glyph.
pub fn paint_section_header(
    header: &SectionHeader,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // Collapsed sections get a darker background plate so the
    // visual hierarchy (open = transparent on panel; closed =
    // tinted) reads as "this block is folded". Centralized here so
    // every panel using `SectionHeader` gets the same affordance.
    // ⚠️ **A placa DESVANECE com o mesmo `t` do chevron, e não é enfeite:** sem isto ela trocava
    //    no instante SEMÂNTICO enquanto a seta desliza, e o cabeçalho ficava com duas metades a
    //    discordar sobre quando a secção fechou — precisamente a classe de defeito que esta wave
    //    existe para remover, um sistema acima. Nos dois extremos é byte-idêntica: `t = 0` pinta
    //    a placa cheia de hoje, `t = 1` não pinta nada.
    // ⚠️ **Quem gateia é o `t`, NUNCA o flag semântico** — e a primeira versão desta linha errava
    //    numa direcção só. Com `matches!(collapsible, Some(false))` a placa desvanecia a FECHAR
    //    (o flag já virou, o `t` ainda desce) e **sumia de repente a ABRIR** (o flag vira para
    //    `Some(true)` no clique e a placa deixa de ser pintada nesse quadro). Um efeito que só é
    //    simétrico num sentido é um efeito que ninguém escreveu.
    if let Some(plate) = plate_color(header, theme) {
        fill_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            crate::paint::token_to_vello(plate),
        );
    }
    let pad_x = Spacing::Md.px();
    let mut cursor_x = rect.x + pad_x;
    let icon_w = (rect.h * 0.7).clamp(12.0, 18.0); // LITERAL-PX-OK: section header chev icon size scales 70% of row height with min/max

    // Collapse chevron — always painted now; non-collapsible headers
    // simply default to "open" via `is_open()`. Single visual anchor
    // for the "this is a section title, click me to fold" affordance.
    let chev_rect = Rect::new(cursor_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
    paint_chevron(
        scene,
        chev_rect,
        header.fold_t(),
        resolve(ColorToken::Text2, theme),
    );
    cursor_x += icon_w + Spacing::Sm.px();

    // Label in UPPERCASE, painted at SEMI_BOLD (600) to match panel
    // titles — user feedback 2026-05-24 ("quase negrito"). Parley
    // supports the InterVariable axis (text/system.rs `layout_with_weight`),
    // so the editor stack DOES expose weights now (the older
    // text/lib.rs comment "no italics, no weight" is stale).
    let font = TypeToken::Sm.px();
    let label_y = rect.y + (rect.h - font) * 0.5;
    let label_w = if header.count.is_some() {
        (rect.x + rect.w - cursor_x - Spacing::Xl4.px() - pad_x).max(0.0)
    } else {
        (rect.x + rect.w - cursor_x - pad_x).max(0.0)
    };
    let upper = header.label.to_uppercase();
    paint_text_title(
        text_system,
        scene,
        &upper,
        cursor_x,
        label_y,
        font,
        label_w,
        resolve(ColorToken::Text1, theme),
    );

    // Right-edge ornament. Priority: color circle > count chip.
    // The color circle is the user-clickable "open the picker for
    // this section" affordance; the count chip is legacy and only
    // shown when no color is configured.
    if let Some(rgba) = header.color {
        let radius_px = 7.0_f32; // LITERAL-PX-OK: section header color circle radius (chrome-specific accent)
        let cx = rect.x + rect.w - pad_x - radius_px;
        let cy = rect.y + rect.h * 0.5;
        let circle = Circle::new(Point::new(cx as f64, cy as f64), radius_px as f64);
        let fill = VelloColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: user-color — `rgba` is the user's per-section accent, not a theme token
        scene.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &circle,
        );
        // 1-px ring so the circle still reads against same-colored
        // backgrounds (e.g. a light yellow circle on light theme).
        let ring = Circle::new(Point::new(cx as f64, cy as f64), (radius_px + 0.5) as f64);
        scene.inner_mut().stroke(
            &ph2d_vector::Stroke::new(1.0),
            Affine::IDENTITY,
            &Brush::Solid(resolve(ColorToken::Border, theme)),
            None,
            &ring,
        );
    } else if let Some(n) = header.count {
        let chip_w = ICON_BTN_SIZE_PX;
        let chip_h = (rect.h - Spacing::Xs.px()).max(SECTION_GAP_PX);
        let chip_rect = Rect::new(
            rect.x + rect.w - pad_x - chip_w,
            rect.y + (rect.h - chip_h) * 0.5,
            chip_w,
            chip_h,
        );
        fill_rounded_rect(
            scene,
            chip_rect,
            Radius::Xs.px(),
            resolve(ColorToken::Bg3, theme),
        );
        let text = n.to_string();
        paint_text_centered(
            text_system,
            scene,
            &text,
            chip_rect,
            TypeToken::Xs.px(),
            resolve(ColorToken::Text3, theme),
        );
    }
}

/// Rect of the color-circle hit zone in screen coordinates. Hosts
/// register this rect for hit-testing the "open picker for this
/// section" gesture. Returns `None` for headers that have no color
/// circle (the painter falls back to the count chip / no ornament).
pub fn color_circle_hit_rect(header: &SectionHeader, host: Rect) -> Option<Rect> {
    header.color?;
    let pad_x = Spacing::Md.px();
    let radius_px = 7.0_f32; // LITERAL-PX-OK: hit-zone radius mirrors paint geometry
    let size = radius_px * 2.0 + Spacing::Xs.px();
    Some(Rect::new(
        host.x + host.w - pad_x - size,
        host.y + (host.h - size) * 0.5,
        size,
        size,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O NEUTRO do `t` é o BINÁRIO de hoje** — um cabeçalho que ninguém migrou pinta exactamente
    /// o que pintava. *Mutação: `fold_t` a cair em `0.0` ⇒ toda secção aberta não-migrada desenha
    /// a seta fechada.*
    #[test]
    fn an_unmigrated_header_keeps_the_binary_chevron() {
        let open = SectionHeader::new(NodeId(1), "x").collapsible(true);
        let shut = SectionHeader::new(NodeId(1), "x").collapsible(false);
        assert!((open.fold_t() - 1.0).abs() < f32::EPSILON);
        assert!(shut.fold_t().abs() < f32::EPSILON);
        // e o `t` VIVO vence quando o chamador o passa
        assert!((open.open_t(0.25).fold_t() - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn defaults_no_count_no_collapsible() {
        let h = SectionHeader::new(NodeId(1), "Params");
        assert!(h.count.is_none());
        assert!(h.collapsible.is_none());
        assert!(h.is_open(), "non-collapsible defaults to open");
    }

    #[test]
    fn paint_emits_no_panic_for_uppercase_label_and_chevron() {
        // Smoke: header with all the new chrome bits (chevron +
        // uppercase + count chip) renders without geometry asserts.
        let h = SectionHeader::new(NodeId(1), "vector")
            .count(3)
            .collapsible(true);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_section_header(
            &h,
            Rect::new(0.0, 0.0, 280.0, 28.0),
            &mut scene,
            &mut text,
            Theme::Forge,
        );
    }

    #[test]
    fn count_setter_round_trips() {
        let h = SectionHeader::new(NodeId(1), "x").count(12);
        assert_eq!(h.count, Some(12));
    }

    #[test]
    fn collapsible_open_false() {
        let h = SectionHeader::new(NodeId(1), "x").collapsible(false);
        assert!(!h.is_open());
    }

    #[test]
    fn a11y_role_is_label() {
        let node = SectionHeader::new(NodeId(1), "Params").build_a11y(0.0, 0.0, 280.0, 24.0);
        assert_eq!(node.role(), Role::Label);
    }

    #[test]
    fn a11y_collapsible_supports_click() {
        let node = SectionHeader::new(NodeId(1), "x")
            .collapsible(false)
            .build_a11y(0.0, 0.0, 280.0, 24.0);
        assert!(node.supports_action(Action::Click));
    }

    fn smoke(h: SectionHeader, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_section_header(
            &h,
            Rect::new(0.0, 0.0, 280.0, 24.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_plain() {
        smoke(SectionHeader::new(NodeId(1), "Params"), Theme::Forge);
    }

    #[test]
    fn paint_smoke_with_count() {
        smoke(
            SectionHeader::new(NodeId(1), "Params").count(12),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_collapsible_closed() {
        smoke(
            SectionHeader::new(NodeId(1), "Advanced")
                .count(7)
                .collapsible(false),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_collapsible_open() {
        smoke(
            SectionHeader::new(NodeId(1), "Inputs")
                .count(24)
                .collapsible(true),
            Theme::Workshop,
        );
    }
}
