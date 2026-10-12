//! **A LUT do halo no dispositivo** — o endereço do sampler contra a lei do bake (o irmão
//! `motion_fx_tests.rs` passou do tecto de LOC).

use super::*;

/// **A LUT DO HALO É LIDA ONDE FOI ASSADA** — a amostra `k` está em `t = k/(n−1)` (a lei do
/// `ph2d_color::ramp_lut::bake`); o sampler endereçado por `u = t` lia meia célula ao lado.
/// Fixtura: halo cinzento uniforme (saturação `0`, limiar `0`) e uma tabela cujo VERMELHO é o
/// próprio `t` e o VERDE é `1` ⇒ no destino preto `verde = lum` e `vermelho/verde = lut(lum)`, que
/// tem de ser `lum`. Antes da cura a razão era `(lum·n − ½)/(n−1)`: `−1,2 %` a `lum = 0,01`.
#[test]
fn the_halo_lut_is_read_at_the_baked_positions() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[motion_fx] SEM ADAPTER -- este gate NAO correu");
        return;
    };
    const SIZE: (u32, u32) = (64, 64);
    let mut fx = MotionFx::new(&gpu, SIZE);
    let limpa = |view: &wgpu::TextureView, v: f64| {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("limpa"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: v,
                        g: v,
                        b: v,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        gpu.queue.submit([enc.finish()]);
    };
    #[expect(clippy::cast_precision_loss, reason = "HALO_LUT_TEXELS <= 4096")]
    let lut: Vec<[f32; 4]> = (0..HALO_LUT_TEXELS)
        .map(|k| [k as f32 / (HALO_LUT_TEXELS - 1) as f32, 1.0, 0.0, 1.0])
        .collect();
    let params = BloomParams {
        threshold: 0.0,
        knee: 0.0,
        saturation: 0.0,
        intensity: 1.0,
        ..BloomParams::default()
    };
    for semente in [0.001f64, 0.003] {
        let alvo = crate::GameRt::new(&gpu, SIZE);
        limpa(alvo.view(), 0.0);
        limpa(fx.rt_view(), semente);
        fx.bloom_over(&gpu, alvo.view(), &params, Some(&lut), None);
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
        let px = read_rt(&gpu, &alvo, SIZE);
        let i = ((SIZE.1 / 2 * SIZE.0 + SIZE.0 / 2) * 8) as usize;
        let h = |o: usize| ph2d_color::half_to_f32(u16::from_le_bytes([px[i + o], px[i + o + 1]]));
        let (r, g) = (h(0), h(2));
        let lum = g / params.tint[3];
        eprintln!(
            "[lut] semente {semente}: lum {lum:.6}, lut(lum) {:.6}",
            r / g
        );
        assert!(
            (0.001..0.2).contains(&lum),
            "fixtura: a luminancia do halo ({lum}) tem de cair no pe' da rampa"
        );
        let erro = (r / g - lum).abs() / lum;
        assert!(
            erro < 4e-3,
            "a LUT foi lida a {:.6} em vez de {lum:.6} ({:.2} %): o endereco nao e' o do bake",
            r / g,
            erro * 100.0
        );
    }
}
