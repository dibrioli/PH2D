//! **A lista de pincéis** do Pixel Art (os espaços Alt+1, Alt+2… do Aseprite), no COMPUTADOR —
//! `~/.ph2d/pixel_brushes/brushes.txt` (dono 02/11: valem em todo projecto, nunca no `.ph2dproj`).
//! Texto canónico, uma secção `brush … end` por espaço; o padrão em RGBA hex (`.` = fora da
//! máscara, `-` = transparente), para valer em sprites de qualquer modo de cor.

use ph2d_pixel_core::brush::BrushShape;
use ph2d_pixel_core::ink::Ink;
use ph2d_pixel_core::raster::Cell;

const HEAD: &str = "ph2d-pixel-brushes 1";

/// As células de um padrão guardado: `None` = fora da máscara, `Some(None)` = transparente.
pub type PatternCells = Vec<Option<Option<[u8; 4]>>>;

/// A forma de um espaço.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrushKind {
    Shape(BrushShape),
    /// O padrão (Ctrl+B): `w × h`, as células e o canto de onde foi copiado (o `Source`).
    Pattern {
        size: (u32, u32),
        cells: PatternCells,
        origin: Cell,
    },
}

/// Os parâmetros que o espaço GUARDA (as caixas do menu do tipo de pincel do Aseprite); um
/// desligado deixa ficar o valor actual ao recuperar o espaço.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BrushParams {
    pub kind: bool,
    pub size: bool,
    pub angle: bool,
    pub image_color: bool,
    pub ink: bool,
    pub fg: bool,
    pub bg: bool,
    pub pixel_perfect: bool,
}

impl BrushParams {
    /// Um espaço do Ctrl+B: só *Brush Type* e *Image Color* (Trial, `rb2b_espacos`).
    pub const NEW_BRUSH: Self = Self {
        kind: true,
        size: false,
        angle: false,
        image_color: true,
        ink: false,
        fg: false,
        bg: false,
        pixel_perfect: false,
    };
    /// *Save Brush Here*: tudo menos as cores e o pixel-perfect (Trial, foto 66).
    pub const SAVE_HERE: Self = Self {
        kind: true,
        size: true,
        angle: true,
        image_color: true,
        ink: true,
        fg: false,
        bg: false,
        pixel_perfect: false,
    };

    const NAMES: [&'static str; 8] = [
        "type",
        "size",
        "angle",
        "image_color",
        "ink",
        "fg",
        "bg",
        "pixel_perfect",
    ];

    fn flags(self) -> [bool; 8] {
        [
            self.kind,
            self.size,
            self.angle,
            self.image_color,
            self.ink,
            self.fg,
            self.bg,
            self.pixel_perfect,
        ]
    }

    fn from_flags(f: [bool; 8]) -> Self {
        Self {
            kind: f[0],
            size: f[1],
            angle: f[2],
            image_color: f[3],
            ink: f[4],
            fg: f[5],
            bg: f[6],
            pixel_perfect: f[7],
        }
    }
}

/// Um espaço da lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushSlot {
    /// Trancado: guardado de propósito (*Save Brush Here*); o Ctrl+B cria-os destrancados.
    pub locked: bool,
    pub params: BrushParams,
    pub kind: BrushKind,
    pub size: u32,
    pub angle: i32,
    pub image_color: bool,
    pub ink: Ink,
    pub fg: [u8; 4],
    pub bg: [u8; 4],
    pub pixel_perfect: bool,
}

/// Porque o ficheiro foi recusado (a linha, contada de 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushFileError {
    pub line: usize,
    pub what: String,
}

fn shape_name(s: BrushShape) -> &'static str {
    match s {
        BrushShape::Circle => "circle",
        BrushShape::Square => "square",
        BrushShape::Line => "line",
    }
}

const INKS: [(Ink, &str); 5] = [
    (Ink::Simple, "simple"),
    (Ink::AlphaCompositing, "alpha_compositing"),
    (Ink::CopyAlphaColor, "copy_alpha_color"),
    (Ink::LockAlpha, "lock_alpha"),
    (Ink::Shading, "shading"),
];

fn hex(c: [u8; 4]) -> String {
    format!("{:02x}{:02x}{:02x}{:02x}", c[0], c[1], c[2], c[3])
}

/// O texto canónico da lista.
#[must_use]
pub fn write(slots: &[BrushSlot]) -> String {
    let mut o = format!("{HEAD}\n");
    for s in slots {
        o.push_str("brush\n");
        o.push_str(&format!("locked {}\n", u8::from(s.locked)));
        let on = BrushParams::NAMES
            .iter()
            .zip(s.params.flags())
            .filter(|(_, f)| *f)
            .map(|(n, _)| *n);
        o.push_str("params");
        for n in on {
            o.push(' ');
            o.push_str(n);
        }
        o.push('\n');
        match &s.kind {
            BrushKind::Shape(sh) => o.push_str(&format!("type {}\n", shape_name(*sh))),
            BrushKind::Pattern {
                size: (w, h),
                cells,
                origin,
            } => {
                o.push_str(&format!("type pattern {w} {h} {} {}\n", origin.0, origin.1));
                for row in cells.chunks(*w as usize) {
                    let r: Vec<String> = row
                        .iter()
                        .map(|c| match c {
                            None => ".".to_owned(),
                            Some(None) => "-".to_owned(),
                            Some(Some(c)) => hex(*c),
                        })
                        .collect();
                    o.push_str(&format!("row {}\n", r.join(" ")));
                }
            }
        }
        let ink = INKS
            .iter()
            .find(|(i, _)| *i == s.ink)
            .map_or("simple", |x| x.1);
        o.push_str(&format!(
            "size {}\nangle {}\nimage_color {}\nink {ink}\nfg {}\nbg {}\npixel_perfect {}\nend\n",
            s.size,
            s.angle,
            u8::from(s.image_color),
            hex(s.fg),
            hex(s.bg),
            u8::from(s.pixel_perfect)
        ));
    }
    o
}

fn err(line: usize, what: impl Into<String>) -> BrushFileError {
    BrushFileError {
        line,
        what: what.into(),
    }
}

fn num<T: std::str::FromStr>(n: usize, v: Option<&str>) -> Result<T, BrushFileError> {
    v.and_then(|v| v.parse().ok())
        .ok_or_else(|| err(n, "number"))
}

fn bit(n: usize, v: Option<&str>) -> Result<bool, BrushFileError> {
    match v {
        Some("0") => Ok(false),
        Some("1") => Ok(true),
        _ => Err(err(n, "0 or 1")),
    }
}

/// `rrggbbaa`, exactamente oito dígitos hex.
fn color(n: usize, v: Option<&str>) -> Result<[u8; 4], BrushFileError> {
    let rgba = |t: &str| -> Option<[u8; 4]> {
        if t.len() != 8 || !t.is_ascii() {
            return None;
        }
        let b = |i: usize| u8::from_str_radix(&t[2 * i..2 * i + 2], 16).ok();
        Some([b(0)?, b(1)?, b(2)?, b(3)?])
    };
    v.and_then(rgba).ok_or_else(|| err(n, "colour"))
}

fn blank(kind: BrushKind) -> BrushSlot {
    BrushSlot {
        locked: false,
        params: BrushParams::default(),
        kind,
        size: 1,
        angle: 0,
        image_color: true,
        ink: Ink::Simple,
        fg: [0, 0, 0, 255],
        bg: [0, 0, 0, 0],
        pixel_perfect: false,
    }
}

/// Lê a lista. Tolerante a CRLF e a linhas vazias; tudo o resto que não percebe é ERRO com a linha.
pub fn read(text: &str) -> Result<Vec<BrushSlot>, BrushFileError> {
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty());
    match lines.next() {
        Some((_, HEAD)) => {}
        Some((n, _)) => return Err(err(n, "header")),
        None => return Err(err(1, "header")),
    }
    let mut out = Vec::new();
    let mut cur: Option<BrushSlot> = None;
    let mut rows: (usize, u32) = (0, 0);
    for (n, l) in lines {
        let mut w = l.split_whitespace();
        let key = w.next().unwrap_or_default();
        if key == "brush" {
            if cur.is_some() {
                return Err(err(n, "brush without end"));
            }
            cur = Some(blank(BrushKind::Shape(BrushShape::Circle)));
            continue;
        }
        let Some(s) = cur.as_mut() else {
            return Err(err(n, "outside a brush"));
        };
        match key {
            "locked" => s.locked = bit(n, w.next())?,
            "params" => {
                let mut f = [false; 8];
                for p in w {
                    let k = BrushParams::NAMES
                        .iter()
                        .position(|x| *x == p)
                        .ok_or_else(|| err(n, "param"))?;
                    f[k] = true;
                }
                s.params = BrushParams::from_flags(f);
            }
            "type" => {
                s.kind = match w.next() {
                    Some("circle") => BrushKind::Shape(BrushShape::Circle),
                    Some("square") => BrushKind::Shape(BrushShape::Square),
                    Some("line") => BrushKind::Shape(BrushShape::Line),
                    Some("pattern") => {
                        let (pw, ph): (u32, u32) = (num(n, w.next())?, num(n, w.next())?);
                        let origin = (num(n, w.next())?, num(n, w.next())?);
                        if pw == 0 || ph == 0 || pw > 4096 || ph > 4096 {
                            return Err(err(n, "pattern size"));
                        }
                        rows = (0, ph);
                        BrushKind::Pattern {
                            size: (pw, ph),
                            cells: Vec::with_capacity((pw * ph) as usize),
                            origin,
                        }
                    }
                    _ => return Err(err(n, "type")),
                };
            }
            "row" => {
                let BrushKind::Pattern {
                    size: (pw, _),
                    cells,
                    ..
                } = &mut s.kind
                else {
                    return Err(err(n, "row without pattern"));
                };
                let mut k = 0;
                for c in w {
                    cells.push(match c {
                        "." => None,
                        "-" => Some(None),
                        h => Some(Some(color(n, Some(h))?)),
                    });
                    k += 1;
                }
                if k != *pw || rows.0 >= rows.1 as usize {
                    return Err(err(n, "row width"));
                }
                rows.0 += 1;
            }
            "size" => s.size = num(n, w.next())?,
            "angle" => s.angle = num(n, w.next())?,
            "image_color" => s.image_color = bit(n, w.next())?,
            "ink" => {
                let v = w.next();
                s.ink = INKS
                    .iter()
                    .find(|(_, k)| Some(*k) == v)
                    .map(|x| x.0)
                    .ok_or_else(|| err(n, "ink"))?;
            }
            "fg" => s.fg = color(n, w.next())?,
            "bg" => s.bg = color(n, w.next())?,
            "pixel_perfect" => s.pixel_perfect = bit(n, w.next())?,
            "end" => {
                let s = cur.take().expect("dentro de um brush");
                if let BrushKind::Pattern { size, cells, .. } = &s.kind
                    && cells.len() != (size.0 * size.1) as usize
                {
                    return Err(err(n, "pattern rows"));
                }
                out.push(s);
                continue;
            }
            _ => return Err(err(n, "key")),
        }
    }
    if cur.is_some() {
        return Err(err(n_of(text), "brush without end"));
    }
    Ok(out)
}

fn n_of(text: &str) -> usize {
    text.lines().count().max(1)
}
