//! **Quanto um instantâneo do tocador de uma macro RETÉM** (W5c, `docs/Macros/02_plano_W5.md` §3) — o
//! número que ele declara ao razão da RAM tem de ser este (gate: o retido fica entre `1,00×` e
//! [`FOLGA`] do declarado; medido a 14/10: `1,05–1,09×`, o resto é o rasto do pincel e os rascunhos). Toca a trilha duas vezes no mesmo processo,
//! sem e com um instantâneo no início de cada etapa: a diferença do que fica vivo no fim, por instantâneo,
//! é o que cada um retém (a cópia profunda da água e o que a escrita seguinte do tocador desdobrou da
//! tela que partilhavam).
//!
//! Um `#[test]` por binário: os contadores do dhat são do processo inteiro.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test medida_instantaneo_memoria -- --nocapture
//! ```

#[allow(dead_code)]
#[path = "it/macro_comum.rs"]
mod macro_comum;

use ph2d_macro::{LeitorDaFamilia, MacroDoc};
use ph2d_tool_painter::macro_track::etapas::LeitorDoPainter;
use ph2d_tool_painter::macro_track::roteiro::grava_o_splash;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;

/// O retido pode passar o declarado até isto: o razão conta por baixo, nunca por cima.
const FOLGA: f64 = 1.15;

fn vivo() -> u64 {
    dhat::HeapStats::get().curr_bytes as u64
}

/// O retido ÷ o declarado.
fn mede(nome: &str, doc: &MacroDoc, lado: u32) -> f64 {
    let tr = &doc.tracks[0];
    let cargas = || tr.events.iter().map(|e| e.payload.as_slice());
    let inicios: Vec<usize> = LeitorDoPainter
        .etapas(&tr.events)
        .iter()
        .map(|e| e.inicio)
        .collect();

    let base = vivo();
    let mut toc = TocadorDoPainter::novo(&tr.initial, cargas()).expect("nasce");
    for e in &tr.events {
        toc.aplica(&e.payload).expect("toca");
    }
    let sem = vivo() - base;
    drop(toc);

    let base = vivo();
    let mut toc = TocadorDoPainter::novo(&tr.initial, cargas()).expect("nasce");
    let mut fotos = Vec::new();
    let mut na_hora = Vec::new();
    let mut declarado = 0usize;
    for (i, e) in tr.events.iter().enumerate() {
        if inicios.contains(&i) {
            let antes = vivo();
            fotos.push(toc.instantaneo().expect("o tocador copia-se"));
            declarado += toc.bytes_retidos();
            na_hora.push((vivo() - antes) as f64 / MB);
        }
        toc.aplica(&e.payload).expect("toca");
    }
    let com = vivo() - base;
    let k = fotos.len() as f64;
    let por_foto = (com - sem) as f64 / k;
    let px = f64::from(lado) * f64::from(lado);
    eprintln!(
        "[instantâneo] {nome}: {k} instantâneos · ao tirar (MB) {:?} · retido por instantâneo {:.2} MB = {:.1} B/px · declarado {:.2} MB (retido ÷ declarado {:.3}) · tocador sem eles {:.1} MB",
        na_hora
            .iter()
            .map(|m| (m * 100.0).round() / 100.0)
            .collect::<Vec<_>>(),
        por_foto / MB,
        por_foto / px,
        declarado as f64 / k / MB,
        por_foto * k / declarado as f64,
        sem as f64 / MB,
    );
    drop(fotos);
    drop(toc);
    por_foto * k / declarado as f64
}

#[test]
fn um_instantaneo_declara_o_que_retem() {
    let _profiler = dhat::Profiler::builder().testing().build();
    eprintln!(
        "[instantâneo] loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let (bytes, _) = grava_o_splash(1024, 0).expect("arma no Wet Paint");
    let doc = ph2d_macro::codec::read(&bytes).expect("lê-se");
    for (nome, doc, lado) in [
        ("splash 1024² (relógio fixo)", doc, 1024),
        ("worker 512² (fita)", macro_comum::grava_no_worker(512), 512),
    ] {
        let r = mede(nome, &doc, lado);
        assert!(
            (1.0..=FOLGA).contains(&r),
            "{nome}: retido ÷ declarado = {r:.3}"
        );
    }
}
