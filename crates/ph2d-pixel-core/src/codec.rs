//! O formato gravado: `PH2DPIXEL` + major/minor + registos de campos etiquetados.
//!
//! ```text
//! blob     = MAGIC · major:u16le · minor:u16le · registo(Library)
//! registo  = { etiqueta:varint≥1 · comprimento:varint · bytes }* · 0
//! Library  : 1 Entry* (repetido)
//! Entry    : 1 chave (varint) · 2 Sprite (registo)
//! Sprite   : 1 largura · 2 altura · 3 modo · 4 paleta (n×4 bytes) · 5 transparente · 6 Layer*
//!            · 7 Frame* · 8 Cel* · 9 próximo id · 10 Tag*                (inteiros em varint)
//!            · 13 Slice* (`codec/slices.rs`)                                         [1.5]
//!            · 14 UserData · 15 Prefs (`codec/user.rs`)                              [1.6]
//!            · 16 slice do pivô (id; sem = o centro)                                 [1.7]
//!            · 17 Ramp* · 18 Cycle* · 19 Guias · 20 Audio* (`codec/doc.rs`)           [1.9]
//!            Layer 9 = Efeito* (`codec/doc.rs`)                                      [1.9]
//!            Layer 8 · Cel 12 · Tag 7 · Tileset 8 = UserData; sem = o de omissão     [1.6]
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
use crate::tiles::{Terrain, Tileset, TilesetId};

pub const MAGIC: &[u8; 9] = b"PH2DPIXEL";
pub const MAJOR: u16 = 1;
pub const MINOR: u16 = 9;

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
    let entries: Vec<Vec<u8>> = sprites.iter().map(|(&k, s)| write_entry(k, s)).collect();
    write_library_from_entries(entries.iter().map(Vec::as_slice))
}

/// Uma entrada da biblioteca já com a moldura (`1 · comprimento · Entry`): o que
/// [`write_library_from_entries`] concatena. A biblioteca é a sequência delas, então uma entrada
/// guardada ([`EntryCache`]) dá os mesmos bytes que a escrita inteira.
#[must_use]
pub fn write_entry(key: u64, sprite: &Sprite) -> Vec<u8> {
    let mut entry = Vec::new();
    put_uint(&mut entry, 1, key);
    put_field(&mut entry, 2, &sprite_record(sprite));
    entry.push(0);
    let mut framed = Vec::with_capacity(entry.len() + 10);
    put_field(&mut framed, 1, &entry);
    framed
}

/// O blob a partir das entradas de [`write_entry`], em ordem de chave.
#[must_use]
pub fn write_library_from_entries<'a>(entries: impl Iterator<Item = &'a [u8]>) -> Vec<u8> {
    let entries: Vec<&[u8]> = entries.collect();
    let body: usize = entries.iter().map(|e| e.len()).sum();
    let mut out = Vec::with_capacity(MAGIC.len() + 4 + body + 1);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&MAJOR.to_le_bytes());
    out.extend_from_slice(&MINOR.to_le_bytes());
    for e in &entries {
        out.extend_from_slice(e);
    }
    out.push(0);
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
    sprite_record_with(s, true)
}

/// `pixels = false` salta as células (`Cel.data`, mapas, tiles): o resto do documento, pequeno —
/// a parte que muda sem passar pelo `History` ([`EntryCache`]).
fn sprite_record_with(s: &Sprite, pixels: bool) -> Vec<u8> {
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
            LayerKind::Image | LayerKind::Tilemap { .. } => (false, false),
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
                | (u64::from(l.continuous) << 6)
                | (u64::from(l.normal) << 7),
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
        // MINOR 4. Um binário antigo salta o campo e vê uma camada de imagem com a mesma arte.
        if let Some(t) = l.tileset() {
            put_uint(&mut b, 7, u64::from(t.0));
        }
        user::put_user(&mut b, 8, &l.user);
        doc::put_effects(&mut b, &l.efeitos);
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
        if pixels {
            put_field(&mut b, 7, &c.data);
        }
        if c.opacity != 255 {
            put_uint(&mut b, 8, u64::from(c.opacity));
        }
        if c.z != 0 {
            put_uint(&mut b, 9, zigzag(i32::from(c.z)));
        }
        if let Some(o) = c.link {
            put_uint(&mut b, 10, u64::from(o.0));
        }
        // MINOR 4: `data` continua gravado, para o binário antigo; ao ler, deriva-se do mapa.
        if let Some(g) = c.tiles.as_ref().filter(|_| pixels) {
            put_field(&mut b, 11, &grid_record(g));
        }
        user::put_user(&mut b, 12, &c.user);
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
        user::put_user(&mut b, 7, &t.user);
        b.push(0);
        put_field(&mut r, 10, &b);
    }
    // MINOR 5: as slices.
    for c in &s.slices {
        put_field(&mut r, 13, &slices::slice_record(c));
    }
    // MINOR 4: os tilesets.
    for t in &s.tilesets {
        let mut b = Vec::new();
        put_uint(&mut b, 1, u64::from(t.id.0));
        put_field(&mut b, 2, t.name.as_bytes());
        put_uint(&mut b, 3, u64::from(t.tile_w));
        put_uint(&mut b, 4, u64::from(t.tile_h));
        if pixels {
            put_field(&mut b, 5, &t.tiles.concat());
        }
        for k in &t.terrains {
            let mut c = Vec::new();
            put_field(&mut c, 1, k.name.as_bytes());
            put_uint(&mut c, 2, u64::from(k.first));
            c.push(0);
            put_field(&mut b, 6, &c);
        }
        if t.forma != crate::forma::Forma::Square {
            put_uint(&mut b, 7, u64::from(t.forma.code()));
        }
        user::put_user(&mut b, 8, &t.user);
        b.push(0);
        put_field(&mut r, 12, &b);
    }
    // MINOR 3: o último export. Sem export, o registo fica como no MINOR 2.
    if let Some(p) = &s.export_preset {
        let mut b = Vec::new();
        put_field(&mut b, 1, p.path.as_bytes());
        put_field(&mut b, 2, p.options.as_bytes());
        b.push(0);
        put_field(&mut r, 11, &b);
    }
    user::put_user(&mut r, 14, &s.user);
    // MINOR 7: a slice do pivô (P14).
    if let Some(id) = s.pivot_slice {
        put_uint(&mut r, 16, u64::from(id.0));
    }
    if let Some(p) = user::prefs_record(&s.prefs) {
        put_field(&mut r, 15, &p);
    }
    doc::put_sprite(&mut r, s);
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
        tilesets: Vec::new(),
        tags: Vec::new(),
        slices: Vec::new(),
        pivot_slice: None,
        next_id: 0,
        mask: None,
        last_mask: None,
        export_preset: None,
        user: crate::user::UserData::default(),
        prefs: crate::prefs::PixelPrefs::default(),
        ramps: crate::ramp::PaletteRamps::default(),
        guias: crate::guia::Guias::default(),
        audio: Vec::new(),
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
                            l.normal = f & 128 != 0;
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
                        7 => {
                            l.kind = LayerKind::Tilemap {
                                tileset: TilesetId(u32_of(v, "layer.tileset")?),
                            };
                        }
                        8 => l.user = user::read_user(v)?,
                        9 => l.efeitos.push(doc::read_effect(v)?),
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
                        11 => cel.tiles = Some(Box::new(read_grid(v)?)),
                        12 => cel.user = user::read_user(v)?,
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
                    user: crate::user::UserData::default(),
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
                        7 => tag.user = user::read_user(v)?,
                        _ => {}
                    }
                }
                s.tags.push(tag);
            }
            13 => s.slices.push(slices::read_slice(b)?),
            14 => s.user = user::read_user(b)?,
            15 => s.prefs = user::read_prefs(b)?,
            16 => s.pivot_slice = Some(crate::slice::SliceId(u32_of(b, "sprite.pivot_slice")?)),
            17..=20 => _ = doc::read_sprite_field(&mut s, tag, b)?,
            12 => {
                let mut t = Tileset::new(TilesetId(0), "", 0, 0, Vec::new());
                for (tag, v) in Fields::new(b)? {
                    match tag {
                        1 => t.id = TilesetId(u32_of(v, "tileset.id")?),
                        2 => {
                            t.name = String::from_utf8(v.to_vec())
                                .map_err(|_| CodecError::Invalid("tileset.name"))?;
                        }
                        3 => t.tile_w = u16_of(v, "tileset.w")?,
                        4 => t.tile_h = u16_of(v, "tileset.h")?,
                        5 => t.tiles = vec![v.to_vec()],
                        6 => {
                            let mut k = Terrain {
                                name: String::new(),
                                first: 0,
                            };
                            for (i, c) in Fields::new(v)? {
                                match i {
                                    1 => {
                                        k.name = String::from_utf8(c.to_vec())
                                            .map_err(|_| CodecError::Invalid("terrain.name"))?;
                                    }
                                    2 => k.first = u32_of(c, "terrain.first")?,
                                    _ => {}
                                }
                            }
                            t.terrains.push(k);
                        }
                        7 => {
                            t.forma = u8::try_from(uint(v)?)
                                .ok()
                                .and_then(crate::forma::Forma::from_code)
                                .ok_or(CodecError::Invalid("tileset.forma"))?;
                        }
                        8 => t.user = user::read_user(v)?,
                        _ => {}
                    }
                }
                s.tilesets.push(t);
            }
            11 => {
                let mut p = crate::sprite::ExportPreset {
                    path: String::new(),
                    options: String::new(),
                };
                for (t, v) in Fields::new(b)? {
                    let texto = || {
                        String::from_utf8(v.to_vec())
                            .map_err(|_| CodecError::Invalid("sprite.export_preset"))
                    };
                    match t {
                        1 => p.path = texto()?,
                        2 => p.options = texto()?,
                        _ => {}
                    }
                }
                s.export_preset = Some(p);
            }
            _ => {}
        }
    }
    split_tilesets(&mut s)?;
    validate(&mut s, raw_cels)?;
    Ok(s)
}

/// Os campos de um documento vindo de fora (Open de `.ase`, GIF, PNG…).
#[derive(Debug, Clone, Default)]
pub struct SpriteParts {
    pub width: u32,
    pub height: u32,
    pub mode: Option<ColorMode>,
    pub palette: Vec<[u8; 4]>,
    pub transparent: u8,
    pub layers: Vec<Layer>,
    pub frames: Vec<Frame>,
    pub cels: Vec<((LayerId, FrameId), Cel)>,
    pub tags: Vec<Tag>,
    pub tilesets: Vec<Tileset>,
    pub slices: Vec<crate::slice::Slice>,
    /// P12: o user data do sprite.
    pub user: crate::user::UserData,
}

impl Sprite {
    /// Monta o documento com as MESMAS leis de um `.ph2dproj` lido: ids únicos, camadas, tags, cels.
    ///
    /// # Errors
    /// [`CodecError::Invalid`] com o campo que não passou.
    pub fn from_parts(p: SpriteParts) -> Result<Sprite, CodecError> {
        let last = p
            .layers
            .iter()
            .map(|l| l.id.0)
            .chain(p.frames.iter().map(|f| f.id.0))
            .chain(p.tilesets.iter().map(|t| t.id.0))
            .chain(p.slices.iter().map(|c| c.id.0))
            .max();
        let mut s = Sprite {
            width: p.width,
            height: p.height,
            mode: p.mode.unwrap_or(ColorMode::Rgba),
            palette: p.palette,
            transparent: p.transparent,
            layers: p.layers,
            frames: p.frames,
            cels: BTreeMap::new(),
            tags: p.tags,
            slices: p.slices,
            pivot_slice: None,
            tilesets: p.tilesets,
            next_id: last.map_or(1, |n| n + 1),
            mask: None,
            last_mask: None,
            export_preset: None,
            user: p.user,
            prefs: crate::prefs::PixelPrefs::default(),
            ramps: crate::ramp::PaletteRamps::default(),
            guias: crate::guia::Guias::default(),
            audio: Vec::new(),
        };
        validate(&mut s, p.cels)?;
        Ok(s)
    }
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
    let set_ids = s.tilesets.iter().map(|t| t.id.0);
    for id in layer_ids.chain(frame_ids).chain(set_ids) {
        if id == 0 || id >= s.next_id || !ids.insert(id) {
            return Err(CodecError::Invalid("id"));
        }
    }
    crate::op::check_layers(&s.layers, s.next_id).map_err(|_| CodecError::Invalid("layers"))?;
    crate::op::check_frames(&s.frames, &s.tags, s.next_id)
        .map_err(|_| CodecError::Invalid("frames/tags"))?;
    crate::slice::check_slices(&s.slices, s.next_id).map_err(|_| CodecError::Invalid("slices"))?;
    if s.pivot_slice.is_some_and(|id| s.slice(id).is_none()) {
        return Err(CodecError::Invalid("sprite.pivot_slice"));
    }
    crate::ramp::check_ramps(&s.ramps, s.palette.len(), s.next_id).map_err(CodecError::Invalid)?;
    let all: Vec<_> = raw_cels.into_iter().map(|(k, c)| (k, Some(c))).collect();
    crate::op::check_cels(s, &s.layers, &s.frames, &all, true, s.mode.bpp())
        .map_err(|_| CodecError::Invalid("cel"))?;
    for t in &s.tilesets {
        t.check(s.mode.bpp())
            .map_err(|_| CodecError::Invalid("tileset"))?;
    }
    crate::op::check_layer_sets(&s.layers, &s.tilesets)
        .map_err(|_| CodecError::Invalid("layer.tileset"))?;
    crate::op::check_tile_cels(&s.layers, &s.tilesets, &all)
        .map_err(|_| CodecError::Invalid("cel.tiles"))?;
    for (key, cel) in all.into_iter().filter_map(|(k, c)| Some((k, c?))) {
        if s.cels.insert(key, cel).is_some() {
            return Err(CodecError::Invalid("cel.key repeated"));
        }
    }
    crate::tiles::rederive_all(s);
    Ok(())
}

/// Parte a tira gravada em tiles, agora que o modo de cor é conhecido.
fn split_tilesets(s: &mut Sprite) -> Result<(), CodecError> {
    let bpp = s.mode.bpp();
    for t in &mut s.tilesets {
        let size = usize::from(t.tile_w) * usize::from(t.tile_h) * bpp;
        let raw = t.tiles.concat();
        if size == 0 || raw.is_empty() || raw.len() % size != 0 {
            return Err(CodecError::Invalid("tileset.size"));
        }
        t.tiles = raw.chunks_exact(size).map(<[u8]>::to_vec).collect();
    }
    Ok(())
}

mod cache;
mod doc;
mod grade;
pub use cache::{Entries, EntryCache};
mod slices;
mod user;
use grade::{grid_record, read_grid};

mod wire;
use wire::{Fields, put_field, put_uint, put_varint, u16_of, u32_of, uint, unzigzag, zigzag};
