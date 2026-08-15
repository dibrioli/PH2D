//! **A FAMÍLIA DOS ÍCONES** — como um glifo do manifesto chega ao ecrã.
//!
//! Irmão do [`super::paint`] pela linha que separa os dois assuntos: aquele é o passe de
//! LOWERING (a árvore de widgets → comandos do Vello), isto é *com que forma um ícone é
//! desenhado*. O corte nasceu no tecto de 700 LOC quando a rotação chegou.
//!
//! ⚠️ **E o `canonical_icon_button` aponta para AQUI agora.** Ele reserva o
//! [`paint_icon_path`] ao botão de ícone canónico e à definição dele — a allowlist segue o
//! código, senão o gate passaria a proibir o ficheiro que define a função que ele protege.

use crate::icons::{IconId, cmd_to_path};
use crate::zones::Rect;
use ph2d_vector::{Affine, BezPath, Brush, Color, Stroke, VectorScene};

/// Render an icon centered inside `rect`. The source icons are 24x24
/// and stroked with a 1.5pt line; we scale uniformly to fit, keeping
/// 2px of padding so glyphs don't kiss the rect edge.
pub fn paint_icon(
    scene: &mut VectorScene,
    icon: IconId,
    rect: Rect,
    color: Color,
    stroke_width: f32,
) {
    let mut path = BezPath::new();
    for cmd in icon.cmds() {
        path.extend(cmd_to_path(*cmd));
    }
    paint_icon_path(scene, &path, rect, color, stroke_width);
}

/// **Um ícone RODADO em torno do centro da própria viewbox.**
///
/// ⚠️ **A rotação acontece DENTRO da viewbox (24×24), antes do encaixe** — o [`paint_icon_path`]
/// escala a VIEWBOX e não a bbox do caminho, então rodar depois faria a caixa passar de
/// larga-e-baixa a alta-e-estreita e o glifo **respiraria de tamanho** a meio do gesto.
///
/// ⚠️ **E existe aqui, e não no chamador, por causa de um arch-gate:** o `paint_icon_path` é
/// reservado ao botão de ícone canónico (`canonical_icon_button`), e com razão — um chrome novo
/// que o chame à mão está a re-fazer um botão. *Rodar um glifo* é capacidade da camada de ícones,
/// não licença para desenhar um botão, e é por isso que a porta é esta e o gate fica intacto.
pub fn paint_icon_rotated(
    scene: &mut VectorScene,
    icon: IconId,
    rect: Rect,
    color: Color,
    stroke_width: f32,
    radians: f64,
) {
    const VIEWBOX_MID: f64 = 12.0; // LITERAL-PX-OK: centro da viewbox 24x24 dos ícones
    let mut path = BezPath::new();
    for cmd in icon.cmds() {
        path.extend(cmd_to_path(*cmd));
    }
    path.apply_affine(Affine::rotate_about(
        radians,
        ph2d_vector::Point::new(VIEWBOX_MID, VIEWBOX_MID),
    ));
    paint_icon_path(scene, &path, rect, color, stroke_width);
}

/// Same as [`paint_icon`] but takes a pre-built [`BezPath`] in the
/// canonical 24×24 icon design space. Used by Wave 2 PR 11.4 chrome
/// derivation — manifest `icon_fn` returns a `BezPath` directly, so
/// the editor can paint registry-derived pills without an
/// `IconId` enum round-trip.
pub fn paint_icon_path(
    scene: &mut VectorScene,
    path: &BezPath,
    rect: Rect,
    color: Color,
    stroke_width: f32,
) {
    const VIEWBOX: f64 = 24.0;
    let pad = 2.0_f32.min(rect.w.min(rect.h) * 0.1);
    let avail_w = (rect.w - pad * 2.0).max(0.0) as f64;
    let avail_h = (rect.h - pad * 2.0).max(0.0) as f64;
    if avail_w <= 0.0 || avail_h <= 0.0 {
        return;
    }
    let scale = (avail_w / VIEWBOX).min(avail_h / VIEWBOX);
    let drawn = VIEWBOX * scale;
    // Pixel-snap the icon origin: with `stroke_width = 1.5px`, sub-pixel
    // translation makes MSAA16 stipple the stroke across two rows of
    // pixels. Rounding to the nearest integer pixel keeps strokes
    // anchored to a predictable grid position (up to 0.5 px visual
    // shift from "ideal" centering — invisible at icon sizes, big
    // crispness win on near-horizontal/vertical strokes).
    let tx = (rect.x as f64 + (rect.w as f64 - drawn) * 0.5).round();
    let ty = (rect.y as f64 + (rect.h as f64 - drawn) * 0.5).round();
    let transform = Affine::translate((tx, ty)) * Affine::scale(scale);
    let stroke = Stroke::new(stroke_width as f64);
    let brush = Brush::Solid(color);
    scene
        .inner_mut()
        .stroke(&stroke, transform, &brush, None, path);
}
