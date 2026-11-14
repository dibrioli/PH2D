//! A tela do «+» nasce sem tocar páginas — filho de `mutate.rs` (`#[path]`) pelo tecto de LOC.

use super::{PISO_DO_MMAP, TELA_NOVA_PELO_MMAP, tela_transparente};

/// As faltas de página menores deste fio × 4 KiB.
fn faltas_do_fio() -> u64 {
    let s = std::fs::read_to_string("/proc/thread-self/stat").unwrap_or_default();
    let resto = s.rsplit_once(") ").map_or("", |(_, r)| r);
    resto
        .split(' ')
        .nth(7)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        * 4096
}

/// **A tela do «+» a 4K não toca páginas e é só zeros** (n.º 7 item 3). O alocador é aquecido como no app: três
/// planos de 4K alocados e libertados sobem o limiar dinâmico acima do plano. Controlo, no mesmo fio: a
/// chave desligada dá os mesmos bytes; o tamanho abaixo do mínimo fica no `vec!` (capacidade exacta).
#[test]
fn a_tela_nova_de_4k_nasce_sem_tocar_paginas() {
    let n = 3840 * 2160 * 4;
    for _ in 0..3 {
        let v = std::hint::black_box(vec![7u8; n]);
        drop(v);
    }
    let f0 = faltas_do_fio();
    let t = tela_transparente(n);
    let tocado = faltas_do_fio() - f0;
    assert!(t.capacity() >= PISO_DO_MMAP, "capacidade {}", t.capacity());
    assert!(tocado < 1 << 20, "a tela nova tocou {tocado} bytes");
    assert_eq!(t.len(), n);
    TELA_NOVA_PELO_MMAP.set(false);
    let antes = tela_transparente(n);
    TELA_NOVA_PELO_MMAP.set(true);
    assert_eq!(antes.capacity(), n);
    assert!(t == antes, "os bytes da tela nova são os de antes");
    assert!(t.iter().all(|&b| b == 0));
    assert_eq!(tela_transparente(64 * 64 * 4).capacity(), 64 * 64 * 4);
}
