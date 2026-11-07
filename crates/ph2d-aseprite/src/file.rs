//! **O documento do ficheiro, sem compor**: camadas, cels, paleta, tags. É o que o `.ase` DIZ; quem
//! quer quadros RGBA passa pelo [`crate::parse`], quem edita (Pixel Art) lê isto. Chunks: spec pública.

use crate::read::Reader;
use crate::{AseError, AseTag};

const HEADER_LEN: usize = 128;
const FLAG_LAYER_OPACITY_VALID: u32 = 1;
const FILE_MAGIC: u16 = 0xA5E0;
const FRAME_MAGIC: u16 = 0xF1FA;
const FRAME_HEADER_LEN: usize = 16;

const CHUNK_OLD_PALETTE_8: u16 = 0x0004;
const CHUNK_OLD_PALETTE_6: u16 = 0x0011;
const CHUNK_LAYER: u16 = 0x2004;
const CHUNK_CEL: u16 = 0x2005;
const CHUNK_TAGS: u16 = 0x2018;
const CHUNK_PALETTE: u16 = 0x2019;
const CHUNK_SLICE: u16 = 0x2022;
const CHUNK_USER_DATA: u16 = 0x2020;
const CHUNK_TILESET: u16 = 0x2023;

/// Flags de camada da spec.
pub const LAYER_VISIBLE: u16 = 1;
pub const LAYER_EDITABLE: u16 = 2;
pub const LAYER_BACKGROUND: u16 = 8;
pub const LAYER_PREFER_LINKED: u16 = 16;
pub const LAYER_COLLAPSED: u16 = 32;
pub const LAYER_REFERENCE: u16 = 64;

/// Tipos de camada da spec.
pub const KIND_IMAGE: u16 = 0;
pub const KIND_GROUP: u16 = 1;
pub const KIND_TILEMAP: u16 = 2;

/// Um `.ase` como está no ficheiro.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AseFile {
    pub width: u16,
    pub height: u16,
    /// 32 RGBA · 16 cinza + alfa · 8 indexado.
    pub depth: u16,
    pub transparent_index: u8,
    pub palette: Vec<[u8; 4]>,
    /// Pela ordem do ficheiro: de baixo para cima, cada grupo antes dos filhos.
    pub layers: Vec<AseLayer>,
    pub frames: Vec<AseFileFrame>,
    pub tags: Vec<AseTag>,
    pub tilesets: Vec<AseTileset>,
    /// Pela ordem do ficheiro.
    pub slices: Vec<AseSlice>,
}

/// Uma slice como está no ficheiro, com o user data que a segue.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AseSlice {
    pub name: String,
    pub color: Option<[u8; 4]>,
    pub text: String,
    /// O user data trazia propriedades tipadas (a P12 é que as guarda).
    pub has_properties: bool,
    /// Em ordem de quadro, seja qual for a do ficheiro (F3).
    pub keys: Vec<AseSliceKey>,
}

/// Uma key. `center` e `pivot` relativos à caixa; centro com largura e altura 0 é sem centro (F3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AseSliceKey {
    pub frame: u32,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub center: Option<(i32, i32, u32, u32)>,
    pub pivot: Option<(i32, i32)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AseLayer {
    pub name: String,
    pub flags: u16,
    pub kind: u16,
    pub child_level: u16,
    /// O número do ficheiro; [`crate::BlendMode::from_file`] traduz.
    pub blend: u16,
    pub opacity: u8,
    /// Só em tilemap: o índice do tileset.
    pub tileset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AseFileFrame {
    pub duration_ms: u16,
    pub cels: Vec<AseCel>,
    /// A paleta inteira, se o ficheiro a troca A PARTIR deste quadro (`None` no quadro 0: é a de `palette`).
    pub palette: Option<Vec<[u8; 4]>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AseCel {
    pub layer: u16,
    pub x: i16,
    pub y: i16,
    pub opacity: u8,
    pub z: i16,
    pub body: AseCelBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AseCelBody {
    /// `w * h * depth/8` bytes.
    Image {
        w: u16,
        h: u16,
        pixels: Vec<u8>,
    },
    /// A imagem é a do cel da mesma camada no quadro `frame`.
    Linked {
        frame: u16,
    },
    Tilemap(AseTilemap),
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AseTilemap {
    pub w: u16,
    pub h: u16,
    pub tile_mask: u32,
    pub x_flip: u32,
    pub y_flip: u32,
    pub d_flip: u32,
    /// `w * h` valores de 32 bits (tile, flips).
    pub tiles: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AseTileset {
    pub id: u32,
    pub name: String,
    /// O bit 4 da spec. Desligado (versões internas antigas): o vazio é `0xffffffff`, e 0 é um tile.
    pub zero_is_empty: bool,
    pub tile_w: u16,
    pub tile_h: u16,
    pub count: u32,
    /// `tile_w * tile_h * count * depth/8` bytes (uma tira vertical); vazio se a tira for externa.
    pub pixels: Vec<u8>,
}

impl AseFile {
    /// A imagem de um tilemap: `(w, h, píxeis)` na profundidade do ficheiro, com os flips da spec.
    /// `None` se a camada não disser que tileset usa.
    #[must_use]
    pub fn expand_tilemap(
        &self,
        map: &AseTilemap,
        tileset_index: u32,
    ) -> Option<(u16, u16, Vec<u8>)> {
        let set = self.tilesets.iter().find(|t| t.id == tileset_index)?;
        let bpp = usize::from(self.depth / 8);
        let (tw, th) = (usize::from(set.tile_w), usize::from(set.tile_h));
        let w = usize::from(map.w) * tw;
        let h = usize::from(map.h) * th;
        let mut out = vec![0_u8; w * h * bpp];
        if bpp == 1 {
            out.fill(self.transparent_index);
        }
        for (i, &t) in map.tiles.iter().enumerate() {
            let id = (t & map.tile_mask) as usize;
            // O tile 0 é o vazio (spec).
            if id == 0 || id >= set.count as usize {
                continue;
            }
            let (cx, cy) = (i % usize::from(map.w), i / usize::from(map.w));
            let (xf, yf, df) = (
                t & map.x_flip != 0,
                t & map.y_flip != 0,
                t & map.d_flip != 0,
            );
            for sy in 0..th {
                for sx in 0..tw {
                    let (mut dx, mut dy) = if df { (sy, sx) } else { (sx, sy) };
                    if xf {
                        dx = tw.saturating_sub(1) - dx;
                    }
                    if yf {
                        dy = th.saturating_sub(1) - dy;
                    }
                    let s = (id * th + sy) * tw * bpp + sx * bpp;
                    let d = ((cy * th + dy) * w + cx * tw + dx) * bpp;
                    if let (Some(src), Some(dst)) =
                        (set.pixels.get(s..s + bpp), out.get_mut(d..d + bpp))
                    {
                        dst.copy_from_slice(src);
                    }
                }
            }
        }
        Some((u16::try_from(w).ok()?, u16::try_from(h).ok()?, out))
    }
}

/// **Lê o ficheiro inteiro, sem compor.**
///
/// # Errors
/// [`AseError`], uma variante por causa nomeada.
pub fn parse_file(bytes: &[u8]) -> Result<AseFile, AseError> {
    let mut r = Reader::new(bytes);
    r.skip(4).ok_or(AseError::Truncated("header"))?; // file size, não confiável
    if r.u16().ok_or(AseError::Truncated("header"))? != FILE_MAGIC {
        return Err(AseError::NotAseprite);
    }
    let frame_count = r.u16().ok_or(AseError::Truncated("header"))?;
    let width = r.u16().ok_or(AseError::Truncated("header"))?;
    let height = r.u16().ok_or(AseError::Truncated("header"))?;
    let depth = r.u16().ok_or(AseError::Truncated("header"))?;
    if width == 0 || height == 0 {
        return Err(AseError::EmptyCanvas);
    }
    if frame_count == 0 {
        return Err(AseError::NoFrames);
    }
    if !matches!(depth, 8 | 16 | 32) {
        return Err(AseError::UnknownColorDepth(depth));
    }
    let flags = r.u32().ok_or(AseError::Truncated("header"))?;
    r.skip(2 + 8).ok_or(AseError::Truncated("header"))?; // speed, dois zeros
    let transparent_index = r.u8().ok_or(AseError::Truncated("header"))?;
    r.seek(HEADER_LEN).ok_or(AseError::Truncated("header"))?;

    let mut b = Build {
        file: AseFile {
            width,
            height,
            depth,
            transparent_index,
            ..AseFile::default()
        },
        modern_palette: false,
        layer_opacity_valid: flags & FLAG_LAYER_OPACITY_VALID != 0,
        after_slice: false,
    };
    for index in 0..usize::from(frame_count) {
        let frame_start = r.pos();
        let size = r.u32().ok_or(AseError::Truncated("frame header"))? as usize;
        if r.u16().ok_or(AseError::Truncated("frame header"))? != FRAME_MAGIC {
            return Err(AseError::BadFrameMagic(index));
        }
        let old_chunks = r.u16().ok_or(AseError::Truncated("frame header"))?;
        let duration_ms = r.u16().ok_or(AseError::Truncated("frame header"))?;
        r.skip(2).ok_or(AseError::Truncated("frame header"))?;
        let new_chunks = r.u32().ok_or(AseError::Truncated("frame header"))?;
        // ⚠️ A spec tem DOIS contadores, e o velho satura em 0xFFFF. O novo manda quando existe.
        let chunks = if new_chunks == 0 {
            u32::from(old_chunks)
        } else {
            new_chunks
        };
        b.file.frames.push(AseFileFrame {
            duration_ms,
            cels: Vec::new(),
            palette: None,
        });
        let mut chunk_at = frame_start + FRAME_HEADER_LEN;
        for _ in 0..chunks {
            r.seek(chunk_at).ok_or(AseError::Truncated("chunk"))?;
            let csize = r.u32().ok_or(AseError::Truncated("chunk"))? as usize;
            let ctype = r.u16().ok_or(AseError::Truncated("chunk"))?;
            // Um chunk mente sobre o próprio tamanho num ficheiro corrompido; avançar por um
            // tamanho de zero seria um laço infinito.
            let body = csize.max(6) - 6;
            let mut c = r.window(body).ok_or(AseError::Truncated("chunk"))?;
            b.chunk(ctype, &mut c, index)?;
            chunk_at += csize.max(6);
        }
        // O tamanho declarado do quadro é a forma de saltar chunks que não conhecemos.
        r.seek(frame_start + size.max(FRAME_HEADER_LEN))
            .ok_or(AseError::Truncated("frame"))?;
    }
    Ok(b.file)
}

fn tilemap(r: &mut Reader<'_>) -> Option<AseTilemap> {
    let (w, h, _bits) = (r.u16()?, r.u16()?, r.u16()?);
    let (tile_mask, x_flip, y_flip, d_flip) = (r.u32()?, r.u32()?, r.u32()?, r.u32()?);
    r.skip(10)?;
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib(r.rest()).ok()?;
    let tiles = raw
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| u32::from_le_bytes(*b))
        .collect();
    Some(AseTilemap {
        w,
        h,
        tile_mask,
        x_flip,
        y_flip,
        d_flip,
        tiles,
    })
}

/// Uma key do chunk 0x2022, com as flags da slice (1 = 9-slice, 2 = pivot).
fn slice_key(r: &mut Reader<'_>, flags: u32) -> Option<AseSliceKey> {
    let i32_of = |v: u32| v as i32;
    let mut k = AseSliceKey {
        frame: r.u32()?,
        x: i32_of(r.u32()?),
        y: i32_of(r.u32()?),
        w: r.u32()?,
        h: r.u32()?,
        ..AseSliceKey::default()
    };
    if flags & 1 != 0 {
        let c = (i32_of(r.u32()?), i32_of(r.u32()?), r.u32()?, r.u32()?);
        k.center = ((c.2, c.3) != (0, 0)).then_some(c);
    }
    if flags & 2 != 0 {
        k.pivot = Some((i32_of(r.u32()?), i32_of(r.u32()?)));
    }
    Some(k)
}

struct Build {
    file: AseFile,
    /// Viu-se um 0x2019: os chunks antigos de paleta deixam de contar (spec).
    modern_palette: bool,
    /// Bit 1 do cabeçalho: sem ele, a opacidade da camada não vale (spec).
    layer_opacity_valid: bool,
    /// O chunk anterior era uma slice: o user data a seguir é dela (spec: «last read chunk»).
    after_slice: bool,
}

impl Build {
    fn chunk(&mut self, ctype: u16, r: &mut Reader<'_>, frame: usize) -> Result<(), AseError> {
        let after_slice = std::mem::replace(&mut self.after_slice, ctype == CHUNK_SLICE);
        match ctype {
            CHUNK_LAYER => self.layer(r),
            CHUNK_CEL => self.cel(r, frame)?,
            CHUNK_PALETTE => self.palette(r),
            CHUNK_OLD_PALETTE_8 if !self.modern_palette => self.old_palette(r, false),
            CHUNK_OLD_PALETTE_6 if !self.modern_palette => self.old_palette(r, true),
            CHUNK_TAGS => self.tags(r),
            CHUNK_SLICE => self.slice(r),
            CHUNK_USER_DATA if after_slice => self.slice_user_data(r),
            CHUNK_TILESET => self.tileset(r, frame)?,
            _ => {}
        }
        Ok(())
    }

    fn layer(&mut self, r: &mut Reader<'_>) {
        let Some(flags) = r.u16() else { return };
        let Some(kind) = r.u16() else { return };
        let Some(child_level) = r.u16() else { return };
        let _ = r.skip(4); // largura/altura default, que a spec manda ignorar
        let Some(blend) = r.u16() else { return };
        let Some(opacity) = r.u8() else { return };
        let _ = r.skip(3);
        let name = r.string().unwrap_or_default();
        let tileset = if kind == KIND_TILEMAP {
            r.u32().unwrap_or(0)
        } else {
            0
        };
        self.file.layers.push(AseLayer {
            name,
            flags,
            kind,
            child_level,
            blend,
            opacity: if self.layer_opacity_valid {
                opacity
            } else {
                255
            },
            tileset,
        });
    }

    /// A paleta do quadro corrente: o 0.º escreve em `file.palette`, os seguintes numa cópia dele.
    fn palette_now(&mut self) -> &mut Vec<[u8; 4]> {
        let f = &mut self.file;
        if f.frames.len() <= 1 {
            return &mut f.palette;
        }
        let vigente = f.frames[..f.frames.len() - 1]
            .iter()
            .rev()
            .find_map(|q| q.palette.clone())
            .unwrap_or_else(|| f.palette.clone());
        let ultimo = f.frames.len() - 1;
        f.frames[ultimo].palette.get_or_insert(vigente)
    }

    fn palette(&mut self, r: &mut Reader<'_>) {
        self.modern_palette = true;
        let Some(size) = r.u32() else { return };
        let Some(first) = r.u32() else { return };
        let Some(last) = r.u32() else { return };
        let _ = r.skip(8);
        let palette = self.palette_now();
        if palette.len() < size as usize {
            palette.resize(size as usize, [0, 0, 0, 0]);
        }
        for i in first..=last.max(first) {
            let Some(flags) = r.u16() else { return };
            let (Some(red), Some(green), Some(blue), Some(alpha)) =
                (r.u8(), r.u8(), r.u8(), r.u8())
            else {
                return;
            };
            if flags & 1 != 0 {
                let _ = r.string(); // o nome da cor, que não usamos
            }
            if let Some(slot) = palette.get_mut(i as usize) {
                *slot = [red, green, blue, alpha];
            }
        }
    }

    /// Os chunks antigos: pacotes de `(salta, n, n × RGB)`; `n == 0` quer dizer 256.
    fn old_palette(&mut self, r: &mut Reader<'_>, six_bits: bool) {
        let Some(packets) = r.u16() else { return };
        let mut at = 0_usize;
        for _ in 0..packets {
            let (Some(skip), Some(n)) = (r.u8(), r.u8()) else {
                return;
            };
            at += usize::from(skip);
            let n = if n == 0 { 256 } else { usize::from(n) };
            for _ in 0..n {
                let (Some(red), Some(green), Some(blue)) = (r.u8(), r.u8(), r.u8()) else {
                    return;
                };
                let up = |v: u8| {
                    if six_bits {
                        ((u16::from(v) * 255 + 31) / 63) as u8
                    } else {
                        v
                    }
                };
                if self.file.palette.len() <= at {
                    self.file.palette.resize(at + 1, [0, 0, 0, 0]);
                }
                self.file.palette[at] = [up(red), up(green), up(blue), 255];
                at += 1;
            }
        }
    }

    fn slice(&mut self, r: &mut Reader<'_>) {
        let (Some(n), Some(flags)) = (r.u32(), r.u32()) else {
            return;
        };
        let _ = r.skip(4);
        let mut slice = AseSlice {
            name: r.string().unwrap_or_default(),
            ..AseSlice::default()
        };
        for _ in 0..n {
            let Some(key) = slice_key(r, flags) else {
                break;
            };
            match slice.keys.binary_search_by_key(&key.frame, |k| k.frame) {
                Ok(i) => slice.keys[i] = key,
                Err(i) => slice.keys.insert(i, key),
            }
        }
        self.file.slices.push(slice);
    }

    fn slice_user_data(&mut self, r: &mut Reader<'_>) {
        let Some(slice) = self.file.slices.last_mut() else {
            return;
        };
        let Some(flags) = r.u32() else { return };
        if flags & 1 != 0 {
            slice.text = r.string().unwrap_or_default();
        }
        if flags & 2 != 0 {
            let (Some(cr), Some(g), Some(b), Some(a)) = (r.u8(), r.u8(), r.u8(), r.u8()) else {
                return;
            };
            slice.color = Some([cr, g, b, a]);
        }
        slice.has_properties = flags & 4 != 0;
    }

    fn tags(&mut self, r: &mut Reader<'_>) {
        let Some(n) = r.u16() else { return };
        let _ = r.skip(8);
        for _ in 0..n {
            let (Some(from), Some(to), Some(direction)) = (r.u16(), r.u16(), r.u8()) else {
                return;
            };
            let repeat = r.u16().unwrap_or(0);
            let _ = r.skip(6);
            let rgb = [
                r.u8().unwrap_or(0),
                r.u8().unwrap_or(0),
                r.u8().unwrap_or(0),
            ];
            let _ = r.u8();
            let name = r.string().unwrap_or_default();
            self.file.tags.push(AseTag {
                name,
                from,
                to,
                direction,
                repeat,
                rgb,
            });
        }
    }

    fn cel(&mut self, r: &mut Reader<'_>, frame: usize) -> Result<(), AseError> {
        let Some(layer) = r.u16() else { return Ok(()) };
        let (Some(x), Some(y), Some(opacity), Some(kind), Some(z)) =
            (r.i16(), r.i16(), r.u8(), r.u16(), r.i16())
        else {
            return Ok(());
        };
        let _ = r.skip(5);
        let body = match kind {
            // Cru e comprimido só diferem no zlib.
            0 | 2 => {
                let (Some(w), Some(h)) = (r.u16(), r.u16()) else {
                    return Ok(());
                };
                let need = usize::from(w) * usize::from(h) * usize::from(self.file.depth / 8);
                let mut pixels = if kind == 0 {
                    r.rest().to_vec()
                } else {
                    miniz_oxide::inflate::decompress_to_vec_zlib(r.rest())
                        .map_err(|_| AseError::BadZlib(frame))?
                };
                if pixels.len() < need {
                    return Err(AseError::Truncated("cel pixels"));
                }
                pixels.truncate(need);
                AseCelBody::Image { w, h, pixels }
            }
            1 => {
                let Some(frame) = r.u16() else { return Ok(()) };
                AseCelBody::Linked { frame }
            }
            // Um tilemap ilegível fica vazio: a camada continua a existir, e quem compõe avisa.
            3 => AseCelBody::Tilemap(tilemap(r).unwrap_or_default()),
            _ => return Ok(()),
        };
        if let Some(f) = self.file.frames.last_mut() {
            f.cels.push(AseCel {
                layer,
                x,
                y,
                opacity,
                z,
                body,
            });
        }
        Ok(())
    }

    fn tileset(&mut self, r: &mut Reader<'_>, frame: usize) -> Result<(), AseError> {
        let (Some(id), Some(flags), Some(count), Some(tile_w), Some(tile_h)) =
            (r.u32(), r.u32(), r.u32(), r.u16(), r.u16())
        else {
            return Ok(());
        };
        let _ = r.skip(2 + 14); // índice base (só de UI) + reservados
        let name = r.string().unwrap_or_default();
        if flags & 1 != 0 {
            let _ = r.skip(8); // ficheiro externo
        }
        let mut pixels = Vec::new();
        if flags & 2 != 0 {
            let Some(n) = r.u32() else { return Ok(()) };
            if let Some(mut w) = r.window(n as usize) {
                pixels = miniz_oxide::inflate::decompress_to_vec_zlib(w.rest())
                    .map_err(|_| AseError::BadZlib(frame))?;
            }
        }
        self.file.tilesets.push(AseTileset {
            id,
            name,
            zero_is_empty: flags & 4 != 0,
            tile_w,
            tile_h,
            count,
            pixels,
        });
        Ok(())
    }
}
