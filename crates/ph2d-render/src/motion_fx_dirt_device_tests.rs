//! **OS GATES DE DEVICE DA MÁSCARA DE SUJIDADE** — os que precisam de um adapter e de ler
//! pixels de volta.
//!
//! ⚠️ **O corte foi FORÇADO pelo teto de LOC** (700), e a costura é por responsabilidade: o
//! `motion_fx_tests.rs` fica com a aritmética do passe (o que se mede sem GPU) e este com o que
//! só um device responde — o FIO entre o Rust e o WGSL, e a contagem de bind groups.
//!
//! ⚠️ **FILHO e não irmão**: ele usa o `try_headless_gpu` e o `read_rt` do pai, que são privados
//! de propósito — duplicá-los seria uma segunda leitura de textura a divergir da primeira.

use super::*;

/// ⛔⛔ **DOIS PASSES POR QUADRO NÃO PODEM ALTERNAR A CHAVE ENTRE SI.**
///
/// Medido em 2026-08-27: o `bloom_over` tem dois chamadores por quadro (`render_loop::present`),
/// o do emissivo com `dirt: None` e o do glow do Motion com `dirt: Some(..)`, e eles **não são
/// exclusivos**. A chave alternava duas vezes por quadro ⇒ ≈24 bind groups por quadro, 1 440/s,
/// que é a *"alocação de descritor a 60 Hz"* que a chave existe para evitar.
///
/// ⚠️ A cerca não é uma chave por passe: um passe com `dirt_intensity == 0` multiplica a máscara
/// por zero e **não se importa** com o que está ligado. Este gate encena os dois passes na ordem
/// real e conta.
#[test]
fn a_pass_that_does_not_read_the_mask_does_not_rebind_it() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[motion_fx] SEM ADAPTER -- este gate NAO correu");
        return;
    };
    let mut fx = MotionFx::new(&gpu, (64, 64));
    let target = crate::GameRt::new(&gpu, (64, 64));
    let img = crate::GameRt::new(&gpu, (32, 32));
    let quiet = BloomParams::default(); // o passe do emissivo: `dirt_intensity = 0`
    let loud = BloomParams {
        dirt_intensity: 3.0,
        ..BloomParams::default()
    };
    let mask = || {
        Some(crate::DirtMask {
            view: img.view(),
            key: 11,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            aspect: 1.0,
        })
    };
    // Três quadros do par (emissivo sem máscara → motion com máscara).
    for _ in 0..3 {
        fx.bloom_over(&gpu, target.view(), &quiet, None, None);
        fx.bloom_over(&gpu, target.view(), &loud, None, mask());
    }
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    assert_eq!(
        fx.dirt_rebinds, 1,
        "os dois passes do quadro estao a alternar a chave — 3 quadros deviam custar UM rebind"
    );
    // ⚠️ **CONTROLE — sem ele, um `wanted` que nunca mudasse passaria com zero rebinds e a
    // feature ficaria desligada em silêncio.** Trocar de imagem TEM de rebindar.
    fx.bloom_over(
        &gpu,
        target.view(),
        &loud,
        None,
        Some(crate::DirtMask {
            view: img.view(),
            key: 12,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            aspect: 1.0,
        }),
    );
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    assert_eq!(fx.dirt_rebinds, 2, "escolher outra imagem TEM de rebindar");
}

/// Lê um `GameRt` de volta para bytes — `Rgba16Float`, 8 B/texel.
/// ⛔⛔ **O ENQUADRAMENTO CHEGA AO SHADER NA ORDEM EM QUE O SHADER O LÊ.**
///
/// A lei (`dirt::scale_offset`) está bem defendida **em isolamento** — nenhuma mutação dentro
/// dela sobrevive. O que não tinha gate era o **fio**: o Rust escreve `so[0..3]` nas posições
/// `8..11` de um `[f32; 12]` e o WGSL lê `in.uv * P.v3.xy + P.v3.zw`. Nada afirmava que a ordem
/// é essa, e a auditoria de 2026-08-27 **provou a mutação a sobreviver**: trocar `xy` com `zw`
/// (escala e deslocamento invertidos) deixava `ph2d-render` 337/337 verde.
///
/// ⚠️ **É a MESMA classe que esta feature já pagou uma vez** — uma convenção lida ao contrário,
/// 13 gates verdes — só que um nível abaixo, no fio em vez do valor.
///
/// ⚠️ **A régua tem de ser um mapeamento NÃO-IDENTIDADE, e é por isso que os gates que existiam
/// não podiam vê-lo:** eles usam `uv_rect = [0,0,1,1]` com o aspecto do ecrã, o que dá
/// `so = [1,1,0,0]` — trocar `xy` com `zw` ali é quase um no-op —, e a máscara deles é um clear
/// CHAPADO, sobre o qual *qualquer* enquadramento devolve o mesmo pixel. *Uma fixtura plana não
/// pode medir um mapeamento.*
///
/// Aqui a máscara é assimétrica e o rect é um sub-rect de átlas (`so = [0.5, 0.5, 0, 0]`), então:
/// - ordem certa ⇒ `dirt_uv = uv·0,5` varre meia textura ⇒ **o quadro herda a assimetria**;
/// - ordem trocada ⇒ `dirt_uv = uv·0 + 0,5` é a constante `(0,5; 0,5)` ⇒ **o quadro fica plano**.
///
/// ⭐ E a comparação é de **bytes crus**, sem descodificar `f16`: o que se pergunta é se as duas
/// metades do quadro diferem, e o controle sem máscara prova que a diferença é da sujidade.
#[test]
fn the_framing_reaches_the_shader_in_the_order_the_shader_reads_it() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[motion_fx] SEM ADAPTER -- este gate NAO correu");
        return;
    };
    const SIZE: (u32, u32) = (64, 64);
    // Uma máscara 4×4 cujo QUADRANTE superior-esquerdo (os texels que o sub-rect varre) vai de
    // preto a branco da esquerda para a direita. O resto é cinzento, e nunca é amostrado.
    let px: [[u8; 4]; 16] = {
        let mut p = [[128, 128, 128, 255]; 16];
        for y in 0..2 {
            p[y * 4] = [0, 0, 0, 255];
            p[y * 4 + 1] = [255, 255, 255, 255];
        }
        p
    };
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("dirt framing fixture"),
        size: wgpu::Extent3d {
            width: 4,
            height: 4,
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
        bytemuck::cast_slice(&px),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(16),
            rows_per_image: Some(4),
        },
        wgpu::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        },
    );
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

    let mut fx = MotionFx::new(&gpu, SIZE);
    let params = BloomParams {
        dirt_intensity: 4.0,
        ..BloomParams::default()
    };
    let seed = |fx: &MotionFx| {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("seed"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: fx.rt_view(),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 2.0,
                        g: 2.0,
                        b: 2.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        gpu.queue.submit([enc.finish()]);
    };
    let run = |fx: &mut MotionFx, dirt: Option<crate::DirtMask<'_>>| {
        let target = crate::GameRt::new(&gpu, SIZE);
        seed(fx);
        fx.bloom_over(&gpu, target.view(), &params, None, dirt);
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
        read_rt(&gpu, &target, SIZE)
    };
    // Uma linha do meio, partida em duas metades — 8 bytes por pixel (`Rgba16Float`).
    let halves = |buf: &[u8]| -> (Vec<u8>, Vec<u8>) {
        let row = (SIZE.1 as usize / 2) * SIZE.0 as usize * 8;
        let half = SIZE.0 as usize / 2 * 8;
        (
            buf[row..row + half].to_vec(),
            buf[row + half..row + half * 2].to_vec(),
        )
    };
    let with_mask = run(
        &mut fx,
        Some(crate::DirtMask {
            view: &view,
            key: 7,
            // O quadrante superior-esquerdo — `so = [0,5 · 1, 0,5 · 1, 0, 0]`.
            uv_rect: [0.0, 0.0, 0.5, 0.5],
            aspect: 1.0,
        }),
    );
    let (l, r) = halves(&with_mask);
    assert_ne!(
        l, r,
        "o quadro saiu PLANO com uma mascara assimetrica — o enquadramento nao chegou ao \
         shader na ordem que ele le' (`P.v3.xy` = escala, `P.v3.zw` = deslocamento)"
    );
    // ⚠️ **CONTROLE — sem ele, um quadro que fosse assimétrico por outro motivo (o halo, o
    // vinhetado, um artefacto de mip) passaria a asserção acima sem a sujidade ter feito nada.**
    let no_mask = run(&mut fx, None);
    let (l0, r0) = halves(&no_mask);
    assert_eq!(
        l0, r0,
        "controle: sem mascara as duas metades TEM de ser iguais, senao este gate mede o halo"
    );
}
