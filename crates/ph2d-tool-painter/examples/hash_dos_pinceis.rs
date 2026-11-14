//! **B43.3 — o hash de cada pincel de fábrica, por sistema** (fora do produto). Pinta a miniatura de cada preset
//! pelo motor de hoje (sem `cfg(test)`, como o app) e imprime `pincel-fnv <ficheiro> <fnv>` + um `pinceis-fnv <fnv>`
//! que os junta. O CI corre-o nos três SO e compara a última linha (`spike.yml`, `determinism-compare`).
//! ```text
//! cargo run -p ph2d-tool-painter --example hash_dos_pinceis --profile smoke
//! ```

fn main() {
    let fnv = |h: u64, v: &[u8]| {
        v.iter()
            .fold(h, |a, x| (a ^ u64::from(*x)).wrapping_mul(0x0100_0000_01b3))
    };
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let mut todos = 0xcbf2_9ce4_8422_2325u64;
    let mut n = 0usize;
    for e in ph2d_brush_presets::Biblioteca::carregar(&raiz, None).entradas {
        let m = ph2d_tool_painter::miniatura_do_preset(&e.preset);
        let h = fnv(0xcbf2_9ce4_8422_2325, &m.rgba);
        println!("pincel-fnv {} {h:#018x}", e.relativo);
        todos = fnv(todos, &h.to_le_bytes());
        n += 1;
    }
    assert!(n > 0, "nenhum pincel de fábrica em {}", raiz.display());
    println!("pinceis-fnv {n} {todos:#018x}");
}
