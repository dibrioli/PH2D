//! **Tilemap**: o tileset do sprite e o mapa de cada cel (`docs/PixelArt/P10_plano.md`).
//!
//! A lei: num cel de tilemap, `Cel.tiles` é a verdade e `Cel.data` é sempre `expand(tiles, tileset)`.
//! Quem lê píxeis (compor, exportar, miniatura) não sabe de tiles; quem escreve passa por [`Op::Tiles`].
//!
//! [`Op::Tiles`]: crate::Op::Tiles

use std::collections::{BTreeMap, BTreeSet};

use crate::op::OpError;
use crate::sprite::{Cel, CelKey, LayerId, LayerKind, Rect, Sprite};

/// O id de um tileset; sai do mesmo contador das camadas e dos frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilesetId(pub u32);

/// Uma célula do mapa: o índice do tile e os flips, no arranjo de bits da spec do `.ase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileRef(pub u32);

impl TileRef {
    /// O tile 0: sempre vazio.
    pub const EMPTY: TileRef = TileRef(0);
    pub const ID_MASK: u32 = 0x1fff_ffff;
    pub const FLIP_X: u32 = 0x8000_0000;
    pub const FLIP_Y: u32 = 0x4000_0000;
    pub const FLIP_D: u32 = 0x2000_0000;

    #[must_use]
    pub const fn new(id: u32, x: bool, y: bool, d: bool) -> Self {
        let mut v = id & Self::ID_MASK;
        if x {
            v |= Self::FLIP_X;
        }
        if y {
            v |= Self::FLIP_Y;
        }
        if d {
            v |= Self::FLIP_D;
        }
        Self(v)
    }

    #[must_use]
    pub const fn id(self) -> u32 {
        self.0 & Self::ID_MASK
    }

    #[must_use]
    pub const fn flip_x(self) -> bool {
        self.0 & Self::FLIP_X != 0
    }

    #[must_use]
    pub const fn flip_y(self) -> bool {
        self.0 & Self::FLIP_Y != 0
    }

    #[must_use]
    pub const fn flip_d(self) -> bool {
        self.0 & Self::FLIP_D != 0
    }

    /// Onde a célula `(sx, sy)` do tile cai na célula do mapa: D troca os eixos, depois X e Y.
    #[must_use]
    pub const fn place(self, sx: u32, sy: u32, tw: u32, th: u32) -> (u32, u32) {
        let (mut dx, mut dy) = if self.flip_d() { (sy, sx) } else { (sx, sy) };
        if self.flip_x() {
            dx = tw - 1 - dx;
        }
        if self.flip_y() {
            dy = th - 1 - dy;
        }
        (dx, dy)
    }
}

/// Um terreno de gabarito: 47 tiles seguidos a partir de `first`, um por forma de vizinhança.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terrain {
    pub name: String,
    pub first: u32,
}

/// Os tiles de um gabarito (`docs/Pixel Art/02_spec_engine_e_porte.md` §15: 256 vizinhanças caem em 47).
pub const TERRAIN_TILES: u32 = 47;

/// O conjunto de tiles. `tiles[0]` é o vazio e nunca muda; cada tile tem `tile_w * tile_h * bpp` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tileset {
    pub id: TilesetId,
    pub name: String,
    pub tile_w: u16,
    pub tile_h: u16,
    pub tiles: Vec<Vec<u8>>,
    pub terrains: Vec<Terrain>,
}

impl Tileset {
    /// Um tileset só com o tile vazio.
    #[must_use]
    pub fn new(
        id: TilesetId,
        name: impl Into<String>,
        tile_w: u16,
        tile_h: u16,
        empty: Vec<u8>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            tile_w,
            tile_h,
            tiles: vec![empty],
            terrains: Vec::new(),
        }
    }

    #[must_use]
    pub fn bytes(&self) -> usize {
        self.tiles.iter().map(Vec::len).sum::<usize>() + std::mem::size_of::<Self>()
    }

    /// O tile 0 fica de fora dos gabaritos, e todos cabem em `count`.
    fn terrains_fit(&self, count: u32) -> bool {
        self.terrains
            .iter()
            .all(|t| t.first > 0 && t.first.saturating_add(TERRAIN_TILES) <= count)
    }

    pub(crate) fn check(&self, bpp: usize) -> Result<(), OpError> {
        let size = usize::from(self.tile_w) * usize::from(self.tile_h) * bpp;
        if size == 0 || self.tiles.is_empty() || self.tiles.iter().any(|t| t.len() != size) {
            return Err(OpError::BadTiles("tileset.size"));
        }
        if !self.terrains_fit(self.tiles.len() as u32) {
            return Err(OpError::BadTiles("tileset.terrain"));
        }
        Ok(())
    }
}

/// O mapa de um cel: `tiles` tem `w * h` células, linha a linha; `(x, y)` é o canto em células do
/// sprite. A grelha é sempre retangular nesta forma; iso e hex nascem com a P10c.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TileGrid {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub tiles: Vec<TileRef>,
}

impl TileGrid {
    /// A caixa, em células do sprite, com tiles de `tw × th`.
    #[must_use]
    pub fn rect(&self, tw: u16, th: u16) -> Rect {
        Rect::new(
            self.x,
            self.y,
            self.w * u32::from(tw),
            self.h * u32::from(th),
        )
    }

    #[must_use]
    pub fn bytes(&self) -> usize {
        self.tiles.len() * size_of::<TileRef>() + size_of::<Self>()
    }
}

/// Escreve o tile `t` na célula `(col, row)` da imagem `out` (`w` células de largura).
fn blit(out: &mut [u8], w: usize, bpp: usize, set: &Tileset, t: TileRef, col: usize, row: usize) {
    let (tw, th) = (u32::from(set.tile_w), u32::from(set.tile_h));
    let Some(src) = set.tiles.get(t.id() as usize) else {
        return;
    };
    for sy in 0..th {
        for sx in 0..tw {
            let (dx, dy) = t.place(sx, sy, tw, th);
            let s = (sy * tw + sx) as usize * bpp;
            let d = ((row * th as usize + dy as usize) * w + col * tw as usize + dx as usize) * bpp;
            out[d..d + bpp].copy_from_slice(&src[s..s + bpp]);
        }
    }
}

/// **A imagem do mapa**: os píxeis que `Cel.data` tem de guardar.
#[must_use]
pub fn expand(grid: &TileGrid, set: &Tileset, bpp: usize) -> Vec<u8> {
    let w = grid.w as usize * usize::from(set.tile_w);
    let h = grid.h as usize * usize::from(set.tile_h);
    let mut out = vec![0; w * h * bpp];
    for (i, &t) in grid.tiles.iter().enumerate() {
        blit(
            &mut out,
            w,
            bpp,
            set,
            t,
            i % grid.w as usize,
            i / grid.w as usize,
        );
    }
    out
}

/// O que [`Op::Tiles`](crate::Op::Tiles) põe no sprite, por esta ordem: tilesets inteiros, número
/// de tiles (novos nascem vazios), píxeis de tiles, e por fim os mapas. `Cel.data` é derivado: o
/// que vier aqui é ignorado, e `apply` escreve-o de novo.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TilesSet {
    pub sets: Vec<(TilesetId, Option<Tileset>)>,
    pub counts: Vec<(TilesetId, u32)>,
    pub tiles: Vec<(TilesetId, u32, Vec<u8>)>,
    pub cels: Vec<(CelKey, Option<Cel>)>,
}

impl TilesSet {
    #[must_use]
    pub fn bytes(&self) -> usize {
        let sets: usize = self
            .sets
            .iter()
            .filter_map(|(_, s)| s.as_ref().map(Tileset::bytes))
            .sum();
        let tiles: usize = self.tiles.iter().map(|(_, _, p)| p.len() + 16).sum();
        let cels: usize = self
            .cels
            .iter()
            .filter_map(|(_, c)| c.as_ref()?.tiles.as_ref().map(|g| g.bytes()))
            .sum();
        sets + tiles + cels + size_of::<Self>() + self.counts.len() * 8
    }
}

/// Layer ▸ New ▸ Tilemap Layer: uma camada acima de `above` com um tileset novo, vazio, de
/// `tile_w × tile_h`. Um passo de desfazer.
pub fn new_tilemap_layer(
    s: &mut Sprite,
    above: LayerId,
    tile_w: u16,
    tile_h: u16,
) -> Option<(crate::Op, LayerId, TilesetId)> {
    if tile_w == 0 || tile_h == 0 {
        return None;
    }
    let id = TilesetId(s.alloc_id());
    let (op, layer) = crate::layers::new_layer(s, above)?;
    let crate::Op::Layers(mut set) = op else {
        return None;
    };
    set.layers.iter_mut().find(|l| l.id == layer)?.kind = LayerKind::Tilemap { tileset: id };
    let bpp = s.mode().bpp();
    let vazio = s.transparent_value().to_le_bytes()[..bpp]
        .repeat(usize::from(tile_w) * usize::from(tile_h));
    let t = Tileset::new(id, "Tileset", tile_w, tile_h, vazio);
    let novo = TilesSet {
        sets: vec![(id, Some(t))],
        ..TilesSet::default()
    };
    let op = crate::Op::Batch(vec![
        crate::Op::Tiles(Box::new(novo)),
        crate::Op::Layers(set),
    ]);
    Some((op, layer, id))
}

/// O tileset de uma camada de tilemap.
fn layer_set(s: &Sprite, id: LayerId) -> Option<TilesetId> {
    match s.layer(id)?.kind {
        LayerKind::Tilemap { tileset } => Some(tileset),
        _ => None,
    }
}

/// O tamanho e o número de tiles de cada tileset DEPOIS de `op`, ou a recusa.
fn final_sizes(s: &Sprite, op: &TilesSet) -> Result<BTreeMap<TilesetId, (u16, u16, u32)>, OpError> {
    let bad = OpError::BadTiles;
    let bpp = s.mode().bpp();
    let mut out: BTreeMap<TilesetId, (u16, u16, u32)> = s
        .tilesets
        .iter()
        .map(|t| (t.id, (t.tile_w, t.tile_h, t.tiles.len() as u32)))
        .collect();
    let mut whole = BTreeSet::new();
    for (id, t) in &op.sets {
        if !whole.insert(*id) {
            return Err(bad("sets.repeated"));
        }
        match t {
            Some(t) => {
                if t.id != *id || id.0 == 0 || id.0 >= s.next_id {
                    return Err(bad("sets.id"));
                }
                t.check(bpp)?;
                out.insert(*id, (t.tile_w, t.tile_h, t.tiles.len() as u32));
            }
            None => {
                if s.layers().iter().any(|l| l.tileset() == Some(*id)) {
                    return Err(bad("sets.in_use"));
                }
                out.remove(id);
            }
        }
    }
    for &(id, n) in &op.counts {
        let e = out.get_mut(&id).ok_or(bad("counts.id"))?;
        if whole.contains(&id) || n == 0 || n > TileRef::ID_MASK {
            return Err(bad("counts"));
        }
        if s.tileset(id).is_some_and(|t| !t.terrains_fit(n)) {
            return Err(bad("counts.terrain"));
        }
        e.2 = n;
    }
    for (id, i, px) in &op.tiles {
        let e = out.get(id).ok_or(bad("tiles.id"))?;
        let size = usize::from(e.0) * usize::from(e.1) * bpp;
        if *i == 0 || *i >= e.2 || px.len() != size {
            return Err(bad("tiles.index"));
        }
    }
    Ok(out)
}

/// As leis de um mapa sob o tileset de tamanho `(tw, th, n)`.
pub(crate) fn check_grid(g: &TileGrid, (tw, th, n): (u16, u16, u32)) -> Result<(), OpError> {
    let bad = OpError::BadTiles;
    if g.w == 0 || g.h == 0 || g.tiles.len() != g.w as usize * g.h as usize {
        return Err(bad("grid.size"));
    }
    if g.tiles.iter().any(|t| t.id() >= n) {
        return Err(bad("grid.tile"));
    }
    if tw != th && g.tiles.iter().any(|t| t.flip_d()) {
        return Err(bad("grid.flip_d"));
    }
    Ok(())
}

/// As leis de `op` sobre o estado atual. Falha sem mudar nada.
fn check(s: &Sprite, op: &TilesSet) -> Result<BTreeMap<TilesetId, (u16, u16, u32)>, OpError> {
    let sizes = final_sizes(s, op)?;
    let novos: BTreeSet<CelKey> = op.cels.iter().map(|(k, _)| *k).collect();
    for (k, c) in &op.cels {
        let Some(c) = c else { continue };
        let set = layer_set(s, k.0).ok_or(OpError::BadCel(*k))?;
        let size = *sizes.get(&set).ok_or(OpError::BadCel(*k))?;
        let g = c.tiles.as_ref().ok_or(OpError::BadCel(*k))?;
        if c.link.is_some() || !s.frames().iter().any(|f| f.id == k.1) {
            return Err(OpError::BadCel(*k));
        }
        check_grid(g, size)?;
    }
    // Encolher ou trocar um tileset não pode deixar um mapa que fica a apontar para o nada.
    let mudam: BTreeSet<TilesetId> = op
        .sets
        .iter()
        .map(|(i, _)| *i)
        .chain(op.counts.iter().map(|(i, _)| *i))
        .collect();
    for (k, c) in s.entries() {
        if novos.contains(k) {
            continue;
        }
        let (Some(g), Some(set)) = (c.tiles.as_ref(), layer_set(s, k.0)) else {
            continue;
        };
        if mudam.contains(&set) {
            check_grid(g, *sizes.get(&set).ok_or(OpError::BadCel(*k))?)?;
        }
    }
    Ok(sizes)
}

/// Aplica e devolve o inverso, sem píxeis derivados dentro dele.
pub(crate) fn apply_tiles(s: &mut Sprite, op: TilesSet) -> Result<TilesSet, OpError> {
    check(s, &op)?;
    let bpp = s.mode().bpp();
    let mut inv = TilesSet::default();
    let mut tudo: BTreeSet<TilesetId> = BTreeSet::new();
    for (id, t) in op.sets {
        let antes = s.tilesets.iter().position(|x| x.id == id);
        inv.sets.push((
            id,
            match (antes, t) {
                (Some(i), Some(t)) => Some(std::mem::replace(&mut s.tilesets[i], t)),
                (Some(i), None) => Some(s.tilesets.remove(i)),
                (None, Some(t)) => {
                    s.tilesets.push(t);
                    None
                }
                (None, None) => None,
            },
        ));
        tudo.insert(id);
    }
    let mut tocados: BTreeMap<TilesetId, BTreeSet<u32>> = BTreeMap::new();
    for (id, n) in op.counts {
        let set = s
            .tilesets
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(OpError::BadTiles("counts.id"))?;
        let velho = set.tiles.len() as u32;
        inv.counts.push((id, velho));
        for i in n..velho {
            inv.tiles
                .push((id, i, std::mem::take(&mut set.tiles[i as usize])));
        }
        let vazio = set.tiles[0].clone();
        set.tiles.resize(n as usize, vazio);
    }
    for (id, i, px) in op.tiles {
        let set = s
            .tilesets
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(OpError::BadTiles("tiles.id"))?;
        let velho = std::mem::replace(&mut set.tiles[i as usize], px);
        // Um tile que nasceu neste op some pelo `counts` do inverso.
        let nasceu = inv.counts.iter().any(|(a, n)| *a == id && i >= *n);
        if !nasceu && !inv.tiles.iter().any(|(a, b, _)| *a == id && *b == i) {
            inv.tiles.push((id, i, velho));
        }
        tocados.entry(id).or_default().insert(i);
    }
    let mut novos = BTreeSet::new();
    for (key, cel) in op.cels {
        let old = match cel {
            Some(mut c) => {
                c.data = Vec::new();
                novos.insert(key);
                s.cels.insert(key, c)
            }
            None => s.cels.remove(&key),
        };
        inv.cels.push((
            key,
            old.map(|o| Cel {
                data: Vec::new(),
                ..o
            }),
        ));
    }
    inv.cels.reverse();
    rederive(s, bpp, &novos, &tudo, &tocados);
    Ok(inv)
}

/// Refaz o `data` dos cels novos e dos que usam um tile mudado; os outros ficam como estão.
fn rederive(
    s: &mut Sprite,
    bpp: usize,
    novos: &BTreeSet<CelKey>,
    tudo: &BTreeSet<TilesetId>,
    tocados: &BTreeMap<TilesetId, BTreeSet<u32>>,
) {
    let chaves: Vec<(CelKey, TilesetId)> = s
        .cels
        .iter()
        .filter(|(_, c)| c.tiles.is_some())
        .filter_map(|(k, _)| Some((*k, layer_set(s, k.0)?)))
        .collect();
    let Sprite { cels, tilesets, .. } = s;
    for (key, id) in chaves {
        let (Some(set), Some(c)) = (tilesets.iter().find(|t| t.id == id), cels.get_mut(&key))
        else {
            continue;
        };
        let Some(g) = c.tiles.as_deref() else {
            continue;
        };
        if novos.contains(&key) || tudo.contains(&id) {
            c.bbox = g.rect(set.tile_w, set.tile_h);
            c.data = expand(g, set, bpp);
        } else if let Some(mudou) = tocados.get(&id) {
            // Só as células com um tile mudado: um traço num tile não refaz o mapa inteiro.
            let w = c.bbox.w as usize;
            for (i, &t) in g.tiles.iter().enumerate() {
                if mudou.contains(&t.id()) {
                    let (col, row) = (i % g.w as usize, i / g.w as usize);
                    blit(&mut c.data, w, bpp, set, t, col, row);
                }
            }
        }
    }
}

/// `Cel.data` de todos os cels de tilemap, de novo (depois de ler um ficheiro).
pub(crate) fn rederive_all(s: &mut Sprite) {
    let bpp = s.mode().bpp();
    let ids: BTreeSet<TilesetId> = s.tilesets.iter().map(|t| t.id).collect();
    rederive(s, bpp, &BTreeSet::new(), &ids, &BTreeMap::new());
}

/// **A lei do módulo**, para os gates: todo cel de tilemap tem o `data` que o mapa dá.
#[must_use]
pub fn derived_is_consistent(s: &Sprite) -> bool {
    let bpp = s.mode().bpp();
    s.entries().all(|(k, c)| {
        let Some(g) = &c.tiles else { return true };
        layer_set(s, k.0)
            .and_then(|id| s.tileset(id))
            .is_some_and(|set| {
                c.bbox == g.rect(set.tile_w, set.tile_h) && c.data == expand(g, set, bpp)
            })
    })
}
