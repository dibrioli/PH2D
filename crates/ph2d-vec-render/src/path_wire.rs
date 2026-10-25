//! **O FIO DO CAMINHO** (F30 §E, report do dono 25/10) — 1 px de ECRÃ no acento do tema, por cima
//! da arte e por baixo dos nós, sem olhar para o traço nem para o preenchimento do objecto: o path
//! em construção e as formas seleccionadas veem-se com largura 0 e tinta transparente. É o idioma
//! do Figma/Illustrator; o porquê e as medições: `docs/UI_New_and_Simple/F30_plano.md` §E.
//!
//! ⚠️ px de TELA sob `Affine::IDENTITY` (a lei do [`super::marquee`]): no Vello o transform do
//! `stroke` multiplica a largura.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState, VecXforms};
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// A espessura do fio, em px de TELA.
pub const WIRE_PX: f64 = 1.0;

/// O token do fio — o do gizmo (as alças da caixa, `gizmo/paint.rs`). ⚠️ Não a `Selection`: ela é o
/// acento a 27,5 % sobre a base do chrome (`ph2d-tokens/src/derive.rs:541`), um tinto para linhas de
/// lista que, a 1 px sobre o canvas, some.
pub const WIRE_TOKEN: ColorToken = ColorToken::Accent;

/// Um token do tema na tinta do Vello — a porta dos overlays desta crate.
#[must_use]
pub fn token_ink(token: ColorToken, theme: Theme) -> Color {
    let c = token.resolve(theme);
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

/// **Pinta um caminho JÁ em px de ecrã** com uma caneta de `px` — a porta única dos contornos sem
/// estilo (o fio, o hover, o Trim).
pub fn stroke_screen(screen: &BezPath, ink: Color, px: f64, target: &mut VectorScene) {
    if screen.elements().is_empty() {
        return;
    }
    target.inner_mut().stroke(
        &Stroke::new(px),
        Affine::IDENTITY,
        &Brush::Solid(ink),
        None,
        screen,
    );
}

/// **O fio de cada `ids`**, pela MESMA projecção das âncoras ([`super::overlay_transform`]); um
/// caminho escondido não mostra fio.
pub fn draw_path_wires(
    scene: &VecScene,
    view: &VecViewState,
    xforms: &VecXforms,
    ids: &[VecPathId],
    camera: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let ink = token_ink(WIRE_TOKEN, theme);
    for &id in ids {
        if view.is_hidden(id) {
            continue;
        }
        let Some(path) = scene.path(id) else {
            continue;
        };
        let t = super::overlay_transform(view, xforms, id, camera);
        stroke_screen(&(t * super::build_bezpath(path)), ink, WIRE_PX, target);
    }
}

/// **O segmento que o próximo clique da Caneta faria** — da última âncora de `id` (com a alça de
/// saída dela) até `cursor_world`.
#[allow(clippy::too_many_arguments)]
pub fn draw_pen_reach(
    scene: &VecScene,
    view: &VecViewState,
    xforms: &VecXforms,
    id: VecPathId,
    cursor_world: [f64; 2],
    camera: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let Some(tip) = scene.path(id).and_then(|p| p.verts.last()) else {
        return;
    };
    let t = super::overlay_transform(view, xforms, id, camera);
    let a = t * Point::new(tip.anchor[0], tip.anchor[1]);
    let h = t * Point::new(tip.out_handle[0], tip.out_handle[1]);
    let c = camera * Point::new(cursor_world[0], cursor_world[1]);
    let mut seg = BezPath::new();
    seg.move_to(a);
    seg.curve_to(h, c, c);
    stroke_screen(&seg, token_ink(WIRE_TOKEN, theme), WIRE_PX, target);
}
