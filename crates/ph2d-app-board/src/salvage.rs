//! W18 — **o que se LÊ de uma imagem estragada** (o «salvador» do desenho,
//! [`ph2d_board_render::install_salvager`]): um PNG ou JPEG cortado a meio mostra as linhas de cima,
//! como o navegador (oráculo `excalidraw_oracle/saidas/w17`: o Chromium desenha 173 das 300 linhas do
//! PNG cortado a 60 % e 142 do JPEG cortado a 50 %, o resto transparente). A imagem continua
//! ESTRAGADA para o aviso: isto só decide o que se desenha.

use ph2d_board_model::mime_of;
use ph2d_board_render::Pixels;

/// As linhas legíveis de um PNG ou JPEG que o descodificador inteiro recusou; o resto transparente.
/// `None` = nada se lê (ou outro formato).
#[must_use]
pub fn salvage(bytes: &[u8]) -> Option<Pixels> {
    match mime_of(bytes)? {
        "image/png" => png_rows(bytes),
        "image/jpeg" => jpeg_rows(bytes),
        _ => None,
    }
}

/// O PNG linha a linha até ao erro (um entrelaçado não tem «linhas de cima»: nada).
fn png_rows(bytes: &[u8]) -> Option<Pixels> {
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::normalize_to_color8());
    let mut r = dec.read_info().ok()?;
    let (w, h) = (r.info().width, r.info().height);
    if r.info().interlaced {
        return None;
    }
    let (color, _) = r.output_color_type();
    let mut rgba = vec![0u8; w as usize * h as usize * 4];
    let mut y = 0usize;
    while let Ok(Some(row)) = r.next_row() {
        let out = &mut rgba[y * w as usize * 4..(y + 1) * w as usize * 4];
        for (o, i) in out.chunks_exact_mut(4).zip(row.data().chunks_exact(color.samples())) {
            o.copy_from_slice(&match *i {
                [g] => [g, g, g, 255],
                [g, a] => [g, g, g, a],
                [r, g, b] => [r, g, b, 255],
                [r, g, b, a] => [r, g, b, a],
                _ => return None,
            });
        }
        y += 1;
        if y == h as usize {
            break;
        }
    }
    (y > 0).then_some(Pixels { size: [w, h], rgba })
}

/// Um JPEG com a marca do fim (`FFD9`) depois do começo dos dados (`FFDA`): o que não a tem foi cortado.
#[must_use]
pub(crate) fn jpeg_ends(bytes: &[u8]) -> bool {
    let sos = bytes.windows(2).position(|w| w == [0xFF, 0xDA]);
    let eoi = bytes.windows(2).rposition(|w| w == [0xFF, 0xD9]);
    matches!((sos, eoi), (Some(s), Some(e)) if e > s)
}

/// Um JPEG sem a marca do fim: inteiro só se o tolerante escreveu TODOS os píxeis (o `image` aceita-o
/// e pinta o que falta — a metade de baixo de um cortado, sem aviso).
#[must_use]
pub(crate) fn jpeg_whole(bytes: &[u8]) -> Option<Pixels> {
    jpeg_rows(bytes).filter(|p| p.rgba.chunks_exact(4).all(|q| q[3] != 0))
}

/// O JPEG pelo `zune-jpeg` tolerante (pára no erro, «matching browsers» — `mcu.rs`). Os blocos que ele
/// não leu saem com os coeficientes a zero — o nível médio, `[128; 4]` com o ALFA também (medido no
/// `w17_jpeg_truncado`: 158 linhas lidas, o resto assim); um JPEG lido tem sempre alfa 255 ⇒ alfa ≠ 255
/// é «não lido», e fica transparente.
fn jpeg_rows(bytes: &[u8]) -> Option<Pixels> {
    use zune_core::bytestream::ZCursor;
    use zune_core::colorspace::ColorSpace;
    use zune_core::options::DecoderOptions;
    let opts = DecoderOptions::default()
        .set_strict_mode(false)
        .jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut dec = zune_jpeg::JpegDecoder::new_with_options(ZCursor::new(bytes), opts);
    dec.decode_headers().ok()?;
    let info = dec.info()?;
    let (w, h) = (u32::from(info.width), u32::from(info.height));
    let mut rgba = vec![0u8; dec.output_buffer_size()?];
    dec.decode_into(&mut rgba).ok()?;
    for p in rgba.chunks_exact_mut(4).filter(|p| p[3] != u8::MAX) {
        p.fill(0);
    }
    rgba.chunks_exact(4)
        .any(|p| p[3] != 0)
        .then_some(Pixels { size: [w, h], rgba })
}

#[cfg(test)]
#[path = "salvage_tests.rs"]
mod tests;
