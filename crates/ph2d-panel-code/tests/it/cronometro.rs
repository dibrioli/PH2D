//! ⏱ MEDIÇÃO (não é gate): o custo de `paint` com ~100 linhas visíveis × ~100 glifos, cache quente
//! e fria. `cargo test --profile smoke -p ph2d-panel-code --test it pintar_cem_linhas -- --ignored --nocapture`

use crate::{Deco, area_para, frame, pinta_em, texto, ts};
use ph2d_tokens::Theme;
use ph2d_code_buffer::Buffer;
use ph2d_panel_code::CodeView;
use ph2d_vector::VectorScene;
use std::time::Instant;

fn resumo(nome: &str, mut v: Vec<f64>) {
    v.sort_by(f64::total_cmp);
    let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
    println!("{nome:>6}: n={} min={:.0} µs  p50={:.0} µs  p99={:.0} µs", v.len(), v[0], q(0.5), q(0.99));
}

#[test]
#[ignore = "medição: corre-se à mão com --ignored"]
fn pintar_cem_linhas() {
    let buf = Buffer::new(&texto(7_500, 100));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 100);
    // Uma vista por variante: a cache de uma não é a da outra.
    let (mut vf, mut vs) = (CodeView::new(), CodeView::new());
    let d = Deco::default();
    let mut scene = VectorScene::new();
    // Três variantes intercaladas no MESMO processo: quente, fria, e quente SEM cor (um só trecho
    // por linha — separa o custo por corrida de glifos do custo por linha).
    let (mut quente, mut fria, mut sem_cor) = (Vec::new(), Vec::new(), Vec::new());
    let mut nada = |_: std::ops::Range<usize>| Vec::new();
    for i in 0..600 {
        scene.reset();
        let t = Instant::now();
        match i % 3 {
            0 => pinta_em(&mut v, &buf, &mut ts, area, &d, &mut scene),
            1 => {
                vf.clear_cache();
                pinta_em(&mut vf, &buf, &mut ts, area, &d, &mut scene);
            }
            _ => {
                let mut m = crate::marcas(&d);
                vs.paint(frame(&buf, &d, &mut nada, &mut m), area, &mut scene, &mut ts, &Theme::default());
            }
        }
        let us = t.elapsed().as_secs_f64() * 1e6;
        if i >= 30 {
            [&mut quente, &mut fria, &mut sem_cor][i % 3].push(us);
        }
    }
    assert_eq!(v.cache().len(), 100);
    let perfil = if cfg!(debug_assertions) { "debug" } else { "otimizado (smoke/release)" };
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("perfil: {perfil} · loadavg: {}", load.trim());
    resumo("quente", quente);
    resumo("fria", fria);
    resumo("sem cor", sem_cor);
    // O realce de brincar do teste, sozinho, sobre as mesmas linhas (o custo que é do TESTE).
    let fim = buf.text().line_end(99);
    let mut realce = Vec::new();
    for _ in 0..100 {
        let t = Instant::now();
        std::hint::black_box(crate::spans_de(&buf, 0..fim));
        realce.push(t.elapsed().as_secs_f64() * 1e6);
    }
    resumo("realce", realce);
}
