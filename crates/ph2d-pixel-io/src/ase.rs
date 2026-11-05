//! `Sprite ⇄ .ase`: só a tradução; ler e gravar bytes é do `ph2d-aseprite`.

use std::collections::BTreeMap;

use ph2d_aseprite::file::{
    KIND_GROUP, KIND_IMAGE, KIND_TILEMAP, LAYER_BACKGROUND, LAYER_COLLAPSED, LAYER_EDITABLE,
    LAYER_PREFER_LINKED, LAYER_REFERENCE, LAYER_VISIBLE,
};
use ph2d_aseprite::{AseCel, AseCelBody, AseFile, AseFileFrame, AseLayer, AseTag};
use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::codec::SpriteParts;
use ph2d_pixel_core::palette;
use ph2d_pixel_core::{
    Cel, CodecError, ColorMode, Direction, Frame, FrameId, Layer, LayerId, LayerKind, Rect, Sprite,
    Tag,
};

/// O que o ficheiro trazia e o documento não guarda (o artista é avisado, nunca é perda muda).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aviso {
    /// Slices ficam de fora até a P11.
    Slices(usize),
    /// Tilemap virou camada de imagem comum (P10).
    Tilemap,
    /// O ficheiro troca a paleta entre quadros: fica a do primeiro.
    PaletaPorQuadro,
    /// Modo de mistura desconhecido: compõe em Normal.
    ModoDesconhecido,
    /// Uma duração maior que o WORD do formato ficou em 65 535 ms.
    DuracaoCortada,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AseErro {
    Ler(ph2d_aseprite::AseError),
    Escrever(ph2d_aseprite::WriteError),
    Documento(CodecError),
    /// Limite do FORMATO: mais quadros que o WORD do cabeçalho.
    QuadrosDemais(usize),
    /// Limite do FORMATO: um cel fora do SHORT da posição.
    CelForaDoFormato { x: i32, y: i32 },
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
        (matches!(l.kind, LayerKind::Group { collapsed: true }), LAYER_COLLAPSED),
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

/// O documento em `.ase`.
///
/// # Errors
/// [`AseErro`]: limites do formato.
pub fn escrever(s: &Sprite) -> Result<(Vec<u8>, Vec<Aviso>), AseErro> {
    if s.frames().len() > usize::from(u16::MAX) {
        return Err(AseErro::QuadrosDemais(s.frames().len()));
    }
    let ordem = pre_ordem(s);
    let layers = ordem
        .iter()
        .map(|&(l, nivel)| AseLayer {
            name: l.name.clone(),
            flags: bandeiras(l),
            kind: if l.is_group() { KIND_GROUP } else { KIND_IMAGE },
            child_level: nivel,
            blend: u16::from(l.blend.code()),
            opacity: l.opacity,
            tileset: 0,
        })
        .collect();
    let mut avisos = Vec::new();
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
            cels.push(cel_saida(s, i as u16, l.id, c)?);
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
            })
            .collect(),
        ..AseFile::default()
    };
    let bytes = ph2d_aseprite::write_file(&file).map_err(AseErro::Escrever)?;
    Ok((bytes, avisos))
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
                body: AseCelBody::Image { w: 1, h: 1, pixels },
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
    let file = ph2d_aseprite::parse_file(bytes).map_err(AseErro::Ler)?;
    let mut avisos = Vec::new();
    if file.slices > 0 {
        avisos.push(Aviso::Slices(file.slices));
    }
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
        c.parent = abertos.last().copied();
        c.background = l.flags & LAYER_BACKGROUND != 0 && i == 0 && c.parent.is_none() && l.kind != KIND_GROUP;
        c.blend = u8::try_from(l.blend).ok().and_then(BlendMode::from_code).unwrap_or_else(|| {
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
        if l.kind == KIND_TILEMAP && !avisos.contains(&Aviso::Tilemap) {
            avisos.push(Aviso::Tilemap);
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

    let cels = cels_de(&file, &layers, &frames, mode);

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
            color: [t.rgb[0], t.rgb[1], t.rgb[2], 255],
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
    })
    .map_err(AseErro::Documento)?;
    Ok((sprite, avisos))
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

fn cels_de(
    file: &AseFile,
    layers: &[Layer],
    frames: &[Frame],
    mode: ColorMode,
) -> Vec<((LayerId, FrameId), Cel)> {
    let bpp = mode.bpp();
    let mut out: BTreeMap<(LayerId, FrameId), Cel> = BTreeMap::new();
    for (fi, f) in file.frames.iter().enumerate() {
        for c in &f.cels {
            let Some(l) = file.layers.get(usize::from(c.layer)).and(layers.get(usize::from(c.layer))) else {
                continue;
            };
            if l.is_group() {
                continue;
            }
            let key = (l.id, frames[fi].id);
            let cel = match &c.body {
                AseCelBody::Linked { frame } => ligado(file, layers, frames, c, *frame, bpp, &out),
                _ => imagem(file, c).filter(|(b, px)| px.len() == b.area() * bpp).map(|(b, px)| Cel {
                    bbox: b,
                    data: px,
                    opacity: c.opacity,
                    z: c.z,
                    link: None,
                }),
            };
            if let Some(cel) = cel {
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
    })
}
