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
    if icon.cheio()
        && let Some(transform) = encaixe_do_icone(rect)
    {
        scene.fill_path(&path, &Brush::Solid(color), transform);
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
    let Some(transform) = encaixe_do_icone(rect) else {
        return;
    };
    let stroke = Stroke::new(stroke_width as f64);
    let brush = Brush::Solid(color);
    // Pela porta que GRAVA (mesma codificação): a régua da F18 tem de ver o glifo.
    scene.stroke_path(&stroke, transform, &brush, path);
}

/// A viewbox 24×24 posta no `rect`; `None` se o `rect` não tem lugar para o glifo.
fn encaixe_do_icone(rect: Rect) -> Option<Affine> {
    const VIEWBOX: f64 = 24.0;
    let pad = icon_pad(rect.w.min(rect.h));
    let avail_w = (rect.w - pad * 2.0).max(0.0) as f64;
    let avail_h = (rect.h - pad * 2.0).max(0.0) as f64;
    if avail_w <= 0.0 || avail_h <= 0.0 {
        return None;
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
    Some(Affine::translate((tx, ty)) * Affine::scale(scale))
}

/// O respiro que [`paint_icon_path`] deixa à volta do ícone num rectângulo de lado `side`.
#[must_use]
pub fn icon_pad(side: f32) -> f32 {
    2.0_f32.min(side * 0.1) // LITERAL-PX-OK: o respiro de sempre do ícone (2 px, ou 10 % do lado)
}

/// O lado do DESENHO de um ícone pintado num quadrado de lado `side` (o lado menos o respiro).
#[must_use]
pub fn icon_glyph_side(side: f32) -> f32 {
    (side - 2.0 * icon_pad(side)).max(0.0)
}

/// O lado do quadrado em que um ícone desenha `glyph` px — a inversa de [`icon_glyph_side`].
#[must_use]
pub fn icon_side_for_glyph(glyph: f32) -> f32 {
    // Com o respiro de 2 px (lado ≥ 20) o lado é o desenho + 4; abaixo, o respiro é 10 % do lado.
    let fixed = glyph + 4.0; // LITERAL-PX-OK: 2 × o respiro máximo do `icon_pad`
    if fixed * 0.1 >= 2.0 {
        fixed
    } else {
        glyph / 0.8
    } // LITERAL-PX-OK: 1 − 2 × 10 %
}

/// ⭐ **O glifo na tinta do TEMA** — o ícone de uma cor, salvo os de DUAS tintas: a Position do
/// traço (dono 01/11, F44c: *«o stroke de cor diferente da linha central»*) pinta o anel do traço no
/// acento e a borda da forma (a linha central, contínua) no texto forte — a escolha lê-se pelo
/// fundo da peça. ⛔ Tracejada, ou na tinta apagada (`ink`) das peças por escolher, a 22 px a borda
/// sumia (fotos F44c).
pub fn paint_icon_toned(
    scene: &mut VectorScene,
    icon: IconId,
    rect: Rect,
    ink: Color,
    theme: ph2d_tokens::Theme,
    stroke_width: f32,
) {
    // Em unidades do `viewBox` 24 dos SVG irmãos (`stroke-*.svg`): a borda e o anel — o de fora
    // (`EDGE_R + RING_W`) encosta na margem do desenho.
    const EDGE_R: f64 = 7.0; // LITERAL-PX-OK: raio da borda no viewBox 24
    const RING_W: f64 = 5.0; // LITERAL-PX-OK: espessura do anel no viewBox 24
    let shift = match icon {
        IconId::StrokeInside => -RING_W * 0.5,
        IconId::StrokeCenter => 0.0,
        IconId::StrokeOutside => RING_W * 0.5,
        _ => return paint_icon(scene, icon, rect, ink, stroke_width),
    };
    let side = icon_glyph_side(rect.w.min(rect.h));
    let scale = f64::from(side) / 24.0; // LITERAL-PX-OK: o viewBox dos SVG
    let c = ph2d_vector::Point::new(
        f64::from(rect.x + rect.w * 0.5),
        f64::from(rect.y + rect.h * 0.5),
    );
    let ring = crate::paint::resolve(ph2d_tokens::ColorToken::Accent, theme);
    let anel = ph2d_vector::Circle::new(c, (EDGE_R + shift) * scale);
    scene.stroke_path(
        &Stroke::new(RING_W * scale),
        Affine::IDENTITY,
        &Brush::Solid(ring),
        &ph2d_vector::Shape::to_path(&anel, 0.1),
    );
    let borda = ph2d_vector::Circle::new(c, EDGE_R * scale);
    scene.stroke_path(
        &Stroke::new(f64::from(stroke_width)),
        Affine::IDENTITY,
        &Brush::Solid(crate::paint::resolve(ph2d_tokens::ColorToken::Text1, theme)),
        &ph2d_vector::Shape::to_path(&borda, 0.1),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O SVG cheio (`fill="currentColor"`) e o `cheio()` dizem o mesmo, e a tinta vem com ele: miolo + contorno.
    #[test]
    fn a_full_icon_is_the_one_whose_svg_says_so_and_it_paints_its_fill() {
        let pasta = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/design/icons");
        let mut cheios = 0;
        for id in IconId::all() {
            let svg = std::fs::read_to_string(format!("{pasta}/{}.svg", id.slug())).expect("o SVG");
            assert_eq!(svg.contains(r#"fill="currentColor""#), id.cheio(), "{id:?}");
            cheios += usize::from(id.cheio());
            let mut scene = VectorScene::new();
            paint_icon(
                &mut scene,
                *id,
                Rect::new(0.0, 0.0, 24.0, 24.0),
                Color::WHITE,
                1.5,
            );
            let desenhos = scene.inner().encoding().draw_data.len();
            assert_eq!(desenhos, 1 + usize::from(id.cheio()), "{id:?}");
        }
        assert!(cheios >= 1, "controlo: a régua viu um ícone cheio");
    }
}
