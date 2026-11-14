//! B43.2 (Bug #58): o premul `_in_linear` faz `powf(2.4)` por canal; pelo `det_powf` o byte é o do `libm` fixado,
//! não o da libm do sistema. A prova ao vivo corre este teste sob `scripts/prova-libm` (shim que erra 1 ulp em
//! TUDO, `LIBM_ADVERSARIA=1`): o hash não muda. O censo barato é `o_motor_nao_chama_a_libm_do_sistema`.

use ph2d_render::premul::premultiply_rgba8_in_linear;

/// FNV-1a dos 65 536 pares (canal, alfa) — todo byte que a função pode devolver.
fn hash_de_todos_os_pares() -> u64 {
    let mut px = Vec::with_capacity(256 * 256 * 4);
    for a in 0..=255_u8 {
        for c in 0..=255_u8 {
            px.extend_from_slice(&[c, c, c, a]);
        }
    }
    premultiply_rgba8_in_linear(&mut px);
    px.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[test]
fn o_premul_em_linear_e_o_mesmo_byte_em_toda_maquina() {
    assert_eq!(hash_de_todos_os_pares(), HASH, "o premul em linear mudou de byte");
}

const HASH: u64 = 4_612_054_009_264_338_934;
