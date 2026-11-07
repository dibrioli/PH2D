//! **Tilemap**: o tileset do sprite e o mapa de cada cel (`docs/PixelArt/P10_plano.md`).
//!
//! A lei: num cel de tilemap, `Cel.tiles` é a verdade e `Cel.data` é sempre `expand(tiles, tileset)`.
//! Quem lê píxeis (compor, exportar, miniatura) não sabe de tiles; quem escreve passa por [`Op::Tiles`].
//!
//! [`Op::Tiles`]: crate::Op::Tiles

use std::collections::{BTreeMap, BTreeSet};

use crate::blend::BlendMode;
use crate::forma::Forma;
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
    pub forma: Forma,
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
            forma: Forma::Square,
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

/// O mapa de um cel: `tiles` tem `w * h` células, linha a linha; `(x, y)` é o píxel do sprite onde
/// começa a primeira. `(col, row)` é o índice dela: só iso e hex o guardam, em quadrado são 0 e o
/// índice deriva de `(x, y)`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TileGrid {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub tiles: Vec<TileRef>,
    pub col: i32,
    pub row: i32,
}

impl TileGrid {
    /// O índice da primeira célula e o píxel do sprite onde começa a célula `(0, 0)` da forma.
    #[must_use]
    pub fn ancora(&self, forma: Forma, tw: i32, th: i32) -> ((i32, i32), (i32, i32)) {
        if forma == Forma::Square {
            let col = (self.x.div_euclid(tw), self.y.div_euclid(th));
            return (col, (self.x.rem_euclid(tw), self.y.rem_euclid(th)));
        }
        let (ox, oy) = forma.origem(self.col, self.row, tw, th);
        ((self.col, self.row), (self.x - ox, self.y - oy))
    }

    /// Um mapa com a primeira célula em `(col, row)` e a célula `(0, 0)` em `fase`.
    #[must_use]
    pub fn com_ancora(
        forma: Forma,
        (tw, th): (i32, i32),
        (col, row): (i32, i32),
        fase: (i32, i32),
        (w, h): (u32, u32),
        tiles: Vec<TileRef>,
    ) -> Self {
        let (ox, oy) = forma.origem(col, row, tw, th);
        let quadrado = forma == Forma::Square;
        Self {
            x: fase.0 + ox,
            y: fase.1 + oy,
            w,
            h,
            tiles,
            col: if quadrado { 0 } else { col },
            row: if quadrado { 0 } else { row },
        }
    }

    /// A caixa em píxeis do sprite.
    #[must_use]
    pub fn caixa(&self, set: &Tileset) -> Rect {
        let (tw, th) = (u32::from(set.tile_w), u32::from(set.tile_h));
        if set.forma == Forma::Square {
            return Rect::new(self.x, self.y, self.w * tw, self.h * th);
        }
        let lados = (tw as i32, th as i32);
        let ((c, r), (fx, fy)) = self.ancora(set.forma, lados.0, lados.1);
        let (x0, y0, x1, y1) = set
            .forma
            .limites((c, r), (self.w as i32, self.h as i32), lados);
        Rect::new(fx + x0, fy + y0, (x1 - x0) as u32, (y1 - y0) as u32)
    }

    #[must_use]
    pub fn bytes(&self) -> usize {
        self.tiles.len() * size_of::<TileRef>() + size_of::<Self>()
    }
}

/// A imagem de um tile com os flips de `t` já aplicados.
fn virada(src: &[u8], t: TileRef, tw: u32, th: u32, bpp: usize) -> Vec<u8> {
    let mut out = vec![0; src.len()];
    for sy in 0..th {
        for sx in 0..tw {
            let (dx, dy) = t.place(sx, sy, tw, th);
            let s = (sy * tw + sx) as usize * bpp;
            let d = (dy * tw + dx) as usize * bpp;
            out[d..d + bpp].copy_from_slice(&src[s..s + bpp]);
        }
    }
    out
}

/// As linhas `[a, b)` de um tile, depois dos flips de `t`: D torna linhas em colunas, logo são todas.
fn linhas_viradas(t: TileRef, (a, b): (u32, u32), th: u32) -> (u32, u32) {
    if t.flip_d() {
        (0, th)
    } else if t.flip_y() {
        (th - b, th - a)
    } else {
        (a, b)
    }
}

/// Escreve em `out` as células do mapa para as quais `linhas` diz que linhas do tile mudaram, por
/// linha de píxeis. Cada combinação de flips usada vira uma imagem uma vez só (M6:
/// `docs/PixelArt/P10_plano.md` §6.1).
fn desenha(
    out: &mut [u8],
    g: &TileGrid,
    set: &Tileset,
    bpp: usize,
    linhas: impl Fn(u32) -> Option<(u32, u32)>,
) {
    if g.w == 0 {
        return;
    }
    let (tw, th) = (u32::from(set.tile_w), u32::from(set.tile_h));
    let linha = tw as usize * bpp;
    let passo = g.w as usize * linha;
    let mut viradas: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
    for &t in &g.tiles {
        if t.id() != t.0
            && linhas(t.id()).is_some()
            && !viradas.contains_key(&t.0)
            && let Some(src) = set.tiles.get(t.id() as usize)
        {
            viradas.insert(t.0, virada(src, t, tw, th, bpp));
        }
    }
    let mut fontes: Vec<(usize, &[u8], u32, u32)> = Vec::with_capacity(g.w as usize);
    for (row, banda) in g.tiles.chunks(g.w as usize).enumerate() {
        fontes.clear();
        let (mut de, mut ate) = (th, 0);
        for (col, &t) in banda.iter().enumerate() {
            let Some(sujas) = linhas(t.id()) else {
                continue;
            };
            let src = if t.id() == t.0 {
                set.tiles.get(t.id() as usize)
            } else {
                viradas.get(&t.0)
            };
            if let Some(src) = src {
                let (a, b) = linhas_viradas(t, sujas, th);
                fontes.push((col * linha, src, a, b));
                de = de.min(a);
                ate = ate.max(b);
            }
        }
        for ly in de..ate {
            let ini = (row * th as usize + ly as usize) * passo;
            let destino = &mut out[ini..ini + passo];
            let l = ly as usize;
            for &(x, src, a, b) in &fontes {
                if a <= ly && ly < b {
                    destino[x..x + linha].copy_from_slice(&src[l * linha..(l + 1) * linha]);
                }
            }
        }
    }
}

/// **A imagem do mapa**: os píxeis que `Cel.data` tem de guardar.
#[must_use]
pub fn expand(grid: &TileGrid, set: &Tileset, bpp: usize) -> Vec<u8> {
    if set.forma != Forma::Square {
        return expand_forma(grid, set, bpp);
    }
    let w = grid.w as usize * usize::from(set.tile_w);
    let h = grid.h as usize * usize::from(set.tile_h);
    let mut out = vec![0; w * h * bpp];
    let tudo = Some((0, u32::from(set.tile_h)));
    desenha(&mut out, grid, set, bpp, |_| tudo);
    out
}

/// Um píxel de tile por cima do que já lá está: em iso e hex as caixas sobrepõem-se.
fn por_cima(dst: &mut [u8], src: &[u8], vazio: &[u8]) {
    if src == vazio {
        return;
    }
    let meio = src.len() == 4 && src[3] != u8::MAX && dst != vazio;
    if meio {
        let (d, s) = (
            [dst[0], dst[1], dst[2], dst[3]],
            [src[0], src[1], src[2], src[3]],
        );
        dst.copy_from_slice(&crate::blend::blend(BlendMode::Normal, d, s, u8::MAX));
    } else {
        dst.copy_from_slice(src);
    }
}

/// A imagem do mapa em iso e hex. Coluna a coluna, de cima para baixo: a ordem que o Godot desenha
/// (`docs/PixelArt/P10_oraculos.md`, M8a).
fn expand_forma(g: &TileGrid, set: &Tileset, bpp: usize) -> Vec<u8> {
    let (tw, th) = (i32::from(set.tile_w), i32::from(set.tile_h));
    let caixa = g.caixa(set);
    let (lw, lh) = (caixa.w as usize, caixa.h as usize);
    let vazio = &set.tiles[0][..bpp];
    let mut out = vazio.repeat(lw * lh);
    let ((c0, r0), (fx, fy)) = g.ancora(set.forma, tw, th);
    let mut viradas: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
    for i in 0..g.w as i32 {
        for j in 0..g.h as i32 {
            let t = g.tiles[j as usize * g.w as usize + i as usize];
            let Some(src) = set.tiles.get(t.id() as usize).filter(|_| t.id() != 0) else {
                continue;
            };
            let img = viradas
                .entry(t.0)
                .or_insert_with(|| virada(src, t, tw as u32, th as u32, bpp));
            let (ox, oy) = set.forma.origem(c0 + i, r0 + j, tw, th);
            let (dx, dy) = (fx + ox - caixa.x, fy + oy - caixa.y);
            for y in 0..th {
                for x in 0..tw {
                    let s = (y * tw + x) as usize * bpp;
                    let d = ((dy + y) as usize * lw + (dx + x) as usize) * bpp;
                    por_cima(&mut out[d..d + bpp], &img[s..s + bpp], vazio);
                }
            }
        }
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
    new_tilemap_layer_forma(s, above, tile_w, tile_h, Forma::Square)
}

/// [`new_tilemap_layer`] com um tileset da forma pedida.
pub fn new_tilemap_layer_forma(
    s: &mut Sprite,
    above: LayerId,
    tile_w: u16,
    tile_h: u16,
    forma: Forma,
) -> Option<(crate::Op, LayerId, TilesetId)> {
    if tile_w == 0 || tile_h == 0 {
        return None;
    }
    let id = TilesetId(s.alloc_id());
    let (op, layer) = crate::layers::new_layer(s, above)?;
    let crate::Op::Layers(mut set) = op else {
        return None;
    };
    let nova = set.layers.iter_mut().find(|l| l.id == layer)?;
    nova.kind = LayerKind::Tilemap { tileset: id };
    nova.name = nova.name.replacen("Layer", "Tilemap", 1);
    let bpp = s.mode().bpp();
    let vazio = s.transparent_value().to_le_bytes()[..bpp]
        .repeat(usize::from(tile_w) * usize::from(tile_h));
    let t = Tileset {
        forma,
        ..Tileset::new(id, "Tileset", tile_w, tile_h, vazio)
    };
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

/// **Layer ▸ New Tilemap Layer sobre um tileset que já existe**: a camada desenha com os mesmos tiles.
pub fn tilemap_layer_using(
    s: &mut Sprite,
    above: LayerId,
    set: TilesetId,
) -> Option<(crate::Op, LayerId)> {
    s.tileset(set)?;
    let (op, layer) = crate::layers::new_layer(s, above)?;
    let crate::Op::Layers(mut camadas) = op else {
        return None;
    };
    let nova = camadas.layers.iter_mut().find(|l| l.id == layer)?;
    nova.kind = LayerKind::Tilemap { tileset: set };
    nova.name = nova.name.replacen("Layer", "Tilemap", 1);
    Some((crate::Op::Layers(camadas), layer))
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
    let mut tocados: Tocados = BTreeMap::new();
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
        let linha = usize::from(set.tile_w) * bpp;
        let sujas = linhas_sujas(&set.tiles[i as usize], &px, linha);
        let velho = std::mem::replace(&mut set.tiles[i as usize], px);
        // Um tile que nasceu neste op some pelo `counts` do inverso.
        let nasceu = inv.counts.iter().any(|(a, n)| *a == id && i >= *n);
        if !nasceu && !inv.tiles.iter().any(|(a, b, _)| *a == id && *b == i) {
            inv.tiles.push((id, i, velho));
        }
        if let Some((a, b)) = sujas {
            let e = tocados.entry(id).or_default().entry(i).or_insert((a, b));
            *e = (e.0.min(a), e.1.max(b));
        }
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

/// As linhas `[a, b)` que mudaram, por tile, por tileset.
type Tocados = BTreeMap<TilesetId, BTreeMap<u32, (u32, u32)>>;

/// A primeira e a última linha em que dois tiles diferem, ou `None` se são iguais.
fn linhas_sujas(velho: &[u8], novo: &[u8], linha: usize) -> Option<(u32, u32)> {
    let difere = |(a, b): (&[u8], &[u8])| a != b;
    let pares = || velho.chunks(linha).zip(novo.chunks(linha));
    let de = pares().position(difere)?;
    let ate = pares().rposition(difere)?;
    Some((de as u32, ate as u32 + 1))
}

/// Refaz o `data` dos cels novos e dos que usam um tile mudado; os outros ficam como estão.
fn rederive(
    s: &mut Sprite,
    bpp: usize,
    novos: &BTreeSet<CelKey>,
    tudo: &BTreeSet<TilesetId>,
    tocados: &Tocados,
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
        let inteiro = set.forma != Forma::Square && tocados.contains_key(&id);
        if inteiro || novos.contains(&key) || tudo.contains(&id) {
            c.bbox = g.caixa(set);
            c.data = expand(g, set, bpp);
        } else if let Some(mudou) = tocados.get(&id) {
            // Só as linhas mudadas: um traço num tile não refaz o mapa inteiro.
            let mut marca = vec![None; set.tiles.len()];
            for (&i, &sujas) in mudou {
                marca[i as usize] = Some(sujas);
            }
            let linhas = |id: u32| marca.get(id as usize).copied().flatten();
            desenha(&mut c.data, g, set, bpp, linhas);
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
            .is_some_and(|set| c.bbox == g.caixa(set) && c.data == expand(g, set, bpp))
    })
}
