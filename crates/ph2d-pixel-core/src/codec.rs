//! O formato gravado: `PH2DPIXEL` + major/minor + registos de campos etiquetados.
//!
//! ```text
//! blob     = MAGIC · major:u16le · minor:u16le · registo(Library)
//! registo  = { etiqueta:varint≥1 · comprimento:varint · bytes }* · 0
//! Library  : 1 Entry* (repetido)
//! Entry    : 1 chave (varint) · 2 Sprite (registo)
//! Sprite   : 1 largura · 2 altura · 3 modo · 4 paleta (n×4 bytes) · 5 transparente · 6 Layer*
//!            · 7 Frame* · 8 Cel* · 9 próximo id · 10 Tag*                (inteiros em varint)
//! Layer    : 1 id · 2 nome (utf8) · 3 bandeiras (bit0 visível, bit1 background, bit2 trancada,
//!            bit3 referência, bit4 grupo, bit5 grupo recolhido, bit6 continuous [1.2]) · 4 opacidade (sem = 255)
//!            · 5 modo de mistura (id do `.ase`, sem = Normal) · 6 pai (id; sem = raiz)   [1.1]
//! Frame    : 1 id · 2 duração ms
//! Cel      : 1 camada · 2 frame · 3 x (zigzag) · 4 y (zigzag) · 5 w · 6 h · 7 bytes
//!            · 8 opacidade (sem = 255) · 9 z (zigzag, sem = 0) · 10 frame do dono (ligado)  [1.2]
//! Tag      : 1 nome · 2 de · 3 até (posições) · 4 direcção (id do `.ase`) · 5 passadas
//!            · 6 cor (4 bytes)                                                       [1.2]
//! ```
//! Dentro da mesma MAJOR uma etiqueta desconhecida SALTA-SE (o molde `.riv` que o `ph2d-macro`
//! segue); MAJOR diferente recusa-se. A escrita é em ordem fixa ⇒ o mesmo documento dá os
//! mesmos bytes.

use std::collections::BTreeMap;

use crate::blend::BlendMode;
use crate::sprite::{
    Cel, ColorMode, Direction, Frame, FrameId, Layer, LayerId, LayerKind, MAX_SIDE, Rect, Sprite,
    Tag,
};

pub const MAGIC: &[u8; 9] = b"PH2DPIXEL";
pub const MAJOR: u16 = 1;
pub const MINOR: u16 = 2;

/// Ficheiro recusado; nunca um documento meio lido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    BadMagic,
    MajorMismatch {
        found: u16,
    },
    Truncated,
    /// O conteúdo contradiz as leis do documento (o campo diz qual).
    Invalid(&'static str),
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::BadMagic => write!(f, "not a PH2DPIXEL blob"),
            CodecError::MajorMismatch { found } => {
                write!(f, "format {found}.x, this build reads {MAJOR}.x")
            }
            CodecError::Truncated => write!(f, "truncated"),
            CodecError::Invalid(what) => write!(f, "invalid {what}"),
        }
    }
}

impl std::error::Error for CodecError {}

/// Grava um conjunto de sprites por chave (a do objecto que o possui).
#[must_use]
pub fn write_library(sprites: &BTreeMap<u64, Sprite>) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&MAJOR.to_le_bytes());
    out.extend_from_slice(&MINOR.to_le_bytes());
    let mut lib = Vec::new();
    for (&key, sprite) in sprites {
        let mut entry = Vec::new();
        put_uint(&mut entry, 1, key);
        put_field(&mut entry, 2, &sprite_record(sprite));
        entry.push(0);
        put_field(&mut lib, 1, &entry);
    }
    lib.push(0);
    out.extend_from_slice(&lib);
    out
}

/// Lê o que [`write_library`] gravou. Vazio = biblioteca vazia (projecto sem Pixel Sprites).
pub fn read_library(bytes: &[u8]) -> Result<BTreeMap<u64, Sprite>, CodecError> {
    let mut out = BTreeMap::new();
    if bytes.is_empty() {
        return Ok(out);
    }
    let rest = bytes.strip_prefix(MAGIC).ok_or(CodecError::BadMagic)?;
    if rest.len() < 4 {
        return Err(CodecError::Truncated);
    }
    let major = u16::from_le_bytes([rest[0], rest[1]]);
    if major != MAJOR {
        return Err(CodecError::MajorMismatch { found: major });
    }
    for (tag, body) in Fields::new(&rest[4..])? {
        if tag != 1 {
            continue;
        }
        let mut key = None;
        let mut sprite = None;
        for (t, b) in Fields::new(body)? {
            match t {
                1 => key = Some(uint(b)?),
                2 => sprite = Some(read_sprite(b)?),
                _ => {}
            }
        }
        let key = key.ok_or(CodecError::Invalid("entry.key"))?;
        let sprite = sprite.ok_or(CodecError::Invalid("entry.sprite"))?;
        if out.insert(key, sprite).is_some() {
            return Err(CodecError::Invalid("entry.key repeated"));
        }
    }
    Ok(out)
}

fn sprite_record(s: &Sprite) -> Vec<u8> {
    let mut r = Vec::new();
    put_uint(&mut r, 1, u64::from(s.width));
    put_uint(&mut r, 2, u64::from(s.height));
    put_uint(&mut r, 3, u64::from(s.mode.code()));
    put_field(&mut r, 4, &s.palette.concat());
    put_uint(&mut r, 5, u64::from(s.transparent));
    for l in &s.layers {
        let mut b = Vec::new();
        put_uint(&mut b, 1, u64::from(l.id.0));
        put_field(&mut b, 2, l.name.as_bytes());
        let (group, collapsed) = match l.kind {
            LayerKind::Image => (false, false),
            LayerKind::Group { collapsed } => (true, collapsed),
        };
        put_uint(
            &mut b,
            3,
            u64::from(l.visible)
                | (u64::from(l.background) << 1)
                | (u64::from(l.locked) << 2)
                | (u64::from(l.reference) << 3)
                | (u64::from(group) << 4)
                | (u64::from(collapsed) << 5)
                | (u64::from(l.continuous) << 6),
        );
        if l.opacity != 255 {
            put_uint(&mut b, 4, u64::from(l.opacity));
        }
        if l.blend != BlendMode::Normal {
            put_uint(&mut b, 5, u64::from(l.blend.code()));
        }
        if let Some(p) = l.parent {
            put_uint(&mut b, 6, u64::from(p.0));
        }
        b.push(0);
        put_field(&mut r, 6, &b);
    }
    for f in &s.frames {
        let mut b = Vec::new();
        put_uint(&mut b, 1, u64::from(f.id.0));
        put_uint(&mut b, 2, u64::from(f.duration_ms));
        b.push(0);
        put_field(&mut r, 7, &b);
    }
    for (&(l, f), c) in &s.cels {
        let mut b = Vec::new();
        put_uint(&mut b, 1, u64::from(l.0));
        put_uint(&mut b, 2, u64::from(f.0));
        put_uint(&mut b, 3, zigzag(c.bbox.x));
        put_uint(&mut b, 4, zigzag(c.bbox.y));
        put_uint(&mut b, 5, u64::from(c.bbox.w));
        put_uint(&mut b, 6, u64::from(c.bbox.h));
        put_field(&mut b, 7, &c.data);
        if c.opacity != 255 {
            put_uint(&mut b, 8, u64::from(c.opacity));
        }
        if c.z != 0 {
            put_uint(&mut b, 9, zigzag(i32::from(c.z)));
        }
        if let Some(o) = c.link {
            put_uint(&mut b, 10, u64::from(o.0));
        }
        b.push(0);
        put_field(&mut r, 8, &b);
    }
    put_uint(&mut r, 9, u64::from(s.next_id));
    for t in &s.tags {
        let mut b = Vec::new();
        put_field(&mut b, 1, t.name.as_bytes());
        put_uint(&mut b, 2, u64::from(t.from));
        put_uint(&mut b, 3, u64::from(t.to));
        put_uint(&mut b, 4, u64::from(t.direction.code()));
        put_uint(&mut b, 5, u64::from(t.repeat));
        put_field(&mut b, 6, &t.color);
        b.push(0);
        put_field(&mut r, 10, &b);
    }
    r.push(0);
    r
}

fn read_sprite(bytes: &[u8]) -> Result<Sprite, CodecError> {
    let mut s = Sprite {
        width: 0,
        height: 0,
        mode: ColorMode::Rgba,
        palette: Vec::new(),
        transparent: 0,
        layers: Vec::new(),
        frames: Vec::new(),
        cels: BTreeMap::new(),
        tags: Vec::new(),
        next_id: 0,
        mask: None,
        last_mask: None,
    };
    let mut raw_cels = Vec::new();
    for (tag, b) in Fields::new(bytes)? {
        match tag {
            1 => s.width = u32_of(b, "sprite.width")?,
            2 => s.height = u32_of(b, "sprite.height")?,
            3 => {
                s.mode = ColorMode::from_code(u8::try_from(uint(b)?).unwrap_or(u8::MAX))
                    .ok_or(CodecError::Invalid("sprite.mode"))?;
            }
            4 => {
                if b.len() % 4 != 0 || b.len() > 256 * 4 {
                    return Err(CodecError::Invalid("sprite.palette"));
                }
                s.palette = b.as_chunks::<4>().0.to_vec();
            }
            5 => {
                s.transparent = u8::try_from(uint(b)?)
                    .map_err(|_| CodecError::Invalid("sprite.transparent"))?;
            }
            6 => {
                let mut l = Layer::image(LayerId(0), String::new());
                for (t, v) in Fields::new(b)? {
                    match t {
                        1 => l.id = LayerId(u32_of(v, "layer.id")?),
                        2 => {
                            l.name = String::from_utf8(v.to_vec())
                                .map_err(|_| CodecError::Invalid("layer.name"))?;
                        }
                        3 => {
                            let f = uint(v)?;
                            l.visible = f & 1 != 0;
                            l.background = f & 2 != 0;
                            l.locked = f & 4 != 0;
                            l.reference = f & 8 != 0;
                            l.continuous = f & 64 != 0;
                            if f & 16 != 0 {
                                l.kind = LayerKind::Group {
                                    collapsed: f & 32 != 0,
                                };
                            }
                        }
                        4 => {
                            l.opacity = u8::try_from(uint(v)?)
                                .map_err(|_| CodecError::Invalid("layer.opacity"))?;
                        }
                        5 => {
                            l.blend = u8::try_from(uint(v)?)
                                .ok()
                                .and_then(BlendMode::from_code)
                                .ok_or(CodecError::Invalid("layer.blend"))?;
                        }
                        6 => l.parent = Some(LayerId(u32_of(v, "layer.parent")?)),
                        _ => {}
                    }
                }
                s.layers.push(l);
            }
            7 => {
                let mut f = Frame {
                    id: FrameId(0),
                    duration_ms: 0,
                };
                for (t, v) in Fields::new(b)? {
                    match t {
                        1 => f.id = FrameId(u32_of(v, "frame.id")?),
                        2 => f.duration_ms = u32_of(v, "frame.duration")?,
                        _ => {}
                    }
                }
                s.frames.push(f);
            }
            8 => {
                let (mut l, mut f, mut r) = (0, 0, Rect::default());
                let mut cel = Cel::image(Rect::default(), Vec::new());
                for (t, v) in Fields::new(b)? {
                    match t {
                        1 => l = u32_of(v, "cel.layer")?,
                        2 => f = u32_of(v, "cel.frame")?,
                        3 => r.x = unzigzag(uint(v)?)?,
                        4 => r.y = unzigzag(uint(v)?)?,
                        5 => r.w = u32_of(v, "cel.w")?,
                        6 => r.h = u32_of(v, "cel.h")?,
                        7 => cel.data = v.to_vec(),
                        8 => {
                            cel.opacity = u8::try_from(uint(v)?)
                                .map_err(|_| CodecError::Invalid("cel.opacity"))?;
                        }
                        9 => {
                            cel.z = i16::try_from(unzigzag(uint(v)?)?)
                                .map_err(|_| CodecError::Invalid("cel.z"))?;
                        }
                        10 => cel.link = Some(FrameId(u32_of(v, "cel.link")?)),
                        _ => {}
                    }
                }
                cel.bbox = r;
                raw_cels.push(((LayerId(l), FrameId(f)), cel));
            }
            9 => s.next_id = u32_of(b, "sprite.next_id")?,
            10 => {
                let mut tag = Tag {
                    name: String::new(),
                    from: 0,
                    to: 0,
                    direction: Direction::Forward,
                    repeat: 0,
                    color: [0, 0, 0, 255],
                };
                for (t, v) in Fields::new(b)? {
                    match t {
                        1 => {
                            tag.name = String::from_utf8(v.to_vec())
                                .map_err(|_| CodecError::Invalid("tag.name"))?;
                        }
                        2 => tag.from = u32_of(v, "tag.from")?,
                        3 => tag.to = u32_of(v, "tag.to")?,
                        4 => {
                            tag.direction = u8::try_from(uint(v)?)
                                .ok()
                                .and_then(Direction::from_code)
                                .ok_or(CodecError::Invalid("tag.direction"))?;
                        }
                        5 => {
                            tag.repeat = u16::try_from(uint(v)?)
                                .map_err(|_| CodecError::Invalid("tag.repeat"))?;
                        }
                        6 => {
                            tag.color = <[u8; 4]>::try_from(v)
                                .map_err(|_| CodecError::Invalid("tag.color"))?;
                        }
                        _ => {}
                    }
                }
                s.tags.push(tag);
            }
            _ => {}
        }
    }
    validate(&mut s, raw_cels)?;
    Ok(s)
}

fn validate(s: &mut Sprite, raw_cels: Vec<((LayerId, FrameId), Cel)>) -> Result<(), CodecError> {
    if s.width == 0 || s.height == 0 || s.width > MAX_SIDE || s.height > MAX_SIDE {
        return Err(CodecError::Invalid("sprite.size"));
    }
    if s.layers.is_empty() || s.frames.is_empty() {
        return Err(CodecError::Invalid("sprite.layers/frames"));
    }
    if s.mode == ColorMode::Indexed && usize::from(s.transparent) >= s.palette.len().max(1) {
        return Err(CodecError::Invalid("sprite.transparent"));
    }
    let mut ids = std::collections::BTreeSet::new();
    let layer_ids = s.layers.iter().map(|l| l.id.0);
    let frame_ids = s.frames.iter().map(|f| f.id.0);
    for id in layer_ids.chain(frame_ids) {
        if id == 0 || id >= s.next_id || !ids.insert(id) {
            return Err(CodecError::Invalid("id"));
        }
    }
    crate::op::check_layers(&s.layers, s.next_id).map_err(|_| CodecError::Invalid("layers"))?;
    crate::op::check_frames(&s.frames, &s.tags, s.next_id)
        .map_err(|_| CodecError::Invalid("frames/tags"))?;
    let all: Vec<_> = raw_cels.into_iter().map(|(k, c)| (k, Some(c))).collect();
    crate::op::check_cels(s, &s.layers, &s.frames, &all, true, s.mode.bpp())
        .map_err(|_| CodecError::Invalid("cel"))?;
    for (key, cel) in all.into_iter().filter_map(|(k, c)| Some((k, c?))) {
        if s.cels.insert(key, cel).is_some() {
            return Err(CodecError::Invalid("cel.key repeated"));
        }
    }
    Ok(())
}

// ── registos ──────────────────────────────────────────────────────────────────

fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

fn put_field(out: &mut Vec<u8>, tag: u64, body: &[u8]) {
    put_varint(out, tag);
    put_varint(out, body.len() as u64);
    out.extend_from_slice(body);
}

fn put_uint(out: &mut Vec<u8>, tag: u64, v: u64) {
    let mut b = Vec::new();
    put_varint(&mut b, v);
    put_field(out, tag, &b);
}

fn zigzag(v: i32) -> u64 {
    u64::from(((v << 1) ^ (v >> 31)) as u32)
}

fn unzigzag(v: u64) -> Result<i32, CodecError> {
    let v = u32::try_from(v).map_err(|_| CodecError::Invalid("zigzag"))?;
    Ok(((v >> 1) as i32) ^ -((v & 1) as i32))
}

fn read_varint(b: &[u8], at: &mut usize) -> Result<u64, CodecError> {
    let mut v = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *b.get(*at).ok_or(CodecError::Truncated)?;
        *at += 1;
        v |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(v);
        }
    }
    Err(CodecError::Invalid("varint"))
}

fn uint(b: &[u8]) -> Result<u64, CodecError> {
    let mut at = 0;
    read_varint(b, &mut at)
}

fn u32_of(b: &[u8], what: &'static str) -> Result<u32, CodecError> {
    u32::try_from(uint(b)?).map_err(|_| CodecError::Invalid(what))
}

/// Os campos `(etiqueta, corpo)` de um registo, até ao 0 final.
struct Fields<'a> {
    items: std::vec::IntoIter<(u64, &'a [u8])>,
}

impl<'a> Fields<'a> {
    fn new(b: &'a [u8]) -> Result<Self, CodecError> {
        let mut items = Vec::new();
        let mut at = 0;
        loop {
            let tag = read_varint(b, &mut at)?;
            if tag == 0 {
                break;
            }
            let len =
                usize::try_from(read_varint(b, &mut at)?).map_err(|_| CodecError::Truncated)?;
            let end = at.checked_add(len).ok_or(CodecError::Truncated)?;
            let body = b.get(at..end).ok_or(CodecError::Truncated)?;
            items.push((tag, body));
            at = end;
        }
        Ok(Self {
            items: items.into_iter(),
        })
    }
}

impl<'a> Iterator for Fields<'a> {
    type Item = (u64, &'a [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}
