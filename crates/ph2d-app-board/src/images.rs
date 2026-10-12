//! W14 — **as imagens a ENTRAR no quadro** (soltar, colar, importar, um `.excalidraw`): a porta de
//! entrada ([`ingest`], o `Ingest` do modelo) e o descodificador do desenho ([`decode`]) — os dois
//! pela MESMA descodificação que a cena usa ao largar uma imagem (`ph2d_asset::decode_image`, com os
//! tectos e as recusas do registo `imageio`): o que a cena aceita, o quadro aceita.
//!
//! W15 — e um SVG: entra como IMAGEM (os bytes de texto, um `.svgz` já descomprimido), com o tamanho
//! do `<svg>`, e desenha-se pelo `ph2d_imageio_svg::raster` ao tamanho de cada nível. Os `<text>` não
//! se desenham (sem fontes de SVG): quem o larga é avisado ([`Placed::texts`]).

use std::path::PathBuf;
use std::sync::Arc;

use ph2d_board_model::{ImageFile, mime_of};
use ph2d_imageio_svg::raster;
use ph2d_board_render::Pixels;
use ph2d_editor_core::Toast;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::board_images::{insert_images, takes_image_paste};
use ph2d_i18n::tr_with;

/// O MIME de um ficheiro vectorial.
const SVG: &str = "image/svg+xml";

/// Os píxeis RGBA de 8 bits de uns bytes de imagem (os de 16 bits descem a 8); um SVG a `at` (o
/// natural, sem ele). `None` = não é uma imagem que o app lê.
#[must_use]
pub fn decode(bytes: &[u8], at: Option<[u32; 2]>) -> Option<Pixels> {
    if mime_of(bytes) == Some(SVG) {
        let tree = raster::parse(bytes).ok()?;
        let size = at.unwrap_or_else(|| raster::natural_size(&tree));
        return Some(Pixels {
            size,
            rgba: raster::rasterize(&tree, size)?,
        });
    }
    let asset = ph2d_asset::decode_image(bytes).ok()?;
    let (w, h, px) = asset.image_rgba8()?;
    Some(Pixels {
        size: [w, h],
        rgba: px.into_owned(),
    })
}

/// ⭐ A porta de entrada: os bytes viram um ficheiro do quadro com o tamanho natural; os de um formato
/// que um navegador não mostra (TIFF, PSD, ORA…) entram já em PNG — o SVG, o `.excalidraw` e o par
/// ao vivo mostram-nos.
/// O que o navegador mostra entra pelo tamanho do CABEÇALHO (o fio da janela não descodifica os
/// píxeis — o desenho descodifica-os fora dele); sem cabeçalho legível, descodifica.
#[must_use]
pub fn ingest(bytes: &[u8]) -> Option<ImageFile> {
    if let Some(svg) = raster::plain(bytes).filter(|b| mime_of(b) == Some(SVG)) {
        let tree = raster::parse(&svg).ok()?;
        return Some(ImageFile::new(svg.into_owned(), raster::natural_size(&tree)));
    }
    if mime_of(bytes).is_some() {
        let size = header_size(bytes).or_else(|| decode(bytes, None).map(|p| p.size))?;
        return Some(ImageFile::new(bytes.to_vec(), size));
    }
    let px = decode(bytes, None)?;
    let png = encode_png(&px)?;
    Some(ImageFile::new(png, px.size))
}

/// O tamanho que o cabeçalho declara.
fn header_size(bytes: &[u8]) -> Option<[u32; 2]> {
    let r = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let (w, h) = r.into_dimensions().ok()?;
    (w > 0 && h > 0).then_some([w, h])
}

pub(crate) fn encode_png(px: &Pixels) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut e = png::Encoder::new(&mut out, px.size[0], px.size[1]);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    // No fio da janela (colar uma captura): a compressão rápida.
    e.set_compression(png::Compression::Fast);
    e.write_header().ok()?.write_image_data(&px.rgba).ok()?;
    Some(out)
}

/// O que entrou de uma leva de ficheiros no quadro.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Placed {
    pub placed: usize,
    /// Os que não são uma imagem que o app lê (ou que o disco não deu).
    pub refused: usize,
    /// W15 — os textos dos SVG que entraram e que o desenho não mostra.
    pub texts: usize,
}

/// Os ficheiros `paths` no quadro activo, à volta do píxel de ecrã `at` (ou do meio da vista).
pub fn insert_paths(hero: &mut HeroScreen, paths: &[PathBuf], at: Option<[f64; 2]>) -> Placed {
    let files: Vec<ImageFile> = paths
        .iter()
        .filter_map(|p| ingest(&std::fs::read(p).ok()?))
        .collect();
    let refused = paths.len() - files.len();
    let texts = files
        .iter()
        .filter(|f| f.mime() == Some(SVG))
        .map(|f| raster::dropped_texts(&f.bytes.0))
        .sum();
    let placed = if files.is_empty() {
        0
    } else {
        insert_images(hero, files, at)
    };
    Placed {
        placed,
        refused,
        texts,
    }
}

/// ⭐ A porta da shell para ficheiros LARGADOS num quadro: as imagens entram onde se largou; o aviso
/// (para a shell mostrar) conta o que não é imagem e os textos de SVG que não se desenham.
pub fn drop_files(hero: &mut HeroScreen, paths: &[PathBuf], at: [f64; 2]) -> Option<Toast> {
    let p = insert_paths(hero, paths, Some(at));
    p.warning().map(Toast::warning)
}

impl Placed {
    /// O aviso do que entrou a menos: os ficheiros recusados e os textos de SVG sem desenho.
    #[must_use]
    pub fn warning(&self) -> Option<String> {
        let mut parts = Vec::new();
        if self.refused > 0 {
            let n = self.refused.to_string();
            parts.push(tr_with("board.images.skipped", &[("count", &n)]));
        }
        if self.texts > 0 {
            let n = self.texts.to_string();
            parts.push(tr_with("board.images.svg_texts", &[("count", &n)]));
        }
        (!parts.is_empty()).then(|| parts.join(". "))
    }
}

/// ⭐ A porta da shell para o `Ctrl+V` num quadro com uma IMAGEM na área de transferência (`rgba`
/// reto, `size` em píxeis): entra no meio da vista. `false` = não é para aqui — a escrever num texto,
/// ou o texto da área de transferência é a última cópia de elementos do quadro (essa cola-se a si).
pub fn paste(hero: &mut HeroScreen, text: Option<&str>, size: [u32; 2], rgba: Vec<u8>) -> bool {
    if !takes_image_paste(hero, text) {
        return false;
    }
    let Some(png) = encode_png(&Pixels { size, rgba }) else {
        return false;
    };
    insert_images(hero, vec![ImageFile::new(png, size)], None) > 0
}

/// Instala o descodificador no desenho (uma vez; as seguintes não fazem nada).
pub fn install() {
    ph2d_board_render::install_decoder(Arc::new(decode));
}

#[cfg(test)]
#[path = "images_tests.rs"]
mod tests;
