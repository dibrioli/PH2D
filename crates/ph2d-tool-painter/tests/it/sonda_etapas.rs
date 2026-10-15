//! **Sonda (ignorada, W5a das macros): editar uma ETAPA gravada do Wet Paint** — a pincelada `k` com
//! outra cor, outro tamanho, outro caminho, ou mais tarde (uma espera antes dela) —, com a água no
//! WORKER (a fita, como o produto grava) e no relógio FIXO (o Splash da cena `=1`), tocada pela fita ou
//! passando ao relógio fixo na etapa editada; e o custo de tocar do zero até ao fim de cada pincelada
//! (não há instantâneos). Medição, nunca barra. Plano: `docs/Macros/02_plano_W5.md`.
//!
//! Corre: `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it --
//! --ignored --nocapture --test-threads=1 sonda_etapas`.

use super::macro_comum::{grava_no_worker, texels_diferentes};
use ph2d_editor_core::tool::{PointerPhase, RasterEditTool};
use ph2d_macro::{Event, MacroDoc, MacroTime};
use ph2d_tool_painter::macro_track::Passo;
use ph2d_tool_painter::macro_track::roteiro::grava_o_splash;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;
use std::time::Instant;

const QUADRO_US: u64 = 16_666;
/// A espera acrescentada antes da pincelada editada: meio segundo.
const ESPERA_QUADROS: u64 = 30;

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
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

/// **W5c: o preço dos instantâneos** — tirar um no início de cada etapa (a reprodução da cena paga-o
/// nesse quadro) e reeditar a pincelada `k` a partir dele contra um palco novo do zero. Mínimo de
/// [`RONDAS_W5C`] rondas intercaladas.
const RONDAS_W5C: usize = 5;

#[test]
#[ignore = "sonda de medição (W5c): imprime, não reprova"]
fn sonda_instantaneos() {
    use ph2d_core::razao::Razao;
    use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
    use ph2d_tool_painter::macro_track::etapas;
    let docs = [
        ("splash 1024²", splash(1024)),
        ("splash 2048²", splash(2048)),
    ];
    eprintln!("[w5c] loadavg {}", carga().trim());
    let mut tirar = vec![f64::MAX; docs.len()];
    let mut reedita = vec![f64::MAX; docs.len()];
    let mut do_zero = vec![f64::MAX; docs.len()];
    let mut com = vec![f64::MAX; docs.len()];
    let mut sem = vec![f64::MAX; docs.len()];
    for r in 0..RONDAS_W5C {
        for j in 0..docs.len() {
            let i = (j + r) % docs.len();
            let doc = &docs[i].1;
            let bytes = ph2d_macro::codec::write(doc);
            let k = etapas::etapas(&doc.tracks[0]).expect("lê-se").len() - 1;
            let mut editada = doc.clone();
            // LITERAL-COLOR-OK: a cor nova de uma pincelada GRAVADA, não da UI.
            etapas::edita_cor(&mut editada.tracks[0], k, [20, 200, 40]).expect("pincelada");
            let novos = ph2d_macro::codec::write(&editada);
            // Tocar até ao fim com e sem instantâneos (o razão a zero não guarda nenhum).
            let a = Instant::now();
            let mut m = MacroEmCena::no_razao(&bytes, Razao::new(usize::MAX))
                .unwrap()
                .unwrap();
            m.segue(60.0).unwrap();
            com[i] = com[i].min(a.elapsed().as_secs_f64() * 1e3);
            let a = Instant::now();
            let mut z = MacroEmCena::no_razao(&bytes, Razao::new(0))
                .unwrap()
                .unwrap();
            z.segue(60.0).unwrap();
            sem[i] = sem[i].min(a.elapsed().as_secs_f64() * 1e3);
            // Um instantâneo, sozinho, no início da última pincelada (a água da anterior ainda molhada).
            let inicio = pinceladas(doc)[k].0;
            let tr = &doc.tracks[0];
            let mut t =
                TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
                    .unwrap();
            for e in &tr.events[..inicio] {
                t.aplica(&e.payload).unwrap();
            }
            let a = Instant::now();
            let f = t.instantaneo().unwrap();
            tirar[i] = tirar[i].min(a.elapsed().as_secs_f64() * 1e3);
            drop(f);
            // Editar a última pincelada: do instantâneo dela contra um palco novo.
            let a = Instant::now();
            m.reedita(&novos).unwrap();
            m.segue(60.0).unwrap();
            reedita[i] = reedita[i].min(a.elapsed().as_secs_f64() * 1e3);
            let a = Instant::now();
            let mut n = MacroEmCena::no_razao(&novos, Razao::new(0))
                .unwrap()
                .unwrap();
            n.segue(60.0).unwrap();
            do_zero[i] = do_zero[i].min(a.elapsed().as_secs_f64() * 1e3);
        }
    }
    eprintln!("[w5c] loadavg {}", carga().trim());
    for (i, (nome, doc)) in docs.iter().enumerate() {
        eprintln!(
            "[w5c] {nome}: macro {:.2} s · um instantâneo {:.1} ms · tocar ao fim com instantâneos {:.0} ms, \
             sem {:.0} ms · editar a última pincelada: do instantâneo {:.0} ms, do zero {:.0} ms",
            doc.duration().0 as f64 / 1e6,
            tirar[i],
            com[i],
            sem[i],
            reedita[i],
            do_zero[i]
        );
    }
}
