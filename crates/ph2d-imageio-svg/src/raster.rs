//! W15 do Quadro — **um SVG como IMAGEM**: a árvore pelas mesmas defesas da porta de importação (o
//! tecto de bytes, nenhum `href` que leia o disco) e os píxeis a um tamanho pedido pelo `resvg` (a
//! série do `usvg` desta casa). Um SVG é vector: quem o mostra pede-o ao tamanho do ECRÃ, e ele fica
//! nítido a qualquer zoom.
//!
//! ⛔ Sem a feature `text` do `usvg` (recusa medida, `ph2d-vec-svg/Cargo.toml`): os `<text>` não se
//! desenham — [`dropped_texts`] conta-os para quem mostra o aviso.

use ph2d_imageio::{Error, MAX_ARCHIVE_TEXT_BYTES};

/// A árvore de um SVG (`.svgz` incluído), com o tecto de bytes e sem ler o disco por um `href`. Todo
/// gzip — o do ficheiro e o de um SVG ANINHADO num `data:` (o usvg descomprime-o sem tecto, e a
/// recursão passa pelo mesmo resolvedor) — passa pelo [`plain`] com tecto.
pub fn parse(src: &[u8]) -> Result<usvg::Tree, Error> {
    if src.is_empty() {
        return Err(Error::Truncated);
    }
    let src = plain(src).ok_or_else(|| {
        Error::Decode(format!(
            "SVGZ over MAX_ARCHIVE_TEXT_BYTES={MAX_ARCHIVE_TEXT_BYTES} or broken (DoS defence)"
        ))
    })?;
    let src = &*src;
    if (src.len() as u64) > MAX_ARCHIVE_TEXT_BYTES {
        return Err(Error::Decode(format!(
            "SVG source {} bytes > MAX_ARCHIVE_TEXT_BYTES={MAX_ARCHIVE_TEXT_BYTES} (DoS defence)",
            src.len()
        )));
    }
    let opts = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: capped_data_resolver(),
            resolve_string: Box::new(|_href, _opts| None),
        },
        ..Default::default()
    };
    usvg::Tree::from_data(src, &opts)
        .map_err(|e| Error::from_decoder_message(format!("SVG parse: {e}")))
}

/// O resolvedor de `data:` do usvg, com o gzip (um SVG aninhado comprimido) pelo [`plain`].
fn capped_data_resolver() -> usvg::ImageHrefDataResolverFn<'static> {
    let default = usvg::ImageHrefResolver::default_data_resolver();
    Box::new(move |mime, data, opts| {
        if data.starts_with(&[0x1f, 0x8b]) {
            let p = plain(&data)?.into_owned();
            return default(mime, std::sync::Arc::new(p), opts);
        }
        default(mime, data, opts)
    })
}

/// O tamanho natural em píxeis: o do `<svg>` (`width`/`height`, senão o `viewBox`), para cima.
#[must_use]
pub fn natural_size(tree: &usvg::Tree) -> [u32; 2] {
    let s = tree.size();
    [s.width(), s.height()].map(|v| (v.ceil() as u32).max(1))
}

/// Os píxeis RGBA de 8 bits, alfa RETO, sRGB, da árvore esticada a `size` (a caixa do `<svg>` inteira
/// em `size`, sem manter a proporção — quem pede mantém-na). `None` = tamanho nulo.
#[must_use]
pub fn rasterize(tree: &usvg::Tree, size: [u32; 2]) -> Option<Vec<u8>> {
    let mut pm = resvg::tiny_skia::Pixmap::new(size[0], size[1])?;
    let s = tree.size();
    let t = resvg::tiny_skia::Transform::from_scale(
        size[0] as f32 / s.width(),
        size[1] as f32 / s.height(),
    );
    resvg::render(tree, t, &mut pm.as_mut());
    let mut px = pm.take();
    // O tiny-skia guarda pré-multiplicado; a porta das imagens fala alfa reto.
    for p in px.as_chunks_mut::<4>().0 {
        let a = u32::from(p[3]);
        if a != 0 && a != 255 {
            for c in &mut p[..3] {
                *c = ((u32::from(*c) * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    Some(px)
}

/// O texto de um SVG: os próprios bytes, ou os de um `.svgz` (gzip) descomprimidos ATÉ ao tecto de
/// bytes (uma bomba de compressão não passa dele). `None` = gzip estragado ou acima do tecto.
#[must_use]
pub fn plain(src: &[u8]) -> Option<std::borrow::Cow<'_, [u8]>> {
    use std::io::Read as _;
    if !src.starts_with(&[0x1f, 0x8b]) {
        return Some(std::borrow::Cow::Borrowed(src));
    }
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(src)
        .take(MAX_ARCHIVE_TEXT_BYTES + 1)
        .read_to_end(&mut out)
        .ok()?;
    ((out.len() as u64) <= MAX_ARCHIVE_TEXT_BYTES).then_some(std::borrow::Cow::Owned(out))
}

/// Quantos `<text>` um SVG tem (o desenho não os mostra). Um `.svgz` (não é texto) conta zero.
#[must_use]
pub fn dropped_texts(src: &[u8]) -> usize {
    let Ok(txt) = std::str::from_utf8(src) else {
        return 0;
    };
    usvg::roxmltree::Document::parse(txt).map_or(0, |doc| {
        doc.descendants()
            .filter(|d| d.is_element() && d.tag_name().name() == "text")
            .count()
    })
}

#[cfg(test)]
#[path = "raster_tests.rs"]
mod tests;
