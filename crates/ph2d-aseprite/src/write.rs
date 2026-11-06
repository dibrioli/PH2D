//! **O escritor do `.ase`**, pela spec pública. Leis medidas no LibreSprite (`docs/PixelArt/P9_plano.md`
//! §5): o cel ligado repete posição e opacidade do dono (sem isso ele o lê em 0,0, e o Trial separa-o);
//! os dois contadores de chunks levam o número (com `0xFFFF` no velho o Trial desalinha).

use crate::file::{AseCelBody, AseFile, AseTilemap, AseTileset, KIND_TILEMAP};

const FILE_MAGIC: u16 = 0xA5E0;
const FRAME_MAGIC: u16 = 0xF1FA;
const HEADER_LEN: usize = 128;
const FLAG_LAYER_OPACITY_VALID: u32 = 1;
/// Deprecado na spec, mas é o que os leitores antigos usam; 100 ms é o que o LibreSprite grava.
const LEGACY_SPEED: u16 = 100;
const GRID_SIDE: u16 = 16;
const ZLIB_LEVEL: u8 = 6;

const CHUNK_LAYER: u16 = 0x2004;
const CHUNK_CEL: u16 = 0x2005;
const CHUNK_COLOR_PROFILE: u16 = 0x2007;
const CHUNK_TAGS: u16 = 0x2018;
const CHUNK_PALETTE: u16 = 0x2019;
const CHUNK_TILESET: u16 = 0x2023;
/// Tileset com os píxeis dentro do ficheiro, e o tile 0 é o vazio (spec).
const TILESET_EMBEDDED_ZERO_EMPTY: u32 = 2 | 4;
/// O índice que a tela mostra para o tile 1 (spec: 1 por omissão).
const TILESET_BASE_INDEX: u16 = 1;
const BITS_PER_TILE: u16 = 32;
const PROFILE_SRGB: u16 = 1;

/// O que o formato não consegue dizer. Limites do FORMATO (WORD da spec), não nossos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// Uma camada de tilemap aponta para um tileset que o ficheiro não traz.
    Tilemap,
    /// Um cel ligado aponta para um quadro sem imagem naquela camada.
    DanglingLink { frame: usize, layer: u16 },
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tilemap => write!(f, "a tilemap layer has no tileset"),
            Self::DanglingLink { frame, layer } => {
                write!(
                    f,
                    "frame {frame}, layer {layer}: linked cel without an owner"
                )
            }
        }
    }
}

impl std::error::Error for WriteError {}

fn string(out: &mut Vec<u8>, t: &str) {
    let b = t.as_bytes();
    let n = b.len().min(usize::from(u16::MAX));
    out.extend_from_slice(&(n as u16).to_le_bytes());
    out.extend_from_slice(&b[..n]);
}

fn chunk(out: &mut Vec<u8>, kind: u16, body: &[u8]) {
    out.extend_from_slice(&((body.len() + 6) as u32).to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(body);
}

fn u16s(out: &mut Vec<u8>, v: &[u16]) {
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
}

fn palette_chunk(file: &AseFile) -> Vec<u8> {
    let n = file.palette.len() as u32;
    let mut b = Vec::with_capacity(20 + file.palette.len() * 6);
    b.extend_from_slice(&n.to_le_bytes());
    b.extend_from_slice(&0_u32.to_le_bytes());
    b.extend_from_slice(&n.saturating_sub(1).to_le_bytes());
    b.extend_from_slice(&[0; 8]);
    for c in &file.palette {
        b.extend_from_slice(&0_u16.to_le_bytes());
        b.extend_from_slice(c);
    }
    b
}

fn layer_chunk(l: &crate::file::AseLayer) -> Vec<u8> {
    let mut b = Vec::with_capacity(18 + l.name.len());
    u16s(&mut b, &[l.flags, l.kind, l.child_level, 0, 0, l.blend]);
    b.push(l.opacity);
    b.extend_from_slice(&[0; 3]);
    string(&mut b, &l.name);
    if l.kind == KIND_TILEMAP {
        b.extend_from_slice(&l.tileset.to_le_bytes());
    }
    b
}

fn tileset_chunk(t: &AseTileset) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&t.id.to_le_bytes());
    b.extend_from_slice(&TILESET_EMBEDDED_ZERO_EMPTY.to_le_bytes());
    b.extend_from_slice(&t.count.to_le_bytes());
    u16s(&mut b, &[t.tile_w, t.tile_h, TILESET_BASE_INDEX]);
    b.extend_from_slice(&[0; 14]);
    string(&mut b, &t.name);
    let z = miniz_oxide::deflate::compress_to_vec_zlib(&t.pixels, ZLIB_LEVEL);
    b.extend_from_slice(&(z.len() as u32).to_le_bytes());
    b.extend_from_slice(&z);
    b
}

fn tilemap_body(b: &mut Vec<u8>, m: &AseTilemap) {
    u16s(b, &[m.w, m.h, BITS_PER_TILE]);
    for v in [m.tile_mask, m.x_flip, m.y_flip, m.d_flip] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b.extend_from_slice(&[0; 10]);
    let raw: Vec<u8> = m.tiles.iter().flat_map(|t| t.to_le_bytes()).collect();
    b.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(
        &raw, ZLIB_LEVEL,
    ));
}

fn tags_chunk(file: &AseFile) -> Vec<u8> {
    let mut b = Vec::new();
    u16s(&mut b, &[file.tags.len().min(usize::from(u16::MAX)) as u16]);
    b.extend_from_slice(&[0; 8]);
    for t in &file.tags {
        u16s(&mut b, &[t.from, t.to]);
        b.push(t.direction);
        u16s(&mut b, &[t.repeat]);
        b.extend_from_slice(&[0; 6]);
        b.extend_from_slice(&t.rgb);
        b.push(0);
        string(&mut b, &t.name);
    }
    b
}

fn cel_chunk(c: &crate::file::AseCel) -> Result<Vec<u8>, WriteError> {
    let mut b = Vec::new();
    b.extend_from_slice(&c.layer.to_le_bytes());
    b.extend_from_slice(&c.x.to_le_bytes());
    b.extend_from_slice(&c.y.to_le_bytes());
    b.push(c.opacity);
    let kind: u16 = match c.body {
        AseCelBody::Image { .. } => 2,
        AseCelBody::Linked { .. } => 1,
        AseCelBody::Tilemap(_) => 3,
    };
    b.extend_from_slice(&kind.to_le_bytes());
    b.extend_from_slice(&c.z.to_le_bytes());
    b.extend_from_slice(&[0; 5]);
    match &c.body {
        AseCelBody::Image { w, h, pixels } => {
            u16s(&mut b, &[*w, *h]);
            b.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(
                pixels, ZLIB_LEVEL,
            ));
        }
        AseCelBody::Linked { frame } => u16s(&mut b, &[*frame]),
        AseCelBody::Tilemap(m) => tilemap_body(&mut b, m),
    }
    Ok(b)
}

fn header(file: &AseFile, total: usize) -> Vec<u8> {
    let mut h = Vec::with_capacity(HEADER_LEN);
    h.extend_from_slice(&(total as u32).to_le_bytes());
    u16s(
        &mut h,
        &[
            FILE_MAGIC,
            file.frames.len() as u16,
            file.width,
            file.height,
            file.depth,
        ],
    );
    h.extend_from_slice(&FLAG_LAYER_OPACITY_VALID.to_le_bytes());
    u16s(&mut h, &[LEGACY_SPEED]);
    h.extend_from_slice(&[0; 8]);
    h.push(file.transparent_index);
    h.extend_from_slice(&[0; 3]);
    // Número de cores: 0 quer dizer 256 (spec).
    let n = file.palette.len();
    u16s(&mut h, &[if n >= 256 { 0 } else { n as u16 }]);
    h.extend_from_slice(&[1, 1]); // pixel ratio
    h.extend_from_slice(&0_i16.to_le_bytes());
    h.extend_from_slice(&0_i16.to_le_bytes());
    u16s(&mut h, &[GRID_SIDE, GRID_SIDE]);
    h.resize(HEADER_LEN, 0);
    h
}

/// **Grava o documento.** Determinístico: o mesmo documento dá os mesmos bytes.
///
/// # Errors
/// [`WriteError`].
pub fn write_file(file: &AseFile) -> Result<Vec<u8>, WriteError> {
    let sem_tileset = |l: &crate::file::AseLayer| {
        l.kind == KIND_TILEMAP && !file.tilesets.iter().any(|t| t.id == l.tileset)
    };
    if file.layers.iter().any(sem_tileset) {
        return Err(WriteError::Tilemap);
    }
    let mut body = Vec::new();
    for (i, frame) in file.frames.iter().enumerate() {
        let mut chunks = Vec::new();
        let mut n = 0_usize;
        if i == 0 {
            let mut profile = Vec::new();
            u16s(&mut profile, &[PROFILE_SRGB, 0]);
            profile.extend_from_slice(&[0; 12]);
            chunk(&mut chunks, CHUNK_COLOR_PROFILE, &profile);
            n += 1;
            if !file.palette.is_empty() {
                chunk(&mut chunks, CHUNK_PALETTE, &palette_chunk(file));
                n += 1;
            }
            for t in &file.tilesets {
                chunk(&mut chunks, CHUNK_TILESET, &tileset_chunk(t));
                n += 1;
            }
            for l in &file.layers {
                chunk(&mut chunks, CHUNK_LAYER, &layer_chunk(l));
                n += 1;
            }
            if !file.tags.is_empty() {
                chunk(&mut chunks, CHUNK_TAGS, &tags_chunk(file));
                n += 1;
            }
        }
        for c in &frame.cels {
            if let AseCelBody::Linked { frame: owner } = c.body {
                let has = file.frames.get(usize::from(owner)).is_some_and(|f| {
                    f.cels
                        .iter()
                        .any(|o| {
                            o.layer == c.layer
                                && matches!(o.body, AseCelBody::Image { .. } | AseCelBody::Tilemap(_))
                        })
                });
                if !has {
                    return Err(WriteError::DanglingLink {
                        frame: i,
                        layer: c.layer,
                    });
                }
            }
            chunk(&mut chunks, CHUNK_CEL, &cel_chunk(c)?);
            n += 1;
        }
        body.extend_from_slice(&((16 + chunks.len()) as u32).to_le_bytes());
        u16s(
            &mut body,
            &[
                FRAME_MAGIC,
                n.min(usize::from(u16::MAX) - 1) as u16,
                frame.duration_ms,
                0,
            ],
        );
        body.extend_from_slice(&(n as u32).to_le_bytes());
        body.extend_from_slice(&chunks);
    }
    let mut out = header(file, HEADER_LEN + body.len());
    out.extend_from_slice(&body);
    Ok(out)
}
