//! ⭐ **O canvas diz o USO da imagem** (smoke do dono 09/11: *«belos indicativos visuais de que estamos lidando com
//! Animated ou Repeated ou Region… molduras por quadro, moldura global com label»*).
//!
//! Só com a folha INTEIRA à vista ([`ph2d_ecs::imagem_aberta`]: Edit, a pintura): uma moldura por fora dela, na cor do
//! uso, com a etiqueta (`Animated · 6 × 2`) na quina superior-esquerda, e a moldura de cada quadro, com o que toca em
//! traço grosso. ⛔ Sobre o resultado final no canvas a etiqueta atrapalha (dono 09/11, com foto): lá não se desenha.
//!
//! Em pixels de TELA (a lei do [`ph2d_sheet_bounds::overlay`]): é legenda, e lê-se igual em qualquer zoom.

use crate::uso::{ImageUse, image_use};
use ph2d_ecs::{Entity, SpriteGrid, World};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
use ph2d_vector::{
    Affine, BezPath, Brush, Color, Fill, Point, Rect, RoundedRect, Stroke, VectorScene,
};

/// Folga entre o quad e a moldura, em pixels de tela: a caixa do gizmo pousa NA borda, a moldura fica fora dela.
const GAP_PX: f64 = 6.0;
/// Espessura da moldura, em pixels de tela.
const FRAME_PX: f64 = 1.5;
/// Espessura da linha entre quadros, em pixels de tela.
const CELL_PX: f64 = 1.0;
/// Espessura do quadro que toca, em pixels de tela.
const LIVE_PX: f64 = 2.5;
/// Folga horizontal do texto dentro da etiqueta, em pixels de tela.
const PAD_X_PX: f64 = 6.0;
/// Folga vertical do texto dentro da etiqueta, em pixels de tela.
const PAD_Y_PX: f64 = 2.0;
/// Raio dos cantos da etiqueta, em pixels de tela.
const RADIUS_PX: f64 = 4.0;
/// Largura máxima do texto da etiqueta, em pixels de tela.
const LABEL_MAX_W_PX: f32 = 260.0;

/// A cor de cada uso — uma por uso, para o olho separar antes de ler.
#[must_use]
pub fn use_color(u: ImageUse) -> Option<ColorToken> {
    match u {
        ImageUse::Still => None,
        ImageUse::Animated => Some(ColorToken::Success),
        ImageUse::Repeated => Some(ColorToken::Warn),
        ImageUse::Region => Some(ColorToken::Info),
    }
}

/// O texto da etiqueta de `e`: o uso, e a grelha da folha à vista quando é animada.
#[must_use]
pub fn label(world: &World, e: Entity) -> Option<String> {
    let u = image_use(world, e)?;
    let nome = match u {
        ImageUse::Still => return None,
        ImageUse::Animated => tr("panel.inspector.image.use_animated"),
        ImageUse::Repeated => tr("panel.inspector.image.use_repeated"),
        ImageUse::Region => tr("panel.inspector.image.use_region"),
    };
    let grid = folha(world, e).map(|(_, g, _)| g);
    Some(
        match grid.filter(|g| u == ImageUse::Animated && g.hframes * g.vframes > 1) {
            Some(g) => format!("{nome} \u{00b7} {} \u{00d7} {}", g.hframes, g.vframes),
            None => nome.to_owned(),
        },
    )
}

/// A geometria da folha à vista: o `Sprite` dela (tamanho de UMA célula), a grelha, e se o quadro que toca é dela.
fn folha(world: &World, e: Entity) -> Option<(ph2d_render::Sprite, SpriteGrid, bool)> {
    let k = ph2d_ecs::shown_sheet(world, e);
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let toca = g.sheet == k;
    let (spr, grid) = ph2d_render::sheet_geometry(world, e, k, if toca { g.frame } else { 0 })?;
    Some((spr, grid.unwrap_or(SpriteGrid::SINGLE), toca))
}

/// O retângulo LOCAL do que está desenhado (`x0, y0` = quina superior-esquerda, `y` para cima) e, em Edit, a grelha.
struct Desenho {
    x0: f64,
    y0: f64,
    w: f64,
    h: f64,
    /// A grelha quando a folha inteira está à vista.
    celulas: Option<Celulas>,
}

/// `(colunas, linhas, centro do quadro que toca)`.
type Celulas = (u32, u32, Option<(f64, f64)>);

fn desenho(world: &World, e: Entity, ppm: f32) -> Option<Desenho> {
    ph2d_ecs::imagem_aberta(world, e)?;
    let (spr, grid, toca) = folha(world, e)?;
    let l = ph2d_sprite_screen::sheet_lattice::lattice(&spr, grid, ppm, true)?;
    let viva = toca.then_some((l.live_cx, l.live_cy));
    Some(Desenho {
        x0: l.x0,
        y0: l.y0,
        w: l.w,
        h: l.h,
        celulas: Some((grid.hframes.max(1), grid.vframes.max(1), viva)),
    })
}

/// Desenha a moldura do uso de cada objecto de `selected`.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    world: &World,
    selected: impl IntoIterator<Item = u64>,
    pixels_per_meter: f32,
    cam: Affine,
    px_per_world: f64,
    theme: Theme,
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
) {
    if px_per_world <= 0.0 {
        return;
    }
    for bits in selected {
        let e = Entity::from_bits(bits);
        let Some(token) = image_use(world, e).and_then(use_color) else {
            continue;
        };
        let (Some(d), Some(texto), Some(wt)) = (
            desenho(world, e, pixels_per_meter),
            label(world, e),
            ph2d_ecs::world_transform(world, e),
        ) else {
            continue;
        };
        let xf = cam
            * Affine::translate((f64::from(wt.translation.x), f64::from(wt.translation.y)))
            * Affine::rotate(f64::from(wt.rotation))
            * Affine::scale_non_uniform(f64::from(wt.scale.x), f64::from(wt.scale.y));
        // As constantes são de tela; no espaço local a escala do objecto entra (a lei da grelha da folha).
        let (sx, sy) = (
            f64::from(wt.scale.x.abs()).max(f64::EPSILON),
            f64::from(wt.scale.y.abs()).max(f64::EPSILON),
        );
        let um = px_per_world * sx.max(sy);
        let (gx, gy) = (GAP_PX / (px_per_world * sx), GAP_PX / (px_per_world * sy));
        let cor = resolve(token, theme);
        let fora = retangulo(d.x0 - gx, d.y0 + gy, d.w + 2.0 * gx, d.h + 2.0 * gy);
        stroke(scene, &fora, xf, cor, FRAME_PX / um);
        if let Some((cols, rows, viva)) = d.celulas {
            let (cw, ch) = (d.w / f64::from(cols), d.h / f64::from(rows));
            let mut linhas = BezPath::new();
            for c in 0..=cols {
                let x = d.x0 + f64::from(c) * cw;
                linhas.move_to((x, d.y0));
                linhas.line_to((x, d.y0 - d.h));
            }
            for r in 0..=rows {
                let y = d.y0 - f64::from(r) * ch;
                linhas.move_to((d.x0, y));
                linhas.line_to((d.x0 + d.w, y));
            }
            stroke(scene, &linhas, xf, cor, CELL_PX / um);
            if let Some((cx, cy)) = viva {
                let r = retangulo(cx - 0.5 * cw, cy + 0.5 * ch, cw, ch);
                stroke(scene, &r, xf, cor, LIVE_PX / um);
            }
        }
        etiqueta(
            scene,
            text_system,
            &texto,
            xf * Point::new(d.x0 - gx, d.y0 + gy),
            cor,
            theme,
        );
    }
}

/// A etiqueta, em espaço de TELA, apoiada por cima da quina superior-esquerda da moldura (o rótulo de moldura do Figma).
fn etiqueta(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    texto: &str,
    quina: Point,
    fundo: Color,
    theme: Theme,
) {
    let size = TypeToken::Xs.px();
    let w = f64::from(ph2d_editor_core::text_elide::title_elided_width(
        text_system,
        texto,
        size,
    ))
    .min(f64::from(LABEL_MAX_W_PX));
    let h = f64::from(size) + 2.0 * PAD_Y_PX;
    let caixa = Rect::new(quina.x, quina.y - h, quina.x + w + 2.0 * PAD_X_PX, quina.y);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(fundo),
        None,
        &RoundedRect::from_rect(caixa, RADIUS_PX),
    );
    ph2d_editor_core::paint_text(
        text_system,
        scene,
        texto,
        (caixa.x0 + PAD_X_PX) as f32,
        (caixa.y0 + PAD_Y_PX) as f32,
        size,
        LABEL_MAX_W_PX,
        resolve(ColorToken::AccentFg, theme),
    );
}

/// Um retângulo local de quina superior-esquerda `(x0, y0)`, `y` para cima.
fn retangulo(x0: f64, y0: f64, w: f64, h: f64) -> BezPath {
    let mut p = BezPath::new();
    p.move_to((x0, y0));
    p.line_to((x0 + w, y0));
    p.line_to((x0 + w, y0 - h));
    p.line_to((x0, y0 - h));
    p.close_path();
    p
}

fn stroke(scene: &mut VectorScene, path: &BezPath, xf: Affine, color: Color, width: f64) {
    scene.inner_mut().stroke(
        &Stroke::new(width.max(f64::EPSILON)),
        xf,
        &Brush::Solid(color),
        None,
        path,
    );
}

fn resolve(token: ColorToken, theme: Theme) -> Color {
    ph2d_editor_core::paint::token_to_vello(token.resolve(theme))
}

#[cfg(test)]
#[path = "use_frame_tests.rs"]
mod tests;
