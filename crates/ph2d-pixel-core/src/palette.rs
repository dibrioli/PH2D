//! A paleta: a de omissão (DB32) e as leis que a reordenam sem mudar a imagem (P2).

/// **DB32** (DawnBringer, 2012, livre): a paleta que o Aseprite 1.3.18 mostra num sprite novo
/// (`docs/PixelArt/aseprite/01_editor.png`: 32 amostras, preto em 0, branco em 21).
pub const DEFAULT: [[u8; 4]; 32] = [
    [0x00, 0x00, 0x00, 0xff],
    [0x22, 0x20, 0x34, 0xff],
    [0x45, 0x28, 0x3c, 0xff],
    [0x66, 0x39, 0x31, 0xff],
    [0x8f, 0x56, 0x3b, 0xff],
    [0xdf, 0x71, 0x26, 0xff],
    [0xd9, 0xa0, 0x66, 0xff],
    [0xee, 0xc3, 0x9a, 0xff],
    [0xfb, 0xf2, 0x36, 0xff],
    [0x99, 0xe5, 0x50, 0xff],
    [0x6a, 0xbe, 0x30, 0xff],
    [0x37, 0x94, 0x6e, 0xff],
    [0x4b, 0x69, 0x2f, 0xff],
    [0x52, 0x4b, 0x24, 0xff],
    [0x32, 0x3c, 0x39, 0xff],
    [0x3f, 0x3f, 0x74, 0xff],
    [0x30, 0x60, 0x82, 0xff],
    [0x5b, 0x6e, 0xe1, 0xff],
    [0x63, 0x9b, 0xff, 0xff],
    [0x5f, 0xcd, 0xe4, 0xff],
    [0xcb, 0xdb, 0xfc, 0xff],
    [0xff, 0xff, 0xff, 0xff],
    [0x9b, 0xad, 0xb7, 0xff],
    [0x84, 0x7e, 0x87, 0xff],
    [0x69, 0x6a, 0x6a, 0xff],
    [0x59, 0x56, 0x52, 0xff],
    [0x76, 0x42, 0x8a, 0xff],
    [0xac, 0x32, 0x32, 0xff],
    [0xd9, 0x57, 0x63, 0xff],
    [0xd7, 0x7b, 0xba, 0xff],
    [0x8f, 0x97, 0x4a, 0xff],
    [0x8a, 0x6f, 0x30, 0xff],
];

/// O maior número de cores de uma paleta: um índice é um byte (o `.ase` guarda até 256 no modo
/// Indexed) — o recurso é o FORMATO da célula.
pub const MAX_COLORS: usize = 256;

/// **Mover uma cor** de `from` para `to` (o arrasto na paleta): `perm[novo] = antigo`, o resto
/// desliza um lugar (MVP `movePermutation`).
#[must_use]
pub fn move_permutation(size: usize, from: usize, to: usize) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..size).collect();
    if from < size && to < size {
        let v = perm.remove(from);
        perm.insert(to, v);
    }
    perm
}

/// O inverso de uma permutação `perm[novo] = antigo`: `inv[antigo] = novo`.
#[must_use]
pub fn invert_permutation(perm: &[usize]) -> Vec<usize> {
    let mut inv = vec![0; perm.len()];
    for (new, &old) in perm.iter().enumerate() {
        inv[old] = new;
    }
    inv
}

/// É uma permutação de `0..perm.len()`?
#[must_use]
pub fn is_permutation(perm: &[usize]) -> bool {
    let mut seen = vec![false; perm.len()];
    perm.iter()
        .all(|&o| o < seen.len() && !std::mem::replace(&mut seen[o], true))
}

/// As cores repetidas: `duplicata → a primeira igual a ela` (RGBA inteiro); o transparente nem
/// conta nem é contado (MVP `findDuplicates`).
#[must_use]
pub fn find_duplicates(colors: &[[u8; 4]], transparent: Option<usize>) -> Vec<(usize, usize)> {
    let mut first = std::collections::BTreeMap::new();
    let mut out = Vec::new();
    for (i, c) in colors.iter().enumerate() {
        if Some(i) == transparent {
            continue;
        }
        match first.get(c) {
            Some(&k) => out.push((i, k)),
            None => {
                first.insert(*c, i);
            }
        }
    }
    out
}

/// `#rrggbb` (minúsculas).
#[must_use]
pub fn to_hex(c: [u8; 4]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// `#rgb`, `#rrggbb`, com ou sem `#`, espaços à volta; qualquer outra coisa é `None` (MVP `parseHex`).
#[must_use]
pub fn parse_hex(text: &str) -> Option<[u8; 4]> {
    let t = text.trim();
    let t = t.strip_prefix('#').unwrap_or(t);
    if !t.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let digit = |i: usize| u8::from_str_radix(&t[i..=i], 16).ok();
    match t.len() {
        3 => Some([digit(0)? * 17, digit(1)? * 17, digit(2)? * 17, 255]),
        6 => {
            let byte = |i: usize| u8::from_str_radix(&t[i..i + 2], 16).ok();
            Some([byte(0)?, byte(2)?, byte(4)?, 255])
        }
        _ => None,
    }
}

/// O critério de **Sort** da paleta (o menu da paleta do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Hue,
    Saturation,
    Brightness,
    Luminance,
    Red,
    Green,
    Blue,
    Alpha,
}

impl SortKey {
    pub const ALL: [SortKey; 8] = [
        SortKey::Hue,
        SortKey::Saturation,
        SortKey::Brightness,
        SortKey::Luminance,
        SortKey::Red,
        SortKey::Green,
        SortKey::Blue,
        SortKey::Alpha,
    ];

    /// A chave inteira: matiz em centésimos de grau, saturação e brilho do HSV em 0..=255·255,
    /// luminância = [`crate::color::luma`].
    fn key(self, c: [u8; 4]) -> u32 {
        let [r, g, b, a] = c.map(u32::from);
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        match self {
            SortKey::Hue => hue_centi(r, g, b, max, min),
            SortKey::Saturation => (max - min) * 255 * 255 / max.max(1),
            SortKey::Brightness => max,
            SortKey::Luminance => u32::from(crate::color::luma(c)),
            SortKey::Red => r,
            SortKey::Green => g,
            SortKey::Blue => b,
            SortKey::Alpha => a,
        }
    }
}

/// O matiz HSV em centésimos de grau (`0..36000`), inteiro; cinza = 0.
fn hue_centi(r: u32, g: u32, b: u32, max: u32, min: u32) -> u32 {
    let d = i64::from(max - min);
    if d == 0 {
        return 0;
    }
    let (r, g, b) = (i64::from(r), i64::from(g), i64::from(b));
    let h = if max as i64 == r {
        6000 * (g - b) / d
    } else if max as i64 == g {
        6000 * (b - r) / d + 12000
    } else {
        6000 * (r - g) / d + 24000
    };
    h.rem_euclid(36000) as u32
}

/// A permutação (`perm[novo] = antigo`) que ordena `colors[range]` por `key`; o resto fica no
/// lugar, empates guardam a ordem (estável). O transparente não se move.
#[must_use]
pub fn sort_permutation(
    colors: &[[u8; 4]],
    range: std::ops::Range<usize>,
    key: SortKey,
    descending: bool,
    transparent: Option<usize>,
) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..colors.len()).collect();
    let range = range.start.min(colors.len())..range.end.min(colors.len());
    let slots: Vec<usize> = range.filter(|&i| Some(i) != transparent).collect();
    let mut moving = slots.clone();
    moving.sort_by(|&a, &b| {
        let o = key.key(colors[a]).cmp(&key.key(colors[b]));
        if descending { o.reverse() } else { o }
    });
    for (slot, old) in slots.into_iter().zip(moving) {
        perm[slot] = old;
    }
    perm
}
