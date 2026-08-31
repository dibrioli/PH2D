//! `PaintCtx` — per-frame context handed to every panel's
//! `Panel::paint` impl.
//!
//! ADR-0029 §4.5. Differs from the pre-ADR `PaintCtx` (which lived
//! in `ph2d-editor::panel_registry` and held `&mut HeroScreen`
//! directly): now holds `&mut dyn PanelHostInternal` so panel
//! crates have zero static dep on the concrete host type.

use super::PanelHostInternal;
use crate::screens::HeroLayout;
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Per-frame context for panel paint thunks.
///
/// Borrow note: `host`, `scene`, and `text_system` are independent
/// `&mut`s; Rust's field-level borrow splitting handles disjoint
/// use within a single paint pass.
pub struct PaintCtx<'a> {
    /// Host (typically `HeroScreen`). Panels read/write store +
    /// theme + project + per-panel-state through this trait object.
    pub host: &'a mut dyn PanelHostInternal,
    /// Pre-computed sub-region rects — a área de desenho, as bandas do chrome, a divisão do
    /// centro. ⚠️ **Para saber ONDE ELE PRÓPRIO fica, um painel lê [`Self::slot`]**, não um campo
    /// com o nome de outro painel.
    pub layout: &'a HeroLayout,
    /// ⭐⭐⭐ **O RECT DO ENCAIXE DESTE PAINEL** — já sem a faixa de abas, se houver.
    ///
    /// > *«Lugares pré-definidos. O artista escolhe QUAL painel vai em cada lugar.»* — decisão D4
    ///
    /// ⛔⛔ **Isto substitui `layout.inspector`, `layout.hierarchy`, `layout.padding`,
    /// `layout.bgremoval`, `layout.painter_layers`, `layout.timeline` e `layout.flip_strip`**, que
    /// eram **oito nomes para a mesma pergunta** — *onde é que eu fico?* — cada um com o nome do
    /// painel que ali morava quando a posição era fixa.
    ///
    /// ⚠️⚠️ **E os campos substituídos têm de MORRER, não ficar por perto.** O `layout.flip_strip`
    /// sobreviveu à conversão como um rect que ninguém pintava — e a **reserva** da área de desenho
    /// continuou a lê-lo: ela reservava 132 px enquanto a tira ocupava 240, deixando
    /// **147 528 px²** de painel por cima do desenho durante uma entrega inteira (2026-08-31).
    /// *Um campo de geometria que perdeu o pintor ainda tem leitores, e todos eles passam a
    /// responder pelo sítio errado.*
    ///
    /// ⚠️ Enquanto a posição era uma constante isso lia-se bem. No dia em que o artista passa a
    /// **mover** um painel, um campo chamado `inspector` lido pelo painel de Física deixa de poder
    /// estar certo: mover a Física mudava quem a **contava** e não onde ela **pintava**.
    ///
    /// ⚠️ **Um painel que FLUTUA também o lê** — para ele é a posição de NASCIMENTO, e o encaixe
    /// que ele declara é a resposta mais significativa que existe («onde eu estaria se estivesse
    /// encaixado»).
    pub slot: Rect,
    /// Outer viewport — needed by floating-panel clamping math.
    pub viewport: Rect,
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
}
