//! O documento: [`Sprite`] = modo de cor + paleta + camadas + frames + cels esparsos.

use std::collections::BTreeMap;

use crate::palette;
pub use crate::rect::Rect;
use crate::selection::Mask;

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

/// O que uma camada é (Aseprite: Layer ▸ New Layer / New Group).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    /// Tem cels com células.
    Image,
    /// Não tem cels: compõe os filhos (`parent == Some(id)`) e mistura o resultado.
    Group { collapsed: bool },
    /// Tem cels com um mapa de tiles; a imagem do cel deriva do mapa e deste tileset.
    Tilemap { tileset: crate::tiles::TilesetId },
}

/// Uma camada. Ordem no `Vec` = z entre irmãos (0 = fundo); a árvore vem de `parent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    /// A camada Background do Aseprite: opaca, no fundo; a borracha pinta a cor de fundo nela.
    pub background: bool,
    /// Trancada (Aseprite: o cadeado, *editable* desligado): nenhuma ferramenta escreve nela.
    pub locked: bool,
    /// Camada de referência (Aseprite: New Reference Layer): vê-se, não se exporta nem funde.
    pub reference: bool,
    /// Camada *continuous* (Layer Properties): o New Frame liga os cels dela em vez de copiar.
    pub continuous: bool,
    pub opacity: u8,
    pub blend: crate::blend::BlendMode,
    pub kind: LayerKind,
    /// O grupo que a contém; `None` = na raiz.
    pub parent: Option<LayerId>,
    /// P12: o user data (Layer Properties).
    pub user: crate::user::UserData,
}

impl Layer {
    /// Uma camada de imagem visível, opaca, Normal, na raiz.
    #[must_use]
    pub fn image(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            background: false,
            locked: false,
            reference: false,
            continuous: false,
            opacity: 255,
            blend: crate::blend::BlendMode::Normal,
            kind: LayerKind::Image,
            parent: None,
            user: crate::user::UserData::default(),
        }
    }

    #[must_use]
    pub fn is_group(&self) -> bool {
        matches!(self.kind, LayerKind::Group { .. })
    }

    /// O tileset, se for camada de tilemap.
    #[must_use]
    pub fn tileset(&self) -> Option<crate::tiles::TilesetId> {
        match self.kind {
            LayerKind::Tilemap { tileset } => Some(tileset),
            _ => None,
        }
    }
}

/// Um frame, com a duração própria em milissegundos (Aseprite: 100 ms por omissão).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub id: FrameId,
    pub duration_ms: u32,
}

/// A imagem de uma camada num frame: `bbox` em células do sprite; fora dela é transparente.
///
/// **Cel ligado** (`link = Some(f)`): a imagem, a posição e a opacidade são as do cel `(camada, f)`
/// — o DONO, que não é ligado —, e só o `z` é dele (medido no Trial, `lk_*.ase`: o Aseprite separa
/// dois cels cujo ficheiro lhes dá posição ou opacidade diferentes, e liga-os com `z` diferente).
/// O cel ligado guarda `bbox` vazia e `data` vazio. O dono de cels ligados pode ter a imagem vazia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cel {
    pub bbox: Rect,
    /// `bbox.area() * bpp` bytes, linha a linha.
    pub data: Vec<u8>,
    /// A opacidade do cel; a efectiva é `mul(camada, cel)` (Trial `co_*.ase`, 7/7 exactos).
    pub opacity: u8,
    /// Desloca o cel na ordem das camadas: a chave de desenho é `(índice + z, z)` (Trial `z_*.ase`).
    pub z: i16,
    pub link: Option<FrameId>,
    /// Num cel de tilemap, a verdade; `data` e `bbox` derivam dele (`tiles::expand`).
    pub tiles: Option<Box<crate::tiles::TileGrid>>,
    /// P12: o user data. Por cel, também num ligado (no `.ase` o `0x2020` segue cada chunk de cel).
    pub user: crate::user::UserData,
}

impl Cel {
    /// Um cel dono, opaco, sem deslocamento.
    #[must_use]
    pub fn image(bbox: Rect, data: Vec<u8>) -> Self {
        Self {
            bbox,
            data,
            opacity: 255,
            z: 0,
            link: None,
            tiles: None,
            user: crate::user::UserData::default(),
        }
    }

    /// Um cel de tilemap; `data` e `bbox` escreve-os quem aplica o [`crate::Op::Tiles`].
    #[must_use]
    pub fn tilemap(grid: crate::tiles::TileGrid) -> Self {
        Self {
            tiles: Some(Box::new(grid)),
            ..Self::image(Rect::default(), Vec::new())
        }
    }

    /// Um cel ligado ao dono do frame `owner`.
    #[must_use]
    pub fn linked(owner: FrameId, z: i16) -> Self {
        Self {
            bbox: Rect::default(),
            data: Vec::new(),
            opacity: 255,
            z,
            link: Some(owner),
            tiles: None,
            user: crate::user::UserData::default(),
        }
    }
}

/// A direcção de uma tag (ids do `.ase`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Forward,
    Reverse,
    PingPong,
    PingPongReverse,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Forward,
        Direction::Reverse,
        Direction::PingPong,
        Direction::PingPongReverse,
    ];

    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Direction::Forward => 0,
            Direction::Reverse => 1,
            Direction::PingPong => 2,
            Direction::PingPongReverse => 3,
        }
    }

    #[must_use]
    pub const fn from_code(c: u8) -> Option<Self> {
        match c {
            0 => Some(Direction::Forward),
            1 => Some(Direction::Reverse),
            2 => Some(Direction::PingPong),
            3 => Some(Direction::PingPongReverse),
            _ => None,
        }
    }
}

/// Uma tag de animação: o trecho `from..=to` (POSIÇÕES na linha do tempo — inserir/apagar um frame
/// mexe nelas, mover não), a direcção e as passadas (`repeat` 0 = para sempre).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub direction: Direction,
    pub repeat: u16,
    pub color: [u8; 4],
    /// P12: o user data. A cor da tag continua a ser `color`; esta é a do user data.
    pub user: crate::user::UserData,
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
    pub(crate) tilesets: Vec<crate::tiles::Tileset>,
    pub(crate) tags: Vec<Tag>,
    pub(crate) slices: Vec<crate::slice::Slice>,
    pub(crate) next_id: u32,
    /// A selecção (Aseprite: estado do DOCUMENTO, desfaz-se; não se grava no `PH2DPIXEL`).
    pub(crate) mask: Option<Mask>,
    /// A última selecção desfeita por Deselect (Select ▸ Reselect).
    pub(crate) last_mask: Option<Mask>,
    /// O último export (File ▸ Repeat Last Export). Não é passo do desfazer.
    pub(crate) export_preset: Option<ExportPreset>,
    /// P12: o user data do sprite (Sprite Properties).
    pub(crate) user: crate::user::UserData,
    /// P12: grade, grade de píxel, xadrez e onion DESTE documento.
    pub(crate) prefs: crate::prefs::PixelPrefs,
}

/// O último export deste sprite: o caminho e as opções em texto (quem as lê é `ph2d-pixel-io`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPreset {
    pub path: String,
    pub options: String,
}

/// A duração por omissão de um frame novo (Aseprite: Frame Properties, 100 ms).
pub const DEFAULT_FRAME_MS: u32 = 100;

impl Sprite {
    /// O diálogo New Sprite: uma camada («Background» se o fundo for opaco, senão «Layer 1»), um
    /// frame, a paleta por omissão.
    pub fn new(spec: NewSprite) -> Result<Self, SpriteError> {
        Self::with_palette(spec, &palette::DEFAULT)
    }

    /// O mesmo, com a paleta `colors` (a *Default Palette* do artista) desde o nascimento — o fundo
    /// opaco de um Indexed procura-se NELA. Vazia ou com mais de [`palette::MAX_COLORS`]: a de
    /// fábrica.
    pub fn with_palette(spec: NewSprite, colors: &[[u8; 4]]) -> Result<Self, SpriteError> {
        let colors = if colors.is_empty() || colors.len() > palette::MAX_COLORS {
            &palette::DEFAULT[..]
        } else {
            colors
        };
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
            palette: colors.to_vec(),
            transparent: 0,
            layers: Vec::new(),
            frames: Vec::new(),
            cels: BTreeMap::new(),
            tilesets: Vec::new(),
            tags: Vec::new(),
            slices: Vec::new(),
            next_id: 1,
            mask: None,
            last_mask: None,
            export_preset: None,
            user: crate::user::UserData::default(),
            prefs: crate::prefs::PixelPrefs::default(),
        };
        let opaque = background != Background::Transparent;
        let layer = s.alloc_id();
        let mut first = Layer::image(
            LayerId(layer),
            if opaque { "Background" } else { "Layer 1" },
        );
        first.background = opaque;
        s.layers.push(first);
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
                .insert((LayerId(layer), FrameId(frame)), Cel::image(bbox, data));
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
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }
    /// A posição do frame `id` na linha do tempo.
    #[must_use]
    pub fn frame_index(&self, id: FrameId) -> Option<usize> {
        self.frames.iter().position(|f| f.id == id)
    }
    /// A IMAGEM do cel `key` (a do dono, se for ligado): caixa, bytes e opacidade.
    #[must_use]
    pub fn cel(&self, key: CelKey) -> Option<&Cel> {
        let c = self.cels.get(&key)?;
        match c.link {
            Some(f) => self.cels.get(&(key.0, f)),
            None => Some(c),
        }
    }
    /// O cel `key` tal como está guardado (um ligado é só `link` e `z`).
    #[must_use]
    pub fn entry(&self, key: CelKey) -> Option<&Cel> {
        self.cels.get(&key)
    }
    /// Os tilesets do sprite.
    #[must_use]
    pub fn tilesets(&self) -> &[crate::tiles::Tileset] {
        &self.tilesets
    }
    /// O tileset `id`.
    #[must_use]
    pub fn tileset(&self, id: crate::tiles::TilesetId) -> Option<&crate::tiles::Tileset> {
        self.tilesets.iter().find(|t| t.id == id)
    }
    /// Todos os cels guardados, por chave.
    pub fn entries(&self) -> impl Iterator<Item = (&CelKey, &Cel)> {
        self.cels.iter()
    }
    /// A chave do dono da imagem de `key` (ela própria, se não for ligado).
    #[must_use]
    pub fn owner(&self, key: CelKey) -> CelKey {
        match self.cels.get(&key).and_then(|c| c.link) {
            Some(f) => (key.0, f),
            None => key,
        }
    }
    /// Os frames cujos cels da camada partilham a imagem de `key` (ele incluído), na ordem da linha
    /// do tempo; vazio se não houver cel.
    #[must_use]
    pub fn linked_frames(&self, key: CelKey) -> Vec<FrameId> {
        if !self.cels.contains_key(&key) {
            return Vec::new();
        }
        let o = self.owner(key);
        self.frames
            .iter()
            .map(|f| f.id)
            .filter(|&f| self.cels.contains_key(&(key.0, f)) && self.owner((key.0, f)) == o)
            .collect()
    }
    #[must_use]
    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }
    /// Visível ela e todos os grupos que a contêm.
    #[must_use]
    pub fn is_shown(&self, id: LayerId) -> bool {
        let mut cur = self.layer(id);
        while let Some(l) = cur {
            if !l.visible {
                return false;
            }
            cur = l.parent.and_then(|p| self.layer(p));
        }
        true
    }

    /// Um id novo para uma camada que um [`crate::Op::Layers`] vai pôr (ids nunca se reciclam,
    /// e o desfazer não os devolve: o refazer reata os cels pelo MESMO id).
    pub fn reserve_id(&mut self) -> u32 {
        self.alloc_id()
    }

    /// O último export deste sprite.
    #[must_use]
    pub fn export_preset(&self) -> Option<&ExportPreset> {
        self.export_preset.as_ref()
    }

    /// Guarda o último export; não é passo do desfazer.
    pub fn set_export_preset(&mut self, p: Option<ExportPreset>) {
        self.export_preset = p;
    }

    /// O user data do sprite; muda por [`crate::Op::UserData`].
    #[must_use]
    pub fn user(&self) -> &crate::user::UserData {
        &self.user
    }

    /// O user data de `owner`, se o dono existe.
    #[must_use]
    pub fn user_of(&self, owner: crate::user::Owner) -> Option<&crate::user::UserData> {
        use crate::user::Owner;
        match owner {
            Owner::Sprite => Some(&self.user),
            Owner::Layer(id) => self.layers.iter().find(|l| l.id == id).map(|l| &l.user),
            Owner::Cel(k) => self.cels.get(&k).map(|c| &c.user),
            Owner::Tag(i) => self.tags.get(i).map(|t| &t.user),
            Owner::Tileset(id) => self.tilesets.iter().find(|t| t.id == id).map(|t| &t.user),
            Owner::Slice(id) => self.slices.iter().find(|c| c.id == id).map(|c| &c.user),
        }
    }

    /// Grade, xadrez e onion deste documento.
    #[must_use]
    pub fn prefs(&self) -> &crate::prefs::PixelPrefs {
        &self.prefs
    }

    /// Como no Aseprite, mudar a grade ou o onion não é passo do desfazer.
    pub fn set_prefs(&mut self, p: crate::prefs::PixelPrefs) {
        self.prefs = p;
    }

    /// A selecção activa (`None` = nada seleccionado).
    #[must_use]
    pub fn mask(&self) -> Option<&Mask> {
        self.mask.as_ref()
    }
    /// A selecção que o Reselect devolve.
    #[must_use]
    pub fn last_mask(&self) -> Option<&Mask> {
        self.last_mask.as_ref()
    }
    /// A célula pode ser escrita pela selecção: sem selecção tudo pode; com ela, só o que ela cobre.
    #[must_use]
    pub fn in_selection(&self, x: i32, y: i32) -> bool {
        self.mask.as_ref().is_none_or(|m| m.get(x, y))
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
            ColorMode::Gray => u32::from(crate::color::luma(c)) | (u32::from(c[3]) << 8),
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
        let Some(cel) = self.cel(key) else {
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
