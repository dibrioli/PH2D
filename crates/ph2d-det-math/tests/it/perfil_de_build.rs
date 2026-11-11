//! B43.5 (ADR-0174): o nível de processador entregue e a premissa de que nenhum alvo contrai `a*b + c`.

use std::hint::black_box;

/// `RUSTFLAGS` à mão SUBSTITUI os `rustflags` do `.cargo/config.toml`: o binário volta ao x86-64 de 2003 em silêncio.
#[test]
#[cfg(target_arch = "x86_64")]
fn o_binario_e_x86_64_v2_e_o_rustflags_nao_o_apagou() {
    assert!(
        cfg!(target_feature = "sse4.2") && cfg!(target_feature = "popcnt"),
        "sem SSE4.2/POPCNT: um RUSTFLAGS no ambiente apagou o `target-cpu=x86-64-v2` do .cargo/config.toml"
    );
}

/// O Rust nunca funde `a*b + c` (nem no `v3`, que tem FMA): só o `mul_add` explícito funde, e com o mesmo bit
/// em software (`v2`) ou em `vfmadd` (`v3`). Correr também com `RUSTFLAGS="-C target-cpu=x86-64-v3"`.
#[test]
fn a_vezes_b_mais_c_nao_se_contrai_em_nenhum_alvo() {
    let (mut separado, mut fundido) = (0_u64, 0_u64);
    let mut difere = 0;
    for i in 0..100_000_u32 {
        let a = 1.0_f32 + i as f32 * 3.1e-5;
        let b = 0.7_f32 - i as f32 * 1.3e-6;
        let c = -0.69_f32;
        let direto = a * b + c;
        let em_dois = black_box(a * b) + c;
        assert_eq!(direto.to_bits(), em_dois.to_bits(), "{a}*{b}+{c} contraiu");
        let f = a.mul_add(b, c);
        difere += usize::from(f.to_bits() != direto.to_bits());
        separado = separado.rotate_left(5) ^ u64::from(direto.to_bits());
        fundido = fundido.rotate_left(5) ^ u64::from(f.to_bits());
    }
    assert!(difere > 1000, "o mul_add só difere em {difere}");
    println!("hash separado {separado:#018x} · fundido {fundido:#018x} · difere {difere}");
}
