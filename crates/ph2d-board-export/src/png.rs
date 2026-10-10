//! ⭐ **O PNG** (E3): a página pela PLACA — o mesmo `VelloPass` do ecrã, ao píxel — em LADRILHOS do
//! tecto de textura do DISPOSITIVO (§0.0: o tecto é o do hardware; o Large do Miro, 49 MP, tem um lado
//! de 13 359 px no exemplo do frame comprido). Cada ladrilho redesenha só o seu pedaço do mundo (o
//! recorte do `paint` poupa o resto); as fronteiras caem em píxeis inteiros, e a mesma afinidade
//! mundo → px atravessa os ladrilhos sem costura.

#[cfg(feature = "gpu")]
use ph2d_board_model::Board;
#[cfg(feature = "gpu")]
use ph2d_gpu::GpuContext;
#[cfg(feature = "gpu")]
use ph2d_render::VelloPass;
#[cfg(feature = "gpu")]
use ph2d_text::TextSystem;
#[cfg(feature = "gpu")]
use ph2d_vector::Color;

#[cfg(feature = "gpu")]
use crate::{Page, paint_page};

/// Uma imagem RGBA8 (alfa directo, sRGB), linha a linha.
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// O lado de um ladrilho: o tecto de textura do dispositivo.
#[cfg(feature = "gpu")]
#[must_use]
pub fn tile_side(gpu: &GpuContext) -> u32 {
    gpu.device.limits().max_texture_dimension_2d.max(1)
}

/// ⭐ Desenha `page` a `scale` px por unidade, pela placa, em ladrilhos de `tile` px.
///
/// # Errors
/// O que a placa recusar (a mensagem do `VelloPass`).
#[cfg(feature = "gpu")]
#[allow(clippy::too_many_arguments)]
pub fn render(
    gpu: &GpuContext,
    pass: &mut VelloPass,
    board: &Board,
    page: &Page,
    scale: f64,
    paper: Option<Color>,
    tile: u32,
    ts: &mut TextSystem,
) -> Result<Image, String> {
    let [w, h] = page.size();
    let (width, height) = (
        ((w * scale).round() as u32).max(1),
        ((h * scale).round() as u32).max(1),
    );
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    let tile = tile.max(1);
    let mut ty = 0;
    while ty < height {
        let th = tile.min(height - ty);
        let mut tx = 0;
        while tx < width {
            let tw = tile.min(width - tx);
            let x0 = page.rect[0] + f64::from(tx) / scale;
            let y0 = page.rect[1] + f64::from(ty) / scale;
            let piece = Page {
                rect: [
                    x0,
                    y0,
                    x0 + f64::from(tw) / scale,
                    y0 + f64::from(th) / scale,
                ],
                frame: page.frame,
            };
            let (scene, _) = paint_page(board, &piece, scale, paper, false, ts);
            let px = pass.render_and_readback(gpu, scene.inner(), (tw, th))?;
            for row in 0..th as usize {
                let src = &px[row * tw as usize * 4..(row + 1) * tw as usize * 4];
                let at = ((ty as usize + row) * width as usize + tx as usize) * 4;
                rgba[at..at + src.len()].copy_from_slice(src);
            }
            tx += tw;
        }
        ty += th;
    }
    Ok(Image {
        width,
        height,
        rgba,
    })
}

/// Os bytes PNG de `img` (RGBA, 8 bits).
#[must_use]
pub fn encode(img: &Image) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut e = png::Encoder::new(&mut out, img.width, img.height);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        if let Ok(mut w) = e.write_header() {
            let _ = w.write_image_data(&img.rgba);
        }
    }
    out
}
