//! **O BAKE do traço** — a metade do `draw` que precisa da shell (W2/L5 Fase B, 2026-09-11).
//!
//! A LEI do traço (a reamostragem, o ajuste, a construção) vive em [`crate::draw`]: ela
//! só fala `ph2d_flip` e `crate::smooth`. O que sobra aqui é **uma função** —
//! [`bake_stroke`] — e o que a prende é nomeável: ela pergunta a [`crate::autokey`] qual é
//! o desenho-alvo e escreve na [`crate::strip`], e esses dois estão presos atrás do
//! `vec_transform` (§3 do handoff).
//!
//! ⭐ **Separá-la foi o que destravou os TRÊS roteadores do fim da linha** — o
//! `hardness`/`pressure`/`resample` precisavam de UMA função do `draw`
//! (`stroke_from_samples`), e não desta.

use ph2d_core::Vec2;
use ph2d_flip::{FlipDoc, LayerId};
use ph2d_tool_flip::FlipStyleSnapshot;
use ph2d_vec_scene::Xform;

/// Assa `(points, pressures)` (mundo) num `FlipStroke` e o empurra no desenho
/// ativo do 1º objeto na CAMADA ATIVA (fallback: topo) no quadro atual. Cria uma
/// chave se o quadro ainda não tem desenho. `px_to_world` = mundo por pixel de
/// tela (a largura do brush é em px → convertida pra mundo). Uma camada TRAVADA
/// (`locked`) recusa o traço. Devolve `true` se assou.
#[allow(clippy::too_many_arguments)] // doc+playhead+estilo+camada+amostras+afim são intrínsecos
pub fn bake_stroke(
    flip: &mut FlipDoc,
    playhead: &ph2d_core::Playhead,
    style: &FlipStyleSnapshot,
    active_layer: Option<LayerId>,
    strip: &mut crate::strip::FlipStrip,
    points: &[Vec2],
    pressures: &[f32],
    world_to_local: &Xform,
) -> Option<(ph2d_flip::FlipObjectId, ph2d_flip::DrawingId, usize)> {
    if points.len() < 2 {
        return None;
    }
    // **O autokey por-tool (W3.T3.4)**: quem decide o desenho-alvo — e se uma chave
    // nova nasce (em branco, ou como cópia sob *Additive*) — é o `flip_autokey`, o
    // mesmo ponto que a borracha usa. A caneta nunca resolve isso na mão.
    let (oid, _lid, did) = crate::autokey::target_drawing(
        flip,
        playhead,
        active_layer,
        strip,
        crate::autokey::FlipEdit::Draw,
    )?;
    let drawing = flip.object_mut(oid)?.drawing_mut(did)?;

    // Active smoothing (T2.7): assa EXATAMENTE o traço que o preview mostrou — o
    // mesmo `active_smooth`, sem decimar. O RDP do 1º corte (0.75px) deixava o
    // traço assado mais anguloso que o preview (Enio 2026-07-11: "o desenho em
    // tempo real está mais suave que o traço cosido após mouse up"); mantê-los
    // idênticos vale mais que "enxuto". As pressões seguem 1:1 (o smooth só move
    // posições). Uma decimação visualmente-perdida-zero (RDP fininho) tira só
    // pontos EXATAMENTE colineares, sem cortar curva.
    drawing.strokes.push(crate::draw::stroke_from_samples(
        style,
        points,
        pressures,
        world_to_local,
    ));
    Some((oid, did, drawing.strokes.len() - 1))
}
