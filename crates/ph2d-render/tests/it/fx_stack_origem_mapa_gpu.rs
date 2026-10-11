//! **A ORIGEM POR TEXEL da turbulência** (doc 129 §5.5 A6) — `Origem::Mapa` é, texel a texel, o
//! `Origem::Ponto` do valor que o mapa guarda ali.
//!
//! As células do átlas (`ph2d-fx-cells`) ancoram o ruído de cada forma na âncora da SUA instância,
//! e o único canal por onde isso chega ao shader é o `org_map` dos globals. Os gates do átlas
//! comparam células entre si com a origem em zero, onde o mapa e o ponto coincidem; aqui o mapa
//! guarda origens NÃO nulas e diferentes em cada metade.
//!
//! `PH2D_GPU=1 cargo test -p ph2d-render --test it fx_stack_origem_mapa -- --ignored --nocapture`

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, Origem, make_output_texture};

use crate::fx_stack_common::{camada, op, readback, try_headless_gpu};

const W: u32 = 96;
const H: u32 = 64;
/// As origens das duas metades — fora da grade e longe uma da outra.
const ORIGEM_A: [f32; 2] = [13.25, -7.5];
const ORIGEM_B: [f32; 2] = [-21.0, 30.5];

/// Uma camada com estrutura nos dois eixos: qualquer deformação muda pixels.
fn listras() -> Vec<[f32; 4]> {
    let mut v = Vec::with_capacity((W * H) as usize);
    for y in 0..H {
        for x in 0..W {
            let fx = (f64::from(x) * 0.7).sin() * 0.5 + 0.5;
            let fy = (f64::from(y) * 0.45).cos() * 0.5 + 0.5;
            #[allow(clippy::cast_possible_truncation)]
            v.push([fx as f32, fy as f32, 0.4, 1.0]);
        }
    }
    v
}

/// O mapa `Rgba32Float` da origem por texel: `ORIGEM_A` à esquerda, `ORIGEM_B` à direita.
fn mapa(gpu: &ph2d_gpu::GpuContext) -> wgpu::Texture {
    let mut bytes = Vec::with_capacity((W * H * 16) as usize);
    for _ in 0..H {
        for x in 0..W {
            let o = if x < W / 2 { ORIGEM_A } else { ORIGEM_B };
            for v in [o[0], o[1], 0.0, 0.0] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    let size = wgpu::Extent3d {
        width: W,
        height: H,
        depth_or_array_layers: 1,
    };
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test mapa da origem"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
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
            bytes_per_row: Some(W * 16),
            rows_per_image: Some(H),
        },
        size,
    );
    tex
}

fn turbulencia() -> FxOpGpu {
    FxOpGpu {
        noise_scale_px: 24.0,
        detail: 2,
        mode: FxOp::MODE_SMOOTH,
        ..op(FxOp::TURBULENCE, 6.0, [0.0, 0.0, 0.0, 1.0])
    }
}

/// Os bytes que diferem entre `a` e `b` nas colunas `xs`.
fn diferem(a: &[u8], b: &[u8], xs: std::ops::Range<u32>) -> usize {
    let mut n = 0;
    for y in 0..H {
        for x in xs.clone() {
            let o = ((y * W + x) * 4) as usize;
            n += (0..4).filter(|k| a[o + k] != b[o + k]).count();
        }
    }
    n
}

/// **O mapa é o ponto, texel a texel.** Cada metade da saída com `Origem::Mapa` tem de ser BYTE A
/// BYTE a saída com `Origem::Ponto` da origem que o mapa guarda nessa metade.
///
/// Mata a inversão do `org_map` em `FxStackPass::run_with` (`!matches!(origem, Origem::Mapa(_))`):
/// o mapa passaria a ser ignorado (origem zero) e o ponto, a ler a COR da imagem como origem
/// (medido: 4716 e 4771 bytes diferem nas duas metades; o real dá 0 e 0). O lado `Margem` dessa
/// inversão também cai em `the_noise_is_pinned_to_the_shape_not_to_the_scratch`; o lado `Mapa`, só
/// aqui.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_map_is_the_point_it_holds_texel_by_texel() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_origem_mapa] sem adapter — skip");
        return;
    };
    let src = camada(&gpu, W, H, &listras());
    let ops = [turbulencia()];
    let mut pass = FxStackPass::new(&gpu);
    let mut corre = |origem: Origem<'_>| {
        let dst = make_output_texture(&gpu, W, H);
        pass.run_layer(&gpu, &src, &dst, W, H, &ops, origem);
        readback(&gpu, &dst, W, H)
    };
    let m = mapa(&gpu);
    let mv = m.create_view(&wgpu::TextureViewDescriptor::default());
    let com_mapa = corre(Origem::Mapa(&mv));
    let em_a = corre(Origem::Ponto(ORIGEM_A));
    let em_b = corre(Origem::Ponto(ORIGEM_B));

    let (esq, dir) = (0..W / 2, W / 2..W);
    // Controlo: as duas origens desenham ruídos DIFERENTES — senão a igualdade abaixo é vácua.
    let controlo = diferem(&em_a, &em_b, 0..W);
    let erro_esq = diferem(&com_mapa, &em_a, esq);
    let erro_dir = diferem(&com_mapa, &em_b, dir);
    eprintln!(
        "[fx_stack_origem_mapa] controlo A≠B {controlo} bytes | mapa≠A à esquerda {erro_esq} | \
         mapa≠B à direita {erro_dir}"
    );
    assert_eq!(
        (erro_esq, erro_dir),
        (0, 0),
        "a origem POR TEXEL não é a origem que o mapa guarda: o ruído das células não anda com a \
         âncora da instância"
    );
    assert!(
        controlo > (W * H) as usize,
        "as origens A e B desenham quase o mesmo ({controlo} bytes diferem) — o gate não mede a origem"
    );
}
