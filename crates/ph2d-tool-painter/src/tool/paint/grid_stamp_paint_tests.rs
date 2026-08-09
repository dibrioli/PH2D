//! **O que o carimbo de grade de fato PINTA** — o irmão de `grid_stamp_settings`, que mede os
//! controles. A pergunta aqui é a do artista, não a do slider: *a tinta caiu na célula?*
//!
//! ⚠️ O oráculo é o **retângulo de texels escritos**, lido do canvas do produto depois de um gesto de
//! ponteiro real. Ele não conhece `rim_t0`, nem `grid_stamp_frame`, nem o `sample_mask` — é isso que
//! o torna oráculo em vez de espelho: uma regra que mude e continue concordando consigo mesma passa
//! por um gate que a re-derive, e falha aqui.

use crate::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// O retângulo `(x0, x1, y0, y1)` INCLUSIVO dos texels que deixaram de ser a folha branca.
fn painted_box(t: &PainterTool, size: u32) -> (i64, i64, i64, i64) {
    let px = &*t.canvas_rgba;
    let (mut x0, mut x1, mut y0, mut y1) = (i64::MAX, i64::MIN, i64::MAX, i64::MIN);
    for y in 0..i64::from(size) {
        for x in 0..i64::from(size) {
            let i = ((y * i64::from(size) + x) * 4) as usize;
            if px[i] < 250 {
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
    }
    (x0, x1, y0, y1)
}

/// Um tool com folha branca, Grid Stamp armado, célula quadrada e uma silhueta de Shape OPACA.
fn grid_tool(cell: f32, offset: f32, size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_stroke_method(ph2d_painter_brush::StrokeMethod::GridStamp.to_u8());
    t.set_grid_cell_px(super::grid_stamp_settings::GridAxis::X, cell);
    t.set_grid_cell_px(super::grid_stamp_settings::GridAxis::Y, cell);
    t.set_grid_offset_px(super::grid_stamp_settings::GridAxis::X, offset);
    t.set_grid_offset_px(super::grid_stamp_settings::GridAxis::Y, offset);
    // Imagem CHEIA: a silhueta é o quadrado inteiro, então o que o carimbo pinta é exatamente o
    // alcance do footprint — uma imagem com margem transparente mediria a ARTE, não o motor.
    t.set_brush_shape_image(vec![255u8; 32 * 32], 32, 32);
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.brush.strength = 1.0;
    t
}

fn tap(t: &mut PainterTool, p: [f32; 2]) {
    let at = |phase| CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(PointerPhase::Down));
    t.on_canvas_pointer(at(PointerPhase::Up));
}

/// ⚠️ **A imagem enche a célula e NADA fora dela** — o report do Enio de 2026-08-09 (*"a imagem não
/// está perfeitamente ajustada à célula do grid e nem centralizada"*).
///
/// MEDIDO antes da correção: numa célula de 40 px o carimbo pintava **41 colunas cheias**, a sobra
/// inteira à direita e abaixo — a assinatura de um clamp de borda somado a um retângulo de blit
/// assimétrico, e não de um raio errado (um raio errado erraria dos DOIS lados).
///
/// A varredura cobre célula **par e ímpar** (com a ímpar o centro cai num centro de pixel em vez de
/// numa fronteira, e é ali que um erro de meio texel troca de lado) e **com e sem deslocamento** da
/// grade — uma grade ancorada em zero esconde todo erro que seja função do offset.
///
/// **Mutação que tem de sangrar:** apagar o corte de `sample_mask` fora do quadrado unitário.
#[test]
fn a_shape_stamp_paints_the_cell_and_not_one_texel_more() {
    let size = 128u32;
    for cell in [40.0f32, 41.0, 32.0] {
        for offset in [0.0f32, 7.0] {
            let mut t = grid_tool(cell, offset, size);
            // Um toque em QUALQUER ponto da célula 0: o método encaixa no centro dela.
            tap(&mut t, [offset + 1.0, offset + 1.0]);
            let got = painted_box(&t, size);
            let lo = offset as i64;
            let hi = lo + cell as i64 - 1;
            assert_eq!(
                got,
                (lo, hi, lo, hi),
                "célula {cell} px, offset {offset}: a tinta caiu em {got:?} e a célula é \
                 [{lo}, {hi}] nos dois eixos"
            );
        }
    }
}

/// **O Cell Fit encolhe e cresce em torno do MESMO centro** (Enio, 2026-08-09) — a metade que o gate
/// acima não vê, porque ele mede só o encaixe exato.
///
/// O oráculo é a **simetria**: a folga que sobra à esquerda tem de ser a que sobra à direita. Um
/// fator aplicado ao raio sem re-centrar cresceria só para um lado, e um gate que medisse apenas a
/// LARGURA ficaria verde sobre isso.
///
/// ⚠️ **A célula medida é a `(1,1)`, não a `(0,0)`** — e a 1ª versão deste gate reprovou por isso:
/// com `fit > 0` o carimbo passa da célula **de propósito**, e na célula da quina quem o corta é a
/// **borda do canvas**, não o motor (medido: centro em 22,5 onde a célula diz 20). Uma fixture
/// encostada na margem mede o recorte e chama-o de descentramento.
///
/// **Mutação que tem de sangrar:** ignorar `grid_fit_scale()` no raio (a largura para de responder).
#[test]
fn the_cell_fit_grows_and_shrinks_around_the_cells_centre() {
    let size = 160u32;
    let cell = 40.0f32;
    // O centro da célula (1,1) — com folga de uma célula inteira para cada lado, que é o que o fit
    // máximo desta varredura pede.
    let want_centre = cell * 1.5;
    let mut widths = Vec::new();
    for fit in [-0.5f32, -0.25, 0.0, 0.25] {
        let mut t = grid_tool(cell, 0.0, size);
        t.set_grid_fit(fit);
        tap(&mut t, [cell + 1.0, cell + 1.0]);
        let (x0, x1, y0, y1) = painted_box(&t, size);
        let centre = 0.5 * (x0 + x1 + 1) as f32;
        assert!(
            (centre - want_centre).abs() <= 0.5,
            "fit {fit}: a tinta ficou centrada em {centre} e o centro da célula é {want_centre}"
        );
        assert_eq!(
            (x1 - x0, y1 - y0),
            (x1 - x0, x1 - x0),
            "fit {fit}: a célula é quadrada, o carimbo saiu {}×{}",
            x1 - x0 + 1,
            y1 - y0 + 1
        );
        widths.push(x1 - x0 + 1);
    }
    for pair in widths.windows(2) {
        assert!(
            pair[1] > pair[0],
            "as larguras {widths:?} deviam crescer com o fit — o slider não está chegando ao raio"
        );
    }
    // O número MEDIDO ao lado da propriedade (§0): `fit = 0` enche a célula EXATA.
    assert_eq!(
        widths[2], cell as i64,
        "no meio do curso o carimbo tem de medir a célula, e mediu {} px",
        widths[2]
    );
}
