//! **A régua da tira** — a única conversão entre quadro e pixel.
//!
//! Quem PINTA uma célula e quem INTERPRETA um arrasto sobre ela têm de concordar sobre onde
//! o quadro 7 está na tela. Enquanto só existia o paint, a conta morava inline nele; com o
//! gesto ela passou a ter dois consumidores, e duas cópias divergem — foi assim que a régua
//! de scrub e o handle do playhead quase viraram duas respostas para a mesma pergunta
//! (`state::scrub_frame`, que existe *por isso*).
//!
//! A régua é **derivada**, nunca guardada: sai do retângulo das células + o snapshot deste
//! frame. Não há estado a sincronizar, e um snapshot novo produz a régua nova de graça.

use crate::state::FlipStripSnapshot;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

/// Largura máxima de um quadro em px (senão 3 chaves viram 3 painéis gigantes).
pub(crate) const MAX_PX_PER_FRAME: f32 = 26.0; // LITERAL-PX-OK: strip cell scale cap
/// Largura mínima visível de uma célula.
pub(crate) const MIN_CELL_W: f32 = 3.0; // LITERAL-PX-OK: strip minimum cell width

/// **A largura do pega-mão do hold**, em px de tela: a faixa na borda DIREITA da célula
/// que estica a exposição em vez de mover a chave.
///
/// Um `Spacing` token (é medida de design — o alvo que um dedo/mouse acerta), e o mesmo
/// papel do grip de trim de uma strip da timeline. Ele **come** da área de mover, então numa
/// célula estreita a divisão é feita com um teto (ver [`StripRuler::hold_edge_rect`]): abaixo
/// de um limite a borda simplesmente **não é oferecida** — perder o *esticar* neste zoom é
/// honesto; perder o *mover* seria um bug (a mesma lei que a alça de fade da timeline segue).
pub(crate) fn hold_grip_w() -> f32 {
    Spacing::Sm.px()
}

/// A fração máxima da largura da célula que o pega-mão do hold pode tomar. Acima disto a
/// célula é estreita demais para hospedar os dois alvos e o hold não é oferecido.
const HOLD_GRIP_MAX_SHARE: f32 = 0.4; // LITERAL-PX-OK: fração da largura, não medida de design

/// A régua desta pintura: onde cada quadro cai, e o inverso.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct StripRuler {
    /// x do quadro `first` (a borda esquerda da área útil).
    pub origin_x: f32,
    /// Topo das células (abaixo da faixa reservada da régua de scrub).
    pub cells_top: f32,
    /// Altura de uma célula.
    pub cell_h: f32,
    /// Pixels por quadro (já com o teto de [`MAX_PX_PER_FRAME`]).
    pub ppf: f32,
    /// Primeiro quadro exposto.
    pub first: i32,
    /// Um-além do último quadro exposto (o fim do vão).
    pub end: i32,
}

impl StripRuler {
    /// Resolve a régua para o retângulo das células. `None` sem células — sem vão não há
    /// escala, e responder um número aqui seria inventar geometria para uma tira vazia.
    pub(crate) fn resolve(area: Rect, snap: &FlipStripSnapshot) -> Option<Self> {
        if area.w <= 0.0 || area.h <= 0.0 || snap.cells.is_empty() {
            return None;
        }
        let (first, end) = snap.frame_span()?;
        let total = (end - first).max(1) as f32;
        let inner = Rect::new(
            area.x + Spacing::Xs.px(),
            area.y + Spacing::Xs.px(),
            (area.w - Spacing::Xs.px() * 2.0).max(1.0),
            (area.h - Spacing::Xs.px() * 2.0).max(1.0),
        );
        let cells_top = inner.y + crate::paint_cells::scrub_reserved_h();
        Some(Self {
            origin_x: inner.x,
            cells_top,
            cell_h: (inner.y + inner.h - cells_top).max(1.0),
            ppf: (inner.w / total).min(MAX_PX_PER_FRAME),
            first,
            end,
        })
    }

    /// x da borda ESQUERDA do quadro `frame`.
    pub(crate) fn x_of_frame(&self, frame: i32) -> f32 {
        self.origin_x + (frame - self.first) as f32 * self.ppf
    }

    /// O quadro sob `x`.
    ///
    /// **`floor`, não `round`** — e a diferença é o gesto inteiro: um quadro é uma FAIXA de
    /// pixels (a célula), não um ponto. Arredondar faria o ponteiro pertencer ao quadro
    /// seguinte assim que passasse da metade da célula, e arrastar "meia célula" moveria a
    /// chave um quadro inteiro. (A régua de scrub arredonda **de propósito**: lá o handle É
    /// um ponto, e ele deve grudar no quadro mais próximo — `state::scrub_frame`.)
    pub(crate) fn frame_at_x(&self, x: f32) -> i32 {
        if self.ppf <= 0.0 {
            return self.first;
        }
        self.first + ((x - self.origin_x) / self.ppf).floor() as i32
    }

    /// O retângulo da célula `i` (o alvo de MOVER).
    pub(crate) fn cell_rect(&self, i: usize, snap: &FlipStripSnapshot) -> Option<Rect> {
        let cell = snap.cells.get(i)?;
        let w = (cell.exposure.max(1) as f32 * self.ppf - 1.0).max(MIN_CELL_W);
        Some(Rect::new(
            self.x_of_frame(cell.key),
            self.cells_top,
            w,
            self.cell_h,
        ))
    }

    /// O retângulo do pega-mão do hold da célula `i` — a faixa na borda direita dela.
    ///
    /// `None` quando a célula é estreita demais para hospedar os dois alvos: aí ela é toda
    /// de MOVER (ver [`hold_grip_w`]).
    pub(crate) fn hold_edge_rect(&self, i: usize, snap: &FlipStripSnapshot) -> Option<Rect> {
        let r = self.cell_rect(i, snap)?;
        let grip = hold_grip_w();
        if grip > r.w * HOLD_GRIP_MAX_SHARE {
            return None;
        }
        Some(Rect::new(r.x + r.w - grip, r.y, grip, r.h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::FlipCell;

    fn cell(key: i32, exposure: u32) -> FlipCell {
        FlipCell {
            key,
            exposure,
            breakdown: false,
            instanced: false,
            selected: false,
            pinned: false,
            weight: 1.0,
        }
    }

    fn snap(cells: Vec<FlipCell>) -> FlipStripSnapshot {
        FlipStripSnapshot {
            has_layer: true,
            cells,
            ..Default::default()
        }
    }

    /// Uma área generosa: 12 quadros em 600 px batem no teto de `MAX_PX_PER_FRAME`, então
    /// uma FIXTURE de layout precisa ser estreita o bastante para o teto não mascarar a
    /// escala que ela quer medir.
    fn ruler(cells: Vec<FlipCell>, w: f32) -> (StripRuler, FlipStripSnapshot) {
        let s = snap(cells);
        let area = Rect::new(0.0, 0.0, w, 100.0);
        (StripRuler::resolve(area, &s).expect("régua"), s)
    }

    #[test]
    fn without_cells_there_is_no_ruler() {
        let s = FlipStripSnapshot::default();
        assert!(StripRuler::resolve(Rect::new(0.0, 0.0, 100.0, 100.0), &s).is_none());
        // E nem com células, se a área for degenerada (painel fechando).
        let s2 = snap(vec![cell(0, 2)]);
        assert!(StripRuler::resolve(Rect::new(0.0, 0.0, 0.0, 100.0), &s2).is_none());
    }

    /// 🔴 **x→quadro é o inverso de quadro→x, e o ponteiro pertence à célula que está SOB
    /// ele** — em qualquer ponto DENTRO da faixa do quadro, não só na borda esquerda.
    /// Mutação que sangra: trocar o `floor` por `round` (o ponto a 60% da célula passaria a
    /// responder o quadro seguinte, e arrastar meia célula moveria a chave inteira).
    #[test]
    fn a_pointer_anywhere_inside_a_frames_band_belongs_to_that_frame() {
        let (r, _) = ruler(vec![cell(0, 4), cell(4, 4)], 80.0);
        for frame in r.first..r.end {
            let x0 = r.x_of_frame(frame);
            for t in [0.01_f32, 0.5, 0.99] {
                let x = x0 + t * r.ppf;
                assert_eq!(
                    r.frame_at_x(x),
                    frame,
                    "x a {t} da faixa do quadro {frame} caiu fora dele"
                );
            }
        }
    }

    /// A célula tem a largura da EXPOSIÇÃO (o tempo é visível como espaço — TVPaint).
    #[test]
    fn a_cells_width_is_its_exposure() {
        let (r, s) = ruler(vec![cell(0, 1), cell(1, 6)], 80.0);
        let one = r.cell_rect(0, &s).unwrap();
        let six = r.cell_rect(1, &s).unwrap();
        assert!(
            six.w > one.w * 5.0,
            "seis quadros deviam medir ~6x um quadro ({} vs {})",
            six.w,
            one.w
        );
        assert!(
            (six.x - r.x_of_frame(1)).abs() < 0.01,
            "começa na sua chave"
        );
    }

    /// 🔴 **Numa célula estreita o hold NÃO é oferecido** — a célula inteira vira alvo de
    /// mover. Perder o *esticar* num zoom apertado é honesto (a caixa Hold da barra segue
    /// lá); perder o *mover* seria um bug, e é o que aconteceria se o grip tomasse a célula.
    #[test]
    fn a_narrow_cell_offers_no_hold_grip_and_stays_movable() {
        // 40 quadros numa faixa curta ⇒ ppf minúsculo ⇒ células de poucos px.
        let (r, s) = ruler(vec![cell(0, 1), cell(1, 39)], 60.0);
        assert!(
            r.hold_edge_rect(0, &s).is_none(),
            "a célula de 1 quadro é estreita demais para os dois alvos"
        );
        assert!(r.cell_rect(0, &s).is_some(), "mas continua movível");
        // A larga hospeda os dois.
        let wide = r.hold_edge_rect(1, &s).expect("a célula larga tem grip");
        let body = r.cell_rect(1, &s).unwrap();
        assert!(
            wide.x > body.x && (wide.x + wide.w - (body.x + body.w)).abs() < 0.01,
            "o grip mora na borda DIREITA da célula"
        );
    }
}
