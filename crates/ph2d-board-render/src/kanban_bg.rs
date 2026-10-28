//! ⭐ **O fundo de um Kanban pintado** — UMA lei para a aba (`ph2d-app-kanban`) e para o Kanban no
//! canvas do Quadro (o nível baixo de [`crate::kanban`]): a cor da paleta, a cor livre, os nove
//! gradientes do Trello (pares de tokens da paleta, seguem o tema), a imagem recortada como `cover` e
//! os padrões desenhados a VECTOR (nítidos a qualquer zoom).
//!
//! Um padrão é UM caminho com os ladrilhos que tocam o rectângulo, preenchido de uma vez. De longe
//! (o ladrilho abaixo de [`PATTERN_MIN_TILE_PX`] no ecrã) vira a cor média — base e traço misturados
//! pela fracção que o traço cobre —, que é o que o olho vê ali e não custa nada.
//!
//! [`kanban_bg_tone`] diz quão claro o fundo é: o véu do cabeçalho ([`kanban_header_veil`]) escurece
//! o que for preciso para o título branco se ler a 4,5:1, e o chão claro
//! ([`kanban_floor_is_light`]) troca o botão translúcido pelo da lista.

use std::f64::consts::SQRT_2;
use std::sync::Arc;

use ph2d_kanban_model::{Background, Hue, Pattern, PatternKind, Shade, Tint};
use ph2d_tokens::{ColorToken as T, Spacing, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color, Gradient, ImageQuality, Point, Rect, Shape, VectorScene,
    VelloBlend,
};

use crate::token;

/// O ladrilho de um padrão, em px do Kanban.
fn tile() -> f64 {
    f64::from(Spacing::Xl2.px())
}

/// A espessura do traço de um padrão.
fn line_w() -> f64 {
    f64::from(Spacing::Xxs.px())
}

/// O raio de um ponto do padrão `Dots`.
fn dot_r() -> f64 {
    f64::from(Spacing::Sm.px()) / 2.0
}

/// Abaixo disto (px de ecrã por ladrilho) o padrão é a cor média: o traço já não se distingue e
/// cada ladrilho custaria o mesmo que um legível.
pub const PATTERN_MIN_TILE_PX: f64 = 6.0;

/// Os gradientes do Trello, pela ordem do seletor (o índice de `Background::Gradient`; os nomes
/// estão em `ph2d-kanban-import`): snow, ocean, crystal, rainbow, peach, flower, earth, alien,
/// volcano. Cada paragem é um token da paleta, para o gradiente seguir o tema.
pub const GRADIENTS: [&[T]; 9] = [
    &[T::KanbanSkySubtle, T::KanbanBlackSubtle],
    &[T::KanbanBlueBold, T::KanbanSkyBold],
    &[T::KanbanBlueBold, T::KanbanPurpleBold],
    &[T::KanbanPinkBold, T::KanbanOrangeBold, T::KanbanYellowBold],
    &[T::KanbanRedBold, T::KanbanOrangeBold],
    &[T::KanbanPurpleBold, T::KanbanPinkBold],
    &[T::KanbanGreenBold, T::KanbanSkyBold],
    &[T::KanbanBlackBold, T::KanbanGreenBold],
    &[T::KanbanBlackBold, T::KanbanRedBold],
];

/// Uma imagem de fundo já descodificada (RGBA reto) e a luminância mais clara que tem.
#[derive(Clone, Copy, Debug)]
pub struct BgImage<'a> {
    pub rgba: &'a Arc<Vec<u8>>,
    pub size: [u32; 2],
    /// A luminância relativa (WCAG) mais clara da parte que importa (a faixa do cabeçalho), `0..=1`.
    pub brightest: f32,
    /// A luminância média da imagem.
    pub mean: f32,
}

/// O que pintar e onde.
#[derive(Clone, Copy, Debug)]
pub struct KanbanBg<'a> {
    pub bg: Background,
    /// O rectângulo do fundo, em px do Kanban.
    pub rect: Rect,
    /// Px do Kanban → a cena.
    pub to_scene: Affine,
    /// Px de ECRÃ por px do Kanban (o zoom final, para o nível do padrão).
    pub screen_scale: f64,
    /// A imagem de `Background::Image`, se já está pronta (senão, a cor de espera).
    pub image: Option<BgImage<'a>>,
}

fn hue_index(h: Hue) -> usize {
    Hue::ALL.iter().position(|x| *x == h).unwrap_or(0)
}

fn label(theme: Theme, hue: Hue, shade: Shade) -> Color {
    let s = Shade::ALL.iter().position(|x| *x == shade).unwrap_or(0);
    token(ph2d_tokens::kanban::LABELS[s][hue_index(hue)], theme)
}

/// A cor de um matiz como FUNDO: o azul do Trello; outro matiz, a etiqueta forte dele.
fn hue_bg(theme: Theme, h: Hue) -> Color {
    if h == Hue::Blue {
        token(T::KanbanBoardBg, theme)
    } else {
        label(theme, h, Shade::Bold)
    }
}

/// ⭐ A cor de uma `Tint` no papel `ink` (o traço de um padrão: o tom subtil do matiz) ou de fundo.
#[must_use]
pub fn kanban_tint(theme: Theme, t: Tint, ink: bool) -> Color {
    match (t, ink) {
        (Tint::Rgb(c), _) => Color::from_rgb8(c.0[0], c.0[1], c.0[2]),
        (Tint::Hue(h), true) => label(theme, h, Shade::Subtle),
        (Tint::Hue(h), false) => hue_bg(theme, h),
    }
}

/// As paragens do gradiente `i` (um índice fora da lista = o primeiro).
#[must_use]
pub fn kanban_gradient_stops(theme: Theme, i: u16) -> Vec<Color> {
    GRADIENTS
        .get(usize::from(i))
        .copied()
        .unwrap_or(GRADIENTS[0])
        .iter()
        .map(|t| token(*t, theme))
        .collect()
}

/// A fracção do ladrilho que o traço de `kind` cobre (a mistura do padrão visto de longe).
#[must_use]
pub fn pattern_coverage(kind: PatternKind) -> f32 {
    let (t, w, r) = (tile(), line_w(), dot_r());
    let line = w / t;
    let c = match kind {
        PatternKind::Dots => std::f64::consts::PI * r * r / (t * t),
        PatternKind::Grid => 2.0 * line - line * line,
        PatternKind::Lines | PatternKind::Waves => line * 1.1,
        PatternKind::Diagonal => line * SQRT_2,
        PatternKind::CrossHatch => 2.0 * line * SQRT_2,
        PatternKind::Checks => 0.5,
        PatternKind::Triangles => 0.125,
    };
    #[allow(clippy::cast_possible_truncation)]
    let c = c as f32;
    c
}

/// ⭐ A cor plana que representa o fundo (a média): o que se pinta sem imagem, de muito longe, e por
/// baixo de um botão opaco.
#[must_use]
pub fn kanban_board_bg(theme: Theme, bg: Background) -> Color {
    match bg {
        Background::Color(h) => hue_bg(theme, h),
        Background::Custom(c) => kanban_tint(theme, Tint::Rgb(c), false),
        Background::Gradient(i) => {
            let s = kanban_gradient_stops(theme, i);
            s[0].lerp_rect(s[s.len() - 1], 0.5)
        }
        Background::Image(_) => token(T::KanbanBoardBg, theme),
        Background::Pattern(p) => pattern_mean(theme, p),
    }
}

fn pattern_mean(theme: Theme, p: Pattern) -> Color {
    kanban_tint(theme, p.base, false).lerp_rect(
        kanban_tint(theme, p.ink, true),
        pattern_coverage(p.kind),
    )
}

/// A luminância relativa (WCAG) de uma cor.
#[must_use]
pub fn luminance(c: Color) -> f32 {
    let [r, g, b, _] = c.to_rgba8().to_u8_array();
    let lin = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

/// Quão claro é um fundo: a luminância média e a da parte MAIS clara.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BgTone {
    pub mean: f32,
    pub brightest: f32,
}

/// ⭐ O tom de `bg` (com a imagem, se já está pronta).
#[must_use]
pub fn kanban_bg_tone(theme: Theme, bg: Background, image: Option<&BgImage<'_>>) -> BgTone {
    let flat = luminance(kanban_board_bg(theme, bg));
    let mean = match (bg, image) {
        (Background::Image(_), Some(i)) => i.mean,
        _ => flat,
    };
    let brightest = match bg {
        Background::Gradient(i) => kanban_gradient_stops(theme, i)
            .into_iter()
            .map(luminance)
            .fold(0.0, f32::max),
        Background::Pattern(p) => luminance(kanban_tint(theme, p.base, false))
            .max(luminance(kanban_tint(theme, p.ink, true))),
        Background::Image(_) => image.map_or(flat, |i| i.brightest),
        Background::Color(_) | Background::Custom(_) => flat,
    };
    BgTone { mean, brightest }
}

/// O contraste WCAG entre duas luminâncias.
fn contrast(a: f32, b: f32) -> f32 {
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// O contraste que o texto do cabeçalho e do chão tem de ter (WCAG AA, texto normal).
const MIN_CONTRAST: f32 = 4.5;

/// ⭐ O véu do cabeçalho sobre `tone`: o token, com o alfa subido até o título (`KanbanHeaderFg`) se
/// ler a 4,5:1 sobre a parte mais clara do fundo (composto em luz linear; o Vello compõe em sRGB,
/// que dá mais escuro — a conta é do lado seguro).
#[must_use]
pub fn kanban_header_veil(theme: Theme, tone: BgTone) -> Color {
    let veil = token(T::KanbanHeaderBg, theme);
    let floor = veil.components[3];
    let ink = luminance(token(T::KanbanHeaderFg, theme));
    let v = luminance(veil);
    let goal = (ink + 0.05) / MIN_CONTRAST - 0.05;
    let b = tone.brightest;
    let need = if b <= goal || b <= v {
        0.0
    } else {
        (b - goal) / (b - v)
    };
    veil.with_alpha(need.clamp(floor, 1.0))
}

/// O véu do cabeçalho sobre um chão LISO `under` (os botões da edição rápida, pintados sobre o fundo
/// e a sombra): a mesma lei de contraste de [`kanban_header_veil`].
#[must_use]
pub fn kanban_header_veil_over(theme: Theme, under: Color) -> Color {
    let l = luminance(under);
    kanban_header_veil(
        theme,
        BgTone {
            mean: l,
            brightest: l,
        },
    )
}

/// ⭐ O chão do quadro é claro: o texto branco do cabeçalho não se lê sobre a cor média a 4,5:1
/// (o «Add another list» passa a ser um botão opaco da cor da lista).
#[must_use]
pub fn kanban_floor_is_light(theme: Theme, tone: BgTone) -> bool {
    let ink = luminance(token(T::KanbanHeaderFg, theme));
    contrast(ink, tone.mean) < MIN_CONTRAST
}

/// O destino da imagem de `size` que enche `r` sem deformar, centrada.
fn cover(size: [u32; 2], r: Rect) -> Affine {
    let (w, h) = (f64::from(size[0].max(1)), f64::from(size[1].max(1)));
    let s = (r.width() / w).max(r.height() / h);
    let (dw, dh) = (w * s, h * s);
    Affine::translate((
        r.x0 + (r.width() - dw) / 2.0,
        r.y0 + (r.height() - dh) / 2.0,
    )) * Affine::scale(s)
}

/// ⭐ Pinta o fundo.
pub fn paint_kanban_bg(scene: &mut VectorScene, theme: Theme, p: &KanbanBg<'_>) {
    let r = p.rect;
    if r.width() <= 0.0 || r.height() <= 0.0 {
        return;
    }
    let path = r.to_path(0.1);
    match p.bg {
        Background::Color(_) | Background::Custom(_) => {
            let c = kanban_board_bg(theme, p.bg);
            scene.fill_path(&path, &Brush::Solid(c), p.to_scene);
        }
        Background::Gradient(i) => {
            let stops = kanban_gradient_stops(theme, i);
            let g = Gradient::new_linear(Point::new(r.x0, r.y0), Point::new(r.x1, r.y1))
                .with_stops(stops.as_slice());
            scene.fill_path(&path, &Brush::Gradient(g), p.to_scene);
        }
        Background::Image(_) => match p.image {
            Some(img) => {
                scene.fill_path(
                    &path,
                    &Brush::Solid(token(T::KanbanBoardBg, theme)),
                    p.to_scene,
                );
                scene.push_layer_shape(VelloBlend::default(), 1.0, p.to_scene, &r);
                scene.draw_image_rgba_transformed(
                    img.rgba,
                    img.size[0],
                    img.size[1],
                    p.to_scene * cover(img.size, r),
                    ImageQuality::Medium,
                );
                scene.pop_layer();
            }
            None => {
                let c = token(T::KanbanBoardBg, theme);
                scene.fill_path(&path, &Brush::Solid(c), p.to_scene);
            }
        },
        Background::Pattern(pat) => {
            let base = kanban_tint(theme, pat.base, false);
            if tile() * p.screen_scale < PATTERN_MIN_TILE_PX {
                let c = pattern_mean(theme, pat);
                scene.fill_path(&path, &Brush::Solid(c), p.to_scene);
                return;
            }
            scene.fill_path(&path, &Brush::Solid(base), p.to_scene);
            let ink = kanban_tint(theme, pat.ink, true);
            scene.push_layer_shape(VelloBlend::default(), 1.0, p.to_scene, &r);
            scene.fill_path(&pattern_path(pat.kind, r), &Brush::Solid(ink), p.to_scene);
            scene.pop_layer();
        }
    }
}

/// ⭐ O traço de um padrão sobre `r` (os ladrilhos que lhe tocam, ancorados no canto dele), como UM
/// caminho para UM preenchimento.
#[must_use]
pub fn pattern_path(kind: PatternKind, r: Rect) -> BezPath {
    let (t, w) = (tile(), line_w());
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (cols, rows) = (
        (r.width() / t).ceil() as usize + 1,
        (r.height() / t).ceil() as usize + 1,
    );
    #[allow(clippy::cast_precision_loss)]
    let at = |i: usize, j: usize| (r.x0 + i as f64 * t, r.y0 + j as f64 * t);
    let mut path = BezPath::new();
    match kind {
        PatternKind::Dots => {
            for j in 0..rows {
                for i in 0..cols {
                    let (x, y) = at(i, j);
                    add(&mut path, &Circle::new((x + t / 2.0, y + t / 2.0), dot_r()));
                }
            }
        }
        PatternKind::Grid | PatternKind::Lines => {
            for j in 0..rows {
                let (_, y) = at(0, j);
                add(&mut path, &Rect::new(r.x0, y, r.x1, y + w));
            }
            if kind == PatternKind::Grid {
                for i in 0..cols {
                    let (x, _) = at(i, 0);
                    add(&mut path, &Rect::new(x, r.y0, x + w, r.y1));
                }
            }
        }
        PatternKind::Diagonal | PatternKind::CrossHatch => {
            let h = r.height();
            let d = w * SQRT_2;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = ((r.width() + h) / t).ceil() as usize + 1;
            for k in 0..n {
                #[allow(clippy::cast_precision_loss)]
                let c = r.x0 + k as f64 * t;
                add(&mut path, &quad([(c, r.y0), (c + d, r.y0), (c + d - h, r.y1), (c - h, r.y1)]));
                if kind == PatternKind::CrossHatch {
                    let c = c - h;
                    add(&mut path, &quad([(c, r.y0), (c + d, r.y0), (c + d + h, r.y1), (c + h, r.y1)]));
                }
            }
        }
        PatternKind::Checks => {
            for j in 0..rows {
                for i in 0..cols {
                    let (x, y) = at(i, j);
                    let half = t / 2.0;
                    add(&mut path, &Rect::new(x, y, x + half, y + half));
                    add(&mut path, &Rect::new(x + half, y + half, x + t, y + t));
                }
            }
        }
        PatternKind::Waves => {
            // Cada onda é uma faixa fechada (a curva e a mesma descida `w` de volta): um traço
            // expandido custava 4 ms a ecrã inteiro, a faixa custa o de um caminho.
            let amp = t / 6.0;
            #[allow(clippy::cast_precision_loss)]
            let x_at = |i: usize| r.x0 + i as f64 * t;
            for j in 0..rows {
                let (_, y) = at(0, j);
                let y = y + t / 2.0 - w / 2.0;
                let mut band = BezPath::new();
                band.move_to((r.x0, y));
                for i in 0..cols {
                    let x = x_at(i);
                    band.quad_to((x + t / 4.0, y - amp), (x + t / 2.0, y));
                    band.quad_to((x + 3.0 * t / 4.0, y + amp), (x + t, y));
                }
                band.line_to((x_at(cols), y + w));
                for i in (0..cols).rev() {
                    let x = x_at(i);
                    band.quad_to((x + 3.0 * t / 4.0, y + w + amp), (x + t / 2.0, y + w));
                    band.quad_to((x + t / 4.0, y + w - amp), (x, y + w));
                }
                band.close_path();
                add(&mut path, &band);
            }
        }
        PatternKind::Triangles => {
            for j in 0..rows {
                for i in 0..cols {
                    let (x, y) = at(i, j);
                    add(&mut path, &tri([
                        (x + t / 4.0, y + 3.0 * t / 4.0),
                        (x + 3.0 * t / 4.0, y + 3.0 * t / 4.0),
                        (x + t / 2.0, y + t / 4.0),
                    ]));
                }
            }
        }
    }
    path
}

fn add(path: &mut BezPath, s: &impl Shape) {
    path.extend(s.path_elements(0.1));
}

fn quad(p: [(f64, f64); 4]) -> BezPath {
    let mut b = BezPath::new();
    b.move_to(p[0]);
    for q in &p[1..] {
        b.line_to(*q);
    }
    b.close_path();
    b
}

fn tri(p: [(f64, f64); 3]) -> BezPath {
    let mut b = BezPath::new();
    b.move_to(p[0]);
    b.line_to(p[1]);
    b.line_to(p[2]);
    b.close_path();
    b
}

#[cfg(test)]
#[path = "kanban_bg_tests.rs"]
mod tests;
