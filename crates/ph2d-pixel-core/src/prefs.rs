//! **As preferências do módulo, por documento** (P12, `P12_plano.md` §9.6): grade, grade de píxel,
//! xadrez e onion. Cada documento leva as suas; «Set as Defaults» guarda uma cópia para os novos
//! (texto `chave=valor`, [`PixelPrefs::to_lines`] / [`PixelPrefs::from_lines`]).
//!
//! Os valores de fábrica são os do Aseprite Trial 1.3.18.6 medidos em `P12_oraculos.md` O18, O19 e
//! O21. As cores são dados do documento (bytes), não tokens de UI.

use crate::compose::Onion;

/// A grade (O18): `Visible Grid` desligada, origem 0,0, 16×16, `#0000FF`, opacidade 160 `Auto`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub color: [u8; 4],
    pub opacity: u8,
    /// A opacidade desce quando o zoom aproxima as linhas.
    pub auto_opacity: bool,
    pub visible: bool,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            w: 16,
            h: 16,
            color: [0, 0, 255, 255],
            opacity: 160,
            auto_opacity: true,
            visible: false,
        }
    }
}

/// A grade de píxel (O18): desligada, `#C8C8C8`, opacidade 160 `Auto`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelGrid {
    pub color: [u8; 4],
    pub opacity: u8,
    pub auto_opacity: bool,
    pub visible: bool,
}

impl Default for PixelGrid {
    fn default() -> Self {
        Self {
            color: [200, 200, 200, 255],
            opacity: 160,
            auto_opacity: true,
            visible: false,
        }
    }
}

/// O xadrez do fundo (O19): 16×16 `Apply Zoom`, `#808080` e `#C0C0C0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checker {
    pub w: u32,
    pub h: u32,
    pub color_a: [u8; 4],
    pub color_b: [u8; 4],
    /// O quadrado tem `w×h` píxeis do SPRITE (cresce com o zoom); desligado, do ecrã.
    pub apply_zoom: bool,
}

impl Default for Checker {
    fn default() -> Self {
        Self {
            w: 16,
            h: 16,
            color_a: [128, 128, 128, 255],
            color_b: [192, 192, 192, 255],
            apply_zoom: true,
        }
    }
}

/// As preferências de um documento Pixel.
///
/// O onion reusa [`Onion`] (unidade 0–255): o `Opacity 27 %` e o `Opacity Step 11 %` do O21 são
/// `base 68` (27 % de 255 = 68,9 → o Trial mostra 27 % para 68) e `step 28` (11 % de 255 = 28,05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PixelPrefs {
    pub grid: Grid,
    pub pixel_grid: PixelGrid,
    pub checker: Checker,
    pub onion: Onion,
}

fn rgba(c: [u8; 4]) -> String {
    format!("{:02x}{:02x}{:02x}{:02x}", c[0], c[1], c[2], c[3])
}

fn parse_rgba(v: &str) -> Option<[u8; 4]> {
    if v.len() != 8 || !v.is_ascii() {
        return None;
    }
    let b = |i: usize| u8::from_str_radix(&v[i..i + 2], 16).ok();
    Some([b(0)?, b(2)?, b(4)?, b(6)?])
}

fn flag(b: bool) -> &'static str {
    if b { "1" } else { "0" }
}

fn parse_flag(v: &str) -> Option<bool> {
    match v {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

impl PixelPrefs {
    /// As linhas `chave=valor` (prefixo `pixel.`), em ordem fixa.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let (g, p, c, o) = (&self.grid, &self.pixel_grid, &self.checker, &self.onion);
        [
            ("grid.x", g.x.to_string()),
            ("grid.y", g.y.to_string()),
            ("grid.w", g.w.to_string()),
            ("grid.h", g.h.to_string()),
            ("grid.color", rgba(g.color)),
            ("grid.opacity", g.opacity.to_string()),
            ("grid.auto_opacity", flag(g.auto_opacity).into()),
            ("grid.visible", flag(g.visible).into()),
            ("pixel_grid.color", rgba(p.color)),
            ("pixel_grid.opacity", p.opacity.to_string()),
            ("pixel_grid.auto_opacity", flag(p.auto_opacity).into()),
            ("pixel_grid.visible", flag(p.visible).into()),
            ("checker.w", c.w.to_string()),
            ("checker.h", c.h.to_string()),
            ("checker.color_a", rgba(c.color_a)),
            ("checker.color_b", rgba(c.color_b)),
            ("checker.apply_zoom", flag(c.apply_zoom).into()),
            ("onion.prev", o.prev.to_string()),
            ("onion.next", o.next.to_string()),
            ("onion.base", o.base.to_string()),
            ("onion.step", o.step.to_string()),
            ("onion.tint", flag(o.tint).into()),
            ("onion.front", flag(o.front).into()),
            ("onion.loop_tag", flag(o.loop_tag).into()),
            (
                "onion.current_layer_only",
                flag(o.current_layer_only).into(),
            ),
        ]
        .into_iter()
        .map(|(k, v)| format!("pixel.{k}={v}"))
        .collect()
    }

    /// Lê as linhas de [`Self::to_lines`] por cima dos valores de fábrica. Chave alheia, sem o
    /// prefixo ou com valor ilegível salta-se (o ficheiro de preferências é partilhado pelo app).
    #[must_use]
    pub fn from_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> Self {
        let mut s = Self::default();
        for line in lines {
            let Some((k, v)) = line.trim().split_once('=') else {
                continue;
            };
            let Some(k) = k.trim().strip_prefix("pixel.") else {
                continue;
            };
            let _ = s.set(k, v.trim());
        }
        s
    }

    fn set(&mut self, k: &str, v: &str) -> Option<()> {
        let (g, p, c, o) = (
            &mut self.grid,
            &mut self.pixel_grid,
            &mut self.checker,
            &mut self.onion,
        );
        match k {
            "grid.x" => g.x = v.parse().ok()?,
            "grid.y" => g.y = v.parse().ok()?,
            "grid.w" => g.w = v.parse().ok().filter(|&n| n > 0)?,
            "grid.h" => g.h = v.parse().ok().filter(|&n| n > 0)?,
            "grid.color" => g.color = parse_rgba(v)?,
            "grid.opacity" => g.opacity = v.parse().ok()?,
            "grid.auto_opacity" => g.auto_opacity = parse_flag(v)?,
            "grid.visible" => g.visible = parse_flag(v)?,
            "pixel_grid.color" => p.color = parse_rgba(v)?,
            "pixel_grid.opacity" => p.opacity = v.parse().ok()?,
            "pixel_grid.auto_opacity" => p.auto_opacity = parse_flag(v)?,
            "pixel_grid.visible" => p.visible = parse_flag(v)?,
            "checker.w" => c.w = v.parse().ok().filter(|&n| n > 0)?,
            "checker.h" => c.h = v.parse().ok().filter(|&n| n > 0)?,
            "checker.color_a" => c.color_a = parse_rgba(v)?,
            "checker.color_b" => c.color_b = parse_rgba(v)?,
            "checker.apply_zoom" => c.apply_zoom = parse_flag(v)?,
            "onion.prev" => o.prev = v.parse().ok()?,
            "onion.next" => o.next = v.parse().ok()?,
            "onion.base" => o.base = v.parse().ok()?,
            "onion.step" => o.step = v.parse().ok()?,
            "onion.tint" => o.tint = parse_flag(v)?,
            "onion.front" => o.front = parse_flag(v)?,
            "onion.loop_tag" => o.loop_tag = parse_flag(v)?,
            "onion.current_layer_only" => o.current_layer_only = parse_flag(v)?,
            _ => return None,
        }
        Some(())
    }
}
