//! ⏱ **A RÉGUA da facilitação** (W7, `docs/MiroClone/05_facilitacao.md` §4), antes de qualquer tecto:
//!
//! - **agrupar** N notas (100 · 1 000 · 10 000) por cor, por etiqueta e por palavra — a operação
//!   inteira pelo editor (`Editor::cluster`: a ordem de leitura, as palavras e as raízes, a grelha, os
//!   frames, o lote aplicado);
//! - **desenhar** 10 000 notas sem nada, com o selo de voto em TODAS e com três carimbos em todas, a
//!   três zooms: no LIMIAR da leitura (a letra a 6 px — o pior caso: o máximo de notas à vista que
//!   ainda se desenham com as marcas), a zoom 1, e de longe (blocos: as marcas não se pintam — o
//!   controlo). ⛔ A vista «a meio» da régua do encode (letra a 5 px) é de BLOCOS: a 1.ª corrida desta
//!   régua mediu-a e as três variantes deram o mesmo número — não desenhava marca nenhuma.
//!
//! Variantes no MESMO processo, rodadas intercaladas com a ordem rodada, vale o MÍNIMO (mediana ao
//! lado). Corre à mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --test it measure_facilitate -- --ignored --nocapture
//! ```

use super::measure_encode_cost::{FRAMES, ROUNDS, notes, one_frame, routes_of};
use super::measure_gpu_raster_cost::{SIZE, min_med, wall_frame};
use ph2d_board_edit::{ClusterBy, ClusterNames, Editor, Metrics};
use ph2d_board_model::{BoardDoc, BoardOp, BoardSet, History, Rgba, Stamp, Style, Tag};
use ph2d_board_render::RenderCache;
use ph2d_board_route::RouteCache;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::time::Instant;

const NS: [usize; 3] = [100, 1_000, 10_000];
/// As frases das notas: palavras que se repetem (a raiz partilhada) e palavras vazias.
const PHRASES: [&str; 8] = [
    "Faster builds for the team",
    "Builds break at night",
    "More pair programming",
    "Pairing on the hard bugs",
    "Team lunch on Fridays",
    "Coffee machine upstairs",
    "Slow reviews block the release",
    "Release notes are late",
];

fn metrics() -> Metrics {
    Metrics {
        handle: 8.0,
        rotate_offset: 24.0,
        snap: 8.0,
        drag: 3.0,
        click_size: [160.0, 100.0],
        paste_offset: 10.0,
        bind: 8.0,
        dot: 40.0,
        eraser: 10.0,
        frame_title: 20.0,
    }
}

/// `n` notas (cores das 16, uma de oito etiquetas, uma das frases com o número) e os ids delas.
fn board(n: usize) -> (BoardDoc, Vec<ph2d_board_model::ElementId>) {
    let mut set = notes(n, true);
    let doc = &mut set.get_mut(set.boards()[0].id).unwrap().doc;
    let ids: Vec<_> = doc.live().map(|e| e.id).collect();
    for (i, id) in ids.iter().enumerate() {
        let mut el = doc.get(*id).unwrap().clone();
        let s = el.shape_mut().unwrap();
        s.text = format!("{} {i}", PHRASES[i % PHRASES.len()]).into();
        el.tags_mut().push(Tag {
            name: format!("t{}", i % 8),
            color: Rgba([200, 0, 0, 255]),
        });
        BoardOp::Put(el).apply(doc);
    }
    (doc.clone(), ids)
}

/// Um editor com `ids` seleccionados.
fn editor(doc: &BoardDoc, ids: &[ph2d_board_model::ElementId]) -> Editor {
    let ink = Rgba([20, 20, 20, 255]);
    let mut ed = Editor::new(Style::new(None, Some(ink), ink), metrics());
    ed.select(doc, ids.iter().copied());
    ed
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_cluster_cost() {
    let names = ClusterNames {
        no_tag: "no tag".into(),
        uncategorized: "#uncategorized".into(),
    };
    let bys = [
        ("cor", ClusterBy::Color),
        ("etiqueta", ClusterBy::Tag),
        ("palavra", ClusterBy::Keyword),
    ];
    let boards: Vec<(BoardDoc, Vec<ph2d_board_model::ElementId>)> =
        NS.iter().map(|n| board(*n)).collect();
    let n = NS.len() * bys.len();
    let mut ms = vec![vec![]; n];
    for round in 0..ROUNDS {
        for k in 0..n {
            let i = (k + round) % n;
            let (b, v) = (i / bys.len(), i % bys.len());
            let mut doc = boards[b].0.clone();
            let mut ed = editor(&doc, &boards[b].1);
            let mut h = History::default();
            let t = Instant::now();
            assert!(ed.cluster(&mut doc, &mut h, bys[v].1, &names));
            ms[i].push(t.elapsed().as_secs_f64() * 1e3);
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", load.trim());
    println!("| N notas | por | mín ms | mediana |");
    for (i, v) in ms.iter().enumerate() {
        let (m, d) = min_med(v);
        println!(
            "| {} | {} | {m:.2} | {d:.2} |",
            NS[i / bys.len()],
            bys[i % bys.len()].0
        );
    }
}

/// O quadro de 10 000 notas à zoom `zoom` (`None` = de longe, o quadro inteiro) com `marks`: 0 = nada,
/// 1 = o selo em todas, 2 = três carimbos em todas.
fn marked(zoom: Option<f64>, marks: u8) -> BoardSet {
    let mut set = notes(10_000, false);
    let id = set.boards()[0].id;
    if let Some(z) = zoom {
        set.get_mut(id).unwrap().camera.zoom = z;
    }
    let doc = &mut set.get_mut(id).unwrap().doc;
    let ids: Vec<_> = doc.live().map(|e| e.id).collect();
    for (i, id) in ids.into_iter().enumerate() {
        let mut el = doc.get(id).unwrap().clone();
        match marks {
            1 => el.set_votes(1 + (i % 9) as u32),
            2 => {
                for s in [Stamp::Heart, Stamp::Star, Stamp::ThumbsUp] {
                    el.add_stamp(s, 1 + (i % 3) as i64);
                }
            }
            _ => {}
        }
        BoardOp::Put(el).apply(doc);
    }
    set
}

#[test]
#[ignore = "régua: corre-se à mão em --release com a placa (ver o cabeçalho)"]
fn measure_marks_paint_cost() {
    // A letra de nascença (20) a 6 px de ecrã: o limiar da leitura.
    let read = Some(6.0 / ph2d_board_model::DEFAULT_FONT_SIZE);
    let variants: Vec<(&str, BoardSet)> = vec![
        ("letra a 6 px · nada", marked(read, 0)),
        ("letra a 6 px · selos", marked(read, 1)),
        ("letra a 6 px · 3 carimbos", marked(read, 2)),
        ("zoom 1 · nada", marked(Some(1.0), 0)),
        ("zoom 1 · selos", marked(Some(1.0), 1)),
        ("zoom 1 · 3 carimbos", marked(Some(1.0), 2)),
        ("de longe · nada", marked(None, 0)),
        ("de longe · selos", marked(None, 1)),
    ];
    let routes: Vec<RouteCache> = variants.iter().map(|(_, b)| routes_of(b)).collect();
    let gpu = GpuContext::new(GpuContext::default_instance(), None).ok();
    let mut pass = gpu
        .as_ref()
        .map(|g| VelloPass::new(g, wgpu::TextureFormat::Rgba8Unorm, SIZE).expect("VelloPass"));
    let mut ts = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    let mut caches: Vec<RenderCache> = variants
        .iter()
        .zip(&routes)
        .map(|((_, b), r)| {
            let mut c = RenderCache::default();
            one_frame(b, &mut scene, &mut ts, &mut c, r);
            c
        })
        .collect();
    let n = variants.len();
    let (mut cpu, mut wall) = (vec![vec![]; n], vec![vec![]; n]);
    for round in 0..ROUNDS {
        for k in 0..n {
            let i = (k + round) % n;
            let t = Instant::now();
            for _ in 0..FRAMES {
                one_frame(
                    &variants[i].1,
                    &mut scene,
                    &mut ts,
                    &mut caches[i],
                    &routes[i],
                );
            }
            cpu[i].push(t.elapsed().as_secs_f64() * 1e3 / FRAMES as f64);
            if let (Some(g), Some(p)) = (gpu.as_ref(), pass.as_mut()) {
                let w: f64 = (0..FRAMES).map(|_| wall_frame(g, p, &scene)).sum();
                wall[i].push(w / FRAMES as f64);
            }
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", load.trim());
    println!("| 10 mil notas | CPU mín ms | CPU mediana | placa mín | mediana |");
    for (k, (name, _)) in variants.iter().enumerate() {
        let (cm, cd) = min_med(&cpu[k]);
        let (wm, wd) = if wall[k].is_empty() {
            (f64::NAN, f64::NAN)
        } else {
            min_med(&wall[k])
        };
        println!("| {name} | {cm:.3} | {cd:.3} | {wm:.3} | {wd:.3} |");
    }
}
