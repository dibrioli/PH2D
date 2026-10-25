//! O documento: [`Sprite`] = modo de cor + paleta + camadas + frames + cels esparsos.

use std::collections::BTreeMap;

use crate::palette;

/// O maior lado de um sprite: o `WORD` de largura/altura do cabeçalho `.ase` (spec pública do
/// formato) — o recurso é o FORMATO de intercâmbio, não a memória.
pub const MAX_SIDE: u32 = u16::MAX as u32;

/// Modo de cor do sprite inteiro (Sprite ▸ Color Mode do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorMode {
    /// 4 bytes por célula: R, G, B, A (alfa reto).
    Rgba,
    /// 2 bytes por célula: valor de cinza e alfa.
    Gray,
    /// 1 byte por célula: índice na paleta; `palette.transparent` é o transparente.
    Indexed,
}

impl ColorMode {
    /// Bytes por célula.
    #[must_use]
    pub const fn bpp(self) -> usize {
        match self {
            ColorMode::Rgba => 4,
            ColorMode::Gray => 2,
            ColorMode::Indexed => 1,
        }
    }

    pub(crate) const fn code(self) -> u8 {
        match self {
            ColorMode::Rgba => 0,
            ColorMode::Gray => 1,
            ColorMode::Indexed => 2,
        }
    }

    pub(crate) const fn from_code(c: u8) -> Option<Self> {
        match c {
            0 => Some(ColorMode::Rgba),
            1 => Some(ColorMode::Gray),
            2 => Some(ColorMode::Indexed),
            _ => None,
        }
    }
}

/// O fundo do diálogo New Sprite (Transparent · White · Black).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Background {
    #[default]
    Transparent,
    White,
    Black,
}

/// Os campos do diálogo New Sprite. Por omissão: 32 × 32, RGBA, transparente (o Aseprite 1.3.18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewSprite {
    pub width: u32,
    pub height: u32,
    pub mode: ColorMode,
    pub background: Background,
}

impl Default for NewSprite {
    fn default() -> Self {
        Self {
            width: 32,
            height: 32,
            mode: ColorMode::Rgba,
            background: Background::Transparent,
        }
    }
}

/// Id de camada: crescente por sprite, nunca reciclado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerId(pub u32);

/// Id de frame: crescente por sprite, nunca reciclado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameId(pub u32);

/// A chave de um cel: camada × frame.
pub type CelKey = (LayerId, FrameId);

/// Retângulo inteiro em células. `w == 0 || h == 0` é vazio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    #[must_use]
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    #[must_use]
    pub const fn area(self) -> usize {
        self.w as usize * self.h as usize
    }

    #[must_use]
    pub fn contains(self, x: i32, y: i32) -> bool {
        let (x, y) = (i64::from(x), i64::from(y));
        x >= i64::from(self.x)
            && y >= i64::from(self.y)
            && x < i64::from(self.x) + i64::from(self.w)
            && y < i64::from(self.y) + i64::from(self.h)
    }

    /// O menor retângulo que contém os dois; o vazio é o elemento neutro.
    #[must_use]
    pub fn union(self, o: Rect) -> Rect {
        if self.is_empty() {
            return o;
        }
        if o.is_empty() {
            return self;
        }
        let x0 = self.x.min(o.x);
        let y0 = self.y.min(o.y);
        let x1 = (i64::from(self.x) + i64::from(self.w)).max(i64::from(o.x) + i64::from(o.w));
        let y1 = (i64::from(self.y) + i64::from(self.h)).max(i64::from(o.y) + i64::from(o.h));
        Rect::new(
            x0,
            y0,
            (x1 - i64::from(x0)) as u32,
            (y1 - i64::from(y0)) as u32,
        )
    }

    /// A intersecção (vazia quando não se tocam).
    #[must_use]
    pub fn intersect(self, o: Rect) -> Rect {
        let x0 = i64::from(self.x).max(i64::from(o.x));
        let y0 = i64::from(self.y).max(i64::from(o.y));
        let x1 = (i64::from(self.x) + i64::from(self.w)).min(i64::from(o.x) + i64::from(o.w));
        let y1 = (i64::from(self.y) + i64::from(self.h)).min(i64::from(o.y) + i64::from(o.h));
        if x1 <= x0 || y1 <= y0 {
            return Rect::default();
        }
        Rect::new(x0 as i32, y0 as i32, (x1 - x0) as u32, (y1 - y0) as u32)
    }
}

/// Uma camada. Ordem no `Vec` = z (0 = fundo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    /// A camada Background do Aseprite: opaca, no fundo; a borracha pinta a cor de fundo nela.
    pub background: bool,
}

/// Um frame, com a duração própria em milissegundos (Aseprite: 100 ms por omissão).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub id: FrameId,
    pub duration_ms: u32,
}

/// A imagem de uma camada num frame: `bbox` em células do sprite; fora dela é transparente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cel {
    pub bbox: Rect,
    /// `bbox.area() * bpp` bytes, linha a linha.
    pub data: Vec<u8>,
}

/// Erros de construção/validação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpriteError {
    /// Largura/altura fora de `1..=MAX_SIDE`.
    Size { width: u32, height: u32 },
}

/// **O documento de um objecto Pixel Sprite.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) mode: ColorMode,
    pub(crate) palette: Vec<[u8; 4]>,
    pub(crate) transparent: u8,
    pub(crate) layers: Vec<Layer>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) cels: BTreeMap<CelKey, Cel>,
    pub(crate) next_id: u32,
}

/// A duração por omissão de um frame novo (Aseprite: Frame Properties, 100 ms).
pub const DEFAULT_FRAME_MS: u32 = 100;

impl Sprite {
    /// O diálogo New Sprite: uma camada («Background» se o fundo for opaco, senão «Layer 1»), um
    /// frame, a paleta por omissão.
    pub fn new(spec: NewSprite) -> Result<Self, SpriteError> {
        let NewSprite {
            width,
            height,
            mode,
            background,
        } = spec;
        if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
            return Err(SpriteError::Size { width, height });
        }
        let mut s = Sprite {
            width,
            height,
            mode,
            palette: palette::DEFAULT.to_vec(),
            transparent: 0,
            layers: Vec::new(),
            frames: Vec::new(),
            cels: BTreeMap::new(),
            next_id: 1,
        };
        let opaque = background != Background::Transparent;
        let layer = s.alloc_id();
        s.layers.push(Layer {
            id: LayerId(layer),
            name: if opaque { "Background" } else { "Layer 1" }.to_owned(),
            visible: true,
            background: opaque,
        });
        let frame = s.alloc_id();
        s.frames.push(Frame {
            id: FrameId(frame),
            duration_ms: DEFAULT_FRAME_MS,
        });
        if opaque {
            let rgba = match background {
                Background::White => [255, 255, 255, 255],
                _ => [0, 0, 0, 255],
            };
            let v = s.value_of_rgba(rgba);
            let bbox = Rect::new(0, 0, width, height);
            let mut data = Vec::with_capacity(bbox.area() * mode.bpp());
            for _ in 0..bbox.area() {
                s.push_value(&mut data, v);
            }
            s.cels
                .insert((LayerId(layer), FrameId(frame)), Cel { bbox, data });
        }
        Ok(s)
    }

    pub(crate) fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
    #[must_use]
    pub fn mode(&self) -> ColorMode {
        self.mode
    }
    #[must_use]
    pub fn palette(&self) -> &[[u8; 4]] {
        &self.palette
    }
    /// O índice transparente (Indexed).
    #[must_use]
    pub fn transparent_index(&self) -> u8 {
        self.transparent
    }
    #[must_use]
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
    #[must_use]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
    #[must_use]
    pub fn cel(&self, key: CelKey) -> Option<&Cel> {
        self.cels.get(&key)
    }
    #[must_use]
    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }
    /// O retângulo do sprite inteiro.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    /// O valor transparente do modo: RGBA/Gray = tudo zero, Indexed = o índice transparente.
    #[must_use]
    pub fn transparent_value(&self) -> u32 {
        match self.mode {
            ColorMode::Indexed => u32::from(self.transparent),
            _ => 0,
        }
    }

    /// O valor (no modo do sprite) mais próximo de uma cor RGBA. Indexed: o índice de menor
    /// distância euclidiana em RGB entre os opacos (empate → o menor índice); alfa 0 = transparente.
    #[must_use]
    pub fn value_of_rgba(&self, c: [u8; 4]) -> u32 {
        match self.mode {
            ColorMode::Rgba => u32::from_le_bytes(c),
            ColorMode::Gray => {
                let [r, g, b, a] = c;
                // Rec. 601 em inteiros, arredondado (floor(x+0.5)).
                let v = (299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b) + 500) / 1000;
                v | (u32::from(a) << 8)
            }
            ColorMode::Indexed => {
                if c[3] == 0 {
                    return u32::from(self.transparent);
                }
                let mut best = (u32::MAX, u32::from(self.transparent));
                for (i, p) in self.palette.iter().enumerate() {
                    if i == usize::from(self.transparent) {
                        continue;
                    }
                    let d = (0..3)
                        .map(|k| {
                            let e = i32::from(p[k]) - i32::from(c[k]);
                            (e * e) as u32
                        })
                        .sum::<u32>();
                    if d < best.0 {
                        best = (d, i as u32);
                    }
                }
                best.1
            }
        }
    }

    /// A cor RGBA (alfa reto) de um valor no modo do sprite.
    #[must_use]
    pub fn rgba_of_value(&self, v: u32) -> [u8; 4] {
        match self.mode {
            ColorMode::Rgba => v.to_le_bytes(),
            ColorMode::Gray => {
                let g = (v & 0xff) as u8;
                [g, g, g, (v >> 8) as u8]
            }
            ColorMode::Indexed => {
                let i = (v & 0xff) as u8;
                if i == self.transparent {
                    return [0, 0, 0, 0];
                }
                self.palette
                    .get(usize::from(i))
                    .copied()
                    .unwrap_or([0, 0, 0, 0])
            }
        }
    }

    /// O valor da célula `(x, y)` do cel `key` (transparente fora do cel / sem cel).
    #[must_use]
    pub fn value_at(&self, key: CelKey, x: i32, y: i32) -> u32 {
        let Some(cel) = self.cels.get(&key) else {
            return self.transparent_value();
        };
        if !cel.bbox.contains(x, y) {
            return self.transparent_value();
        }
        let i = ((y - cel.bbox.y) as usize * cel.bbox.w as usize + (x - cel.bbox.x) as usize)
            * self.mode.bpp();
        self.read_value(&cel.data[i..i + self.mode.bpp()])
    }

    pub(crate) fn read_value(&self, b: &[u8]) -> u32 {
        match self.mode {
            ColorMode::Rgba => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            ColorMode::Gray => u32::from(b[0]) | (u32::from(b[1]) << 8),
            ColorMode::Indexed => u32::from(b[0]),
        }
    }

    pub(crate) fn push_value(&self, out: &mut Vec<u8>, v: u32) {
        let b = v.to_le_bytes();
        out.extend_from_slice(&b[..self.mode.bpp()]);
    }

    /// Os bytes do retângulo `r` do cel `key` (transparente fora do cel), linha a linha.
    #[must_use]
    pub fn read_rect(&self, key: CelKey, r: Rect) -> Vec<u8> {
        let mut out = Vec::with_capacity(r.area() * self.mode.bpp());
        for y in 0..r.h as i32 {
            for x in 0..r.w as i32 {
                let v = self.value_at(key, r.x + x, r.y + y);
                self.push_value(&mut out, v);
            }
        }
        out
    }
}
