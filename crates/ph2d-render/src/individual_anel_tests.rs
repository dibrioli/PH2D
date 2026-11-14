//! B20.6 — o anel da subida inteira dá a MESMA textura que `write_texture`, ao bit.

use super::PELO_ANEL;
use crate::IndividualTextureStore;
use ph2d_gpu::GpuContext;

fn bgl(gpu: &GpuContext) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("teste anel bgl"),
            entries: &crate::pipeline::entradas_do_material(),
        })
}

/// (texels lidos de volta, subidas pelo anel) depois de 3 `replace_pixels` seguidas na mesma ranhura.
fn sobe(gpu: &GpuContext, pelo_anel: bool, w: u32, h: u32) -> (Vec<u8>, u64) {
    PELO_ANEL.with(|c| c.set(Some(pelo_anel)));
    let bgl = bgl(gpu);
    let mut loja = IndividualTextureStore::new(gpu);
    let n = (w * h * 4) as usize;
    let id = loja
        .acquire(gpu, &bgl, w, h, &vec![0u8; n])
        .expect("ranhura");
    let mut ultimo = Vec::new();
    for k in 0..3u32 {
        let px: Vec<u8> = (0..n)
            .map(|i| ((i as u32 * 37 + k * 101) % 253) as u8)
            .collect();
        loja.replace_pixels(gpu, &bgl, id, w, h, &px).expect("sobe");
        ultimo = px;
    }
    let (_, _, lido) = loja.readback(gpu, id).expect("lê");
    PELO_ANEL.with(|c| c.set(None));
    assert_eq!(lido.len(), ultimo.len());
    assert!(
        lido == ultimo,
        "{w}×{h} pelo_anel={pelo_anel}: a textura ≠ os bytes subidos"
    );
    (lido, loja.subidas_pelo_anel)
}

#[test]
fn a_subida_pelo_anel_da_a_textura_do_write_texture_ao_bit() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem placa: nada a medir");
        return;
    };
    // 64 × 4 = 256 B por linha (sem alinhar) · 37 e 1001 (linha alinhada a 256 B, o laço por linha).
    for (w, h) in [(64, 8), (37, 19), (1001, 3)] {
        let (anel, n_anel) = sobe(&gpu, true, w, h);
        let (antes, n_antes) = sobe(&gpu, false, w, h);
        assert_eq!(n_anel, 3, "{w}×{h}: a subida não foi pelo anel");
        assert_eq!(n_antes, 0, "{w}×{h}: o controlo foi pelo anel");
        assert!(anel == antes, "{w}×{h}: anel ≠ write_texture");
    }
}
