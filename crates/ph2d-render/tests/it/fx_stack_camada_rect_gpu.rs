//! **A pilha só na área da CENA** (ciclo 20 W4 do Motion, doc 129 §5.7): o que se VÊ é o mesmo
//! que a pilha na camada inteira dá — com discos FORA da cena a projectar sombra para dentro dela,
//! um borrão na pirâmide e a turbulência ancorada no mundo. O controlo é o retângulo sem margem:
//! ele perde o que vem de fora e tem de reprovar.
//!
//! Rodar: `PH2D_GPU=1 cargo test -p ph2d-render --test it fx_stack_camada_rect -- --ignored`.

use ph2d_fx_op::FxOp;
use ph2d_render::{
    BandBlit, BandSource, FxOpGpu, FxStackPass, Origem, make_output_texture, retangulo_da_camada,
};

use crate::fx_stack_common::{camada, readback, try_headless_gpu};

/// Grande o bastante para o borrão da pirâmide (σ 45) caber com a margem e o retângulo ser MENOR
/// que a camada — senão o alinhamento à grelha dela não seria medido.
const W: u32 = 1100;
const H: u32 = 760;
/// A área da cena: os painéis tapam o resto.
const CENA: [u32; 4] = [430, 300, 240, 160];
/// O pixel da origem do mundo (a âncora do ruído da camada inteira).
const ORIGEM: [f32; 2] = [537.5, 371.25];
/// Mundo → pixel (20 px por unidade, Y para baixo).
const LINEAR: [f64; 4] = [20.0, 0.0, 0.0, -20.0];

/// Discos de raio 9 numa grelha de 24 px que cobre a camada INTEIRA — também fora da cena.
fn discos() -> Vec<[f32; 4]> {
    (0..H)
        .flat_map(|y| (0..W).map(move |x| (x, y)))
        .map(|(x, y)| {
            let (dx, dy) = ((x % 24) as f32 - 11.5, (y % 24) as f32 - 11.5);
            let a = (9.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
            [x as f32 / W as f32, 0.6, 1.0 - y as f32 / H as f32, a]
        })
        .collect()
}

fn degrau(kind: u8, muda: impl FnOnce(&mut FxOpGpu)) -> FxOpGpu {
    let mut o = FxOpGpu::resolve(&FxOp::new(kind), LINEAR);
    muda(&mut o);
    o
}

fn pilhas() -> Vec<(&'static str, Vec<FxOpGpu>)> {
    vec![
        (
            "sombra de fora para dentro",
            vec![degrau(FxOp::DROP_SHADOW, |o| {
                o.sigma_px = 4.0;
                o.offset_px = [14.0, 11.0];
            })],
        ),
        (
            "sombra interna + borrão na pirâmide",
            vec![
                degrau(FxOp::INNER_SHADOW, |o| {
                    o.sigma_px = 5.0;
                    o.offset_px = [6.0, 6.0];
                }),
                degrau(FxOp::BLUR, |o| o.sigma_px = 45.0),
            ],
        ),
        (
            "turbulência no mundo",
            vec![degrau(FxOp::TURBULENCE, |_| {})],
        ),
    ]
}

/// O pior desvio, DENTRO da cena, entre a pilha na camada inteira e a pilha no retângulo `r`.
fn desvio_na_cena(cheia: &[u8], r: [u32; 4], parcial: &[u8]) -> i32 {
    let mut pior = 0;
    for y in CENA[1]..CENA[1] + CENA[3] {
        for x in CENA[0]..CENA[0] + CENA[2] {
            let a = ((y * W + x) * 4) as usize;
            let b = (((y - r[1]) * r[2] + (x - r[0])) * 4) as usize;
            for k in 0..4 {
                pior = pior.max((i32::from(cheia[a + k]) - i32::from(parcial[b + k])).abs());
            }
        }
    }
    pior
}

#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_stack_on_the_scene_rect_is_the_stack_on_the_whole_layer_where_it_shows() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_camada_rect] sem adapter — skip");
        return;
    };
    let src = camada(&gpu, W, H, &discos());
    let mut pass = FxStackPass::new(&gpu);
    for (nome, ops) in pilhas() {
        let cheia = make_output_texture(&gpu, W, H);
        pass.run_layer(&gpu, &src, &cheia, W, H, &ops, Origem::Ponto(ORIGEM));
        let rc = readback(&gpu, &cheia, W, H);

        let r = retangulo_da_camada(CENA, (W, H), &ops);
        assert!(
            r[0] > 0 && r[1] > 0 && r[0] + r[2] < W && r[1] + r[3] < H,
            "{nome}: o retângulo {r:?} encosta na camada — o gate não mede a margem"
        );
        let parcial = make_output_texture(&gpu, r[2], r[3]);
        pass.run_layer_rect(&gpu, &src, r, &parcial, &ops, Origem::Ponto(ORIGEM));
        let pior = desvio_na_cena(&rc, r, &readback(&gpu, &parcial, r[2], r[3]));

        // O controlo: o retângulo da cena SEM margem.
        let seco = make_output_texture(&gpu, CENA[2], CENA[3]);
        pass.run_layer_rect(&gpu, &src, CENA, &seco, &ops, Origem::Ponto(ORIGEM));
        let controlo = desvio_na_cena(&rc, CENA, &readback(&gpu, &seco, CENA[2], CENA[3]));
        eprintln!(
            "[fx_stack_camada_rect] {nome}: retângulo {r:?} — pior {pior} níveis · sem margem {controlo}"
        );
        assert!(
            pior <= 1,
            "{nome}: na cena a pilha do retângulo desvia {pior} níveis"
        );
        assert!(
            controlo > 4,
            "{nome}: o controlo sem margem não reprova ({controlo}) — o gate não mede a margem"
        );
    }
}

/// A colagem no retângulo: o resultado pousa NO SÍTIO (o mesmo que a colagem da camada inteira,
/// dentro) e o resto do alvo fica intacto.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_rect_blit_lands_in_place_and_leaves_the_rest() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_camada_rect] sem adapter — skip");
        return;
    };
    let src = camada(&gpu, W, H, &discos());
    let mut pass = FxStackPass::new(&gpu);
    let ops = &pilhas()[0].1;
    let r = retangulo_da_camada(CENA, (W, H), ops);
    let cheia = make_output_texture(&gpu, W, H);
    pass.run_layer(&gpu, &src, &cheia, W, H, ops, Origem::Ponto(ORIGEM));
    let parcial = make_output_texture(&gpu, r[2], r[3]);
    pass.run_layer_rect(&gpu, &src, r, &parcial, ops, Origem::Ponto(ORIGEM));

    let blit = BandBlit::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
    let vista = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
    let fundo = [40u8, 40, 40, 255];
    let alvo = || {
        let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test alvo da colagem"),
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &t,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &fundo.repeat((W * H) as usize),
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
        t
    };
    let (a, b) = (alvo(), alvo());
    blit.blit(&gpu, &vista(&a), &vista(&cheia), BandSource::Vector);
    blit.blit_rect(
        &gpu,
        &vista(&b),
        &vista(&parcial),
        BandSource::Vector,
        Some(r),
    );
    let (ra, rb) = (readback(&gpu, &a, W, H), readback(&gpu, &b, W, H));
    let mut dentro = 0;
    let mut fora_mudou = 0;
    for y in 0..H {
        for x in 0..W {
            let i = ((y * W + x) * 4) as usize;
            let na_cena = (CENA[0]..CENA[0] + CENA[2]).contains(&x)
                && (CENA[1]..CENA[1] + CENA[3]).contains(&y);
            let no_rect = (r[0]..r[0] + r[2]).contains(&x) && (r[1]..r[1] + r[3]).contains(&y);
            if na_cena {
                for k in 0..4 {
                    dentro = dentro.max((i32::from(ra[i + k]) - i32::from(rb[i + k])).abs());
                }
            }
            if !no_rect && rb[i..i + 4] != fundo {
                fora_mudou += 1;
            }
        }
    }
    assert!(
        dentro <= 1,
        "na cena a colagem do retângulo desvia {dentro}"
    );
    assert_eq!(
        fora_mudou, 0,
        "a colagem tocou {fora_mudou} pixels fora do retângulo"
    );
    assert!(
        ra.iter().zip(&rb).any(|(p, q)| p != q),
        "o controlo: a colagem inteira pinta fora do retângulo e esta não"
    );
}
