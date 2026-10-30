//! **F35 §2 — o renderer REFEITO depois de uma recusa de memória** (`docs/UI_New_and_Simple/F35_plano.md`).

use super::VelloPass;
use ph2d_gpu::GpuContext;
use vello::kurbo::Affine;
use vello::peniko::{ImageBrush, ImageQuality};

const LADO: u32 = 32;

/// Uma textura de `LADO²` toda vermelha, pronta a registar.
fn vermelha(gpu: &GpuContext) -> wgpu::Texture {
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("F35 vermelha"),
        size: wgpu::Extent3d {
            width: LADO,
            height: LADO,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let px: Vec<u8> = (0..LADO * LADO).flat_map(|_| [255, 0, 0, 255]).collect();
    gpu.queue.write_texture(
        t.as_image_copy(),
        &px,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(LADO * 4),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: LADO,
            height: LADO,
            depth_or_array_layers: 1,
        },
    );
    t
}

/// **G15 (placa) — o renderer refeito herda as texturas registadas nos MESMOS ids.** O FX e o mundo
/// por baixo guardam a `ImageData` do registo; refazer o renderer sem as reaplicar apagava a forma
/// filtrada da tela depois de a placa recuperar.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_renderer_rebuilt_after_out_of_memory_still_draws_the_registered_textures() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    let tela = (64, 64);
    let mut vp = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, tela).expect("vello");
    let img = vp.register_texture(vermelha(&gpu));
    let mut cena = vello::Scene::new();
    cena.draw_image(
        ImageBrush::new(img)
            .with_quality(ImageQuality::Low)
            .as_ref(),
        Affine::translate((16.0, 16.0)),
    );
    let vermelho_em = |px: &[u8]| {
        let i = ((30 * tela.0 + 30) * 4) as usize;
        px[i..i + 4].to_vec()
    };
    let antes = vp.render_and_readback(&gpu, &cena, tela).expect("render");
    assert_eq!(vermelho_em(&antes), [255, 0, 0, 255], "antes da falta");
    vp.finge_falta_de_memoria();
    let depois = vp
        .render_and_readback(&gpu, &cena, tela)
        .expect("render refeito");
    let (a_refazer, refeitos) = vp.estado_do_renderer();
    assert!(
        !a_refazer && refeitos == 1,
        "a entrega depois da falta devia refazer o renderer UMA vez e sair sã ({refeitos} refeitos)"
    );
    assert_eq!(
        vermelho_em(&depois),
        [255, 0, 0, 255],
        "o renderer refeito perdeu a textura registada"
    );
}

/// **Sonda (placa CHEIA de verdade)** — o caminho do smoke de 26/10 sem o `ollama`: um lastro enche a
/// placa até ela recusar, o render recusa e é dito UMA vez, o lastro sai e, depois da espera, o
/// renderer refeito desenha de novo. ⛔ Enche a placa principal por ~1 s: só com `F35_PLACA_CHEIA=1`.
///
///     F35_PLACA_CHEIA=1 PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-render --lib placa_cheia -- --ignored --nocapture
#[test]
#[ignore = "sonda F35: enche a placa principal; só com F35_PLACA_CHEIA=1"]
fn sonda_da_placa_cheia() {
    if std::env::var_os("F35_PLACA_CHEIA").is_none() {
        return;
    }
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    eprintln!("[f35-cheia] placa: {}", gpu.adapter.get_info().name);
    let tela = (64, 64);
    let mut vp = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, tela).expect("vello");
    let img = vp.register_texture(vermelha(&gpu));
    let mut cena = vello::Scene::new();
    cena.draw_image(
        ImageBrush::new(img)
            .with_quality(ImageQuality::Low)
            .as_ref(),
        Affine::translate((16.0, 16.0)),
    );
    // O lastro: blocos de 64 MiB até a placa recusar (no máximo 32 GiB).
    let faltas0 = ph2d_gpu::memoria::VIGIA.faltas();
    let mut lastro = Vec::new();
    for _ in 0..512 {
        let (b, falta) = ph2d_gpu::memoria::sob_escopo_de_memoria(&gpu.device, || {
            let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("F35 lastro"),
                size: 64 << 20,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            enc.clear_buffer(&b, 0, None);
            gpu.queue.submit([enc.finish()]);
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            b
        });
        if falta.is_some() {
            break;
        }
        lastro.push(b);
    }
    eprintln!("[f35-cheia] lastro: {} MiB", lastro.len() * 64);
    let t = std::time::Instant::now();
    let cheia = vp.render_and_readback(&gpu, &cena, tela);
    let dito = cheia.is_err();
    let segunda = vp.render_and_readback(&gpu, &cena, tela);
    drop(lastro);
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    std::thread::sleep(super::ESPERA_SEM_MEMORIA);
    let depois = vp.render_and_readback(&gpu, &cena, tela);
    eprintln!(
        "[f35-cheia] cheia: {} · repetida: {} · faltas anotadas: {} · depois de libertar: {} \
         (refeitos {}) em {:?}",
        if dito { "Err (dito)" } else { "Ok" },
        if segunda.is_ok() {
            "Ok (calada)"
        } else {
            "Err"
        },
        ph2d_gpu::memoria::VIGIA.faltas() - faltas0,
        match &depois {
            Ok(px) => format!(
                "pixel {:?}",
                &px[((30 * 64 + 30) * 4)..((30 * 64 + 30) * 4 + 4)]
            ),
            Err(e) => e.clone(),
        },
        vp.estado_do_renderer().1,
        t.elapsed()
    );
}

/// Um episódio de falta inteiro no vigia do processo: a geração sobe uma vez.
fn acaba_um_episodio() {
    use ph2d_gpu::memoria::{JANELA_MS, VIGIA, geracao};
    let _ = VIGIA.mudou_em(u64::MAX / 2);
    let g = geracao();
    VIGIA.anota_em(0);
    assert_eq!(VIGIA.mudou_em(0), Some(true));
    assert_eq!(VIGIA.mudou_em(JANELA_MS), Some(false));
    assert_eq!(geracao(), g + 1, "o episódio devia ter acabado");
}

/// Uma cena com um quadrado verde opaco em `(8..24)²`.
fn quadrado_verde() -> vello::Scene {
    let mut cena = vello::Scene::new();
    cena.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        vello::peniko::Color::from_rgba8(0, 255, 0, 255),
        None,
        &vello::kurbo::Rect::new(8.0, 8.0, 24.0, 24.0),
    );
    cena
}

/// **G32 (placa) — a intermédia que a placa recusa num resize NÃO é instalada** (F40, cena 41 com a
/// placa cheia: `create_view`/readback sobre a «vello intermediate» inválida). Fica a de antes,
/// válida; o pedido não se repete no mesmo episódio; nenhum trabalho de placa sobre ela (o render e a
/// leitura recusam) e zero `[gpu-erro]`.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_refused_intermediate_resize_keeps_the_valid_one_and_does_no_gpu_work() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    let erros = ph2d_gpu::context::erros_relatados();
    let mut vp = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, (32, 32)).expect("vello");
    let cena = quadrado_verde();
    vp.render_and_readback(&gpu, &cena, (32, 32))
        .expect("antes");
    crate::renasce::testes::forca_recusas(1);
    assert!(
        vp.render_and_readback(&gpu, &cena, (64, 48)).is_err(),
        "a leitura de um tamanho recusado devolveu pixels"
    );
    assert!(vp.em_recusa());
    let t = vp.intermediate_texture();
    assert_eq!(
        (t.width(), t.height()),
        (32, 32),
        "a intermédia recusada foi instalada"
    );
    // O mesmo episódio não volta a pedir (a próxima forçada ficaria por consumir).
    crate::renasce::testes::forca_recusas(1);
    assert!(vp.render_and_readback(&gpu, &cena, (64, 48)).is_err());
    assert!(
        crate::renasce::testes::consome_recusa_forcada(),
        "o mesmo episódio voltou a pedir a intermédia"
    );
    let px = vp
        .render_and_readback(&gpu, &cena, (32, 32))
        .expect("o tamanho de antes");
    let i = ((16 * 32 + 16) * 4) as usize;
    assert_eq!(
        &px[i..i + 4],
        &[0, 255, 0, 255],
        "a intermédia de antes deixou de desenhar"
    );
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    assert_eq!(
        ph2d_gpu::context::erros_relatados(),
        erros,
        "a recusa da intermédia deu [gpu-erro]"
    );
}

/// **G33 (placa) — no fim do episódio a intermédia renasce no tamanho pedido e desenha.**
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_refused_intermediate_is_reborn_at_the_requested_size_when_the_episode_ends() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        return;
    };
    crate::renasce::testes::forca_recusas(1);
    let mut vp = VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8Unorm, (64, 48)).expect("vello");
    assert!(
        vp.em_recusa(),
        "a intermédia recusada à nascença não ficou em recusa"
    );
    let cena = quadrado_verde();
    assert!(vp.render_and_readback(&gpu, &cena, (64, 48)).is_err());
    acaba_um_episodio();
    let px = vp
        .render_and_readback(&gpu, &cena, (64, 48))
        .expect("depois do episódio");
    let t = vp.intermediate_texture();
    assert_eq!((t.width(), t.height()), (64, 48));
    let i = ((16 * 64 + 16) * 4) as usize;
    assert_eq!(
        &px[i..i + 4],
        &[0, 255, 0, 255],
        "a intermédia renascida não desenha"
    );
}
