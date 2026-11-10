//! **Z1 — cópia de plano a 4K: série × `collect` do rayon × cópia em blocos**, no mesmo processo,
//! intercaladas em rodadas rodadas, o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`). Resultado e tabela:
//! `docs/Painter/missao_4k/medicoes/Z1_copias_de_plano.md`.
//!
//! «A frio» = cada amostra SEGURA o destino (páginas novas, como o fork do pen-down com três donos:
//! `tool` · `cursor` · a entrada do traço — o `make_mut` copia para memória recém-pedida).
//! «Reciclado» = o destino da rodada anterior foi libertado antes (o alocador devolve páginas já
//! mapeadas: o 2.º pen-down depois de o histórico largar uma entrada).
//!
//! Rodar: `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib z1_ -- --ignored --nocapture`
use super::*;
use std::time::Instant;

fn serie<T: Copy>(s: &[T]) -> Vec<T> {
    s.to_vec()
}

fn coleta<T: Copy + Send + Sync>(s: &[T]) -> Vec<T> {
    s.par_iter().copied().collect()
}

fn blocos<T: Copy + Default + Send + Sync>(s: &[T], bytes: usize) -> Vec<T> {
    let c = (bytes / size_of::<T>()).max(1);
    let mut d = vec![T::default(); s.len()];
    d.par_chunks_mut(c)
        .zip(s.par_chunks(c))
        .for_each(|(d, s)| d.copy_from_slice(s));
    d
}

const NOMES: [&str; 4] = ["serie", "coleta", "blocos256K", "blocos1M"];

fn uma<T: Copy + Default + Send + Sync>(v: usize, s: &[T]) -> Vec<T> {
    match v {
        0 => serie(s),
        1 => coleta(s),
        2 => blocos(s, 256 << 10),
        _ => blocos(s, 1 << 20),
    }
}

fn mede<T: Copy + Default + Send + Sync + PartialEq + std::fmt::Debug>(
    rotulo: &str,
    n: usize,
    gera: impl Fn(usize) -> T,
) {
    const RODADAS: usize = 15;
    let src: Vec<T> = (0..n).map(&gera).collect();
    let mut frio = [f64::MAX; 4];
    let mut reciclado = [f64::MAX; 4];
    let mut guardadas: Vec<Vec<T>> = Vec::new();
    let mut anterior: [Option<Vec<T>>; 4] = [None, None, None, None];
    for r in 0..RODADAS {
        for k in 0..4 {
            let v = (r + k) % 4;
            let t = Instant::now();
            let c = uma(v, &src);
            frio[v] = frio[v].min(t.elapsed().as_secs_f64() * 1e3);
            assert!(c == src, "{rotulo}: {} não copiou", NOMES[v]);
            guardadas.push(c);
            drop(anterior[v].take());
            let t = Instant::now();
            let c = uma(v, &src);
            reciclado[v] = reciclado[v].min(t.elapsed().as_secs_f64() * 1e3);
            anterior[v] = Some(c);
        }
    }
    let mb = (n * size_of::<T>()) as f64 / 1e6;
    let linha = |a: &[f64; 4]| {
        a.iter()
            .map(|x| format!("{x:>8.2}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    println!(
        "{rotulo:<26} {mb:>7.2} MB | frio {} | reciclado {}",
        linha(&frio),
        linha(&reciclado)
    );
}

#[test]
#[ignore = "sonda Z1 de relógio: corre à mão, --profile smoke"]
fn z1_copias_de_plano_a_4k() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\nZ1 load {} · fios rayon {} · colunas: {}",
        carga.trim(),
        rayon::current_num_threads(),
        NOMES.join(" ")
    );
    let u = |i: usize| (i % 251) as u8;
    let m = |i: usize| [(i % 251) as u8, 1, 2, 3, 4, 5, (i % 13) as u8];
    let f = |i: usize| (i % 1009) as f32 * 0.5;
    mede("u8 3840x2160 (cobertura)", 3840 * 2160, u);
    mede("rgba 1920x1080", 1920 * 1080 * 4, u);
    mede("rgba 2048^2", 2048 * 2048 * 4, u);
    mede("24 MiB u8", 24 << 20, u);
    mede("[u8;7] 2048^2 (material)", 2048 * 2048, m);
    mede("rgba 3840x2160 (tela)", 3840 * 2160 * 4, u);
    mede("f32 3840x2160 (altura)", 3840 * 2160, f);
    mede("[u8;7] 3840x2160 (vidro)", 3840 * 2160, m);
    mede("rgba 4096^2", 4096 * 4096 * 4, u);
}
