//! **O MOTOR do Colorize no shell** — as conversões e a porta única de inserção, irmãs do
//! `flip_colorize.rs` pelo teto de LOC do shell (HR-18, 600).
//!
//! É aqui que mora o que se pode DIRIGIR sem janela: o `flip_colorize_apply` precisa de
//! `gfx` (a janela + a GPU) e nenhum teste o alcança, mas um `FlipDoc` se monta headless —
//! então o laço que colore os quadros do onion fill (fatia C3) é testável de verdade, e não
//! só por arch-gate sobre o fonte.

use super::{FlipStyleSnapshot, LiveFrame, Scribble, boundaries, fill_stroke};
use ph2d_flip::{DrawingId, FlipDrawing, FlipObjectId, FlipStroke};
use ph2d_flip_colorize::ColorRegion;

/// `(precision, trap_px)` a partir do estilo + da vista — a MESMA conversão do balde, numa
/// porta só, para o Apply e o re-Apply ao vivo **nunca** divergirem (duas portas divergem).
///
/// O `trap_px` devolvido é o EFETIVO: o `max` do slider **Trap** com o raio de **selagem** do
/// `Bleed 0` (6º smoke, "Bleed 0 SELA o vão"). Os dois são força de trapped-ball; o `Bleed`
/// alimenta o raio pelo `seal_from_bleed` (o pedágio satura e nunca fecha a lente — só a bola
/// que não passa pelo vão o faz). `Bleed` acima do joelho ⇒ selo 0 ⇒ o Trap sozinho, e a
/// seepage fica com o `squeeze` — o 5º smoke intacto.
pub(super) fn precision_and_trap(
    style: &FlipStyleSnapshot,
    px_to_world: f32,
    obj_scale: f32,
) -> (f32, f32) {
    let doc_per_px = px_to_world * obj_scale;
    // px de tela → px de buffer por unidade de documento (a precisão do balde).
    let precision = 1.6 / doc_per_px.max(1e-6);
    // O Trap chega em px de TELA e atravessa as duas conversões (BUGS #11: subir a
    // Precision encolheria a bola em silêncio se ele não cruzasse `precision`).
    let trap_px = (style.trap as f32) * doc_per_px * precision;
    // O selo do Bleed 0, em DOC units → px de buffer pela MESMA precisão. Combinado por `max`.
    let seal_px = ph2d_flip_colorize::seal_from_bleed(style.colorize_bleed as f32) * precision;
    (precision, trap_px.max(seal_px))
}

/// **A porta única que produz e insere as regiões** — o Apply e o re-Apply ao vivo chamam
/// esta MESMA função, então a borda de uma cor colorida nunca depende de por qual caminho
/// ela foi (re)gerada. Assume `drawing` já na base (line-art + fills pré-Colorize) e devolve
/// quantos strokes foram inseridos.
pub(super) fn insert_regions(
    drawing: &mut FlipDrawing,
    palette: &[[u8; 4]],
    seeds: &[Scribble],
    precision: f32,
    trap_px: f32,
    squeeze: u32,
) -> usize {
    let lines = boundaries(drawing);
    if lines.is_empty() {
        return 0;
    }
    let regions: Vec<ColorRegion> =
        ph2d_flip_colorize::colorize_with(&lines, seeds, precision, trap_px, squeeze);
    // Cada região entra ACIMA dos fills existentes (a cor nova cobre a velha, abaixo da
    // linha — o `Paint` do balde), com a MESMA dilatação (contour_widths + fill_stroke).
    let is_fill = |s: &FlipStroke| s.hide_stroke && s.fill.is_some();
    let mut produced = 0;
    for region in regions {
        let color = crate::flip_draw::srgb8_to_linear(palette[region.label as usize]);
        let widths = ph2d_flip_fill::contour_widths(&lines, &region.fill.outer);
        let stroke = fill_stroke(&region.fill.outer, region.fill.holes, color, 1.0, &widths);
        let at = drawing
            .strokes
            .iter()
            .rposition(is_fill)
            .map_or(0, |i| i + 1);
        drawing.strokes.insert(at, stroke);
        produced += 1;
    }
    produced
}

/// **O fan-out do onion fill** (fatia C3, `09 §5.2`): colore CADA desenho de `dids` com as
/// MESMAS sementes, e devolve um [`LiveFrame`] por quadro que de fato produziu região.
///
/// Extraído do `flip_colorize_apply` para ser DIRIGÍVEL: o Apply precisa de `gfx` (janela +
/// GPU) e nenhum teste o alcança, mas o `FlipDoc` se monta headless — então o laço que
/// escreve nos quadros é testável mesmo que o clique não seja. Um arch-gate irmão
/// (`the_colorize_scribble_crosses_the_selected_frames`) pina a metade que sobra: que o
/// Apply de fato PERGUNTA os alvos à tira.
///
/// **Cada quadro é um solve independente** — a linha se move entre os quadros, a região muda
/// de forma, e não há contorno a reaproveitar (o comentário gêmeo do `flip_fill`). As
/// sementes servem todos porque a `w2l` é do OBJETO, não do desenho.
///
/// **Um quadro que não fecha falha em SILÊNCIO** e é devolvido INTOCADO: a política herdada
/// do balde (`09 §5.2`) — um quadro em que a arte não fecha não pode derrubar o gesto nos
/// outros, e o toast fala pelo quadro ATIVO, que é onde o artista está olhando.
#[expect(clippy::too_many_arguments, reason = "os parâmetros do corte, um a um")]
pub(super) fn colorize_frames(
    flip: &mut ph2d_flip::FlipDoc,
    oid: FlipObjectId,
    dids: &[DrawingId],
    palette: &[[u8; 4]],
    seeds: &[Scribble],
    precision: f32,
    trap_px: f32,
    squeeze: u32,
) -> Vec<LiveFrame> {
    let mut out = Vec::new();
    for &did in dids {
        let Some(dr) = flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
            continue;
        };
        let base = dr.strokes.clone();
        let produced = insert_regions(dr, palette, seeds, precision, trap_px, squeeze);
        if produced == 0 {
            dr.strokes = base; // silêncio: este quadro não fechou, os outros seguem
            continue;
        }
        out.push(LiveFrame {
            did,
            base,
            produced,
        });
    }
    out
}
