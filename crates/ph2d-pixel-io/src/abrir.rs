//! **Open e Import Sprite Sheet**: um ficheiro vira um documento novo.

use ph2d_imageio::{DecodedImage, DisposeOp, ImageImporter as _, ImportOpts};
use ph2d_imageio_png::PixelPng;
use ph2d_pixel_core::codec::SpriteParts;
use ph2d_pixel_core::palette;
use ph2d_pixel_core::{
    Cel, CodecError, ColorMode, DEFAULT_FRAME_MS, Frame, FrameId, Layer, LayerId, Rect, Sprite,
};

use crate::ase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroAbrir {
    /// A extensão não é de nenhum leitor nosso.
    Formato(String),
    Ler(String),
    Documento(CodecError),
    /// O ficheiro não tem imagem nenhuma.
    Vazio,
    /// Os quadros da sequência não têm todos o mesmo tamanho.
    TamanhosDiferentes,
}

impl std::fmt::Display for ErroAbrir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formato(e) => write!(f, "this file type cannot be opened: {e}"),
            Self::Ler(e) => write!(f, "{e}"),
            Self::Documento(e) => write!(f, "{e:?}"),
            Self::Vazio => write!(f, "the file has no image"),
            Self::TamanhosDiferentes => write!(f, "the frames of the sequence differ in size"),
        }
    }
}

/// A caixa mínima de uma imagem no modo do documento, e os bytes nela; `None` = só transparente.
fn apertada(px: &[u8], w: u32, h: u32, bpp: usize, vazio: &dyn Fn(&[u8]) -> bool) -> Option<(Rect, Vec<u8>)> {
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0_u32, 0_u32);
    for y in 0..h {
        for x in 0..w {
            let i = (y as usize * w as usize + x as usize) * bpp;
            if !vazio(&px[i..i + bpp]) {
                (x0, y0) = (x0.min(x), y0.min(y));
                (x1, y1) = (x1.max(x + 1), y1.max(y + 1));
            }
        }
    }
    if x1 == 0 {
        return None;
    }
    let (cw, ch) = (x1 - x0, y1 - y0);
    let mut d = Vec::with_capacity(cw as usize * ch as usize * bpp);
    for y in y0..y1 {
        let i = (y as usize * w as usize + x0 as usize) * bpp;
        d.extend_from_slice(&px[i..i + cw as usize * bpp]);
    }
    Some((Rect::new(x0 as i32, y0 as i32, cw, ch), d))
}

/// Uma camada, N quadros já no modo do documento.
fn documento(
    w: u32,
    h: u32,
    mode: ColorMode,
    pal: Vec<[u8; 4]>,
    transparent: u8,
    quadros: &[(Vec<u8>, u32)],
) -> Result<Sprite, ErroAbrir> {
    let camada = LayerId(1);
    let bpp = mode.bpp();
    let mut frames = Vec::with_capacity(quadros.len());
    let mut cels = Vec::new();
    for (i, (px, ms)) in quadros.iter().enumerate() {
        let id = FrameId(2 + i as u32);
        frames.push(Frame {
            id,
            duration_ms: (*ms).max(1),
        });
        let vazio = |c: &[u8]| match mode {
            ColorMode::Rgba => c[3] == 0,
            ColorMode::Gray => c[1] == 0,
            ColorMode::Indexed => c[0] == transparent,
        };
        if let Some((b, d)) = apertada(px, w, h, bpp, &vazio) {
            cels.push(((camada, id), Cel::image(b, d)));
        }
    }
    Sprite::from_parts(SpriteParts {
        width: w,
        height: h,
        mode: Some(mode),
        palette: if pal.is_empty() { palette::DEFAULT.to_vec() } else { pal },
        transparent,
        layers: vec![Layer::image(camada, "Layer 1")],
        frames,
        cels,
        tags: Vec::new(),
    })
    .map_err(ErroAbrir::Documento)
}

/// Compõe as sub-imagens de um APNG/GIF no quadro inteiro, com o `dispose` de cada uma.
fn quadros_inteiros(fs: &[ph2d_imageio::AnimFrame]) -> (u32, u32, Vec<(Vec<u8>, u32)>) {
    let w = fs.iter().map(|f| f.offset_xy.0.max(0) as u32 + f.image.width).max().unwrap_or(1);
    let h = fs.iter().map(|f| f.offset_xy.1.max(0) as u32 + f.image.height).max().unwrap_or(1);
    let mut tela = vec![0_u8; w as usize * h as usize * 4];
    let mut out = Vec::with_capacity(fs.len());
    for f in fs {
        let antes = tela.clone();
        let (ox, oy) = (f.offset_xy.0.max(0) as usize, f.offset_xy.1.max(0) as usize);
        for y in 0..f.image.height as usize {
            for x in 0..f.image.width as usize {
                let c = f.image.pixels[y * f.image.width as usize + x].0;
                let i = ((oy + y) * w as usize + ox + x) * 4;
                if c[3] == 255 || tela[i + 3] == 0 || matches!(f.blend_op, ph2d_imageio::AnimBlendOp::Source) {
                    tela[i..i + 4].copy_from_slice(&c);
                } else if c[3] != 0 {
                    // Over de alfa reto, em inteiros.
                    let (sa, da) = (u32::from(c[3]), u32::from(tela[i + 3]));
                    let a = sa + da * (255 - sa) / 255;
                    for e in 0..3 {
                        let v = (u32::from(c[e]) * sa + u32::from(tela[i + e]) * da * (255 - sa) / 255) / a.max(1);
                        tela[i + e] = v.min(255) as u8;
                    }
                    tela[i + 3] = a as u8;
                }
            }
        }
        out.push((tela.clone(), f.delay_ms));
        match f.dispose_op {
            DisposeOp::None => {}
            DisposeOp::Background => {
                for y in 0..f.image.height as usize {
                    let i = ((oy + y) * w as usize + ox) * 4;
                    tela[i..i + f.image.width as usize * 4].fill(0);
                }
            }
            DisposeOp::Previous => tela = antes,
        }
    }
    (w, h, out)
}

fn de_decodificado(d: DecodedImage) -> Result<(u32, u32, Vec<(Vec<u8>, u32)>), ErroAbrir> {
    match d {
        DecodedImage::Flat(b) => {
            let px = b.pixels.iter().flat_map(|p| p.0).collect();
            Ok((b.width, b.height, vec![(px, DEFAULT_FRAME_MS)]))
        }
        DecodedImage::Animated(fs) if !fs.is_empty() => Ok(quadros_inteiros(&fs)),
        _ => Err(ErroAbrir::Vazio),
    }
}

fn png_doc(bytes: &[u8]) -> Result<Sprite, ErroAbrir> {
    let ler = |e: ph2d_imageio::Error| ErroAbrir::Ler(e.to_string());
    let (w, h, modo, d) = ph2d_imageio_png::decode_pixel(bytes).map_err(ler)?;
    let um = vec![(d, DEFAULT_FRAME_MS)];
    match modo {
        PixelPng::Rgba => documento(w, h, ColorMode::Rgba, Vec::new(), 0, &um),
        PixelPng::GrayAlpha => documento(w, h, ColorMode::Gray, Vec::new(), 0, &um),
        PixelPng::Indexed(pal) => {
            let t = pal.iter().position(|c| c[3] == 0).unwrap_or(0) as u8;
            documento(w, h, ColorMode::Indexed, pal, t, &um)
        }
    }
}

fn tem_actl(bytes: &[u8]) -> bool {
    bytes.windows(4).any(|w| w == b"acTL")
}

/// **Abre um ficheiro** pela extensão.
///
/// # Errors
/// [`ErroAbrir`].
pub fn abrir(bytes: &[u8], extensao: &str) -> Result<(Sprite, Vec<ase::Aviso>), ErroAbrir> {
    let ler = |e: ph2d_imageio::Error| ErroAbrir::Ler(e.to_string());
    let opts = ImportOpts::default();
    let rgba = |(w, h, q): (u32, u32, Vec<(Vec<u8>, u32)>)| documento(w, h, ColorMode::Rgba, Vec::new(), 0, &q);
    let doc = match extensao.to_ascii_lowercase().as_str() {
        "ase" | "aseprite" => {
            return ase::ler(bytes).map_err(|e| ErroAbrir::Ler(e.to_string()));
        }
        "png" | "apng" if tem_actl(bytes) => rgba(de_decodificado(
            ph2d_imageio_apng::ApngImporter.import(bytes, &opts).map_err(ler)?,
        )?)?,
        "png" | "apng" => png_doc(bytes)?,
        "gif" => rgba(de_decodificado(
            ph2d_imageio_gif::GifImporter.import(bytes, &opts).map_err(ler)?,
        )?)?,
        "webp" => rgba(ph2d_imageio_webp::decode_frames(bytes).map_err(ler)?)?,
        outra => return Err(ErroAbrir::Formato(outra.to_owned())),
    };
    Ok((doc, Vec::new()))
}

/// **Uma sequência de PNG** vira quadros, pela ordem dada. O modo é o do primeiro, se todos tiverem o
/// mesmo; senão RGBA.
///
/// # Errors
/// [`ErroAbrir`].
pub fn abrir_sequencia(pngs: &[Vec<u8>]) -> Result<Sprite, ErroAbrir> {
    let ler = |e: ph2d_imageio::Error| ErroAbrir::Ler(e.to_string());
    let lidos: Vec<_> = pngs
        .iter()
        .map(|b| ph2d_imageio_png::decode_pixel(b).map_err(ler))
        .collect::<Result<_, _>>()?;
    let Some((w, h, modo, _)) = lidos.first().cloned() else {
        return Err(ErroAbrir::Vazio);
    };
    if lidos.iter().any(|l| (l.0, l.1) != (w, h)) {
        return Err(ErroAbrir::TamanhosDiferentes);
    }
    let igual = lidos.iter().all(|l| l.2 == modo);
    let quadros: Vec<_> = lidos.iter().map(|l| (l.3.clone(), DEFAULT_FRAME_MS)).collect();
    match (&modo, igual) {
        (PixelPng::Indexed(p), true) => {
            let t = p.iter().position(|c| c[3] == 0).unwrap_or(0) as u8;
            documento(w, h, ColorMode::Indexed, p.clone(), t, &quadros)
        }
        (PixelPng::GrayAlpha, true) => documento(w, h, ColorMode::Gray, Vec::new(), 0, &quadros),
        (PixelPng::Rgba, true) => documento(w, h, ColorMode::Rgba, Vec::new(), 0, &quadros),
        _ => {
            let conv: Vec<_> = lidos
                .iter()
                .map(|l| (em_rgba(&l.2, &l.3), DEFAULT_FRAME_MS))
                .collect();
            documento(w, h, ColorMode::Rgba, Vec::new(), 0, &conv)
        }
    }
}

fn em_rgba(modo: &PixelPng, d: &[u8]) -> Vec<u8> {
    match modo {
        PixelPng::Rgba => d.to_vec(),
        PixelPng::GrayAlpha => d.as_chunks::<2>().0.iter().flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
        PixelPng::Indexed(p) => d
            .iter()
            .flat_map(|&i| p.get(usize::from(i)).copied().unwrap_or([0, 0, 0, 0]))
            .collect(),
    }
}

/// A grelha do Import Sprite Sheet: origem, tamanho da célula, espaço entre células.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grelha {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub gap_x: u32,
    pub gap_y: u32,
    /// Por colunas em vez de por linhas.
    pub por_colunas: bool,
}

/// Quantas células inteiras cabem: a parcial não entra.
#[must_use]
pub fn celulas(g: &Grelha, iw: u32, ih: u32) -> (u32, u32) {
    let conta = |total: u32, o: u32, c: u32, gap: u32| {
        if c == 0 || o + c > total {
            0
        } else {
            (total - o + gap) / (c + gap)
        }
    };
    (conta(iw, g.x, g.w, g.gap_x), conta(ih, g.y, g.h, g.gap_y))
}

/// **Import Sprite Sheet**: cada célula vira um quadro; as totalmente vazias continuam quadros (a posição
/// faz parte da animação).
///
/// # Errors
/// [`ErroAbrir`].
pub fn importar_folha(rgba: &[u8], iw: u32, ih: u32, g: &Grelha) -> Result<Sprite, ErroAbrir> {
    let (cols, rows) = celulas(g, iw, ih);
    if cols == 0 || rows == 0 {
        return Err(ErroAbrir::Vazio);
    }
    let mut ordem = Vec::with_capacity(cols as usize * rows as usize);
    if g.por_colunas {
        for c in 0..cols {
            ordem.extend((0..rows).map(|r| (c, r)));
        }
    } else {
        for r in 0..rows {
            ordem.extend((0..cols).map(|c| (c, r)));
        }
    }
    let quadros: Vec<_> = ordem
        .iter()
        .map(|&(c, r)| {
            let (x0, y0) = (g.x + c * (g.w + g.gap_x), g.y + r * (g.h + g.gap_y));
            let mut d = Vec::with_capacity(g.w as usize * g.h as usize * 4);
            for y in y0..y0 + g.h {
                let i = (y as usize * iw as usize + x0 as usize) * 4;
                d.extend_from_slice(&rgba[i..i + g.w as usize * 4]);
            }
            (d, DEFAULT_FRAME_MS)
        })
        .collect();
    documento(g.w, g.h, ColorMode::Rgba, Vec::new(), 0, &quadros)
}
