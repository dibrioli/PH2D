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
    assert!(
        vp.refazer_em.is_none() && vp.refeitos == 1,
        "a entrega depois da falta devia refazer o renderer UMA vez e sair sã ({} refeitos)",
        vp.refeitos
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
        vp.refeitos,
        t.elapsed()
    );
}
