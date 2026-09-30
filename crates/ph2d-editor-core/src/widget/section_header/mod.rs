//! [`SectionHeader`] — the single-line section title of every panel.
//!
//! Layout (left → right): collapse chevron · label (as written, the *Grid Settings* title style —
//! owner's order of 2026-09-24) · optional count chip, or the **drag grip** on the far right.
//!
//! ⛔ **Two things LEFT this header on 2026-09-29, by order of the owner** (*«Temos uma linha
//! separadora nos títulos das seções. Vamos retirá-la»* · *«um círculo cuja única função é dar
//! cor ao círculo. Vamos retirar isso»*): the accent rule that ran to the right of the name, and
//! the colour circle. The circle's slot is now the [`grip`] — the Blender panel grip, which drags
//! the section to a new place — and the per-section look it pretended to give is now a real one:
//! a right-click on the title picks the THEME the section is painted with.
//! Used by the editor Inspector to break a body into "Params (12)",
//! "Advanced (7)", "Inputs (24)" etc.

pub mod body;
mod fold;
pub mod grip;
use fold::{paint_chevron, plate_color};

use crate::paint::{fill_rounded_rect, paint_text_centered, paint_text_title, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, ICON_BTN_SIZE_PX, INLINE_ICON_PX, Radius, Spacing, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

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
    /// ⭐ **A pega de arrasto** na ponta direita — `Some(lit)` quando esta secção se REORDENA
    /// (o `lit` acende-a sob o rato ou durante o arrasto). Substitui a pastilha de contagem.
    ///
    /// ⚠️ **`None` num painel que não sabe reordenar é a decisão, não um esquecimento:** uma pega
    /// pintada onde nenhum despacho a lê seria um controlo MORTO com cara de controlo.
    pub grip: Option<bool>,
    /// Pixels a mais reservados à direita do título para um controlo que o CHAMADOR pinta ali (o
    /// botão de repor da Transform, do Pincel) — sem isto um título comprido corria por baixo dele.
    pub reserve_right: f32,
}

impl SectionHeader {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            count: None,
            collapsible: None,
            open_t: None,
            grip: None,
            reserve_right: 0.0,
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

    /// Reserva `px` à direita do título para um controlo que o chamador pinta ali.
    pub fn reserve_right(mut self, px: f32) -> Self {
        self.reserve_right = px.max(0.0);
        self
    }

    /// Pinta a pega de arrasto; `lit` acende-a (rato por cima ou arrasto em curso).
    pub fn grip(mut self, lit: bool) -> Self {
        self.grip = Some(lit);
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

/// ⭐ **O corpo do título de secção** — uma porta só, lida pelo pintor e por quem precisa de saber
/// que altura o título pede (o `Md`, o do título do painel *Grid Settings*: ordem do dono de
/// 2026-09-24).
#[must_use]
pub fn section_title_px() -> f32 {
    TypeToken::Md.px()
}

/// Canonical section-header chrome (Inspector + every collapsible
/// panel uses this). Layout (left → right): collapse chevron + label
/// (as written) + the drag grip or the count pill on the far right. The previous
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
    // ⭐⭐ **Dentro de um CARTÃO a placa não é pintada: o cartão JÁ É a placa** (report do dono,
    //    2026-09-29, com o Blender ao lado: cada secção, aberta ou fechada, é UM cartão). Pintá-la
    //    punha um segundo cartão dentro do primeiro, que é o que tornava tudo *«mal definido»*.
    //    ⚠️ Fora de um cartão (o tema clássico, e os painéis que não abrem o livro) ela fica: ali é
    //    o único sinal de que a secção está dobrada.
    if let Some(plate) =
        plate_color(header, theme).filter(|_| !crate::widget::section_cards::inside_cards())
    {
        // ⭐ O raio da placa é o do cromo do TEMA (`Radius::Sm` no clássico, `4` no moderno).
        fill_rounded_rect(
            scene,
            rect,
            ph2d_tokens::visuals::Chrome::of(theme).plate_radius,
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
    cursor_x += icon_w + ph2d_tokens::icon_label_gap_px();

    // ⭐⭐ **O TÍTULO DE SECÇÃO DO APP É O DO GRID** (ordem do dono, 2026-09-24, com foto do painel
    //    *Grid Settings*: *«quero que essa seja a formatação exata (Font, tamanho da Font, etc) para
    //    todo o APP»*). ⇒ o corpo [`TypeToken::Md`], o peso `SemiBold` do [`paint_text_title`], a
    //    cor `Text1` e o texto **como está escrito na tabela** — ⛔ não mais em CAIXA ALTA (o `Sm` +
    //    `to_uppercase` de 2026-05-24 saiu nesse dia).
    let font = section_title_px();
    let label_y = rect.y + (rect.h - font) * 0.5;
    // ⚠️ O fim do espaço do título é o início do ornamento da direita (a pega ou a pastilha de
    //    contagem), quando há um.
    let right = if header.grip.is_some() {
        rect.x + rect.w - pad_x - grip::grip_w_px() - ph2d_tokens::icon_label_gap_px()
    } else if header.count.is_some() {
        rect.x + rect.w - pad_x - Spacing::Xl4.px()
    } else {
        rect.x + rect.w - pad_x
    } - header.reserve_right;
    let label_w = (right - cursor_x).max(0.0);
    paint_text_title(
        text_system,
        scene,
        &header.label,
        cursor_x,
        label_y,
        font,
        label_w,
        resolve(ColorToken::Text1, theme),
    );
    // Right-edge ornament. Priority: the drag grip > the count chip.
    if let Some(lit) = header.grip {
        let tone = if lit {
            ColorToken::Text1
        } else {
            ColorToken::Text3
        };
        grip::paint_grip(scene, grip::grip_rect(rect), resolve(tone, theme));
    } else if let Some(n) = header.count {
        let chip_w = ICON_BTN_SIZE_PX;
        let chip_h = (rect.h - Spacing::Xs.px()).max(INLINE_ICON_PX);
        let chip_rect = Rect::new(
            rect.x + rect.w - pad_x - chip_w,
            rect.y + (rect.h - chip_h) * 0.5,
            chip_w,
            chip_h,
        );
        fill_rounded_rect(
            scene,
            chip_rect,
            crate::paint::frame_radius(theme, Radius::Xs.px()),
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

/// O rect que a pega regista no hit-index — a banda da direita do cabeçalho, da altura dele.
/// `None` quando o cabeçalho não tem pega. Ver [`grip::grip_hit_rect`].
pub fn grip_hit_rect(header: &SectionHeader, host: Rect) -> Option<Rect> {
    header.grip?;
    Some(grip::grip_hit_rect(host))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **O corpo do título é o do *Grid Settings*** (`TypeToken::Md`), por ordem do dono.
    #[test]
    fn o_titulo_de_seccao_tem_o_corpo_do_grid() {
        assert!((section_title_px() - TypeToken::Md.px()).abs() < f32::EPSILON);
    }

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
