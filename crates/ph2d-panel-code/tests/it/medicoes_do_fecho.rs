//! ⏱ MEDIÇÃO (não é gate) dos tectos do fecho da Onda 1 — UMA rodada intercalada, variantes no MESMO
//! processo (`docs/DevOps/MEDIR_VELOCIDADE.md`), resumo = o MÍNIMO (e o p50 ao lado):
//! - **A** — o console aberto com todas as linhas à vista de `L` bytes (o `CONSOLE_LINE_BYTES_MAX`);
//! - **C** — 40 filas à vista, cada uma uma linha de 200 KB, com o tecto de pintura `K`
//!   (`LINE_RENDER_BYTES_MAX`), cache quente (um quadro parado) e fria (rolar/editar: molda tudo).
//!
//! `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-panel-code --test it medicoes_do_fecho -- --ignored --nocapture`

use crate::{Deco, frame, marcas, spans_de, ts};
use ph2d_code_buffer::Buffer;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{CodeView, ConsoleRow, ConsoleSource, ConsoleView, Tone};
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::time::Instant;

struct Linhas(Vec<String>);

impl ConsoleSource for Linhas {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn row(&self, i: usize) -> ConsoleRow<'_> {
        ConsoleRow { text: &self.0[i], tone: Tone::Plain, line: None }
    }
}

fn resumo(nome: &str, mut v: Vec<f64>) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    let (min, p50) = (v[0], v[v.len() / 2]);
    println!("{nome:>28}: n={:>3} min={min:>9.1} µs  p50={p50:>9.1} µs", v.len());
    (min, p50)
}

#[test]
#[ignore = "medição: corre-se à mão com --ignored --nocapture"]
fn medicoes_do_fecho() {
    let mut ts = ts();
    let theme = Theme::default();
    let mut scene = VectorScene::new();
    println!("loadavg início: {}", std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim());

    // A — o console: 1000×300 px aberto (as filas que cabem, todas com L bytes).
    let comps = [128usize, 1 << 10, 4 << 10, 16 << 10, 64 << 10];
    let area = Rect { x: 0.0, y: 0.0, w: 1000.0, h: 300.0 };
    let consoles: Vec<Linhas> = comps
        .iter()
        .map(|&l| Linhas(vec!["oi 123 x ".repeat(l / 9 + 1)[..l].to_owned(); 64]))
        .collect();
    let mut views: Vec<ConsoleView> = comps.iter().map(|_| ConsoleView::default()).collect();
    let mut a: Vec<Vec<f64>> = vec![Vec::new(); comps.len()];

    // C — 40 filas de 200 KB; uma vista por tecto (a cache de uma não é a da outra).
    let tectos = [2_000usize, 5_000, 10_000, 20_000];
    let linha = "local t={a=1,b=\"é\",c=3} ".repeat(200_000 / 24);
    let buf = Buffer::new(&vec![linha; 40].join("\n"));
    let d = Deco::default();
    let mut cv: Vec<CodeView> = tectos
        .iter()
        .map(|&k| {
            let mut v = CodeView::new();
            v.set_render_cap(k);
            v
        })
        .collect();
    let area_c = Rect { x: 0.0, y: 0.0, w: 1600.0, h: 900.0 };
    let (mut quente, mut fria): (Vec<Vec<f64>>, Vec<Vec<f64>>) =
        (vec![Vec::new(); tectos.len()], vec![Vec::new(); tectos.len()]);
    let pinta = |v: &mut CodeView, scene: &mut VectorScene, ts: &mut ph2d_text::TextSystem| {
        let mut f = |r: std::ops::Range<usize>| spans_de(&buf, r);
        let mut m = marcas(&d);
        v.paint(frame(&buf, &d, &mut f, &mut m), area_c, scene, ts, &theme);
    };
    for v in &mut cv {
        pinta(v, &mut scene, &mut ts); // aquece
    }

    const RONDAS: usize = 10;
    const BLOCO: usize = 5;
    for ronda in 0..RONDAS {
        for j in 0..comps.len() {
            let i = (j + ronda) % comps.len();
            for _ in 0..BLOCO {
                scene.reset();
                let t = Instant::now();
                views[i].paint(&consoles[i], true, 0, area, &mut scene, &mut ts, &theme);
                a[i].push(t.elapsed().as_secs_f64() * 1e6);
            }
        }
        for j in 0..tectos.len() {
            let i = (j + ronda) % tectos.len();
            for _ in 0..BLOCO {
                scene.reset();
                let t = Instant::now();
                pinta(&mut cv[i], &mut scene, &mut ts);
                quente[i].push(t.elapsed().as_secs_f64() * 1e6);
                cv[i].clear_cache();
                scene.reset();
                let t = Instant::now();
                pinta(&mut cv[i], &mut scene, &mut ts);
                fria[i].push(t.elapsed().as_secs_f64() * 1e6);
            }
        }
    }
    println!("A — console aberto, {} filas à vista:", views[0].row_rect(0).map_or(0, |_| 1));
    for (i, l) in comps.iter().enumerate() {
        resumo(&format!("linha de {l} B"), std::mem::take(&mut a[i]));
    }
    println!("C — 40 filas de 200 KB:");
    for (i, k) in tectos.iter().enumerate() {
        resumo(&format!("tecto {k} B quente"), std::mem::take(&mut quente[i]));
        resumo(&format!("tecto {k} B fria"), std::mem::take(&mut fria[i]));
    }
    println!("loadavg fim: {}", std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim());
}
