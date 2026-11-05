//! **Export As**: do pedido aos bytes de cada formato. Lei do que sai em cada quadro: [`crate::render`].

use ph2d_imageio_png::PixelPng;
use ph2d_pixel_core::{ColorMode, Sprite};

use crate::ase;
use crate::gif_cores;
use crate::render::{Pedido, QuadroSaida, quadros_com};

/// Os formatos do Export As.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formato {
    /// Um quadro sai `nome.png`; vários, uma sequência `nome1.png, nome2.png…` (medido).
    Png,
    Gif,
    Apng,
    Webp,
    Ase,
}

impl Formato {
    /// A extensão do nome do ficheiro; `None` se não for nossa.
    #[must_use]
    pub fn de_extensao(e: &str) -> Option<Self> {
        Some(match e.to_ascii_lowercase().as_str() {
            "png" => Self::Png,
            "gif" => Self::Gif,
            "apng" => Self::Apng,
            "webp" => Self::Webp,
            "ase" | "aseprite" => Self::Ase,
            _ => return None,
        })
    }
}

/// Algo que o artista precisa de saber sobre o ficheiro que saiu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvisoSaida {
    /// O GIF tem 256 cores e o desenho tinha estas.
    GifReduziu(usize),
    /// O GIF não tem meia transparência: as células ficaram cheias ou vazias.
    GifAlfa,
    Ase(ase::Aviso),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroSaida {
    Formato(String),
    /// Limite do GIF: lados em 16 bits.
    GifGrandeDemais {
        w: u32,
        h: u32,
    },
    Ase(ase::AseErro),
    /// A escala passa do lado máximo de um documento.
    GrandeDemais {
        w: u64,
        h: u64,
    },
}

impl std::fmt::Display for ErroSaida {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formato(e) => write!(f, "{e}"),
            Self::GifGrandeDemais { w, h } => write!(f, "{w} x {h} does not fit a GIF (65535)"),
            Self::Ase(e) => e.fmt(f),
            Self::GrandeDemais { w, h } => write!(f, "{w} x {h} is too large"),
        }
    }
}

/// O que o Export escreve: cada ficheiro é `(sufixo do nome, bytes)`; o sufixo só existe na sequência.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Saida {
    pub ficheiros: Vec<(String, Vec<u8>)>,
    pub avisos: Vec<AvisoSaida>,
}

/// O modo em que uma imagem sai em PNG, com os dados nele: o do documento quando as células cabem
/// exatas, senão RGBA (uma camada a 50 % dá cores fora da paleta).
#[must_use]
pub fn png_do_documento(s: &Sprite, rgba: &[u8]) -> (PixelPng, Vec<u8>) {
    match s.mode() {
        ColorMode::Indexed => {
            let pal = s.palette();
            let idx: Option<Vec<u8>> = rgba
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| {
                    if c[3] == 0 {
                        return Some(s.transparent_index());
                    }
                    pal.iter().position(|p| p == c).map(|i| i as u8)
                })
                .collect();
            match idx {
                Some(i) => (PixelPng::Indexed(pal.to_vec()), i),
                None => (PixelPng::Rgba, rgba.to_vec()),
            }
        }
        ColorMode::Gray => {
            let cinza = rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|c| c[0] == c[1] && c[1] == c[2]);
            if cinza {
                let d = rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|c| [c[0], c[3]])
                    .collect();
                (PixelPng::GrayAlpha, d)
            } else {
                (PixelPng::Rgba, rgba.to_vec())
            }
        }
        ColorMode::Rgba => (PixelPng::Rgba, rgba.to_vec()),
    }
}

/// O rótulo `meta.format` do JSON para o PNG que saiu (medido: `I8` num indexado).
#[must_use]
pub fn formato_json(modo: &PixelPng) -> &'static str {
    match modo {
        PixelPng::Indexed(_) => "I8",
        _ => "RGBA8888",
    }
}

fn png_de(s: &Sprite, q: &QuadroSaida) -> Result<Vec<u8>, ErroSaida> {
    let (modo, dados) = png_do_documento(s, &q.rgba);
    ph2d_imageio_png::encode_pixel(q.w, q.h, &modo, &dados)
        .map_err(|e| ErroSaida::Formato(e.to_string()))
}

fn pares(qs: &[QuadroSaida]) -> Vec<(Vec<u8>, u32)> {
    qs.iter().map(|q| (q.rgba.clone(), q.duration_ms)).collect()
}

/// **Exporta.** `repeat`: 0 = para sempre (formatos animados).
///
/// # Errors
/// [`ErroSaida`]: limite do formato ou do codificador.
pub fn exportar(s: &Sprite, p: &Pedido, f: Formato, repeat: u16) -> Result<Saida, ErroSaida> {
    exportar_com(s, p, f, repeat, &|_| {})
}

/// [`exportar`], dizendo a fração dos quadros já compostos.
///
/// # Errors
/// [`ErroSaida`]: limite do formato ou do codificador.
pub fn exportar_com(
    s: &Sprite,
    p: &Pedido,
    f: Formato,
    repeat: u16,
    feito: &dyn Fn(f32),
) -> Result<Saida, ErroSaida> {
    let mut avisos = Vec::new();
    if f == Formato::Ase {
        let (b, av) = ase::escrever(s).map_err(ErroSaida::Ase)?;
        avisos.extend(av.into_iter().map(AvisoSaida::Ase));
        return Ok(Saida {
            ficheiros: vec![(String::new(), b)],
            avisos,
        });
    }
    let f_escala = if p.escala > 0.0 { p.escala } else { 1.0 };
    let lado = |l: u32| (f64::from(l) * f_escala).floor() as u64;
    let (ew, eh) = (lado(s.width()), lado(s.height()));
    if ew > u64::from(ph2d_pixel_core::MAX_SIDE) || eh > u64::from(ph2d_pixel_core::MAX_SIDE) {
        return Err(ErroSaida::GrandeDemais { w: ew, h: eh });
    }
    let qs = quadros_com(s, p, feito);
    let (w, h) = qs.first().map_or((s.width(), s.height()), |q| (q.w, q.h));
    let err = |e: ph2d_imageio::Error| ErroSaida::Formato(e.to_string());
    let ficheiros = match f {
        Formato::Png if qs.len() == 1 => vec![(String::new(), png_de(s, &qs[0])?)],
        Formato::Png => qs
            .iter()
            .enumerate()
            .map(|(i, q)| Ok(((i + 1).to_string(), png_de(s, q)?)))
            .collect::<Result<_, ErroSaida>>()?,
        Formato::Gif => {
            let (gw, gh) = match (u16::try_from(w), u16::try_from(h)) {
                (Ok(a), Ok(b)) => (a, b),
                _ => return Err(ErroSaida::GifGrandeDemais { w, h }),
            };
            let refs: Vec<&[u8]> = qs.iter().map(|q| q.rgba.as_slice()).collect();
            let r = gif_cores::reduzir(&refs);
            if let Some(n) = r.reduziu_de {
                avisos.push(AvisoSaida::GifReduziu(n));
            }
            let meio = |c: &[u8; 4]| c[3] != 0 && c[3] != 255;
            if refs.iter().any(|q| q.as_chunks::<4>().0.iter().any(meio)) {
                avisos.push(AvisoSaida::GifAlfa);
            }
            let fr: Vec<_> = r
                .quadros
                .into_iter()
                .zip(qs.iter().map(|q| q.duration_ms))
                .collect();
            let b =
                ph2d_imageio_gif::encode_indexed(gw, gh, &r.paleta, r.transparente, &fr, repeat)
                    .map_err(err)?;
            vec![(String::new(), b)]
        }
        Formato::Apng => {
            let b = ph2d_imageio_apng::encode_animated(w, h, &pares(&qs), u32::from(repeat))
                .map_err(err)?;
            vec![(String::new(), b)]
        }
        Formato::Webp => {
            let b = ph2d_imageio_webp::encode_animated(w, h, &pares(&qs), repeat).map_err(err)?;
            vec![(String::new(), b)]
        }
        Formato::Ase => unreachable!("tratado acima"),
    };
    Ok(Saida { ficheiros, avisos })
}
