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

/// `(largura, altura, [(RGBA, ms)])`.
type QuadrosRgba = (u32, u32, Vec<(Vec<u8>, u32)>);

/// A caixa mínima de uma imagem no modo do documento, e os bytes nela; `None` = só transparente.
fn apertada(
    px: &[u8],
    w: u32,
    h: u32,
    bpp: usize,
    vazio: &dyn Fn(&[u8]) -> bool,
) -> Option<(Rect, Vec<u8>)> {
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
        palette: if pal.is_empty() {
            palette::DEFAULT.to_vec()
        } else {
            pal
        },
        transparent,
        layers: vec![Layer::image(camada, "Layer 1")],
        frames,
        cels,
        tilesets: Vec::new(),
        tags: Vec::new(),
    })
    .map_err(ErroAbrir::Documento)
}

/// Compõe as sub-imagens de um APNG/GIF no quadro inteiro, com o `dispose` de cada uma.
fn quadros_inteiros(fs: &[ph2d_imageio::AnimFrame]) -> (u32, u32, Vec<(Vec<u8>, u32)>) {
    let w = fs
        .iter()
        .map(|f| f.offset_xy.0.max(0) as u32 + f.image.width)
        .max()
        .unwrap_or(1);
    let h = fs
        .iter()
        .map(|f| f.offset_xy.1.max(0) as u32 + f.image.height)
        .max()
        .unwrap_or(1);
    let mut tela = vec![0_u8; w as usize * h as usize * 4];
    let mut out = Vec::with_capacity(fs.len());
    for f in fs {
        let antes = tela.clone();
        let (ox, oy) = (f.offset_xy.0.max(0) as usize, f.offset_xy.1.max(0) as usize);
        for y in 0..f.image.height as usize {
            for x in 0..f.image.width as usize {
                let c = f.image.pixels[y * f.image.width as usize + x].0;
                let i = ((oy + y) * w as usize + ox + x) * 4;
                if c[3] == 255
                    || tela[i + 3] == 0
                    || matches!(f.blend_op, ph2d_imageio::AnimBlendOp::Source)
                {
                    tela[i..i + 4].copy_from_slice(&c);
                } else if c[3] != 0 {
                    // Over de alfa reto, em inteiros.
                    let (sa, da) = (u32::from(c[3]), u32::from(tela[i + 3]));
                    let a = sa + da * (255 - sa) / 255;
                    for e in 0..3 {
                        let v = (u32::from(c[e]) * sa
                            + u32::from(tela[i + e]) * da * (255 - sa) / 255)
                            / a.max(1);
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

fn de_decodificado(d: DecodedImage) -> Result<QuadrosRgba, ErroAbrir> {
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
    let rgba = |(w, h, q): (u32, u32, Vec<(Vec<u8>, u32)>)| {
        documento(w, h, ColorMode::Rgba, Vec::new(), 0, &q)
    };
    let doc = match extensao.to_ascii_lowercase().as_str() {
        "ase" | "aseprite" => {
            return ase::ler(bytes).map_err(|e| ErroAbrir::Ler(e.to_string()));
        }
        "png" | "apng" if tem_actl(bytes) => rgba(de_decodificado(
            ph2d_imageio_apng::ApngImporter
                .import(bytes, &opts)
                .map_err(ler)?,
        )?)?,
        "png" | "apng" => png_doc(bytes)?,
        "gif" => rgba(de_decodificado(
            ph2d_imageio_gif::GifImporter
                .import(bytes, &opts)
                .map_err(ler)?,
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
    let quadros: Vec<_> = lidos
        .iter()
        .map(|l| (l.3.clone(), DEFAULT_FRAME_MS))
        .collect();
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
        PixelPng::GrayAlpha => d
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|c| [c[0], c[0], c[0], c[1]])
            .collect(),
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
    importar_folha_com(rgba, iw, ih, g, None)
}

/// [`importar_folha`] com o `.json` da folha: cada célula leva a duração do quadro que cai nela e, se
/// a grelha tem tantas células quantos quadros o JSON, as tags dele. Sem JSON (ou sem o campo), os
/// quadros ficam com a duração de fábrica: a imagem sozinha não guarda tempo.
///
/// # Errors
/// [`ErroAbrir`].
pub fn importar_folha_com(
    rgba: &[u8],
    iw: u32,
    ih: u32,
    g: &Grelha,
    json: Option<&str>,
) -> Result<Sprite, ErroAbrir> {
    let v: Option<serde_json::Value> = json.and_then(|t| serde_json::from_str(t).ok());
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
            let ms = v
                .as_ref()
                .and_then(|j| duracao_da_celula(j, (x0, y0, g.w, g.h)));
            (d, ms.unwrap_or(DEFAULT_FRAME_MS))
        })
        .collect();
    let mut s = documento(g.w, g.h, ColorMode::Rgba, Vec::new(), 0, &quadros)?;
    if let Some(j) = &v {
        poe_tags(&mut s, j, quadros.len());
    }
    Ok(s)
}

/// Os quadros do JSON, em `array` ou em `hash`.
fn quadros_do_json(j: &serde_json::Value) -> Vec<&serde_json::Value> {
    match &j["frames"] {
        serde_json::Value::Array(a) => a.iter().collect(),
        serde_json::Value::Object(o) => o.values().collect(),
        _ => Vec::new(),
    }
}

/// A duração do quadro cujo retângulo tem o centro nesta célula.
fn duracao_da_celula(j: &serde_json::Value, (x, y, w, h): (u32, u32, u32, u32)) -> Option<u32> {
    let num = |q: &serde_json::Value, k: &str| q["frame"][k].as_u64();
    quadros_do_json(j).into_iter().find_map(|q| {
        let cx = num(q, "x")? + num(q, "w")? / 2;
        let cy = num(q, "y")? + num(q, "h")? / 2;
        let dentro = (u64::from(x)..u64::from(x + w)).contains(&cx)
            && (u64::from(y)..u64::from(y + h)).contains(&cy);
        if !dentro {
            return None;
        }
        u32::try_from(q["duration"].as_u64()?).ok()
    })
}

/// As tags do JSON; só se a grelha tem um quadro para cada um dos dele, senão apontariam para outros.
fn poe_tags(s: &mut Sprite, j: &serde_json::Value, celulas: usize) {
    use ph2d_pixel_core::{Direction, apply, frames};
    if quadros_do_json(j).len() != celulas {
        return;
    }
    let Some(tags) = j["meta"]["frameTags"].as_array() else {
        return;
    };
    for t in tags {
        let (Some(nome), Some(de), Some(ate)) =
            (t["name"].as_str(), t["from"].as_u64(), t["to"].as_u64())
        else {
            continue;
        };
        let (Ok(de), Ok(ate)) = (u32::try_from(de), u32::try_from(ate)) else {
            continue;
        };
        let Some(op) = frames::new_tag(s, nome, de, ate) else {
            continue;
        };
        if apply(s, op).is_err() {
            continue;
        }
        let i = s.tags().len() - 1;
        let direcao = match t["direction"].as_str() {
            Some("reverse") => Direction::Reverse,
            Some("pingpong") => Direction::PingPong,
            Some("pingpong_reverse") => Direction::PingPongReverse,
            _ => Direction::Forward,
        };
        // O formato escreve `repeat` em string; ausente = para sempre.
        let repeat = t["repeat"]
            .as_str()
            .and_then(|r| r.parse().ok())
            .unwrap_or(0);
        let cor = t["color"].as_str().and_then(cor_do_json);
        let op = frames::edit_tag(s, i, |x| {
            x.direction = direcao;
            x.repeat = repeat;
            if let Some(c) = cor {
                x.color = c;
            }
        });
        if let Some(op) = op {
            let _ = apply(s, op);
        }
    }
}

/// `#rrggbbaa`.
fn cor_do_json(t: &str) -> Option<[u8; 4]> {
    let h = t.strip_prefix('#')?;
    if h.len() != 8 {
        return None;
    }
    let b = |i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok();
    Some([b(0)?, b(2)?, b(4)?, b(6)?])
}

/// O `.json` com o mesmo nome, ao lado da imagem, se é desta imagem: `meta.size` tem de ser o dela,
/// senão os tempos e as tags de outra folha entrariam sem aviso.
#[must_use]
pub fn dados_ao_lado(imagem: &std::path::Path, (w, h): (u32, u32)) -> Option<String> {
    let t = std::fs::read_to_string(imagem.with_extension("json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&t).ok()?;
    let lado = |k: &str| v["meta"]["size"][k].as_u64();
    (lado("w") == Some(u64::from(w)) && lado("h") == Some(u64::from(h))).then_some(t)
}

/// A grelha de partida do Import Sprite Sheet: a divisão que não parte figuras; `None` = a imagem não
/// a mostra.
#[must_use]
pub fn sugerir_grelha(rgba: &[u8], w: u32, h: u32) -> Option<Grelha> {
    let alfa: Vec<u8> = rgba.as_chunks::<4>().0.iter().map(|c| c[3]).collect();
    let g = ph2d_sprite_sheet::guess_grid(&alfa, w, h)?;
    Some(Grelha {
        x: 0,
        y: 0,
        w: w / g.cols.max(1),
        h: h / g.rows.max(1),
        gap_x: 0,
        gap_y: 0,
        por_colunas: false,
    })
}

/// Um ficheiro aberto: o documento, os avisos e o nome para o objecto.
pub type Aberto = (Sprite, Vec<ase::Aviso>, String);

/// Abre os ficheiros do File ▸ Open: um é o que ele for; vários PNG são os quadros de uma sequência,
/// pela ordem natural dos nomes.
///
/// # Errors
/// [`ErroAbrir`].
pub fn abrir_caminhos(caminhos: &[std::path::PathBuf]) -> Result<Aberto, ErroAbrir> {
    let ler = |c: &std::path::Path| std::fs::read(c).map_err(|e| ErroAbrir::Ler(e.to_string()));
    let nome_de = |c: &std::path::Path| {
        c.file_stem()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
    };
    match caminhos {
        [] => Err(ErroAbrir::Vazio),
        [um] => {
            let e = um
                .extension()
                .map_or_else(String::new, |x| x.to_string_lossy().into_owned());
            let (s, av) = abrir(&ler(um)?, &e)?;
            Ok((s, av, nome_de(um)))
        }
        varios => {
            let mut ordem: Vec<_> = varios.iter().collect();
            ordem.sort_by_key(|c| chave_natural(&nome_de(c)));
            let bytes = ordem
                .iter()
                .map(|c| ler(c))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((abrir_sequencia(&bytes)?, Vec::new(), nome_de(ordem[0])))
        }
    }
}

/// `walk2` antes de `walk10`: o número no fim do nome conta como número.
fn chave_natural(n: &str) -> (String, u64) {
    let raiz = n.trim_end_matches(|c: char| c.is_ascii_digit());
    (raiz.to_owned(), n[raiz.len()..].parse().unwrap_or(0))
}

/// As extensões que viram documento.
pub const LIDOS: [&str; 6] = ["ase", "aseprite", "png", "apng", "gif", "webp"];

fn extensao_de(c: &std::path::Path) -> String {
    c.extension()
        .map_or_else(String::new, |x| x.to_string_lossy().to_ascii_lowercase())
}

/// Todos estes ficheiros podem virar Pixel Sprite?
#[must_use]
pub fn le_todos(caminhos: &[std::path::PathBuf]) -> bool {
    !caminhos.is_empty()
        && caminhos
            .iter()
            .all(|c| LIDOS.contains(&extensao_de(c).as_str()))
}

/// Uma leva: vários PNG são uma sequência; o resto, um documento por ficheiro.
#[must_use]
pub fn abrir_leva(caminhos: &[std::path::PathBuf]) -> Vec<Result<Aberto, ErroAbrir>> {
    if caminhos.iter().all(|c| extensao_de(c) == "png") {
        return vec![abrir_caminhos(caminhos)];
    }
    caminhos
        .iter()
        .map(|c| abrir_caminhos(std::slice::from_ref(c)))
        .collect()
}
