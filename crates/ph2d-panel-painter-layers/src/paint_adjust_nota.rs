//! A frase do relevo por baixo dos parâmetros de um ajuste (`docs/3D/30` §20) — irmã
//! (`#[path]`) do `paint_adjust.rs` (o tecto de linhas dos painéis).

use ph2d_editor_core::panel::PaintCtx;
use ph2d_tool_painter::AdjustmentParams;

/// ⭐ **O Brilho e as Sombras/Realces não mexem no relevo — e o painel di-lo** (`docs/3D/30` §20):
/// são operações de TOM; o Gaussiano e a Nitidez borram e afiam também a espessura da tinta.
pub(super) fn paint_nota_do_relevo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    params: &AdjustmentParams,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if ph2d_tool_painter::relief_effect(params.kind()) != ph2d_tool_painter::ReliefEffect::Tone {
        return y;
    }
    y + ph2d_editor_core::paint::paint_text_block(
        ctx.text_system,
        ctx.scene,
        ph2d_i18n::tr("panel.painter_layers.adjust.tone_not_relief"),
        x,
        y,
        ph2d_tokens::TypeToken::Base.px(),
        w,
        ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::Text2, theme),
    ) + ph2d_tokens::control_gap_px()
}

#[cfg(test)]
mod nota_do_relevo_tests {
    use super::*;
    use ph2d_editor_core::screens::HeroLayout;
    use ph2d_editor_core::zones::Rect;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;
    use ph2d_tool_painter::AdjustmentKind;
    use ph2d_tool_painter::AdjustmentParams;
    use ph2d_ui_testkit::MockPanelHost;
    use ph2d_vector::VectorScene;

    /// Quanto a frase do relevo ocupa por baixo dos parâmetros deste tipo de ajuste.
    fn altura_da_nota(kind: AdjustmentKind) -> f32 {
        let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let viewport = Rect::new(0.0, 0.0, 360.0, 2000.0);
        let layout = HeroLayout::for_viewport(viewport);
        let mut ctx = PaintCtx {
            host: &mut host,
            layout: &layout,
            slot: layout
                .slot_rects(ph2d_editor_core::screens::slot::SlotSet::ANY_DOCK)
                .get(ph2d_editor_core::screens::slot::Slot::RightTop),
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        let params = AdjustmentParams::neutral_for(kind);
        paint_nota_do_relevo(&mut ctx, Theme::default(), &params, 0.0, 300.0, 100.0) - 100.0
    }

    /// ⭐ O Brilho e as Sombras/Realces dizem que o relevo não muda (`docs/3D/30` §20); o Gaussiano
    /// e a Nitidez, que o borram, não levam a frase; um ajuste por píxel também não.
    #[test]
    fn o_brilho_e_as_sombras_dizem_que_o_relevo_nao_muda() {
        for k in [AdjustmentKind::Bloom, AdjustmentKind::ShadowsHighlights] {
            assert!(
                altura_da_nota(k) > 0.0,
                "{k:?} não diz porquê o relevo fica"
            );
        }
        for k in [
            AdjustmentKind::GaussianBlur,
            AdjustmentKind::Sharpen,
            AdjustmentKind::BrightnessContrast,
        ] {
            assert_eq!(
                altura_da_nota(k),
                0.0,
                "{k:?} levou a frase dos efeitos de tom"
            );
        }
    }
}
