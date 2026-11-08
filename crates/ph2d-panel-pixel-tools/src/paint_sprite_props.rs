//! **Sprite ▸ Properties…** (P12 U, O17): o que o Pixel sabe do sprite (tamanho, modo de cor,
//! frames, camadas), só de leitura, e o cartão User Data do sprite atrás do botão `User Data`.

use super::*;
use ph2d_editor_core::panel::cards::user_data;
use ph2d_editor_core::panel::cards::{Cell, Widget};
use ph2d_editor_core::pixel_panel::{NewSpriteMode, sprite_props};

impl Corpo<'_> {
    /// O botão `Properties…` (o 1.º do menu Sprite) e, se ele a abriu, a secção.
    pub(super) fn propriedades_do_sprite(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let [.., (props, _, rotulo)] = ids::SPRITE_ACTIONS;
        let y = self.botoes(&[(props, rotulo)], y);
        if !sprite_props::is_open() {
            return y;
        }
        let (w, h) = info.size;
        let tamanho = tr_with(
            "panel.pixel.sprite_props.size_value",
            &[("w", &w), ("h", &h)],
        );
        let modo = tr(match info.mode {
            NewSpriteMode::Rgba => "panel.pixel.sprite_props.rgba",
            NewSpriteMode::Gray => "panel.pixel.sprite_props.gray",
            NewSpriteMode::Indexed => "panel.pixel.sprite_props.indexed",
        });
        let (frames, camadas) = (info.frames.len().to_string(), info.layers.len().to_string());
        let titulo = tr("panel.pixel.sprite_props.title");
        let linhas = [
            ("panel.pixel.sprite_props.size", tamanho.as_str()),
            ("panel.pixel.sprite_props.mode", modo),
            ("panel.pixel.sprite_props.frames", frames.as_str()),
            ("panel.pixel.sprite_props.layers", camadas.as_str()),
        ];
        let ud = crate::seam_user::ids_sprite();
        let y = self.topo(y);
        let Some(mut c) = self.cartao_ctx() else {
            let mut y = self.linha(titulo, ColorToken::Text3, y);
            for (k, v) in linhas {
                let t = tr_with("panel.pixel.sprite_props.line", &[("k", &tr(k)), ("v", &v)]);
                y = self.linha(&t, ColorToken::Text2, y);
            }
            return y;
        };
        let mut y = match &info.user.sprite {
            Some(_) => user_data::title_row(&mut c, &ud, titulo, y),
            None => c.row(&[Cell::new(Widget::Readout { text: titulo })], y),
        };
        for par in linhas.chunks(2) {
            let celulas: Vec<Cell<'_>> = par
                .iter()
                .map(|&(k, v)| Cell::new(Widget::Readout { text: v }).label(tr(k)))
                .collect();
            y = c.row(&celulas, y);
        }
        match &info.user.sprite {
            Some(v) => user_data::open_body(&mut c, &ud, v, y),
            None => y,
        }
    }
}
