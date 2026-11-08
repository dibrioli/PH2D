//! **As preferências do documento no ecrã** (P12 item P, `P12_plano.md` §9.6): a grade, a grade de
//! píxel e o xadrez lidos de [`PixelPrefs`]. As cores são dados do documento (bytes), não tokens.
//!
//! **A regra do `Auto` da opacidade** (O18 `Auto`; a NOSSA — o Trial não foi medido por zoom): com `s` = píxeis de ecrã entre duas linhas vizinhas, a
//! opacidade desenhada é `round(opacity · clamp((s − 4) / 12, 0, 1))` — invisível a 4 px ou menos
//! (as linhas cobririam a arte), inteira a partir de 16 px. Sem `Auto`, a opacidade é a gravada.
//! Abaixo de [`MIN_LINE_SPACING_PX`] nenhuma grade se desenha, com ou sem `Auto`.

use std::sync::Arc;

use ph2d_pixel_core::{Checker, Grid, PixelGrid, PixelPrefs};
use ph2d_vector::{
    Affine, BezPath, Color, ImageQuality, Point, StableImage, Stroke as Pen, VectorScene,
};

/// O espaçamento de ecrã abaixo do qual a grade não se desenha (cobriria cada píxel de ecrã).
pub const MIN_LINE_SPACING_PX: f64 = 2.0;
/// Onde o `Auto` começa a mostrar a grade e onde ela fica inteira (píxeis de ecrã entre linhas).
pub const AUTO_FADE_PX: (f64, f64) = (4.0, 16.0);
/// A espessura das linhas, em píxeis de ecrã (a do Trial: uma linha de ecrã).
pub const GRID_LINE_PX: f64 = 1.0;

/// A opacidade desenhada de uma grade cujas linhas distam `spacing_px` no ecrã.
#[must_use]
pub fn line_alpha(opacity: u8, auto: bool, spacing_px: f64) -> u8 {
    if !auto {
        return opacity;
    }
    let (a, b) = AUTO_FADE_PX;
    let f = ((spacing_px - a) / (b - a)).clamp(0.0, 1.0);
    (f64::from(opacity) * f).round() as u8
}

/// Uma grade pronta a desenhar: os segmentos em coordenadas de ECRÃ e a cor RGBA (alfa já com a
/// opacidade).
#[derive(Debug, Clone, PartialEq)]
pub struct GridLines {
    pub segments: Vec<((f64, f64), (f64, f64))>,
    pub rgba: [u8; 4],
}

fn first(phase: i32, step: u32) -> u32 {
    phase.rem_euclid(step.max(1) as i32) as u32
}

fn lines(
    (w, h): (u32, u32),
    (sx, sy): (u32, u32),
    (fx, fy): (i32, i32),
    to_screen: Affine,
) -> Vec<((f64, f64), (f64, f64))> {
    let (fw, fh) = (f64::from(w), f64::from(h));
    let p = |x: f64, y: f64| {
        let q = to_screen * Point::new(x, y);
        (q.x, q.y)
    };
    let mut out = Vec::new();
    for x in (first(fx, sx)..=w).step_by(sx.max(1) as usize) {
        out.push((p(f64::from(x), 0.0), p(f64::from(x), fh)));
    }
    for y in (first(fy, sy)..=h).step_by(sy.max(1) as usize) {
        out.push((p(0.0, f64::from(y)), p(fw, f64::from(y))));
    }
    out
}

fn with_alpha(c: [u8; 4], a: u8) -> [u8; 4] {
    [
        c[0],
        c[1],
        c[2],
        ((u16::from(c[3]) * u16::from(a) + 127) / 255) as u8,
    ]
}

/// A grade do documento (`Visible Grid`) sobre um sprite `w × h`; `None` desligada ou invisível
/// neste zoom.
#[must_use]
pub fn grid_lines(size: (u32, u32), g: &Grid, to_screen: Affine) -> Option<GridLines> {
    let ppc = crate::px_per_cell(to_screen);
    let spacing = ppc * f64::from(g.w.min(g.h));
    let a = line_alpha(g.opacity, g.auto_opacity, spacing);
    (g.visible && spacing >= MIN_LINE_SPACING_PX && a > 0).then(|| GridLines {
        segments: lines(size, (g.w, g.h), (g.x, g.y), to_screen),
        rgba: with_alpha(g.color, a),
    })
}

/// A grade de píxel (uma linha entre cada célula).
#[must_use]
pub fn pixel_grid_lines(size: (u32, u32), g: &PixelGrid, to_screen: Affine) -> Option<GridLines> {
    let spacing = crate::px_per_cell(to_screen);
    let a = line_alpha(g.opacity, g.auto_opacity, spacing);
    (g.visible && spacing >= MIN_LINE_SPACING_PX && a > 0).then(|| GridLines {
        segments: lines(size, (1, 1), (0, 0), to_screen),
        rgba: with_alpha(g.color, a),
    })
}

/// Desenha uma [`GridLines`].
pub fn draw_lines(scene: &mut VectorScene, l: &GridLines) {
    let mut path = BezPath::new();
    for &(a, b) in &l.segments {
        path.move_to(a);
        path.line_to(b);
    }
    let [r, g, b, a] = l.rgba;
    scene.inner_mut().stroke(
        &Pen::new(GRID_LINE_PX),
        Affine::IDENTITY,
        Color::from_rgba8(r, g, b, a),
        None,
        &path,
    );
}

/// A grade de píxel e depois a grade do documento (a do documento por cima, como no Trial).
pub fn draw_doc_grids(scene: &mut VectorScene, size: (u32, u32), p: &PixelPrefs, m: Affine) {
    for l in [
        pixel_grid_lines(size, &p.pixel_grid, m),
        grid_lines(size, &p.grid, m),
    ]
    .into_iter()
    .flatten()
    {
        draw_lines(scene, &l);
    }
}

/// O xadrez de um sprite `w × h` a `ppc` píxeis de ecrã por célula: casas em colunas × linhas e o
/// lado de uma casa em células do sprite. `Apply Zoom` = `w×h` células do sprite (crescem com o
/// zoom); sem ele, `w×h` píxeis do ECRÃ (`w / ppc` células).
#[must_use]
pub fn checker_layout(c: &Checker, (w, h): (u32, u32), ppc: f64) -> (u32, u32, f64, f64) {
    let (kx, ky) = if c.apply_zoom {
        (f64::from(c.w), f64::from(c.h))
    } else {
        let ppc = ppc.max(f64::MIN_POSITIVE);
        (f64::from(c.w) / ppc, f64::from(c.h) / ppc)
    };
    let n = |s: u32, k: f64| ((f64::from(s) / k).ceil() as u32).max(1);
    (n(w, kx), n(h, ky), kx, ky)
}

/// O xadrez do documento, em cache (refaz-se quando as casas ou as cores mudam).
#[derive(Clone, Default)]
pub struct DocChecker(Option<((u32, u32, [u8; 4], [u8; 4]), StableImage)>);

impl DocChecker {
    /// O xadrez `c` por baixo de um sprite `w × h` posto por `to_screen`.
    pub fn draw_under(
        &mut self,
        scene: &mut VectorScene,
        (w, h): (u32, u32),
        c: &Checker,
        to_screen: Affine,
    ) {
        let (cols, rows, kx, ky) = checker_layout(c, (w, h), crate::px_per_cell(to_screen));
        let key = (cols, rows, c.color_a, c.color_b);
        if self.0.as_ref().is_none_or(|(k, _)| *k != key) {
            let mut px = Vec::with_capacity(cols as usize * rows as usize * 4);
            for y in 0..rows {
                for x in 0..cols {
                    px.extend_from_slice(if (x + y) % 2 == 0 {
                        &c.color_b
                    } else {
                        &c.color_a
                    });
                }
            }
            self.0 = StableImage::from_rgba(Arc::new(px), cols, rows).map(|img| (key, img));
        }
        let Some((_, img)) = &self.0 else {
            return;
        };
        let clip = ph2d_vector::Rect::new(0.0, 0.0, f64::from(w), f64::from(h));
        scene
            .inner_mut()
            .push_clip_layer(ph2d_vector::Fill::NonZero, to_screen, &clip);
        scene.draw_stable_image_transformed(
            img,
            to_screen * Affine::scale_non_uniform(kx, ky),
            ImageQuality::Low,
        );
        scene.inner_mut().pop_layer();
    }
}

/// O ficheiro dos valores para documentos NOVOS (`Set as Defaults`): `~/.ph2d/pixel-prefs.txt`,
/// as linhas `pixel.*` de [`PixelPrefs::to_lines`]. Não é o `prefs.txt` da shell: o `serialize`
/// dela reescreve o ficheiro só com as chaves que conhece (apagaria as nossas).
#[must_use]
pub fn defaults_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".ph2d/pixel-prefs.txt"))
}

/// Os valores para documentos novos guardados em `p` (ausente = os de fábrica).
#[must_use]
pub fn load_defaults_from(p: &std::path::Path) -> PixelPrefs {
    PixelPrefs::from_lines(std::fs::read_to_string(p).unwrap_or_default().lines())
}

/// Os valores para documentos novos do ficheiro do utilizador.
#[must_use]
pub fn load_defaults() -> PixelPrefs {
    defaults_path().map_or_else(PixelPrefs::default, |p| load_defaults_from(&p))
}

/// Grava `prefs` como valores para documentos novos em `p` (atómico: irmão temporário + `rename`).
pub fn save_defaults_to(p: &std::path::Path, prefs: &PixelPrefs) -> std::io::Result<()> {
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut tmp = p.as_os_str().to_owned();
    tmp.push(".tmp");
    std::fs::write(&tmp, prefs.to_lines().join("\n") + "\n")?;
    std::fs::rename(tmp, p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grade_do_documento_segue_w_h_x_y_cor_e_opacidade_no_zoom() {
        // 32×16 a zoom 4: grade 8×4 com origem (2, 1), vermelha, opacidade 200 sem Auto.
        let g = Grid {
            x: 2,
            y: 1,
            w: 8,
            h: 4,
            color: [255, 0, 0, 255],
            opacity: 200,
            auto_opacity: false,
            visible: true,
        };
        let m = Affine::scale(4.0);
        let l = grid_lines((32, 16), &g, m).expect("visível");
        let xs: Vec<f64> = l
            .segments
            .iter()
            .filter(|(a, b)| a.0 == b.0)
            .map(|(a, _)| a.0)
            .collect();
        let ys: Vec<f64> = l
            .segments
            .iter()
            .filter(|(a, b)| a.1 == b.1)
            .map(|(a, _)| a.1)
            .collect();
        assert_eq!(xs, [8.0, 40.0, 72.0, 104.0]);
        assert_eq!(ys, [4.0, 20.0, 36.0, 52.0]);
        assert_eq!(l.rgba, [255, 0, 0, 200]);
        // Auto: espaçamento 16 px = inteira; a zoom 1 (4 px) some; a meio (zoom 1,25 → 5 px) 1/12.
        let auto = Grid {
            auto_opacity: true,
            ..g
        };
        assert_eq!(grid_lines((32, 16), &auto, m).expect("16 px").rgba[3], 200);
        assert!(grid_lines((32, 16), &auto, Affine::scale(1.0)).is_none());
        assert_eq!(line_alpha(200, true, 10.0), 100);
        assert!(
            grid_lines(
                (32, 16),
                &Grid {
                    visible: false,
                    ..g
                },
                m
            )
            .is_none()
        );
    }

    #[test]
    fn a_grade_de_pixel_aparece_so_de_perto() {
        let p = PixelGrid {
            visible: true,
            ..PixelGrid::default()
        };
        assert!(pixel_grid_lines((4, 4), &p, Affine::scale(4.0)).is_none());
        let l = pixel_grid_lines((4, 4), &p, Affine::scale(16.0)).expect("16 px");
        assert_eq!(l.segments.len(), 10);
        assert_eq!(l.rgba, [200, 200, 200, 160]);
    }

    #[test]
    fn set_as_defaults_grava_e_o_documento_novo_le() {
        let dir = std::env::temp_dir().join(format!("ph2d-pixel-prefs-{}", std::process::id()));
        let p = dir.join("pixel-prefs.txt");
        assert_eq!(load_defaults_from(&p), PixelPrefs::default());
        let mut v = PixelPrefs::default();
        v.grid.w = 24;
        v.grid.visible = true;
        v.checker.apply_zoom = false;
        v.onion.prev = 3;
        save_defaults_to(&p, &v).expect("grava");
        assert_eq!(load_defaults_from(&p), v);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn o_xadrez_apply_zoom_em_pixeis_do_sprite_ou_do_ecra() {
        let c = Checker {
            w: 8,
            h: 4,
            ..Checker::default()
        };
        // Apply Zoom: casas de 8×4 células, o zoom não muda nada.
        assert_eq!(checker_layout(&c, (64, 32), 4.0), (8, 8, 8.0, 4.0));
        assert_eq!(checker_layout(&c, (64, 32), 16.0), (8, 8, 8.0, 4.0));
        // Sem Apply Zoom: 8×4 píxeis do ECRÃ; a zoom 4 são 2×1 células.
        let s = Checker {
            apply_zoom: false,
            ..c
        };
        assert_eq!(checker_layout(&s, (64, 32), 4.0), (32, 32, 2.0, 1.0));
        assert_eq!(checker_layout(&s, (64, 32), 8.0), (64, 64, 1.0, 0.5));
    }
}
