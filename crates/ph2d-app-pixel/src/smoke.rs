//! **As cenas de smoke** (`PH2D_PIXEL_SMOKE=n`).
//!
//! 1 — um Pixel Sprite 32 × 32 já em Draw, com uma cara desenhada em QUATRO traços (quatro passos
//! do desfazer, para o `Ctrl+Z` ter o que mostrar). O dono desenha por cima, desfaz, grava e reabre.
//! 2 — P1: um Pixel Sprite 64 × 48 em Draw com uma casa feita SÓ com as ferramentas novas, cada
//! forma um passo do desfazer: paredes (Rectangle), telhado (Polygon), porta (Filled Rectangle de
//! canto redondo), sol (Filled Ellipse), chão (Line), fumo (Curve). As paredes FECHAM — o balde
//! enche a casa sem vazar —, e a Rectangle fica na mão.
//! 3 — P2: um Pixel Sprite 64 × 48 INDEXED com a DB32: céu em degradê (seis azuis), chão, e uma bola
//! laranja sombreada com a rampa 4‥7 da DB32. As quatro amostras da rampa vêm seleccionadas e a tinta
//! Shading na mão: pintar a bola com o esquerdo clareia um degrau, o direito escurece, o céu (fora
//! da rampa) não muda. O Color Mode… e o menu da paleta estão no painel PIXEL.
//! 4 — P2, o Color Mode que SE VÊ: um Pixel Sprite 96 × 64 **RGBA** em Draw, pintado com degradês
//! que nenhuma paleta pequena exprime — o céu do azul fundo ao laranja do pôr do sol (cada linha
//! uma cor), uma esfera iluminada (centenas de cores) e uma faixa arco-íris em baixo — sobre a
//! paleta de fábrica, a **DB32** (32 cores: medido, a do Game Boy apagava os dithers ordenados —
//! 3 % de células entre 2×2 e 4×4, 0 % entre 4×4 e 8×8, contra 10 % e 2 % na DB32 — porque com
//! quatro verdes longe do céu o par do dither é quase sempre a mesma cor). Indexed sem dither dá
//! faixas chapadas; Floyd–Steinberg, grão; Ordered 2×2/4×4/8×8, tramas regulares diferentes; Old,
//! outras tramas; Grayscale, a imagem cinza; `Ctrl+Z` volta à imagem lisa. (A cena 3 é Indexed só
//! com cores da paleta: lá RGBA ↔ Indexed é idêntico byte a byte e nenhum dither muda nada.)
//! 5 — a cena 4 com o diálogo Color Mode ABERTO (Indexed + Floyd–Steinberg escolhidos), para a foto.
//! 6 — a cena 4 já convertida para Indexed com Ordered 4×4 (um passo do desfazer), para a foto.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::{NewSprite, Paint, Stroke};

use crate::PixelApp;
use ph2d_editor_core::pixel_panel::{ColorModeChoice, NewSpriteMode, PixelDither, PixelToGray};

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_PIXEL_SMOKE",
    max_level: 6,
}];

/// O nível armado em `PH2D_PIXEL_SMOKE`.
#[must_use]
pub fn armed_level() -> Option<u32> {
    std::env::var(ROUTERS[0].env)
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
}

/// Os quatro traços da cara (células de um sprite 32 × 32) e a cor de cada um (índice da DB32):
/// olho esquerdo e direito (preto), o sorriso (vermelho), a moldura (azul).
pub const FACE: [(&[(i32, i32)], usize); 4] = [
    (&[(11, 11), (11, 13)], 0),
    (&[(20, 11), (20, 13)], 0),
    (&[(9, 19), (12, 22), (19, 22), (22, 19)], 27),
    (&[(6, 6), (25, 6), (25, 25), (6, 25), (6, 6)], 17),
];

/// Encena o nível `level`. `false` = nível que este roteador não conhece.
pub fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    level: u32,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> bool {
    match level {
        1 => {
            let Ok(bits) = crate::object_add::spawn(sim, app, NewSprite::default(), spawn_root)
            else {
                return false;
            };
            let Some(key) = crate::input::key_of(sim.world(), bits) else {
                return false;
            };
            for (path, color) in FACE {
                if !draw_path(app, key, color, path) {
                    return false;
                }
            }
            app.fg = [0, 0, 0, 255];
            app.bg = [255, 255, 255, 255];
            true
        }
        2 => stage_house(app, sim, spawn_root),
        3 => stage_ball(app, sim, spawn_root),
        4 => stage_gradient(app, sim, spawn_root).is_some(),
        5 => stage_gradient(app, sim, spawn_root).is_some_and(|_| {
            app.abre_color_mode = Some(PHOTO_DIALOG);
            true
        }),
        6 => stage_gradient(app, sim, spawn_root).is_some_and(|key| {
            let c = ColorModeChoice {
                dither: PixelDither::Ordered4,
                ..PHOTO_DIALOG
            };
            app.converte(key, c);
            app.docs[&key].sprite.mode() == ph2d_pixel_core::ColorMode::Indexed
        }),
        _ => false,
    }
}

/// As formas da casa da cena 2: a ferramenta, as células que a definem e a cor (índice da DB32).
type Peca = (crate::Tool, &'static [(i32, i32)], usize);

pub const HOUSE: [Peca; 6] = [
    (crate::Tool::Line, &[(0, 40), (63, 40)], 12),
    (crate::Tool::Rectangle, &[(14, 22), (40, 40)], 4),
    (crate::Tool::Polygon, &[(11, 22), (27, 9), (43, 22)], 27),
    (crate::Tool::FilledRectangle, &[(24, 30), (30, 40)], 3),
    (crate::Tool::FilledEllipse, &[(50, 3), (59, 12)], 8),
    (crate::Tool::Curve, &[(34, 13), (28, 1), (44, 2)], 2),
];

fn stage_house(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> bool {
    let spec = NewSprite {
        width: 64,
        height: 48,
        ..NewSprite::default()
    };
    let Ok(bits) = crate::object_add::spawn(sim, app, spec, spawn_root) else {
        return false;
    };
    let Some(key) = crate::input::key_of(sim.world(), bits) else {
        return false;
    };
    app.options.corner_radius = 2;
    for (tool, pts, color) in HOUSE {
        if !draw_shape(app, key, tool, pts, color) {
            return false;
        }
    }
    app.options.corner_radius = 0;
    app.tool = crate::Tool::Rectangle;
    app.fg = [0, 0, 0, 255];
    app.bg = [255, 255, 255, 255];
    true
}

/// Uma forma da cena pela MESMA lei do gesto ([`crate::gesture::reshape`]), como UM passo.
fn draw_shape(
    app: &mut PixelApp,
    key: u32,
    tool: crate::Tool,
    pts: &[(i32, i32)],
    color: usize,
) -> bool {
    use crate::gesture::{Kind, Mods, paint, reshape};
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let s = &doc.sprite;
    let Some(&rgba) = s.palette().get(color) else {
        return false;
    };
    let kind = match (tool, pts) {
        (crate::Tool::Polygon, _) => Kind::Polygon(pts.to_vec()),
        (crate::Tool::Curve, [a, c, b]) => Kind::Curve {
            from: *a,
            to: *b,
            ctrl: Some(*c),
        },
        (_, [a, b]) => Kind::Box { from: *a, to: *b },
        _ => return false,
    };
    let mut st = Stroke::new(
        doc.layer,
        doc.frame,
        s.value_of_rgba(rgba),
        paint(&app.options, doc),
    );
    reshape(&mut st, s, tool, &kind, &app.options, Mods::default());
    let Some(op) = st.op(s) else {
        return false;
    };
    app.commit(key, op).is_ok()
}

/// Um traço pela `path` com a cor `color` da paleta, como UM passo do desfazer.
fn draw_path(app: &mut PixelApp, key: u32, color: usize, path: &[(i32, i32)]) -> bool {
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let s = &doc.sprite;
    let Some(&rgba) = s.palette().get(color) else {
        return false;
    };
    let Some(&(x0, y0)) = path.first() else {
        return false;
    };
    let mut st = Stroke::begin(
        s,
        doc.layer,
        doc.frame,
        s.value_of_rgba(rgba),
        Paint::default(),
        (x0, y0),
    );
    for &(x, y) in &path[1..] {
        st.to(s, x, y);
    }
    let Some(op) = st.op(s) else {
        return false;
    };
    app.commit(key, op).is_ok()
}

/// A rampa da bola da cena 3 (índices da DB32, do escuro ao claro e contíguos: o Shading anda em
/// ordem de PALETA).
pub const BALL_RAMP: [usize; 4] = [4, 5, 6, 7];
/// Os azuis do céu, de cima para baixo (índices contíguos da DB32).
pub const SKY: [usize; 6] = [15, 16, 17, 18, 19, 20];
/// O chão.
pub const GROUND: usize = 12;
/// O centro e o raio da bola.
pub const BALL: ((i32, i32), i32) = ((32, 26), 9);

/// O degrau da rampa de cada célula da bola: a luz vem de cima à esquerda.
fn ball_step(x: i32, y: i32) -> Option<usize> {
    let ((cx, cy), r) = BALL;
    let (dx, dy) = (x - cx, y - cy);
    if dx * dx + dy * dy > r * r {
        return None;
    }
    let (lx, ly) = (x - (cx - 3), y - (cy - 3));
    let d = lx * lx + ly * ly;
    Some(match d {
        0..=6 => 3,
        7..=30 => 2,
        31..=110 => 1,
        _ => 0,
    })
}

fn stage_ball(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> bool {
    let spec = NewSprite {
        width: 64,
        height: 48,
        mode: ph2d_pixel_core::ColorMode::Indexed,
        ..NewSprite::default()
    };
    let Ok(bits) = crate::object_add::spawn(sim, app, spec, spawn_root) else {
        return false;
    };
    let Some(key) = crate::input::key_of(sim.world(), bits) else {
        return false;
    };
    let (w, h) = (64, 48);
    let banda = 36 / SKY.len() as i32;
    let mut pecas: Vec<Vec<((i32, i32), usize)>> = vec![Vec::new(); 3];
    for y in 0..h {
        for x in 0..w {
            if y < 36 {
                pecas[0].push(((x, y), SKY[(y / banda) as usize]));
            } else {
                pecas[1].push(((x, y), GROUND));
            }
            if let Some(k) = ball_step(x, y) {
                pecas[2].push(((x, y), BALL_RAMP[k]));
            }
        }
    }
    for p in &pecas {
        if !draw_cells(app, key, p) {
            return false;
        }
    }
    app.select(BALL_RAMP);
    app.options.ink = ph2d_editor_core::pixel_panel::PixelInk::Shading;
    app.tool = crate::Tool::Pencil;
    app.options.size = 3;
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    app.fg = doc.sprite.palette()[BALL_RAMP[1]];
    app.bg = doc.sprite.palette()[BALL_RAMP[0]];
    true
}

/// Células com os índices dados (um sprite Indexed), como UM passo do desfazer.
fn draw_cells(app: &mut PixelApp, key: u32, cells: &[((i32, i32), usize)]) -> bool {
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let s = &doc.sprite;
    let rect = cells
        .iter()
        .fold(ph2d_pixel_core::Rect::default(), |r, &((x, y), _)| {
            r.union(ph2d_pixel_core::Rect::new(x, y, 1, 1))
        });
    let mut pixels = s.read_rect((doc.layer, doc.frame), rect);
    for &((x, y), i) in cells {
        let Ok(v) = u8::try_from(i) else {
            return false;
        };
        pixels[(y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize] = v;
    }
    let bbox = s
        .cel((doc.layer, doc.frame))
        .map_or(rect, |c| c.bbox.union(rect));
    let op = ph2d_pixel_core::Op::Patch(ph2d_pixel_core::Patch {
        layer: doc.layer,
        frame: doc.frame,
        bbox: Some(bbox),
        rect,
        pixels,
    });
    app.commit(key, op).is_ok()
}

/// A escolha com que a cena 5 abre o diálogo.
const PHOTO_DIALOG: ColorModeChoice = ColorModeChoice {
    target: NewSpriteMode::Indexed,
    dither: PixelDither::FloydSteinberg,
    gray: PixelToGray::Luminance,
};

/// O tamanho da cena 4.
pub const GRADIENT_SIZE: (u32, u32) = (96, 64);
/// As linhas da faixa arco-íris (as últimas da imagem).
const RAINBOW_ROWS: u32 = 10;
/// O centro e o raio da esfera.
const SPHERE: ((f32, f32), f32) = ((48.0, 26.0), 20.0);

/// A cor da célula `(x, y)` da cena 4.
#[must_use]
pub fn gradient_rgba(x: u32, y: u32) -> [u8; 4] {
    let (w, h) = GRADIENT_SIZE;
    let byte = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    if y >= h - RAINBOW_ROWS {
        let [r, g, b] = hue(x as f32 / w as f32);
        return [byte(r * 255.0), byte(g * 255.0), byte(b * 255.0), 255];
    }
    let ((cx, cy), r) = SPHERE;
    let (dx, dy) = ((x as f32 + 0.5 - cx) / r, (y as f32 + 0.5 - cy) / r);
    let d2 = dx * dx + dy * dy;
    if d2 <= 1.0 {
        // Lambert + um brilho, a luz de cima à esquerda e à frente.
        let nz = (1.0 - d2).sqrt();
        let (lx, ly, lz) = (-0.48_f32, -0.6_f32, 0.64_f32);
        let lambert = (dx * lx + dy * ly + nz * lz).max(0.0);
        let spec = lambert.powi(24);
        let k = 0.12 + 0.88 * lambert;
        let base = [90.0_f32, 170.0, 235.0];
        return [
            byte(base[0] * k + 255.0 * spec),
            byte(base[1] * k + 255.0 * spec),
            byte(base[2] * k + 255.0 * spec),
            255,
        ];
    }
    let t = y as f32 / (h - RAINBOW_ROWS - 1) as f32;
    let top = [18.0_f32, 28.0, 96.0];
    let bottom = [252.0_f32, 150.0, 58.0];
    let mix = |i: usize| byte(top[i] + (bottom[i] - top[i]) * t + x as f32 * 0.25);
    [mix(0), mix(1), mix(2), 255]
}

/// O arco-íris: matiz `t ∈ [0, 1)` com saturação e valor cheios.
fn hue(t: f32) -> [f32; 3] {
    let h = (t * 6.0).rem_euclid(6.0);
    let f = h - h.floor();
    match h as u32 {
        0 => [1.0, f, 0.0],
        1 => [1.0 - f, 1.0, 0.0],
        2 => [0.0, 1.0, f],
        3 => [0.0, 1.0 - f, 1.0],
        4 => [f, 0.0, 1.0],
        _ => [1.0, 0.0, 1.0 - f],
    }
}

/// A cena 4: a imagem como UM passo, sobre a paleta de fábrica (DB32). Devolve a chave.
fn stage_gradient(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u32> {
    let (w, h) = GRADIENT_SIZE;
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ph2d_pixel_core::ColorMode::Rgba,
        ..NewSprite::default()
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let doc = app.docs.get(&key)?;
    let rect = ph2d_pixel_core::Rect::new(0, 0, w, h);
    let pixels: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).flat_map(move |x| gradient_rgba(x, y)))
        .collect();
    let op = ph2d_pixel_core::Op::Patch(ph2d_pixel_core::Patch {
        layer: doc.layer,
        frame: doc.frame,
        bbox: Some(rect),
        rect,
        pixels,
    });
    app.commit(key, op).ok()?;
    app.tool = crate::Tool::Pencil;
    app.fg = [255, 255, 255, 255];
    app.bg = [0, 0, 0, 255];
    Some(key)
}
