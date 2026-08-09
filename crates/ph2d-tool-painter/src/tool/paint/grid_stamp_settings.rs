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

    /// The axis a per-axis id ARRAY slot names, or `None` for a slot that is not an axis. The panel's
    /// `PAINTER_BRUSH_GRID_*` ids are two-element arrays, so the router asks for the position and this
    /// turns it back into the named axis — the conversion happens **here** and not at each call site,
    /// which is what keeps `0`/`1` from being re-derived (and eventually re-derived wrong) per router.
    #[must_use]
    pub const fn from_slot(slot: usize) -> Option<Self> {
        match slot {
            0 => Some(Self::X),
            1 => Some(Self::Y),
            _ => None,
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

    /// O tamanho do carimbo em relação à célula, em `[-0.5, 0.5]` (`0` = enche a célula).
    pub fn set_grid_fit(&mut self, v: f32) {
        self.paint.brush.grid_fit = if v.is_finite() {
            v.clamp(
                ph2d_painter_brush::grid_stamp::GRID_FIT_MIN,
                ph2d_painter_brush::grid_stamp::GRID_FIT_MAX,
            )
        } else {
            0.0
        };
    }

    /// O tamanho do carimbo a partir do curso `0..1` do slider.
    ///
    /// ⚠️ A régua é **LINEAR**, ao contrário da do tamanho da célula: aquela é uma medida em pixels
    /// (quadrática, para as células pequenas ganharem a maior parte do curso), esta é uma **razão
    /// com sinal** cujo ponto interessante é o ZERO — e é o meio do curso que tem de cair nele, para
    /// "encher a célula" ser um lugar que a mão acha sem olhar o número.
    pub fn set_grid_fit_norm(&mut self, t: f32) {
        let (lo, hi) = (
            ph2d_painter_brush::grid_stamp::GRID_FIT_MIN,
            ph2d_painter_brush::grid_stamp::GRID_FIT_MAX,
        );
        self.set_grid_fit(lo + t.clamp(0.0, 1.0) * (hi - lo));
    }

    /// O curso `0..1` que representa o tamanho vigente — a inversa de [`Self::set_grid_fit_norm`].
    #[must_use]
    pub fn grid_fit_norm(&self) -> f32 {
        let (lo, hi) = (
            ph2d_painter_brush::grid_stamp::GRID_FIT_MIN,
            ph2d_painter_brush::grid_stamp::GRID_FIT_MAX,
        );
        ((self.paint.brush.grid_fit - lo) / (hi - lo)).clamp(0.0, 1.0)
    }

    /// **O spec com que um traço corre** — para o Grid Stamp, o frame da CÉLULA já resolvido.
    ///
    /// ⚠️ Existe porque `has_shape` só é conhecido AQUI (os pixels do Shape vivem no tool) e é
    /// preciso em DOIS lugares que não podem discordar: o motor, que emite o `radius_px` de cada dab
    /// ao abrir o traço, e o carimbo, que estica a silhueta até as bordas da célula. Resolver uma
    /// vez e entregar o mesmo spec aos dois é o que impede a tinta de encostar num lado e sobrar do
    /// outro; [`ph2d_painter_brush::BrushSpec::as_grid_stamp`] é idempotente, então aplicá-la de novo
    /// no carimbo não é uma segunda resposta.
    #[must_use]
    pub(crate) fn stroke_spec(&self) -> ph2d_painter_brush::BrushSpec {
        let brush = self.paint.brush;
        if brush.stroke_method == ph2d_painter_brush::StrokeMethod::GridStamp {
            brush.as_grid_stamp(self.shape_silhouette_active())
        } else {
            brush
        }
    }

    /// A silhueta de **Shape** está de fato ativa (kind escolhido *e*, para `Image`, pixels
    /// carregados) — a porta que o `stroke_spec` e o carimbo perguntam.
    #[must_use]
    pub(crate) fn shape_silhouette_active(&self) -> bool {
        self.paint
            .brush
            .shape_silhouette_active(self.paint.shape_image.is_some())
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

    /// O gesto vigente do **Grid Stamp** apaga (botão direito). O shell arma antes de entregar o Down
    /// e desarma no Up — o [`ph2d_editor_core::tool::CanvasPaintTool`] é contrato congelado e o
    /// `CanvasPointer` dele não carrega botão, então o botão viaja fora de banda como os
    /// modificadores (o precedente exato do `set_line_constrain`).
    pub fn set_grid_stamp_erase(&mut self, on: bool) {
        self.paint.grid_erase = on;
    }

    /// Este método **carimba numa grade**? A pergunta que o shell faz antes de reivindicar um clique
    /// de botão direito para si: fora do Grid Stamp o secundário continua sendo do menu de contexto e
    /// dos consumidores que já existiam, e roubá-lo seria tirar comportamento em troca de nada.
    #[must_use]
    pub fn stamps_on_a_grid(&self) -> bool {
        self.paint.brush.stroke_method == ph2d_painter_brush::StrokeMethod::GridStamp
    }

    /// **O traço vigente APAGA?** A porta única da pergunta que o carimbo faz — a borracha AUTORADA
    /// (o chip da barra) ou o gesto de botão direito do Grid Stamp.
    ///
    /// ⚠️ Os outros leitores de `paint.eraser` ficam no campo AUTORADO de propósito: eles respondem
    /// *"que pincel é este?"* (o que o painel espelha, se o modo depõe relevo, o que um preset semeia)
    /// e não *"o que a mão está fazendo neste instante"*. Colapsar as duas faria o chip da borracha
    /// piscar durante um arrasto de botão direito.
    #[must_use]
    pub fn stroke_erases(&self) -> bool {
        self.paint.eraser || self.paint.grid_erase
    }

    /// **O retângulo da célula que cobre `p`** (px de imagem): `(centro, meia-extensão)`, ou `None`
    /// fora do Grid Stamp.
    ///
    /// É o que o CURSOR desenha: neste método o dab não pousa onde o ponteiro está, mas no centro da
    /// célula que ele cobre, então um anel preso ao ponteiro mostraria um lugar onde nada acontece.
    /// O encaixe não é um comportamento novo do cursor — ele já era a lei do carimbo; o que faltava
    /// era o desenho contá-la (Enio, 2026-08-09).
    ///
    /// ⚠️ **Sai das MESMAS portas do carimbo** ([`ph2d_painter_brush::BrushSpec::grid_cell_at`] +
    /// `grid_cell_center` + `grid_cell`), como as linhas da grade. Um cursor que re-derivasse a rede
    /// desenharia um encaixe que a tinta não honra — e o artista acredita no desenho.
    #[must_use]
    pub fn grid_cell_rect_at(&self, p: [f32; 2]) -> Option<([f32; 2], [f32; 2])> {
        if !self.stamps_on_a_grid() || !p[0].is_finite() || !p[1].is_finite() {
            return None;
        }
        let b = &self.paint.brush;
        let cell = b.grid_cell();
        Some((
            b.grid_cell_center(b.grid_cell_at(p)),
            [cell[0] * 0.5, cell[1] * 0.5],
        ))
    }

    /// As LINHAS da grade num eixo, dentro de `[lo, hi]` px de imagem: `(primeira, passo, quantas)`.
    /// Delegação pura para a porta do spec ([`ph2d_painter_brush::BrushSpec::grid_line_run`]) — é ela
    /// que o carimbo usa, e o overlay tem de desenhar a MESMA rede ou o artista acredita no desenho.
    #[must_use]
    pub fn grid_line_run(&self, axis: usize, lo: f32, hi: f32) -> (f32, f32, u32) {
        self.paint.brush.grid_line_run(axis, lo, hi)
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
    use ph2d_editor_core::tool::RasterEditTool;
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
    ///
    /// ⚠️ A asserção é sobre o **FLIP**, nunca sobre o valor que o toggle produz: escrita como
    /// `assert!(t.grid_show())` ela **passa a testar o default por acidente** e sangra no dia em que
    /// o default se move — que foi exatamente o que ela fez.
    #[test]
    fn show_grid_is_display_only() {
        let mut t = PainterTool::default();
        let before = t.paint.brush;
        let was = t.grid_show();
        t.toggle_grid_show();
        assert_eq!(t.grid_show(), !was, "o toggle não inverteu a grade");
        t.toggle_grid_show();
        assert_eq!(t.grid_show(), was, "o toggle não é a própria inversa");
        assert_eq!(
            t.paint.brush, before,
            "o interruptor da grade tocou o pincel — ele é de exibição"
        );
    }

    /// Um painter com uma tela branca opaca `size²`, o Grid Stamp escolhido e uma célula de `cell` px.
    fn grid_canvas(size: u32, cell: f32) -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::GridStamp.to_u8());
        t.set_grid_cell_px(GridAxis::X, cell);
        t.set_grid_cell_px(GridAxis::Y, cell);
        t.paint.brush.color = [0.0, 0.0, 0.0];
        t
    }

    fn tap(t: &mut PainterTool, pos: [f32; 2]) {
        use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
        for phase in [PointerPhase::Down, PointerPhase::Up] {
            t.on_canvas_pointer(CanvasPointer {
                pos,
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            });
        }
    }

    fn alpha_at(t: &PainterTool, size: u32, x: u32, y: u32) -> u8 {
        t.canvas_rgba[((y * size + x) * 4 + 3) as usize]
    }

    /// **O botão direito APAGA a célula que o esquerdo pintou.**
    ///
    /// As duas metades juntas de propósito, porque uma sozinha não diz nada: sem a primeira o "apagar"
    /// poderia estar apagando nada, e sem a segunda o flag poderia não fazer diferença nenhuma.
    ///
    /// Mutação que tem de sangrar: `stroke_erases` devolver só `self.paint.eraser`. O gesto de botão
    /// direito volta a PINTAR a célula — a falha exata que o artista veria.
    #[test]
    fn the_right_button_gesture_erases_the_cell_the_left_one_painted() {
        let size = 64;
        let mut t = grid_canvas(size, 32.0);
        tap(&mut t, [16.0, 16.0]);
        assert_eq!(
            alpha_at(&t, size, 16, 16),
            255,
            "o carimbo normal não pintou a célula — a fixture não contém o fenômeno"
        );
        t.set_grid_stamp_erase(true);
        tap(&mut t, [16.0, 16.0]);
        t.set_grid_stamp_erase(false);
        // ⚠️ A barra é 4, não 0, e o número é MEDIDO: o carimbo apagado deixa **1** de 255 no centro —
        // o resíduo de arredondamento de `1 − cobertura` em bytes, não tinta que sobrou. O fosso contra
        // o controle acima (255) é de duas ordens, então a barra não decide nada por si.
        let left = alpha_at(&t, size, 16, 16);
        assert!(
            left <= 4,
            "o gesto de apagar não removeu a tinta (sobrou {left}/255) — o carimbo não perguntou à \
             porta `stroke_erases`"
        );
    }

    /// **O gesto não mexe na borracha AUTORADA.** O chip da barra é a escolha do artista; um arrasto de
    /// botão direito é o que a mão está fazendo agora. Colapsar as duas faria o chip piscar durante o
    /// arrasto e — pior — um gesto abandonado deixaria a borracha ligada por conta própria.
    #[test]
    fn the_gesture_answers_the_door_without_touching_the_authored_eraser() {
        let mut t = PainterTool::default();
        assert!(!t.stroke_erases(), "nada apaga em repouso");
        let authored = t.brush_settings().eraser;
        t.set_grid_stamp_erase(true);
        assert!(t.stroke_erases(), "a porta ignorou o gesto");
        assert_eq!(
            t.brush_settings().eraser,
            authored,
            "o gesto escreveu na borracha autorada — o chip do painel piscaria"
        );
        t.set_grid_stamp_erase(false);
        assert!(!t.stroke_erases(), "o gesto não foi desarmado");
        // E a borracha autorada sozinha continua abrindo a porta (a outra metade da união).
        t.toggle_brush_eraser();
        assert!(
            t.stroke_erases(),
            "a borracha autorada deixou de abrir a porta"
        );
    }

    /// **A pergunta que o shell faz antes de roubar o botão direito.** Fora do Grid Stamp o secundário
    /// continua sendo do menu de contexto e dos consumidores que já existiam.
    #[test]
    fn only_the_grid_stamp_claims_the_secondary_button() {
        let mut t = PainterTool::default();
        assert!(
            !t.stamps_on_a_grid(),
            "o método default não carimba em grade e não pode reivindicar o botão direito"
        );
        t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::GridStamp.to_u8());
        assert!(t.stamps_on_a_grid());
    }

    /// **A régua do Cell Fit e a inversa fecham**, e o MEIO do curso é o zero — é ele que o artista
    /// procura sem olhar o número ("encher a célula"), e uma régua que o deslocasse tornaria o caso
    /// normal um lugar que só se acha por tentativa.
    #[test]
    fn the_fit_track_and_its_inverse_close_and_the_centre_is_the_neutral() {
        let mut t = PainterTool::default();
        for i in 0..=20 {
            let track = i as f32 / 20.0;
            t.set_grid_fit_norm(track);
            assert!(
                (t.grid_fit_norm() - track).abs() < 1e-3,
                "fit track {track}"
            );
        }
        t.set_grid_fit_norm(0.5);
        assert_eq!(
            t.paint.brush.grid_fit, 0.0,
            "o meio do curso TEM de ser o neutro"
        );
        assert!((t.paint.brush.grid_fit_scale() - 1.0).abs() < 1e-6);
    }

    /// **O `stroke_spec` resolve o frame da célula, e SÓ para o Grid Stamp.**
    ///
    /// As duas metades juntas: sem a primeira o motor emite o raio do pincel (o carimbo encolhe para
    /// o tamanho do pincel); sem a segunda o spec de TODO método passaria a carregar um raio derivado
    /// de uma grade que ele não usa.
    ///
    /// **Mutação que tem de sangrar:** `stroke_spec` devolver `self.paint.brush` sempre.
    #[test]
    fn the_stroke_spec_resolves_the_cell_frame_only_for_the_grid_stamp() {
        let mut t = PainterTool::default();
        t.set_brush_size_px(3.0);
        t.set_grid_cell_px(GridAxis::X, 80.0);
        t.set_grid_cell_px(GridAxis::Y, 20.0);
        assert_eq!(
            t.stroke_spec().radius_px,
            t.paint.brush.radius_px,
            "fora do Grid Stamp o spec do traço é o pincel autorado, intocado"
        );
        t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::GridStamp.to_u8());
        let want = t
            .paint
            .brush
            .grid_stamp_frame(t.shape_silhouette_active())
            .0;
        assert_eq!(t.stroke_spec().radius_px, want);
        assert!(
            want > 40.0,
            "…e é o da CÉLULA (>{}), não os 3 px do pincel",
            40.0
        );
    }

    /// **O carimbo enche a célula EXATAMENTE com uma silhueta de Shape** — o report do Enio de
    /// 2026-08-09 (*"a imagem extrapola a célula do grid"*), medido pela porta que o tool usa.
    ///
    /// **Mutação que tem de sangrar:** `stroke_spec` passar `false` ao `as_grid_stamp`.
    #[test]
    fn a_shape_image_stamp_fills_the_cell_instead_of_overflowing_it() {
        let mut t = PainterTool::default();
        t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::GridStamp.to_u8());
        t.set_grid_cell_px(GridAxis::X, 40.0);
        t.set_grid_cell_px(GridAxis::Y, 40.0);
        let bare = t.stroke_spec().radius_px;
        t.set_brush_shape_image(vec![255u8; 64], 8, 8);
        let shaped = t.stroke_spec().radius_px;
        assert!(
            (shaped - 20.0).abs() < 1e-3,
            "com imagem o raio É meia célula ({shaped})"
        );
        assert!(
            bare > shaped * 1.5,
            "controle: sem imagem o falloff macio segue compensado ({bare}) — senão o gate não \
             distingue a correção de um raio simplesmente menor"
        );
    }

    /// **A célula sob o cursor é a que o carimbo usa**, e o retângulo mede a célula inteira — é o que
    /// o cursor desenha, e ele tem de contar o ENCAIXE (o dab não pousa sob o ponteiro).
    ///
    /// **Mutação que tem de sangrar:** `grid_cell_rect_at` devolver `Some` fora do Grid Stamp.
    #[test]
    fn the_cursor_cell_is_the_one_the_stamp_lands_in() {
        let mut t = PainterTool::default();
        t.set_grid_cell_px(GridAxis::X, 40.0);
        t.set_grid_cell_px(GridAxis::Y, 20.0);
        assert!(
            t.grid_cell_rect_at([13.0, 7.0]).is_none(),
            "fora do método não há célula a desenhar — o anel comum é o cursor"
        );
        t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::GridStamp.to_u8());
        let (centre, half) = t.grid_cell_rect_at([13.0, 7.0]).expect("célula");
        assert_eq!(half, [20.0, 10.0], "o retângulo é a célula INTEIRA");
        assert_eq!(
            centre,
            t.paint
                .brush
                .grid_cell_center(t.paint.brush.grid_cell_at([13.0, 7.0])),
            "o centro é o do carimbo, pela mesma porta"
        );
        // …e um ponto qualquer DENTRO da mesma célula devolve o mesmo retângulo: é o encaixe.
        assert_eq!(t.grid_cell_rect_at([39.0, 19.0]), Some((centre, half)));
    }

    /// **Trocar de meio SAI do Grid Stamp** (Enio, 2026-08-09) — ele é exclusivo do Digital, e um
    /// método armado que o dropdown não oferece é um estado que a UI não sabe exprimir nem desfazer.
    ///
    /// O **CONTROLE** é o método comum ao lado: trocar de meio não pode mexer num método que o meio
    /// novo oferece, senão a cura seria "toda troca reseta o traço".
    ///
    /// **Mutação que tem de sangrar:** apagar a chamada a `settle_stroke_method`.
    #[test]
    fn changing_the_medium_leaves_a_method_it_does_not_offer() {
        use super::super::media::PaintMedia;
        let grid = ph2d_painter_brush::StrokeMethod::GridStamp.to_u8();
        for media in [
            PaintMedia::Watercolor,
            PaintMedia::Impasto,
            PaintMedia::WetPaint,
        ] {
            let mut t = PainterTool::default();
            t.set_brush_stroke_method(grid);
            assert_eq!(t.brush_settings().stroke_method, grid, "premissa");
            t.set_paint_media(media);
            assert_ne!(
                t.brush_settings().stroke_method,
                grid,
                "{media:?} não oferece o Grid Stamp e ele ficou armado"
            );
        }
        // **O CONTROLE, e ele isola exatamente a coerção desta wave:** o MESMO gesto com o meio que
        // OFERECE o Grid Stamp não o toca. Sem ele, "sair do Grid Stamp" poderia ter sido escrito
        // como *"toda troca de meio reseta o método"*, e o gate passaria.
        //
        // ⚠️ O controle não pode ser um método qualquer sob um meio qualquer: **armar um meio já
        // reseta o método por lei PRÓPRIA e anterior** (medido: `set_wetpaint_armed(true)` leva
        // Line — e Drag Dot — a Space, via o `set_paint_tool_mode("brush")` dele). Misturar os dois
        // mecanismos faria este gate reprovar por algo que não é desta wave.
        let mut t = PainterTool::default();
        t.set_brush_stroke_method(grid);
        t.set_paint_media(PaintMedia::Digital);
        assert_eq!(
            t.brush_settings().stroke_method,
            grid,
            "o Digital OFERECE o Grid Stamp — a coerção é sobre a OFERTA, não sobre trocar de meio"
        );
    }

    /// O **default** que o Enio pediu, num teste que não menciona nenhum gesto — *um default só é
    /// testado por um teste que não o menciona* (a lição do W6.3 do vetor).
    #[test]
    fn the_lattice_is_drawn_by_default() {
        assert!(
            PainterTool::default().grid_show(),
            "Enio 2026-08-09: 'um checkbox checado por padrão para exibir o grid' — e uma regra de \
             encaixe que não se vê é uma regra que o artista tem de adivinhar"
        );
    }
}
