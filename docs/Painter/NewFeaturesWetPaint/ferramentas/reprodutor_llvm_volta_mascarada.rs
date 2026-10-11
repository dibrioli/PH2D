// Reprodutor mínimo do código mau do LLVM 21–22 (rustc 1.92–1.98) — HANDOFF_CONTINUACAO_…_COMPILADOR §2.
// Não é código do motor: fica aqui para o dono decidir se reporta (nada sai sem ordem dele).
//
//   rustc -C opt-level=2 -C overflow-checks=on reprodutor_llvm_volta_mascarada.rs -o /tmp/r && /tmp/r
//   → «7448 de 373464 diferem» (1.92 e 1.98; o mesmo com opt-level=3; com overflow-checks=off: 0)
//   rustc +nightly (1.100, LLVM 23.1.1) com as mesmas flags: 0.
//
// O laço vectorizado lê 8 texels CONTÍGUOS a partir de `ty + ((x − 1) & 511)`: a verificação em tempo
// de execução que devia recusar a volta da máscara testa o overflow em 32 bits de `ty + 511 + x`, sem a
// máscara; em x = 0 o índice é 511 e os 7 seguintes saem da linha seguinte do ladrilho.

const TS: usize = 512;

#[inline(never)]
fn kernel(y: usize, row: &mut [f32], tile: &[f32]) {
    let ty = ((y as i32 - 1) & 511) as usize * TS;
    for (x, cell) in row.iter_mut().enumerate() {
        *cell = tile[ty + ((x as i32 - 1) & 511) as usize];
    }
}

fn main() {
    let (s, rows) = (702usize, 532usize);
    let tile: Vec<f32> = (0..TS * TS).map(|i| i as f32).collect();
    let mut p = vec![-1.0f32; rows * s];
    for (y, row) in p.chunks_mut(s).enumerate() {
        kernel(std::hint::black_box(y), row, &tile);
    }
    let mut n = 0;
    for (i, v) in p.iter().enumerate() {
        let (x, y) = ((i % s) as i64, (i / s) as i64);
        let o = tile[((y - 1).rem_euclid(TS as i64) * TS as i64 + (x - 1).rem_euclid(TS as i64)) as usize];
        n += usize::from(o.to_bits() != v.to_bits());
    }
    println!("{n} de {} diferem", p.len());
    std::process::exit(i32::from(n != 0));
}
