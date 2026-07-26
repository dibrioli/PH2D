//! **`FxStackPass` na GPU** (a PILHA de FX raster do módulo vetorial, plano 24 W2).
//!
//! `#[ignore]`d: precisa de um dispositivo de verdade, então roda na máquina do dev / na lane de
//! GPU do CI, não nos runners CPU headless (`None` de `try_headless_gpu`). Rodar:
//! `cargo test -p ph2d-render --test fx_stack_gpu -- --ignored`.
//!
//! O gate que carrega a wave é o [`the_order_of_the_stack_changes_the_picture`]: se trocar dois
//! degraus de lugar desenhasse o mesmo, a pilha seria uma lista de coisas independentes e não uma
//! COMPOSIÇÃO — e não haveria nada aqui que um filtro único não fizesse.

use std::sync::OnceLock;

use ph2d_gpu::GpuContext;
use ph2d_render::{FxOpGpu, FxStackPass, VelloPass, make_output_texture, stack_reach};

/// Um degrau, resolvido em pixels.
fn op(kind: u8, sigma_px: f32, tint: [f32; 4]) -> FxOpGpu {
    FxOpGpu {
        kind,
        sigma_px,
        offset_px: [0, 0],
        tint,
        opacity: 1.0,
    }
}

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn try_headless_gpu() -> Option<GpuContext> {
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// Uma textura `Rgba8Unorm` de entrada (`TEXTURE_BINDING | COPY_DST`) com `bytes` (premultiplicado).
fn make_src(gpu: &GpuContext, w: u32, h: u32, bytes: &[u8]) -> wgpu::Texture {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test fx_stack src"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
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
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    tex
}

fn readback(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test fx_stack readback"),
        size: (padded as u64) * (h as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    rx.recv().unwrap().unwrap();
    let view = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded as usize) * (h as usize));
    for row in 0..h as usize {
        let s = row * padded as usize;
        out.extend_from_slice(&view[s..s + unpadded as usize]);
    }
    drop(view);
    staging.unmap();
    out
}

/// Um degrau opaco→transparente vira uma RAMPA monótona de alfa centrada na fronteira, e ela
/// ALARGA com o sigma — a propriedade que separa um borrão de um simples corte de alfa (a queixa
/// que o produto existe para responder). No shader, na GPU, sem CPU no caminho.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_step_edge_becomes_a_monotone_alpha_ramp_that_widens_with_sigma() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_gpu] sem adapter — skip");
        return;
    };
    let (w, h) = (64u32, 8u32);
    // Premultiplicado: metade esquerda branca opaca, direita transparente.
    let mut src_bytes = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w / 2 {
            let o = ((y * w + x) * 4) as usize;
            src_bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let src = make_src(&gpu, w, h, &src_bytes);
    let mut pass = FxStackPass::new(&gpu);

    // Lê a fileira do meio (alfa por coluna) para um dado sigma.
    let alpha_row = |pass: &mut FxStackPass, sigma: f32| -> Vec<u8> {
        let dst = make_output_texture(&gpu, w, h);
        pass.run(&gpu, &src, &dst, w, h, &[op(0, sigma, BLACK)]);
        let bytes = readback(&gpu, &dst, w, h);
        let y = h / 2;
        (0..w)
            .map(|x| bytes[(((y * w + x) * 4) + 3) as usize])
            .collect()
    };
    let ramp_width = |a: &[u8]| a.iter().filter(|&&v| v > 25 && v < 230).count();

    let narrow = alpha_row(&mut pass, 2.0);
    let wide = alpha_row(&mut pass, 6.0);

    for a in [&narrow, &wide] {
        // Monótona não-crescente (opaca → transparente). i32 para não estourar o u8.
        for pair in a.windows(2) {
            assert!(
                i32::from(pair[1]) <= i32::from(pair[0]) + 4,
                "ramp de alfa nao e monotona: {a:?}"
            );
        }
        // Centrada: ~127 na fronteira (col 31/32).
        let mid = i32::from(a[(w / 2 - 1) as usize]);
        assert!(
            (mid - 127).abs() < 40,
            "fronteira nao esta em ~0.5 (deu {mid})"
        );
    }
    assert!(
        ramp_width(&wide) > ramp_width(&narrow),
        "sigma 6 ({}) deveria alargar mais que sigma 2 ({})",
        ramp_width(&wide),
        ramp_width(&narrow),
    );
}

/// **Repro do caminho de RENDER do FX** (o "panic ao abrir"): registrar uma textura de GPU no
/// renderer, desenhá-la numa `Scene` e RENDERIZAR — o que a shell faz por frame. Se o Vello não
/// aguentar uma textura-imagem override desenhada + renderizada, é AQUI que estoura.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn registering_a_texture_and_drawing_it_renders_without_panic() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_gpu] sem adapter — skip");
        return;
    };
    let mut vp =
        VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, (256, 256)).expect("main VelloPass");
    // Uma textura de saída de FX (vazia — o conteúdo não importa para o teste de render).
    let tex = make_output_texture(&gpu, 64, 64);
    let img = vp.register_texture(tex);
    // Desenha a imagem override numa Scene e renderiza pelo MESMO renderer (como a shell).
    let mut scene = vello::Scene::new();
    let brush =
        vello::peniko::ImageBrush::new(img).with_quality(vello::peniko::ImageQuality::Medium);
    scene.draw_image(
        brush.as_ref(),
        vello::kurbo::Affine::translate((10.0, 10.0)),
    );
    vp.render_to_intermediate(
        &gpu,
        &scene,
        (256, 256),
        vello::peniko::Color::TRANSPARENT,
        false,
    )
    .expect("render with an overridden image");
}

/// **O caminho INTEIRO da shell, com DOIS renderers** (o "panic ao abrir"): um scratch renderiza
/// a forma, o `FxBlurPass` borra, o renderer PRINCIPAL registra a textura borrada e a desenha +
/// renderiza. É a sequência exata do `fx_live::recook` + `dispatch` + `present`.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_two_renderer_scratch_blur_register_render_path_does_not_panic() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_gpu] sem adapter — skip");
        return;
    };
    let (w, h) = (64u32, 64u32);
    // 1. Scratch renderer renderiza uma forma isolada (um retângulo preenchido).
    let mut scratch =
        VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (w, h)).expect("scratch");
    let mut shape = vello::Scene::new();
    shape.fill(
        vello::peniko::Fill::NonZero,
        vello::kurbo::Affine::IDENTITY,
        vello::peniko::Color::from_rgba8(230, 170, 60, 255),
        None,
        &vello::kurbo::Rect::new(12.0, 12.0, 52.0, 52.0),
    );
    scratch
        .render_to_intermediate(
            &gpu,
            &shape,
            (w, h),
            vello::peniko::Color::TRANSPARENT,
            false,
        )
        .expect("scratch render");

    // 2. Roda a pilha sobre o intermediate do scratch numa textura de saída.
    let mut stack = FxStackPass::new(&gpu);
    let dst = make_output_texture(&gpu, w, h);
    stack.run(
        &gpu,
        scratch.intermediate_texture(),
        &dst,
        w,
        h,
        &[op(0, 4.0, BLACK)],
    );

    // 3. Renderer PRINCIPAL registra a textura borrada e a desenha + renderiza.
    let mut main =
        VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (256, 256)).expect("main");
    let img = main.register_texture(dst);
    let mut scene = vello::Scene::new();
    let brush =
        vello::peniko::ImageBrush::new(img).with_quality(vello::peniko::ImageQuality::Medium);
    scene.draw_image(
        brush.as_ref(),
        vello::kurbo::Affine::translate((20.0, 20.0)),
    );
    main.render_to_intermediate(
        &gpu,
        &scene,
        (256, 256),
        vello::peniko::Color::TRANSPARENT,
        false,
    )
    .expect("main render with the blurred FX image");
}

/// **O RESIZE re-registra (dims corretas), não faz override** — o "panic ao zoom / deforma ao
/// maximizar". Registrar a 64², depois RE-registrar a 96² (dims novas) e desenhar + renderizar tem
/// de rodar sem overrun. (Com `override_image`, a `ImageData` guardava as dims VELHAS e o Vello
/// copiava além da textura nova → validation error / imagem esticada.)
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn re_registering_on_resize_does_not_overrun() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_gpu] sem adapter — skip");
        return;
    };
    let mut main =
        VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (256, 256)).expect("main");
    // Registra a 64².
    let img_a = main.register_texture(make_output_texture(&gpu, 64, 64));
    // "Resize" para 96²: re-registra (dims novas), desregistra o antigo — o que o recook faz.
    let img_b = main.register_texture(make_output_texture(&gpu, 96, 96));
    main.unregister_texture(img_a);
    // Desenha a NOVA e renderiza — as dims da ImageData batem com a textura ⇒ sem overrun.
    let mut scene = vello::Scene::new();
    let brush =
        vello::peniko::ImageBrush::new(img_b).with_quality(vello::peniko::ImageQuality::Medium);
    scene.draw_image(
        brush.as_ref(),
        vello::kurbo::Affine::translate((10.0, 10.0)),
    );
    main.render_to_intermediate(
        &gpu,
        &scene,
        (256, 256),
        vello::peniko::Color::TRANSPARENT,
        false,
    )
    .expect("render after resize re-register");
}

/// **O halo é do EFEITO e a FORMA sobrevive por cima** — a semântica que faz o degrau ser
/// imagem→imagem (e que matou o `FxMode::Below` da W1). Fixture: forma BRANCA opaca, tint
/// VERMELHO, sigma pequeno — no MIOLO tem de sobrar branco (a forma), e FORA dela vermelho (o
/// halo). Um Glow que pintasse o miolo de vermelho não poderia alimentar o degrau seguinte:
/// ele teria comido a imagem que recebeu.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_glow_paints_its_halo_under_the_shape_which_survives() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_gpu] sem adapter — skip");
        return;
    };
    let (w, h) = (32u32, 32u32);
    let mut src_bytes = vec![0u8; (w * h * 4) as usize];
    for y in 12..20 {
        for x in 12..20 {
            let o = ((y * w + x) * 4) as usize;
            src_bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]); // forma branca
        }
    }
    let src = make_src(&gpu, w, h, &src_bytes);
    let mut pass = FxStackPass::new(&gpu);
    let dst = make_output_texture(&gpu, w, h);
    pass.run(&gpu, &src, &dst, w, h, &[op(1, 2.0, RED)]);
    let bytes = readback(&gpu, &dst, w, h);
    let px = |x: u32, y: u32| {
        let o = (((y * w) + x) * 4) as usize;
        [bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]
    };
    // Miolo: a FORMA (branca) sobreviveu por cima do halo.
    let core = px(16, 16);
    assert!(
        core[0] > 200 && core[1] > 200 && core[2] > 200 && core[3] > 200,
        "a forma tem de sobreviver por cima do halo (miolo deu {core:?})"
    );
    // Fora da forma: o halo, na cor do EFEITO.
    let halo = px(10, 16);
    assert!(
        halo[3] > 20,
        "o halo tem de cobrir fora da forma (deu {halo:?})"
    );
    assert!(
        halo[0] > 150 && halo[1] < 90 && halo[2] < 90,
        "o halo tem de ser a cor do EFEITO, não a da forma (deu {halo:?})"
    );
}

/// **A ORDEM da pilha muda o desenho.** O gate da wave: os MESMOS dois degraus (`Glow` e `Blur`),
/// trocados de lugar, têm de produzir imagens diferentes. `Glow → Blur` lava o halo junto com a
/// forma; `Blur → Glow` faz o halo nascer da silhueta já engordada. Uma pilha que aplicasse os
/// degraus em ordem fixa (a mutação) daria as duas iguais, e a wave inteira seria decorativa.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_order_of_the_stack_changes_the_picture() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_gpu] sem adapter — skip");
        return;
    };
    let (w, h) = (48u32, 48u32);
    let mut src_bytes = vec![0u8; (w * h * 4) as usize];
    for y in 18..30 {
        for x in 18..30 {
            let o = ((y * w + x) * 4) as usize;
            src_bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let src = make_src(&gpu, w, h, &src_bytes);
    let mut pass = FxStackPass::new(&gpu);
    let (glow, blur) = (op(1, 3.0, RED), op(0, 3.0, BLACK));

    let render = |pass: &mut FxStackPass, ops: &[FxOpGpu]| -> Vec<u8> {
        let dst = make_output_texture(&gpu, w, h);
        pass.run(&gpu, &src, &dst, w, h, ops);
        readback(&gpu, &dst, w, h)
    };
    let a = render(&mut pass, &[glow, blur]);
    let b = render(&mut pass, &[blur, glow]);

    let differing = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    let worst = a
        .iter()
        .zip(&b)
        .map(|(x, y)| i32::from(*x) - i32::from(*y))
        .map(i32::abs)
        .max()
        .unwrap_or(0);
    // Os dois eram idênticos com a mutação "aplique sempre o primeiro op"; aqui o fosso é grande.
    assert!(
        differing > 200 && worst > 30,
        "trocar a ordem tem de mudar o desenho (bytes diferentes {differing}, pior delta {worst})"
    );
}

/// **Uma pilha VAZIA é a identidade**, não um caso especial: o `resolve` des-premultiplica a
/// entrada e mais nada. É o que permite ao produtor não conhecer "o caso sem filtro" — quem decide
/// não produzir imagem nenhuma é a shell, uma camada acima.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn an_empty_stack_is_the_identity() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_gpu] sem adapter — skip");
        return;
    };
    let (w, h) = (8u32, 8u32);
    let mut src_bytes = vec![0u8; (w * h * 4) as usize];
    for y in 2..6 {
        for x in 2..6 {
            let o = ((y * w + x) * 4) as usize;
            src_bytes[o..o + 4].copy_from_slice(&[200, 100, 50, 255]); // opaco ⇒ premul == reto
        }
    }
    let src = make_src(&gpu, w, h, &src_bytes);
    let mut pass = FxStackPass::new(&gpu);
    let dst = make_output_texture(&gpu, w, h);
    pass.run(&gpu, &src, &dst, w, h, &[]);
    let bytes = readback(&gpu, &dst, w, h);
    let o = (((4 * w) + 4) * 4) as usize;
    assert_eq!(
        &bytes[o..o + 4],
        &[200, 100, 50, 255],
        "pilha vazia tem de devolver a entrada (opaca ⇒ premul == reto)"
    );
}

/// **O custo por DEGRAU — o número que fixa o `VecFilter::MAX_OPS`.** Mede, não afirma: a wave
/// escolheu 6 e a medição diz o que 6 custa. `--ignored --nocapture` para ver a tabela.
#[test]
#[ignore = "medição manual; rode com --ignored --nocapture na lane de GPU"]
fn the_cost_of_a_stack_is_linear_in_the_number_of_ops() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_gpu] sem adapter — skip");
        return;
    };
    let (w, h) = (512u32, 512u32);
    let src_bytes = vec![128u8; (w * h * 4) as usize];
    let src = make_src(&gpu, w, h, &src_bytes);
    let mut pass = FxStackPass::new(&gpu);
    let dst = make_output_texture(&gpu, w, h);
    let one = op(0, 8.0, BLACK);
    eprintln!("[fx-stack] custo por profundidade de pilha, {w}x{h}, sigma 8 px:");
    for n in [0usize, 1, 2, 3, 4, 6] {
        let ops: Vec<FxOpGpu> = std::iter::repeat_n(one, n).collect();
        // Aquece (compilação de pipeline / alocação das temps) antes de medir.
        pass.run(&gpu, &src, &dst, w, h, &ops);
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        let t0 = std::time::Instant::now();
        const ITERS: u32 = 20;
        for _ in 0..ITERS {
            pass.run(&gpu, &src, &dst, w, h, &ops);
        }
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(ITERS);
        eprintln!("  {n} degrau(s): {ms:.3} ms");
    }
}

/// **A margem SOMA ao longo da pilha e é ASSIMÉTRICA numa sombra.** Função pura — sem GPU, roda em
/// qualquer runner. É a régua que dimensiona a textura: se ela mentisse para menos, o borrão do
/// último degrau seria recortado pela borda.
#[test]
fn the_reach_sums_along_the_stack_and_leans_the_way_the_shadow_falls() {
    let blur = op(0, 4.0, BLACK);
    let (l1, t1, r1, b1) = stack_reach(&[blur]);
    let (l2, t2, r2, b2) = stack_reach(&[blur, blur]);
    assert_eq!(
        (l2, t2, r2, b2),
        (l1 * 2, t1 * 2, r1 * 2, b1 * 2),
        "dois degraus iguais espalham o dobro de um"
    );
    // Uma sombra que cai para a direita e para baixo NÃO paga textura à esquerda nem acima.
    let shadow = FxOpGpu {
        offset_px: [20, 12],
        ..op(2, 1.0, BLACK)
    };
    let (l, t, r, b) = stack_reach(&[shadow]);
    assert!(
        r > l && b > t,
        "a margem tem de pender para onde a sombra cai (l{l} t{t} r{r} b{b})"
    );
    assert_eq!((r - l, b - t), (20, 12), "e pender exatamente o offset");
}
