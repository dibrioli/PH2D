//! ⭐ **O UNIFORME DO PEDIDO, campo a campo** — a ordem da `struct Setup` do WGSL.
//!
//! ⚠️ **Ele saiu do [`crate::trace`] em 2026-09-15 por TETO DE LINHAS**, e a fronteira é real: aqui
//! não há decisão nenhuma, só a tradução de um pedido para bytes. *O sítio onde a ORDEM de uma
//! struct de shader é escrita à mão merece um ficheiro que se possa ler ao lado do WGSL.*
//!
//! ⛔⛔ **A ordem e o enchimento são o contrato**: um `vec3` alinha a `16 B`, e um `array<vec4, N>`
//! de uniforme tem tamanho fixo — escrever menos do que ele deixa a cauda com o lixo do que lá
//! estivesse.

use crate::trace::MarchSetup;

/// Os bytes do uniforme, na ordem que o `Setup` do [`crate::trace_wgsl::COMUM`] declara.
pub(super) fn uniforme_do_pedido(
    device: &wgpu::Device,
    setup: MarchSetup,
    width: u32,
    height: u32,
    // ⭐⭐⭐ O índice do cabeçalho da grade de longe no `k`, MAIS UM — `0` = sem grade. Ver
    // [`crate::longe`]. ⚠️ Ele vem de fora porque é o `marcha_com` que decide onde o cabeçalho
    // cai no `k`, e essa aritmética vive num sítio só.
    longe: u32,
) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;
    // O uniforme, campo a campo — a mesma ordem da `struct Setup`. ⚠️ Um `vec3` alinha a 16 B.
    let mut u: Vec<u8> = Vec::with_capacity(256);
    for v in [
        width,
        height,
        setup.budget,
        setup.ao_rays,
        setup.n_lamps,
        // ⭐ **Há chão?** — a altura vai no `f32` do grupo de baixo. Ver `MarchSetup::ground`.
        u32::from(setup.ground.is_some()),
        // ⭐ **Há borda mole?** — o raio dela vai no `vec3` do grupo de baixo. Ver `MarchSetup::mole`.
        u32::from(setup.mole.is_some()),
        longe,
    ] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    for f in [
        setup.half_extent,
        setup.half_px,
        setup.ortho_start,
        setup.eye_distance,
        setup.hit_eps,
        setup.normal_eps,
        setup.step,
        setup.t_max,
        setup.ball_radius,
        setup.ao_reach,
        setup.edge_cos,
        setup.ground.unwrap_or(0.0),
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    // ⚠️ O quarto `u32` de cada `vec3` é enchimento — menos o do `ball_center`, onde mora o
    // `ceu_passo` (um `u32` a seguir a um `vec3` ocupa os bytes que sobram dele).
    for (v, cauda) in [
        (setup.target, 0),
        (setup.right, 0),
        (setup.up, 0),
        (setup.fwd, 0),
        (setup.ball_center, setup.ceu_passo),
        // ⚠️ **Sem borda mole ele vai a ZERO e ninguém o lê** — o `s.mole` é que decide, e um raio
        // aqui sem a bandeira não acorda passagem nenhuma.
        (setup.mole.unwrap_or([0.0; 3]), 0),
    ] {
        for f in v {
            u.extend_from_slice(&f.to_le_bytes());
        }
        u.extend_from_slice(&cauda.to_le_bytes());
    }
    // ⚠️ **O array vai INTEIRO**, e não só as válidas: um `array<vec4, 8>` de uniforme tem tamanho
    // fixo, e escrever menos deixaria a cauda com o lixo do que lá estivesse.
    for v in setup.lamps {
        for f in v {
            u.extend_from_slice(&f.to_le_bytes());
        }
        u.extend_from_slice(&0f32.to_le_bytes());
    }

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("setup"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    })
}
