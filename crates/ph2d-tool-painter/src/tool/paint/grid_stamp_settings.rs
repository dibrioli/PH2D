//! **Grid Stamp** — os controles do método: o tamanho da célula, o deslocamento da grade e o
//! interruptor que a desenha.
//!
//! Os dois primeiros vivem no [`ph2d_painter_brush::BrushSpec`] (é o que decide ONDE o próximo dab
//! cai — o mesmo tipo de coisa que o Spacing); o terceiro vive no `PaintState`, porque **desenhar a
//! grade é EXIBIÇÃO** e não uma propriedade do pincel — o precedente é o `impasto_show`, que também
//! liga uma coisa que se vê sem mudar um byte do que se pinta.
//!
//! ⚠️ **A régua norm↔px é a MESMA do tamanho do pincel** ([`super::brush_settings::size_norm_to_px`]):
//! uma célula é uma medida em pixels de imagem, e dar-lhe uma segunda curva de slider faria o mesmo
//! gesto (arrastar até o meio) significar tamanhos diferentes em dois lugares que o artista lê como
//! irmãos. Ela é quadrática, então as células pequenas — que são as que se usam — ficam com a maior
//! parte do curso.

use super::brush_settings::size_norm_to_px;
use crate::tool::PainterTool;

/// O eixo de um controle de grade. Nomeado em vez de um `bool` ou um `usize` solto: `set(0, v)` e
/// `set(true, v)` são os dois jeitos de escrever no eixo errado sem o compilador reclamar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridAxis {
    /// Horizontal (largura da célula · deslocamento em X).
    X,
    /// Vertical (altura da célula · deslocamento em Y).
    Y,
}

impl GridAxis {
    const fn idx(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
        }
    }
}

impl PainterTool {
    /// O tamanho da célula num eixo, em px de imagem.
    pub fn set_grid_cell_px(&mut self, axis: GridAxis, px: f32) {
        let v = if px.is_finite() {
            px.clamp(
                ph2d_painter_brush::grid_stamp::GRID_CELL_MIN_PX,
                super::brush_ranges::BRUSH_SIZE_MAX_PX,
            )
        } else {
            ph2d_painter_brush::grid_stamp::GRID_CELL_MIN_PX
        };
        self.paint.brush.grid_cell_px[axis.idx()] = v;
    }

    /// O tamanho da célula a partir do curso `0..1` do slider.
    pub fn set_grid_cell_norm(&mut self, axis: GridAxis, t: f32) {
        self.set_grid_cell_px(axis, size_norm_to_px(t));
    }

    /// O deslocamento da grade num eixo, em px de imagem.
    ///
    /// ⚠️ Ele é **periódico na célula** — deslocar por uma célula inteira devolve a mesma grade —,
    /// então a faixa não precisa de valores negativos: todo deslocamento possível já está em
    /// `[0, célula)`. O slider vai além disso de propósito, para o artista poder afinar sem trocar de
    /// unidade quando muda o tamanho da célula.
    pub fn set_grid_offset_px(&mut self, axis: GridAxis, px: f32) {
        let v = if px.is_finite() {
            px.clamp(0.0, super::brush_ranges::BRUSH_SIZE_MAX_PX)
        } else {
            0.0
        };
        self.paint.brush.grid_offset_px[axis.idx()] = v;
    }

    /// O deslocamento a partir do curso `0..1` do slider. ⚠️ O zero do curso é o zero de px — a régua
    /// do tamanho começa em 1 px (uma célula de zero não existe), e um **deslocamento** de zero é o
    /// caso normal, não um degenerado.
    pub fn set_grid_offset_norm(&mut self, axis: GridAxis, t: f32) {
        let max = super::brush_ranges::BRUSH_SIZE_MAX_PX;
        let t = t.clamp(0.0, 1.0);
        self.set_grid_offset_px(axis, t * t * max);
    }

    /// Liga/desliga o desenho da grade. **Só desenho** — o carimbo pousa nas mesmas células com ela
    /// visível ou não, e há gate afirmando isso.
    pub fn toggle_grid_show(&mut self) {
        self.paint.grid_show = !self.paint.grid_show;
    }

    /// A grade está sendo desenhada?
    #[must_use]
    pub fn grid_show(&self) -> bool {
        self.paint.grid_show
    }

    /// O curso `0..1` que representa um tamanho de célula — a inversa de [`Self::set_grid_cell_norm`],
    /// para o painel desenhar o thumb onde o valor de fato está.
    #[must_use]
    pub fn grid_cell_norm(&self, axis: GridAxis) -> f32 {
        let min = ph2d_painter_brush::grid_stamp::GRID_CELL_MIN_PX;
        let max = super::brush_ranges::BRUSH_SIZE_MAX_PX;
        let px = self.paint.brush.grid_cell_px[axis.idx()];
        (((px - min) / (max - min)).max(0.0)).sqrt().clamp(0.0, 1.0)
    }

    /// O curso `0..1` que representa um deslocamento — a inversa de [`Self::set_grid_offset_norm`].
    #[must_use]
    pub fn grid_offset_norm(&self, axis: GridAxis) -> f32 {
        let max = super::brush_ranges::BRUSH_SIZE_MAX_PX;
        let px = self.paint.brush.grid_offset_px[axis.idx()];
        ((px / max).max(0.0)).sqrt().clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_painter_brush::grid_stamp::GRID_CELL_MIN_PX;

    /// **A régua e a sua inversa têm de fechar.** O painel desenha o thumb por `grid_cell_norm` e o
    /// arrasto escreve por `set_grid_cell_norm`; se as duas discordarem o thumb salta ao ser tocado —
    /// a doença do *seed ≠ sample* que esta base já pagou várias vezes.
    #[test]
    fn the_cell_track_and_its_inverse_close() {
        let mut t = PainterTool::default();
        for axis in [GridAxis::X, GridAxis::Y] {
            for i in 0..=20 {
                let track = i as f32 / 20.0;
                t.set_grid_cell_norm(axis, track);
                let back = t.grid_cell_norm(axis);
                assert!(
                    (back - track).abs() < 1e-3,
                    "cell track {track} voltou {back} (eixo {axis:?})"
                );
            }
        }
    }

    /// O mesmo para o deslocamento — inclusive no zero, que é o caso NORMAL dele (e não um
    /// degenerado, ao contrário de uma célula de tamanho zero).
    #[test]
    fn the_offset_track_and_its_inverse_close_including_at_zero() {
        let mut t = PainterTool::default();
        for axis in [GridAxis::X, GridAxis::Y] {
            for i in 0..=20 {
                let track = i as f32 / 20.0;
                t.set_grid_offset_norm(axis, track);
                assert!((t.grid_offset_norm(axis) - track).abs() < 1e-3);
            }
        }
        t.set_grid_offset_norm(GridAxis::X, 0.0);
        assert_eq!(t.paint.brush.grid_offset_px[0], 0.0, "o zero é alcançável");
    }

    /// Os eixos são **independentes**: escrever em X não pode mexer em Y. É o defeito que um
    /// `idx()` trocado produz, e ele é mudo — a célula fica quadrada quando devia ser retangular.
    #[test]
    fn the_two_axes_are_independent() {
        let mut t = PainterTool::default();
        t.set_grid_cell_px(GridAxis::X, 64.0);
        t.set_grid_cell_px(GridAxis::Y, 16.0);
        assert_eq!(t.paint.brush.grid_cell_px, [64.0, 16.0]);
        t.set_grid_offset_px(GridAxis::Y, 7.0);
        assert_eq!(t.paint.brush.grid_offset_px, [0.0, 7.0]);
        assert_eq!(
            t.paint.brush.grid_cell_px,
            [64.0, 16.0],
            "mexer no offset não move a célula"
        );
    }

    /// Uma entrada degenerada não produz uma grade degenerada: o piso é um pixel, e um NaN não
    /// atravessa para o `floor()` que decide o índice da célula.
    #[test]
    fn a_degenerate_input_never_reaches_the_grid() {
        let mut t = PainterTool::default();
        for bad in [0.0, -50.0, f32::NAN, f32::INFINITY] {
            t.set_grid_cell_px(GridAxis::X, bad);
            assert!(t.paint.brush.grid_cell_px[0] >= GRID_CELL_MIN_PX, "{bad}");
            t.set_grid_offset_px(GridAxis::Y, bad);
            assert!(t.paint.brush.grid_offset_px[1].is_finite(), "{bad}");
        }
    }

    /// **Show Grid é só DESENHO.** Ligá-lo e desligá-lo não pode mover um texel de tinta — senão a
    /// grade deixaria de ser uma ajuda de visão e viraria um parâmetro escondido do carimbo.
    #[test]
    fn show_grid_is_display_only() {
        assert!(
            !PainterTool::default().grid_show(),
            "default: a grade nasce apagada até o método a pedir"
        );
        let mut t = PainterTool::default();
        let before = t.paint.brush;
        t.toggle_grid_show();
        assert!(t.grid_show());
        assert_eq!(
            t.paint.brush, before,
            "o interruptor da grade tocou o pincel — ele é de exibição"
        );
    }
}
