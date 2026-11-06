//! **Manual, Auto e Stack**: as escritas de um traço viram um [`Op::Tiles`]. A lei vem do Pixelorama
//! corrido (`docs/PixelArt/P10_oraculos.md` M1) e o que ela não cobre está no plano, §6.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_pixel_core::{
    CelKey, FrameId, LayerId, Op, Sprite, TERRAIN_TILES, TileGrid, TileRef, TilesSet, Tileset,
    TilesetId,
};

use crate::mapa::Mapa;

/// O destino dos píxeis do traço.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Modo {
    /// Edita o tile no lugar: todas as cópias mudam. Célula vazia não faz nada.
    Manual,
    /// Edita o tile usado uma só vez; dá um tile novo à célula de um tile repartido.
    #[default]
    Auto,
    /// Nunca altera um tile existente.
    Stack,
}

/// Onde o píxel `(dx, dy)` da célula do mapa está no tile: o inverso de [`TileRef::place`].
#[must_use]
pub const fn no_tile(t: TileRef, dx: u32, dy: u32, tw: u32, th: u32) -> (u32, u32) {
    let x = if t.flip_x() { tw - 1 - dx } else { dx };
    let y = if t.flip_y() { th - 1 - dy } else { dy };
    if t.flip_d() { (y, x) } else { (x, y) }
}

/// Os usos de cada tile de `set`, em todo o sprite; um cel ligado não conta à parte.
#[must_use]
pub fn usos(s: &Sprite, set: &Tileset) -> Vec<u32> {
    let mut n = vec![0; set.tiles.len()];
    for g in grelhas(s, set.id).map(|(_, g)| g) {
        for t in &g.tiles {
            if let Some(u) = n.get_mut(t.id() as usize) {
                *u += 1;
            }
        }
    }
    n
}

/// Os mapas (só dos donos) das camadas com este tileset.
fn grelhas(s: &Sprite, id: TilesetId) -> impl Iterator<Item = (CelKey, &TileGrid)> {
    s.entries().filter_map(move |(k, c)| {
        let g = c.tiles.as_deref()?;
        (s.layer(k.0)?.tileset() == Some(id)).then_some((*k, g))
    })
}

fn no_terreno(set: &Tileset, id: u32) -> bool {
    set.terrains
        .iter()
        .any(|k| id >= k.first && id < k.first + TERRAIN_TILES)
}

/// As escritas agrupadas por célula do mapa, em píxeis relativos à célula; a última vence.
fn por_celula(
    s: &Sprite,
    m: &Mapa,
    set: &Tileset,
    escritas: &[(i32, i32, u32)],
) -> BTreeMap<(i32, i32), Vec<(u32, u32, u32)>> {
    let mut out: BTreeMap<_, Vec<_>> = BTreeMap::new();
    let (tw, th) = (i32::from(set.tile_w), i32::from(set.tile_h));
    for &(x, y, v) in escritas.iter().filter(|e| s.bounds().contains(e.0, e.1)) {
        let (c, r) = m.celula(x, y);
        let (ox, oy) = m.origem(c, r);
        debug_assert!(x >= ox && y >= oy && x < ox + tw && y < oy + th);
        out.entry((c, r))
            .or_default()
            .push(((x - ox) as u32, (y - oy) as u32, v));
    }
    out
}

/// O tileset em edição: o de antes, mais o que o gesto já editou e fez nascer.
struct Obra<'a> {
    set: &'a Tileset,
    bpp: usize,
    editados: BTreeMap<u32, Vec<u8>>,
    novos: Vec<Vec<u8>>,
    usos: Vec<u32>,
    /// Os tiles que este gesto deixou sem uso.
    largados: BTreeSet<u32>,
}

impl Obra<'_> {
    fn px(&self, id: u32) -> &[u8] {
        let velhos = self.set.tiles.len() as u32;
        if id >= velhos {
            return &self.novos[(id - velhos) as usize];
        }
        self.editados
            .get(&id)
            .map_or(&self.set.tiles[id as usize], |p| p)
    }

    /// A imagem do tile de `t` depois das escritas, em coordenadas do tile.
    fn resultado(&self, t: TileRef, px: &[(u32, u32, u32)]) -> Vec<u8> {
        let (tw, th) = (u32::from(self.set.tile_w), u32::from(self.set.tile_h));
        let mut out = self.px(t.id()).to_vec();
        for &(dx, dy, v) in px {
            let (sx, sy) = no_tile(t, dx, dy, tw, th);
            let i = (sy * tw + sx) as usize * self.bpp;
            out[i..i + self.bpp].copy_from_slice(&v.to_le_bytes()[..self.bpp]);
        }
        out
    }

    /// Um tile igual a `px`, por cópia exata. O gabarito fica de fora: quem entra nele vira terreno.
    fn igual(&self, px: &[u8]) -> Option<u32> {
        let total = (self.set.tiles.len() + self.novos.len()) as u32;
        (0..total).find(|&id| !no_terreno(self.set, id) && self.px(id) == px)
    }

    fn troca(&mut self, de: u32, para: u32) {
        self.usos[para as usize] += 1;
        self.largados.remove(&para);
        // O vazio não se conta: fora da caixa do mapa também é tile 0.
        if de != 0 {
            self.usos[de as usize] -= 1;
            if self.usos[de as usize] == 0 {
                self.largados.insert(de);
            }
        }
    }

    fn nasce(&mut self, px: Vec<u8>) -> u32 {
        let id = (self.set.tiles.len() + self.novos.len()) as u32;
        self.novos.push(px);
        self.usos.push(0);
        id
    }
}

/// **Uma lista de escritas de células do sprite `(x, y, valor)`, um passo de desfazer.** `None`:
/// nada muda, ou a camada não é de tilemap.
#[must_use]
pub fn aplica(
    s: &Sprite,
    layer: LayerId,
    frame: FrameId,
    escritas: &[(i32, i32, u32)],
    modo: Modo,
) -> Option<Op> {
    let set = s.tileset(s.layer(layer)?.tileset()?)?;
    s.frame_index(frame)?;
    let key = s.owner((layer, frame));
    let mut m = Mapa::de(s, layer, frame, set);
    let grupos = por_celula(s, &m, set, escritas);
    let mut o = Obra {
        set,
        bpp: s.mode().bpp(),
        editados: BTreeMap::new(),
        novos: Vec::new(),
        usos: usos(s, set),
        largados: BTreeSet::new(),
    };
    for ((c, r), px) in grupos {
        let t = m.get(c, r);
        let id = t.id();
        let novo = o.resultado(t, &px);
        if novo == o.px(id) {
            continue;
        }
        if modo == Modo::Manual || no_terreno(set, id) {
            if id != 0 {
                o.editados.insert(id, novo);
            }
            continue;
        }
        if let Some(igual) = o.igual(&novo) {
            o.troca(id, igual);
            m.set(
                c,
                r,
                if igual == 0 {
                    TileRef::EMPTY
                } else {
                    com_flips(t, igual)
                },
            );
        } else if modo == Modo::Auto && id != 0 && o.usos[id as usize] == 1 {
            // Um tile nascido neste gesto também cai aqui: fica nos `novos`.
            let velhos = set.tiles.len() as u32;
            if id >= velhos {
                o.novos[(id - velhos) as usize] = novo;
            } else {
                o.editados.insert(id, novo);
            }
        } else {
            let n = o.nasce(novo);
            o.troca(id, n);
            m.set(c, r, com_flips(t, n));
        }
    }
    fecha(s, key, &m, o, modo)
}

fn com_flips(t: TileRef, id: u32) -> TileRef {
    TileRef::new(id, t.flip_x(), t.flip_y(), t.flip_d())
}

/// O op do gesto. Em Auto, os tiles largados fora de terreno saem do conjunto.
fn fecha(s: &Sprite, key: CelKey, m: &Mapa, o: Obra, modo: Modo) -> Option<Op> {
    let set = o.set;
    let saem: Vec<u32> = o
        .largados
        .iter()
        .copied()
        .filter(|&i| modo == Modo::Auto && !no_terreno(set, i) && o.usos[i as usize] == 0)
        .collect();
    let mudou = m.mudanca(s, key);
    if saem.is_empty() {
        let velhos = set.tiles.len() as u32;
        let counts = if o.novos.is_empty() {
            Vec::new()
        } else {
            vec![(set.id, velhos + o.novos.len() as u32)]
        };
        let editados = o.editados.into_iter().map(|(i, p)| (set.id, i, p));
        let novos = (velhos..).zip(o.novos).map(|(i, p)| (set.id, i, p));
        let t = TilesSet {
            sets: Vec::new(),
            counts,
            tiles: editados.chain(novos).collect(),
            cels: mudou.into_iter().collect(),
        };
        let vazio = t.counts.is_empty() && t.tiles.is_empty() && t.cels.is_empty();
        return (!vazio).then(|| Op::Tiles(Box::new(t)));
    }
    Some(sem_os_largados(s, key, set, o, mudou, &saem))
}

/// O tileset inteiro sem `saem` (ordenados), e todos os mapas renumerados, no mesmo op.
fn sem_os_largados(
    s: &Sprite,
    key: CelKey,
    set: &Tileset,
    o: Obra,
    mudou: Option<(CelKey, Option<ph2d_pixel_core::Cel>)>,
    saem: &[u32],
) -> Op {
    let novo_id = |id: u32| id - saem.partition_point(|&x| x < id) as u32;
    let total = (set.tiles.len() + o.novos.len()) as u32;
    let tiles = (0..total)
        .filter(|i| saem.binary_search(i).is_err())
        .map(|i| o.px(i).to_vec())
        .collect();
    let mut novo_set = Tileset {
        tiles,
        ..set.clone()
    };
    for k in &mut novo_set.terrains {
        k.first = novo_id(k.first);
    }
    let renumera = |g: &TileGrid| TileGrid {
        tiles: g
            .tiles
            .iter()
            .map(|t| com_flips(*t, novo_id(t.id())))
            .collect(),
        ..g.clone()
    };
    let mut cels: Vec<(CelKey, Option<ph2d_pixel_core::Cel>)> = Vec::new();
    let (k_mudou, c_mudou) = match mudou {
        Some((k, c)) => (Some(k), c),
        None => (None, None),
    };
    if k_mudou == Some(key) && c_mudou.is_none() {
        cels.push((key, None));
    }
    let gesto = c_mudou.as_ref().and_then(|c| c.tiles.as_deref());
    for (k, g) in grelhas(s, set.id) {
        if k == key && k_mudou.is_some() {
            continue;
        }
        let r = renumera(g);
        if r != *g {
            let c = s.entry(k).cloned().map(|c| ph2d_pixel_core::Cel {
                tiles: Some(Box::new(r)),
                ..c
            });
            cels.push((k, c));
        }
    }
    if let (Some(k), Some(c), Some(g)) = (k_mudou, c_mudou.clone(), gesto) {
        let tiles = Some(Box::new(renumera(g)));
        cels.push((k, Some(ph2d_pixel_core::Cel { tiles, ..c })));
    }
    Op::Tiles(Box::new(TilesSet {
        sets: vec![(set.id, Some(novo_set))],
        cels,
        ..TilesSet::default()
    }))
}
