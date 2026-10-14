//! **Sonda (ignorada, W5a das macros): editar uma ETAPA gravada do Wet Paint** — a pincelada `k` com
//! outra cor, outro tamanho, outro caminho, ou mais tarde (uma espera antes dela) —, com a água no
//! WORKER (a fita, como o produto grava) e no relógio FIXO (o Splash da cena `=1`), tocada pela fita ou
//! passando ao relógio fixo na etapa editada; e o custo de tocar do zero até ao fim de cada pincelada
//! (não há instantâneos). Medição, nunca barra. Plano: `docs/Macros/02_plano_W5.md`.
//!
//! Corre: `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it --
//! --ignored --nocapture --test-threads=1 sonda_etapas`.

use super::macro_comum::{Gravador, cp, texels_diferentes};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool, Tool};
use ph2d_macro::{Event, MacroDoc, MacroTime};
use ph2d_tool_painter::macro_track::Passo;
use ph2d_tool_painter::macro_track::roteiro::grava_o_splash;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::time::{Duration, Instant};

const QUADRO: Duration = Duration::from_micros(16_666);
const QUADRO_US: u64 = 16_666;
/// A espera acrescentada antes da pincelada editada: meio segundo.
const ESPERA_QUADROS: u64 = 30;

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
}

/// Seis pinceladas com esperas REAIS e a água no worker — a gravação do produto.
fn grava_no_worker(n: u32) -> MacroDoc {
    const ROTEIRO: [([f32; 2], [f32; 2], [u8; 3]); 6] = [
        ([0.15, 0.4], [0.85, 0.4], [200, 50, 30]),
        ([0.2, 0.2], [0.8, 0.7], [30, 80, 200]),
        ([0.5, 0.15], [0.5, 0.85], [230, 200, 30]),
        ([0.15, 0.6], [0.85, 0.62], [40, 160, 70]),
        ([0.3, 0.8], [0.7, 0.3], [150, 40, 160]),
        ([0.25, 0.5], [0.75, 0.5], [20, 20, 20]),
    ];
    const ESPERAS_MS: [u64; 6] = [150, 300, 120, 250, 90, 200];
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(30.0 * n as f32 / 1024.0);
    let mut g = Gravador::arma(&mut t);
    let s = n as f32;
    let quadro = |t: &mut PainterTool, g: &mut Gravador, ev: &[_]| {
        let a = Instant::now();
        for &e in ev {
            t.on_canvas_pointer(e);
        }
        let _ = t.take_preview_arc();
        t.on_tick(1000.0 / 60.0);
        g.quadro(t);
        if let Some(r) = QUADRO.checked_sub(a.elapsed()) {
            std::thread::sleep(r);
        }
    };
    for (k, (de, ate, cor)) in ROTEIRO.iter().enumerate() {
        t.set_brush_color_srgb8(*cor);
        let p = |i: u32| {
            let f = i as f32 / 30.0;
            [
                s * (de[0] + (ate[0] - de[0]) * f),
                s * (de[1] + (ate[1] - de[1]) * f),
            ]
        };
        quadro(&mut t, &mut g, &[cp(p(0), PointerPhase::Down)]);
        for q in 0..10 {
            let ev: Vec<_> = (1..=3)
                .map(|m| cp(p(q * 3 + m), PointerPhase::Move))
                .collect();
            quadro(&mut t, &mut g, &ev);
        }
        quadro(&mut t, &mut g, &[cp(p(30), PointerPhase::Up)]);
        let fim = Instant::now() + Duration::from_millis(ESPERAS_MS[k]);
        while Instant::now() < fim {
            quadro(&mut t, &mut g, &[]);
        }
    }
    g.para(&mut t)
}

fn passo(e: &Event) -> Passo {
    Passo::descodifica(&e.payload).expect("carga lê-se")
}

/// `(Down, Up)` de cada pincelada, por índice de evento.
fn pinceladas(doc: &MacroDoc) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut down = None;
    for (i, e) in doc.tracks[0].events.iter().enumerate() {
        if let Passo::Ponteiro { fase, .. } = passo(e) {
            match fase {
                PointerPhase::Down => down = Some(i),
                PointerPhase::Up => {
                    if let Some(d) = down.take() {
                        out.push((d, i));
                    }
                }
                _ => {}
            }
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Edicao {
    Cor,
    Tamanho,
    Caminho,
    Espera,
}

/// A macro com a pincelada `k` editada, e o índice do 1.º evento que a edição tocou.
fn edita(doc: &MacroDoc, k: usize, ed: Edicao) -> (MacroDoc, usize) {
    let mut d = doc.clone();
    let (down, up) = pinceladas(doc)[k];
    let ev = &mut d.tracks[0].events;
    // O 1.º evento do quadro do toque: a cor e o tamanho escolhidos para esta pincelada estão nele.
    let inicio_do_quadro = (0..=down)
        .find(|&i| ev[i].t == ev[down].t)
        .expect("o toque está no seu quadro");
    match ed {
        Edicao::Cor => {
            let i = (0..down)
                .rev()
                .find(|&i| matches!(passo(&ev[i]), Passo::Cor(_)))
                .expect("a pincelada tem uma cor gravada antes dela");
            // LITERAL-COLOR-OK: a cor nova de uma pincelada GRAVADA, não da UI.
            ev[i].payload = Passo::Cor([20, 200, 40]).codifica();
            (d, i)
        }
        Edicao::Tamanho => {
            let t = ev[down].t;
            ev.insert(
                down,
                Event {
                    t,
                    payload: Passo::TamanhoPx(90.0).codifica(),
                },
            );
            (d, down)
        }
        Edicao::Caminho => {
            for e in &mut ev[down..=up] {
                if let Passo::Ponteiro {
                    pos,
                    pressao,
                    tilt,
                    fase,
                } = passo(e)
                {
                    e.payload = Passo::Ponteiro {
                        pos: [pos[0] + 0.08, pos[1]],
                        pressao,
                        tilt,
                        fase,
                    }
                    .codifica();
                }
            }
            (d, down)
        }
        Edicao::Espera => {
            let t0 = ev[inicio_do_quadro].t.0;
            let atraso = ESPERA_QUADROS * QUADRO_US;
            for e in &mut ev[inicio_do_quadro..] {
                e.t = MacroTime(e.t.0 + atraso);
            }
            // Os quadros da espera: a pré-visualização que a ponte drena e o tique, como os gravados.
            let mut novos = Vec::new();
            for q in 0..ESPERA_QUADROS {
                let t = MacroTime(t0 + q * QUADRO_US);
                novos.push(Event {
                    t,
                    payload: Passo::Preview.codifica(),
                });
                novos.push(Event {
                    t,
                    payload: Passo::Tique(1000.0 / 60.0).codifica(),
                });
            }
            ev.splice(inicio_do_quadro..inicio_do_quadro, novos);
            (d, inicio_do_quadro)
        }
    }
}

struct Toque {
    erro: Option<String>,
    tela: Vec<u8>,
    ms_ate_cada_up: Vec<f64>,
    ms: f64,
}

/// Toca a trilha 0 do zero; em `passa_em` a água passa ao relógio fixo (e larga a fita).
fn toca(doc: &MacroDoc, passa_em: Option<usize>) -> Toque {
    let ups: Vec<usize> = pinceladas(doc).iter().map(|p| p.1).collect();
    let tr = &doc.tracks[0];
    let mut toc =
        TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
            .expect("o tocador nasce");
    let a = Instant::now();
    let mut ms_ate_cada_up = Vec::new();
    let mut erro = None;
    for (i, e) in tr.events.iter().enumerate() {
        if Some(i) == passa_em {
            toc.passa_ao_relogio_fixo();
        }
        if let Err(x) = toc.aplica(&e.payload) {
            erro = Some(format!("{x:?} no evento {i}/{}", tr.events.len()));
            break;
        }
        if ups.contains(&i) {
            ms_ate_cada_up.push(a.elapsed().as_secs_f64() * 1e3);
        }
    }
    let ms = a.elapsed().as_secs_f64() * 1e3;
    let tela = toc.tool_mut().run_full().0;
    Toque {
        erro,
        tela,
        ms_ate_cada_up,
        ms,
    }
}

fn splash(lado: u32) -> MacroDoc {
    let (bytes, _) = grava_o_splash(lado, 0).expect("arma no Wet Paint");
    ph2d_macro::codec::read(&bytes).expect("lê-se")
}

#[test]
#[ignore = "sonda de medição (W5a): imprime, não reprova"]
fn sonda_etapas_editadas() {
    eprintln!("[etapas] loadavg {}", carga().trim());
    let k = 2;
    for (nome, doc) in [
        ("worker 512²", grava_no_worker(512)),
        ("splash 512² (relógio fixo)", splash(512)),
    ] {
        let original = toca(&doc, None);
        let resp = doc.tracks[0]
            .events
            .iter()
            .filter(|e| matches!(passo(e), Passo::Agua(_)))
            .count();
        eprintln!(
            "[etapas] {nome}: {} eventos, {resp} respostas do worker, original {:?}",
            doc.tracks[0].events.len(),
            original.erro
        );
        let mesmo_original = toca(&doc, None);
        eprintln!(
            "[etapas] {nome}: original tocado 2× difere em {} texels",
            texels_diferentes(&original.tela, &mesmo_original.tela)
        );
        for ed in [
            Edicao::Cor,
            Edicao::Tamanho,
            Edicao::Caminho,
            Edicao::Espera,
        ] {
            let (editado, primeiro) = edita(&doc, k, ed);
            for (via, passa) in [("fita", None), ("relógio fixo na etapa", Some(primeiro))] {
                let a = toca(&editado, passa);
                let b = toca(&editado, passa);
                eprintln!(
                    "[etapas] {nome} | pincelada {k} {ed:?} | {via}: erro {:?} · ≠ original {} texels · \
                     2 corridas ≠ {} texels",
                    a.erro,
                    texels_diferentes(&a.tela, &original.tela),
                    texels_diferentes(&a.tela, &b.tela)
                );
            }
        }
        // Controlo: passar ao relógio fixo SEM editar — quanto da tela muda só por largar a fita.
        let (_, primeiro) = edita(&doc, k, Edicao::Cor);
        let so_passa = toca(&doc, Some(primeiro));
        eprintln!(
            "[etapas] {nome} | controlo: sem edição, relógio fixo desde a pincelada {k}: erro {:?} · ≠ \
             original {} texels",
            so_passa.erro,
            texels_diferentes(&so_passa.tela, &original.tela)
        );
    }
}

#[test]
#[ignore = "sonda de medição (W5a): imprime, não reprova"]
fn sonda_custo_de_tocar_ate_cada_pincelada() {
    const RONDAS: usize = 5;
    let docs = [
        ("splash 1024² fixo", splash(1024), None),
        ("splash 2048² fixo", splash(2048), None),
        ("worker 1024² fita", grava_no_worker(1024), None),
        ("worker 1024² fixo desde 0", grava_no_worker(1024), Some(0)),
    ];
    eprintln!("[custo] loadavg {}", carga().trim());
    let mut minimos: Vec<Option<(f64, Vec<f64>)>> = vec![None; docs.len()];
    let mut medias: Vec<Vec<f64>> = vec![Vec::new(); docs.len()];
    for r in 0..RONDAS {
        for j in 0..docs.len() {
            let i = (j + r) % docs.len();
            let (_, doc, passa) = &docs[i];
            let t = toca(doc, *passa);
            assert!(t.erro.is_none(), "{:?}", t.erro);
            medias[i].push(t.ms);
            if minimos[i].as_ref().is_none_or(|(m, _)| t.ms < *m) {
                minimos[i] = Some((t.ms, t.ms_ate_cada_up));
            }
        }
    }
    eprintln!("[custo] loadavg {}", carga().trim());
    for (i, (nome, doc, _)) in docs.iter().enumerate() {
        let (min, ups) = minimos[i].clone().expect("rodou");
        let mut v = medias[i].clone();
        v.sort_by(f64::total_cmp);
        let d = doc.duration().0 as f64 / 1e6;
        eprintln!(
            "[custo] {nome}: macro {d:.2} s · do zero ao fim mín {min:.1} ms (mediana {:.1}) · {:.1} ms \
             por segundo de macro · até ao fim de cada pincelada (mín): {:?}",
            v[v.len() / 2],
            min / d,
            ups.iter().map(|x| x.round()).collect::<Vec<_>>()
        );
    }
}
