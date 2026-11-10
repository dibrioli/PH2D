//! `Sprite ⇄ .ase`: só a tradução; ler e gravar bytes é do `ph2d-aseprite`.

use std::collections::BTreeMap;

use ph2d_aseprite::file::{
    KIND_GROUP, KIND_IMAGE, KIND_TILEMAP, LAYER_BACKGROUND, LAYER_COLLAPSED, LAYER_EDITABLE,
    LAYER_PREFER_LINKED, LAYER_REFERENCE, LAYER_VISIBLE,
};
use ph2d_aseprite::{
    AseCel, AseCelBody, AseFile, AseFileFrame, AseLayer, AseTag, AseTilemap, AseTileset,
};
use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::codec::SpriteParts;
use ph2d_pixel_core::palette;
use ph2d_pixel_core::{
    Cel, CodecError, ColorMode, Direction, Frame, FrameId, Layer, LayerId, LayerKind, Rect, Sprite,
    Tag, TileGrid, TileRef, Tileset, TilesetId,
};
use ph2d_pixel_core::{Forma, UserData};

/// O que o ficheiro trazia e o documento não guarda (o artista é avisado, nunca é perda muda).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aviso {
    /// O `.ase` não guarda os terrenos de gabarito: os tiles vão, a ligação entre eles fica no projeto.
    TerrenosFicam,
    /// O `.ase` não tem forma de grelha: uma camada iso ou hex sai como imagem.
    FormaVaiComoImagem,
    /// Camada de tilemap sem tileset legível (externo, ou tira errada): virou camada de imagem.
    Tilemap,
    /// O ficheiro troca a paleta entre quadros: fica a do primeiro.
    PaletaPorQuadro,
    /// Modo de mistura desconhecido: compõe em Normal.
    ModoDesconhecido,
    /// Uma duração maior que o WORD do formato ficou em 65 535 ms.
    DuracaoCortada,
    /// Uma imagem do ficheiro veio com o tamanho errado e ficou de fora.
    CelCorrompido,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AseErro {
    Ler(ph2d_aseprite::AseError),
    Escrever(ph2d_aseprite::WriteError),
    Documento(CodecError),
    /// Limite do FORMATO: mais quadros que o WORD do cabeçalho.
    QuadrosDemais(usize),
    /// Limite do FORMATO: um cel fora do SHORT da posição.
    CelForaDoFormato {
        x: i32,
        y: i32,
    },
}

impl std::fmt::Display for AseErro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ler(e) => e.fmt(f),
            Self::Escrever(e) => e.fmt(f),
            Self::Documento(e) => write!(f, "{e:?}"),
            Self::QuadrosDemais(n) => write!(f, "{n} frames do not fit the .ase format (65535)"),
            Self::CelForaDoFormato { x, y } => {
                write!(f, "a cel at {x},{y} does not fit the .ase format")
            }
        }
    }
}

/// As camadas pela ordem do ficheiro: cada grupo antes dos filhos, irmãos de baixo para cima.
fn pre_ordem(s: &Sprite) -> Vec<(&Layer, u16)> {
    fn desce<'a>(s: &'a Sprite, pai: Option<LayerId>, nivel: u16, out: &mut Vec<(&'a Layer, u16)>) {
        for l in s.layers().iter().filter(|l| l.parent == pai) {
            out.push((l, nivel));
            desce(s, Some(l.id), nivel + 1, out);
        }
    }
    let mut out = Vec::with_capacity(s.layers().len());
    desce(s, None, 0, &mut out);
    out
}

fn bandeiras(l: &Layer) -> u16 {
    let mut f = 0;
    for (cond, bit) in [
        (l.visible, LAYER_VISIBLE),
        (!l.locked, LAYER_EDITABLE),
        (l.background, LAYER_BACKGROUND),
        (l.continuous, LAYER_PREFER_LINKED),
        (l.reference, LAYER_REFERENCE),
        (
            matches!(l.kind, LayerKind::Group { collapsed: true }),
            LAYER_COLLAPSED,
        ),
    ] {
        if cond {
            f |= bit;
        }
    }
    f
}

fn posicao(b: Rect) -> Result<(i16, i16), AseErro> {
    match (i16::try_from(b.x), i16::try_from(b.y)) {
        (Ok(x), Ok(y)) => Ok((x, y)),
        _ => Err(AseErro::CelForaDoFormato { x: b.x, y: b.y }),
    }
}

mod slices;
pub use slices::{COLLIDER_PROP, Colliders, PIVOT_PROP};
use slices::{slices_de, slices_saida, tira_collider, tira_pivo};
mod efeitos;
pub use efeitos::{CRU_PROP, CYCLES_PROP, EFEITOS_PROP, RAMPS_PROP};
mod saida_tiles;
mod user;
use saida_tiles::{mapa_saida, tilesets_saida, vai_em_tiles};

/// O documento em `.ase`.
///
/// # Errors
/// [`AseErro`]: limites do formato.
pub fn escrever(s: &Sprite) -> Result<(Vec<u8>, Vec<Aviso>), AseErro> {
    if s.frames().len() > usize::from(u16::MAX) {
        return Err(AseErro::QuadrosDemais(s.frames().len()));
    }
    let assado = efeitos::assa(s);
    let s: &Sprite = &assado;
    let ordem = pre_ordem(s);
    let ext = user::extensoes(s);
    let layers = ordem
        .iter()
        .map(|&(l, nivel)| AseLayer {
            name: l.name.clone(),
            flags: bandeiras(l),
            kind: match l.kind {
                LayerKind::Group { .. } => KIND_GROUP,
                LayerKind::Tilemap { .. } if vai_em_tiles(s, l) => KIND_TILEMAP,
                LayerKind::Tilemap { .. } | LayerKind::Image => KIND_IMAGE,
            },
            child_level: nivel,
            blend: u16::from(l.blend.code()),
            opacity: l.opacity,
            tileset: if vai_em_tiles(s, l) {
                l.tileset().map_or(0, |t| t.0)
            } else {
                0
            },
            user_data: user::saida_camada(l, &ext),
        })
        .collect();
    let mut avisos = Vec::new();
    if s.tilesets().iter().any(|t| !t.terrains.is_empty()) {
        avisos.push(Aviso::TerrenosFicam);
    }
    if s.tilesets().iter().any(|t| t.forma != Forma::Square) {
        avisos.push(Aviso::FormaVaiComoImagem);
    }
    let mut frames = Vec::with_capacity(s.frames().len());
    for f in s.frames() {
        let duration_ms = u16::try_from(f.duration_ms).unwrap_or_else(|_| {
            if !avisos.contains(&Aviso::DuracaoCortada) {
                avisos.push(Aviso::DuracaoCortada);
            }
            u16::MAX
        });
        let mut cels = Vec::new();
        for (i, &(l, _)) in ordem.iter().enumerate() {
            let Some(c) = s.entry((l.id, f.id)) else {
                continue;
            };
            let mut a = cel_saida(s, i as u16, l.id, c)?;
            a.user_data = user::saida(&c.user, &ext);
            cels.push(a);
        }
        frames.push(AseFileFrame {
            duration_ms,
            cels,
            palette: None,
        });
    }
    let depth = match s.mode() {
        ColorMode::Rgba => 32,
        ColorMode::Gray => 16,
        ColorMode::Indexed => 8,
    };
    let file = AseFile {
        width: s.width() as u16,
        height: s.height() as u16,
        depth,
        transparent_index: s.transparent_index(),
        palette: s.palette().to_vec(),
        layers,
        frames,
        tags: s
            .tags()
            .iter()
            .map(|t| AseTag {
                name: t.name.clone(),
                from: t.from as u16,
                to: t.to as u16,
                direction: t.direction.code(),
                repeat: t.repeat,
                rgb: [t.color[0], t.color[1], t.color[2]],
                // A cor de uma tag no Aseprite 1.3 é a do user data (o Trial mostra-a, não a `rgb`).
                user_data: user::saida(&t.user, &ext).map(|mut u| {
                    u.color = Some(t.color);
                    u
                }),
            })
            .collect(),
        tilesets: tilesets_saida(s)
            .into_iter()
            .map(|mut a| {
                let t = s.tilesets().iter().find(|t| t.id.0 == a.id);
                a.user_data = t.and_then(|t| user::saida(&t.user, &ext));
                a
            })
            .collect(),
        slices: slices_saida(s, &ext),
        user_data: user::saida(s.user(), &ext),
        external_files: ext.clone(),
    };
    let bytes = ph2d_aseprite::write_file(&file).map_err(AseErro::Escrever)?;
    Ok((bytes, avisos))
}

/// Lê um tileset do ficheiro, com o vazio no índice 0. `None`: externo ou com a tira errada; quem
/// chama cai na imagem expandida, e avisa.
fn tileset_de(
    t: &AseTileset,
    id: TilesetId,
    bpp: usize,
    transparent: u8,
    ext: &[ph2d_aseprite::AseExternalFile],
) -> Option<Tileset> {
    let size = usize::from(t.tile_w) * usize::from(t.tile_h) * bpp;
    if size == 0 || t.count == 0 || t.pixels.len() != size * t.count as usize {
        return None;
    }
    let mut tiles: Vec<Vec<u8>> = t.pixels.chunks_exact(size).map(<[u8]>::to_vec).collect();
    if !t.zero_is_empty {
        let vazio = if bpp == 1 {
            vec![transparent; size]
        } else {
            vec![0; size]
        };
        tiles.insert(0, vazio);
    }
    Some(Tileset {
        id,
        name: t.name.clone(),
        tile_w: t.tile_w,
        tile_h: t.tile_h,
        tiles,
        terrains: Vec::new(),
        forma: ph2d_pixel_core::Forma::Square,
        user: user::de(t.user_data.as_ref(), ext),
    })
}

/// O mapa do ficheiro nos nossos bits. Índice fora do tileset, e D em tile não quadrado, são
/// corrupção: a célula fica vazia ou sem o flip, e devolve `true`.
fn mapa_de(m: &AseTilemap, x: i32, y: i32, set: &Tileset, zero_is_empty: bool) -> (TileGrid, bool) {
    let mut corrompido = false;
    let n = set.tiles.len() as u64;
    let tiles = m
        .tiles
        .iter()
        .map(|&v| {
            let cru = u64::from(v & m.tile_mask);
            let id = if zero_is_empty {
                cru
            } else if v == u32::MAX {
                0
            } else {
                cru + 1
            };
            let d = v & m.d_flip != 0;
            if id >= n || (d && set.tile_w != set.tile_h) {
                corrompido = true;
            }
            let id = if id < n { id as u32 } else { 0 };
            TileRef::new(
                id,
                v & m.x_flip != 0,
                v & m.y_flip != 0,
                d && set.tile_w == set.tile_h,
            )
        })
        .collect();
    let g = TileGrid {
        x,
        y,
        w: u32::from(m.w),
        h: u32::from(m.h),
        tiles,
        ..TileGrid::default()
    };
    (g, corrompido)
}

fn cel_saida(s: &Sprite, layer: u16, id: LayerId, c: &Cel) -> Result<AseCel, AseErro> {
    Ok(match c.link {
        // O ligado repete a posição e a opacidade do dono (medido: sem isso o LibreSprite lê-o em 0,0).
        Some(dono) => {
            let d = s.entry((id, dono));
            let b = d.map_or(Rect::default(), |o| o.bbox);
            let (x, y) = posicao(b)?;
            AseCel {
                layer,
                x,
                y,
                opacity: d.map_or(255, |o| o.opacity),
                z: c.z,
                user_data: None,
                body: AseCelBody::Linked {
                    frame: s.frame_index(dono).unwrap_or(0) as u16,
                },
            }
        }
        None if c.bbox.is_empty() => {
            // O dono vazio de ligados: o formato pede uma imagem; uma célula transparente é ela.
            let (x, y) = posicao(c.bbox)?;
            let bpp = s.mode().bpp();
            let mut pixels = vec![0; bpp];
            if s.mode() == ColorMode::Indexed {
                pixels[0] = s.transparent_index();
            }
            AseCel {
                layer,
                x,
                y,
                opacity: c.opacity,
                z: c.z,
                user_data: None,
                body: AseCelBody::Image { w: 1, h: 1, pixels },
            }
        }
        None if c.tiles.is_some() && s.layer(id).is_some_and(|l| vai_em_tiles(s, l)) => {
            let (x, y) = posicao(c.bbox)?;
            AseCel {
                layer,
                x,
                y,
                opacity: c.opacity,
                z: c.z,
                user_data: None,
                body: AseCelBody::Tilemap(mapa_saida(c)),
            }
        }
        None => {
            let (x, y) = posicao(c.bbox)?;
            AseCel {
                layer,
                x,
                y,
                opacity: c.opacity,
                z: c.z,
                user_data: None,
                body: AseCelBody::Image {
                    w: c.bbox.w as u16,
                    h: c.bbox.h as u16,
                    pixels: c.data.clone(),
                },
            }
        }
    })
}

fn modo_de(depth: u16) -> ColorMode {
    match depth {
        32 => ColorMode::Rgba,
        16 => ColorMode::Gray,
        _ => ColorMode::Indexed,
    }
}

/// O documento de um `.ase`. As camadas e os quadros recebem ids de 1 em diante, na ordem do ficheiro.
///
/// # Errors
/// [`AseErro`]: o ficheiro não se lê ou não forma um documento válido.
pub fn ler(bytes: &[u8]) -> Result<(Sprite, Vec<Aviso>), AseErro> {
    ler_com_colliders(bytes).map(|(s, a, _)| (s, a))
}

/// [`ler`], e as slices que o ficheiro marca `ph2d.collider` (a marca sai do user data: o collider
/// é do mundo, P14e D2).
///
/// # Errors
/// [`AseErro`].
pub fn ler_com_colliders(bytes: &[u8]) -> Result<(Sprite, Vec<Aviso>, Colliders), AseErro> {
    let file = ph2d_aseprite::parse_file(bytes).map_err(AseErro::Ler)?;
    let mut avisos = Vec::new();
    if file.frames.iter().any(|f| f.palette.is_some()) {
        avisos.push(Aviso::PaletaPorQuadro);
    }
    let mode = modo_de(file.depth);

    let mut proximo = 1_u32;
    let mut novo_id = || {
        let v = proximo;
        proximo += 1;
        v
    };
    let mut sets: Vec<(u32, Tileset)> = Vec::new();
    for t in &file.tilesets {
        let id = TilesetId(novo_id());
        if let Some(novo) = tileset_de(
            t,
            id,
            mode.bpp(),
            file.transparent_index,
            &file.external_files,
        ) {
            sets.push((t.id, novo));
        }
    }
    let mut layers: Vec<Layer> = Vec::with_capacity(file.layers.len());
    // O grupo aberto em cada nível: o pai de quem vem a seguir é o do nível de cima.
    let mut abertos: Vec<LayerId> = Vec::new();
    for (i, l) in file.layers.iter().enumerate() {
        let id = LayerId(novo_id());
        let nivel = usize::from(l.child_level).min(abertos.len());
        abertos.truncate(nivel);
        let mut c = Layer::image(id, l.name.clone());
        c.visible = l.flags & LAYER_VISIBLE != 0;
        c.locked = l.flags & LAYER_EDITABLE == 0;
        c.continuous = l.flags & LAYER_PREFER_LINKED != 0;
        c.reference = l.flags & LAYER_REFERENCE != 0;
        c.opacity = l.opacity;
        c.user = user::de(l.user_data.as_ref(), &file.external_files);
        // Fora de RGB a camada não guarda uma normal (P14c-pixel 1b (d)): a bandeira cai e a
        // propriedade fica no user data, como veio.
        c.normal = mode == ColorMode::Rgba && user::tira_normal(&mut c.user);
        c.parent = abertos.last().copied();
        c.background =
            l.flags & LAYER_BACKGROUND != 0 && i == 0 && c.parent.is_none() && l.kind != KIND_GROUP;
        c.blend = u8::try_from(l.blend)
            .ok()
            .and_then(BlendMode::from_code)
            .unwrap_or_else(|| {
                if !avisos.contains(&Aviso::ModoDesconhecido) {
                    avisos.push(Aviso::ModoDesconhecido);
                }
                BlendMode::Normal
            });
        if l.kind == KIND_GROUP {
            c.kind = LayerKind::Group {
                collapsed: l.flags & LAYER_COLLAPSED != 0,
            };
            abertos.push(id);
        }
        if l.kind == KIND_TILEMAP {
            match sets.iter().find(|(f, _)| *f == l.tileset) {
                Some((_, t)) => c.kind = LayerKind::Tilemap { tileset: t.id },
                None if !avisos.contains(&Aviso::Tilemap) => avisos.push(Aviso::Tilemap),
                None => {}
            }
        }
        layers.push(c);
    }
    if !layers.iter().any(|l| !l.is_group()) {
        layers.push(Layer::image(LayerId(novo_id()), "Layer 1"));
    }

    let frames: Vec<Frame> = file
        .frames
        .iter()
        .map(|f| Frame {
            id: FrameId(novo_id()),
            // O Aseprite não tem quadro de 0 ms (o seu campo mínimo é 1).
            duration_ms: u32::from(f.duration_ms).max(1),
        })
        .collect();

    let tilesets: Vec<Tileset> = sets.into_iter().map(|(_, t)| t).collect();
    let cels = cels_de(&file, &layers, &frames, mode, &tilesets, &mut avisos);

    let n = frames.len() as u32;
    let tags = file
        .tags
        .iter()
        .filter(|t| u32::from(t.from.min(t.to)) < n)
        .map(|t| Tag {
            name: t.name.clone(),
            from: u32::from(t.from.min(t.to)),
            to: u32::from(t.from.max(t.to)).min(n - 1),
            direction: Direction::from_code(t.direction).unwrap_or_default(),
            repeat: t.repeat,
            color: t
                .user_data
                .as_ref()
                .and_then(|u| u.color)
                .unwrap_or([t.rgb[0], t.rgb[1], t.rgb[2], 255]),
            user: UserData {
                color: [0; 4],
                ..user::de(t.user_data.as_ref(), &file.external_files)
            },
        })
        .collect();

    let mut pal = file.palette.clone();
    if pal.is_empty() {
        pal = palette::DEFAULT.to_vec();
    }
    pal.truncate(palette::MAX_COLORS);
    if mode == ColorMode::Indexed && usize::from(file.transparent_index) >= pal.len() {
        pal.resize(usize::from(file.transparent_index) + 1, [0, 0, 0, 0]);
    }
    let proximo_id = layers
        .iter()
        .map(|l| l.id.0)
        .chain(frames.iter().map(|f| f.id.0))
        .chain(tilesets.iter().map(|t| t.id.0))
        .max()
        .map_or(1, |n| n + 1);
    let mut slices = slices_de(&file, proximo_id);
    let pivo = tira_pivo(&mut slices);
    let colliders = tira_collider(&mut slices);
    let sprite = Sprite::from_parts(SpriteParts {
        width: u32::from(file.width),
        height: u32::from(file.height),
        mode: Some(mode),
        palette: pal,
        transparent: file.transparent_index,
        layers,
        frames,
        cels,
        tags,
        slices,
        tilesets,
        user: user::de(file.user_data.as_ref(), &file.external_files),
    })
    .map_err(AseErro::Documento)?;
    let mut sprite = sprite;
    // O id veio da MESMA lista de slices do documento: o op não tem como recusar.
    if pivo.is_some() {
        let _ = ph2d_pixel_core::apply(&mut sprite, ph2d_pixel_core::Op::PivotSlice(pivo));
    }
    efeitos::repoe(&mut sprite);
    Ok((sprite, avisos, colliders))
}

/// A imagem própria de um cel: a sua, ou a do tilemap expandido.
fn imagem(file: &AseFile, c: &AseCel) -> Option<(Rect, Vec<u8>)> {
    let (w, h, px) = match &c.body {
        AseCelBody::Image { w, h, pixels } => (*w, *h, pixels.clone()),
        AseCelBody::Tilemap(m) => {
            let ts = file.layers.get(usize::from(c.layer))?.tileset;
            file.expand_tilemap(m, ts)?
        }
        AseCelBody::Linked { .. } => return None,
    };
    (w > 0 && h > 0).then(|| {
        (
            Rect::new(i32::from(c.x), i32::from(c.y), u32::from(w), u32::from(h)),
            px,
        )
    })
}

fn cel_de_mapa(
    file: &AseFile,
    l: &Layer,
    c: &AseCel,
    m: &AseTilemap,
    sets: &[Tileset],
    bpp: usize,
    avisos: &mut Vec<Aviso>,
) -> Option<Cel> {
    let set = sets.iter().find(|t| Some(t.id) == l.tileset())?;
    if m.w == 0 || m.h == 0 || m.tiles.len() != usize::from(m.w) * usize::from(m.h) {
        if !avisos.contains(&Aviso::CelCorrompido) {
            avisos.push(Aviso::CelCorrompido);
        }
        return None;
    }
    let zero = file
        .layers
        .get(usize::from(c.layer))
        .and_then(|f| file.tilesets.iter().find(|t| t.id == f.tileset))
        .is_none_or(|t| t.zero_is_empty);
    let (g, corrompido) = mapa_de(m, i32::from(c.x), i32::from(c.y), set, zero);
    if corrompido && !avisos.contains(&Aviso::CelCorrompido) {
        avisos.push(Aviso::CelCorrompido);
    }
    Some(Cel {
        bbox: g.caixa(set),
        data: ph2d_pixel_core::tiles::expand(&g, set, bpp),
        opacity: c.opacity,
        z: c.z,
        link: None,
        tiles: Some(Box::new(g)),
        user: ph2d_pixel_core::UserData::default(),
    })
}

fn cels_de(
    file: &AseFile,
    layers: &[Layer],
    frames: &[Frame],
    mode: ColorMode,
    sets: &[Tileset],
    avisos: &mut Vec<Aviso>,
) -> Vec<((LayerId, FrameId), Cel)> {
    let bpp = mode.bpp();
    let mut out: BTreeMap<(LayerId, FrameId), Cel> = BTreeMap::new();
    for (fi, f) in file.frames.iter().enumerate() {
        for c in &f.cels {
            let Some(l) = file
                .layers
                .get(usize::from(c.layer))
                .and(layers.get(usize::from(c.layer)))
            else {
                continue;
            };
            if l.is_group() {
                continue;
            }
            let key = (l.id, frames[fi].id);
            let cel = match &c.body {
                AseCelBody::Linked { frame } => ligado(file, layers, frames, c, *frame, bpp, &out),
                AseCelBody::Tilemap(m) if l.tileset().is_some() => {
                    cel_de_mapa(file, l, c, m, sets, bpp, avisos)
                }
                _ => imagem(file, c)
                    .filter(|(b, px)| {
                        let certo = px.len() == b.area() * bpp;
                        if !certo && !avisos.contains(&Aviso::CelCorrompido) {
                            avisos.push(Aviso::CelCorrompido);
                        }
                        certo
                    })
                    .map(|(b, px)| Cel {
                        bbox: b,
                        data: px,
                        opacity: c.opacity,
                        z: c.z,
                        link: None,
                        tiles: None,
                        user: ph2d_pixel_core::UserData::default(),
                    }),
            };
            if let Some(mut cel) = cel {
                cel.user = user::de(c.user_data.as_ref(), &file.external_files);
                out.insert(key, cel);
            }
        }
    }
    out.into_iter().collect()
}

/// O cel ligado continua ligado só se bater com o dono em posição e opacidade (medido no Trial,
/// `lk_*.ase`: o Aseprite separa os outros). Separado, é uma cópia da imagem com os números dele.
fn ligado(
    file: &AseFile,
    layers: &[Layer],
    frames: &[Frame],
    c: &AseCel,
    alvo: u16,
    bpp: usize,
    feitos: &BTreeMap<(LayerId, FrameId), Cel>,
) -> Option<Cel> {
    let l = layers.get(usize::from(c.layer))?;
    let f = frames.get(usize::from(alvo))?;
    let d = feitos.get(&(l.id, f.id))?;
    // Ligado a ligado: o dono é o do alvo.
    let (dono_id, dono) = match d.link {
        Some(raiz) => (raiz, feitos.get(&(l.id, raiz))?),
        None => (f.id, d),
    };
    let _ = (file, bpp);
    let (x, y) = (i32::from(c.x), i32::from(c.y));
    if (dono.bbox.x, dono.bbox.y, dono.opacity) == (x, y, c.opacity) {
        return Some(Cel::linked(dono_id, c.z));
    }
    Some(Cel {
        bbox: Rect::new(x, y, dono.bbox.w, dono.bbox.h),
        data: dono.data.clone(),
        opacity: c.opacity,
        z: c.z,
        link: None,
        user: ph2d_pixel_core::UserData::default(),
        tiles: dono.tiles.as_ref().map(|g| {
            Box::new(TileGrid {
                x,
                y,
                ..(**g).clone()
            })
        }),
    })
}

#[cfg(test)]
mod tests;
