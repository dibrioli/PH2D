//! O PNG do Pixel Art: sem converter, paleta e índices como o artista os deixou.

use ph2d_imageio::Error;

/// O modo de um PNG do Pixel Art.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PixelPng {
    /// 4 bytes por célula.
    Rgba,
    /// 2 bytes por célula: cinza, alfa.
    GrayAlpha,
    /// 1 byte por célula, com `PLTE` e `tRNS` (a tabela inteira, como o LibreSprite grava).
    Indexed(Vec<[u8; 4]>),
}

/// Grava o PNG no modo do documento.
///
/// # Errors
/// [`Error`] do codificador.
pub fn encode_pixel(w: u32, h: u32, modo: &PixelPng, data: &[u8]) -> Result<Vec<u8>, Error> {
    let err = |e: png::EncodingError| Error::Decode(e.to_string());
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_depth(png::BitDepth::Eight);
        match modo {
            PixelPng::Rgba => enc.set_color(png::ColorType::Rgba),
            PixelPng::GrayAlpha => enc.set_color(png::ColorType::GrayscaleAlpha),
            PixelPng::Indexed(pal) => {
                enc.set_color(png::ColorType::Indexed);
                let rgb: Vec<u8> = pal.iter().flat_map(|c| [c[0], c[1], c[2]]).collect();
                let trns: Vec<u8> = pal.iter().map(|c| c[3]).collect();
                enc.set_palette(rgb);
                enc.set_trns(trns);
            }
        }
        let mut wr = enc.write_header().map_err(err)?;
        wr.write_image_data(data).map_err(err)?;
        wr.finish().map_err(err)?;
    }
    Ok(out)
}

/// Lê o PNG SEM converter: a paleta e os índices ficam como o artista os deixou.
/// `(w, h, modo, dados)`; cinza opaco ganha alfa, RGB ganha alfa, 16 bits descem a 8.
///
/// # Errors
/// [`Error`] do leitor.
pub fn decode_pixel(src: &[u8]) -> Result<(u32, u32, PixelPng, Vec<u8>), Error> {
    let err = |e: png::DecodingError| Error::Decode(e.to_string());
    let mut dec = png::Decoder::new(std::io::Cursor::new(src));
    dec.set_transformations(png::Transformations::STRIP_16);
    let mut rd = dec.read_info().map_err(err)?;
    let mut buf = vec![0; rd.output_buffer_size().ok_or(Error::OutOfMemory)?];
    let fr = rd.next_frame(&mut buf).map_err(err)?;
    buf.truncate(fr.buffer_size());
    let (w, h) = (fr.width, fr.height);
    // Menos de 8 bits por célula (paletas pequenas): uma célula por byte.
    let bits = fr.bit_depth as usize;
    if bits < 8 {
        let linha = fr.line_size;
        let cinza = fr.color_type == png::ColorType::Grayscale;
        let topo = (1_usize << bits) - 1;
        let mut d = Vec::with_capacity(w as usize * h as usize);
        for l in buf.chunks(linha) {
            for x in 0..w as usize {
                let b = x * bits;
                let v = (usize::from(l[b / 8]) >> (8 - bits - b % 8)) & topo;
                d.push(if cinza {
                    (v * 255 / topo) as u8
                } else {
                    v as u8
                });
            }
        }
        buf = d;
    }
    let info = rd.info();
    Ok(match fr.color_type {
        png::ColorType::Indexed => {
            let plte = info.palette.as_deref().unwrap_or(&[]);
            let trns = info.trns.as_deref().unwrap_or(&[]);
            let pal = plte
                .as_chunks::<3>()
                .0
                .iter()
                .enumerate()
                .map(|(i, c)| [c[0], c[1], c[2], trns.get(i).copied().unwrap_or(255)])
                .collect();
            (w, h, PixelPng::Indexed(pal), buf)
        }
        png::ColorType::Rgba => (w, h, PixelPng::Rgba, buf),
        png::ColorType::GrayscaleAlpha => (w, h, PixelPng::GrayAlpha, buf),
        png::ColorType::Grayscale => {
            let d = buf.iter().flat_map(|&v| [v, 255]).collect();
            (w, h, PixelPng::GrayAlpha, d)
        }
        png::ColorType::Rgb => {
            let d = buf
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|c| [c[0], c[1], c[2], 255])
                .collect();
            (w, h, PixelPng::Rgba, d)
        }
    })
}
