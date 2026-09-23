//! ⭐⭐⭐ **O mundo chega ao Vello AO BYTE, e uma camada de mistura mistura-se com ELE** — o gate de
//! placa da W2 do doc 118.
//!
//! A fixtura é um «mundo» de `256×2` no MESMO formato do acumulador (`Bgra8Unorm` com vista
//! `Bgra8UnormSrgb`), com os três canais DIFERENTES em cada coluna (uma troca de canais `B↔R` não
//! passaria despercebida) e todos os 256 valores de um byte no canal vermelho:
//!
//! - **linha 0** — sem nada por cima: o intermédio tem de devolver o mundo **ao byte**. É a afirmação
//!   de que descodificar e re-codificar sRGB é exacto para 8 bits, de que o cabeçalho do módulo
//!   depende;
//! - **linha 1** — uma camada `Multiply` com um cinzento `s`: tem de dar `mundo · s`.
//!
//! ⭐ **O CONTROLO é o render de sempre** sobre a mesma cena: sem o mundo por baixo, a linha 1 lê
//! `s` (a camada mistura-se com o vazio) — que é o defeito que a W2 cura. Sem essa metade, uma porta
//! que ignorasse o mundo e pintasse `s` ficaria verde na linha 1 por acaso de valores.

use crate::VelloPass;
use ph2d_gpu::GpuContext;
use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{BlendMode, Color, Compose, Fill, Mix};

const W: u32 = 256;
const H: u32 = 2;
/// O cinzento da camada `Multiply`, em byte.
const S: u8 = 128;

fn try_headless_gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// A cor RGBA do mundo na coluna `x` — três canais diferentes, e o vermelho a percorrer os 256.
/// `virado` inverte os três: é o «quadro seguinte», com o mundo mudado.
#[allow(clippy::cast_possible_truncation)]
fn mundo_em(x: u32, virado: bool) -> [u8; 3] {
    let c = [x as u8, (255 - x) as u8, ((x * 7) % 256) as u8];
    if virado { c.map(|v| 255 - v) } else { c }
}

/// O mundo, como o acumulador o guarda: `Bgra8Unorm` com vista de amostragem sRGB.
fn mundo(gpu: &GpuContext, virado: bool) -> wgpu::TextureView {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mundo de teste"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
    });
    let mut bytes = Vec::with_capacity((W * H * 4) as usize);
    for _ in 0..H {
        for x in 0..W {
            let [r, g, b] = mundo_em(x, virado);
            bytes.extend_from_slice(&[b, g, r, 255]);
        }
    }
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(W * 4),
            rows_per_image: Some(H),
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    tex.create_view(&wgpu::TextureViewDescriptor {
        format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
        ..Default::default()
    })
}

/// A cena: uma camada `Multiply` sobre a linha 1, com o cinzento `S`.
fn cena() -> Scene {
    let mut s = Scene::new();
    let linha1 = Rect::new(0.0, 1.0, f64::from(W), 2.0);
    s.push_layer(
        Fill::NonZero,
        BlendMode::new(Mix::Multiply, Compose::SrcOver),
        1.0,
        Affine::IDENTITY,
        &linha1,
    );
    s.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(S, S, S, 255),
        None,
        &linha1,
    );
    s.pop_layer();
    s
}

#[test]
#[ignore = "precisa de GPU"]
fn o_mundo_chega_ao_byte_e_a_camada_mistura_com_ele() {
    let Some(gpu) = try_headless_gpu() else {
        println!("sem adaptador — saltado");
        return;
    };
    let Ok(mut pass) = VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (W, H)) else {
        println!("sem VelloPass — saltado");
        return;
    };
    let cena = cena();

    // ⭐ O CONTROLO primeiro: o render de sempre mistura a camada com o vazio.
    pass.render_to_intermediate(&gpu, &cena, (W, H), Color::TRANSPARENT)
        .expect("render de sempre");
    let sempre = pass.read_intermediate(&gpu).expect("readback");
    let px = |b: &[u8], x: u32, y: u32| {
        let i = ((y * W + x) * 4) as usize;
        [b[i], b[i + 1], b[i + 2]]
    };
    assert_eq!(
        px(&sempre, 100, 1),
        [S, S, S],
        "o controlo não reproduz o defeito — sem o mundo por baixo a camada devia ler o cinzento \
         puro, e as asserções abaixo não distinguiriam nada"
    );

    // ⚠️ DOIS quadros, com o mundo MUDADO no segundo: desde a `vello` 0.10 o atlas é persistente, e
    // uma textura re-escrita sem a marca de suja serve os pixels VELHOS — o segundo quadro é a
    // metade que o apanha.
    let mut erros = Vec::new();
    for virado in [false, true] {
        let vista = mundo(&gpu, virado);
        pass.render_to_intermediate_over_world(&gpu, &cena, &vista, (W, H))
            .expect("render com o mundo");
        let com = pass.read_intermediate(&gpu).expect("readback");
        for x in 0..W {
            let m = mundo_em(x, virado);
            if px(&com, x, 0) != m {
                erros.push(format!(
                    "linha 0, x={x}: {:?} contra o mundo {m:?}",
                    px(&com, x, 0)
                ));
            }
            let esperado = m.map(|c| (f32::from(c) * f32::from(S) / 255.0).round());
            let lido = px(&com, x, 1);
            for k in 0..3 {
                if (f32::from(lido[k]) - esperado[k]).abs() > 1.0 {
                    erros.push(format!(
                        "linha 1, x={x}: {lido:?} contra mundo·s {esperado:?}"
                    ));
                    break;
                }
            }
        }
    }
    assert!(
        erros.is_empty(),
        "{} píxeis fora (os 8 primeiros):\n{}",
        erros.len(),
        erros.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}
