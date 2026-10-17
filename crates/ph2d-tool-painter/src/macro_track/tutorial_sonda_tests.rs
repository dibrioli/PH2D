//! **A medição da fracção** (W6b, `docs/Macros/03_plano_W6.md` §6.1): o Splash gravado, e um aluno que
//! faz as pinceladas anteriores certas e a `k` de cada maneira — certa, deslocada, fina, pela metade,
//! noutro lugar, nenhuma, «Mostra-me». Imprime a fracção pintada da parte NOVA e da região inteira.
//!
//! `cargo test -p ph2d-tool-painter --profile smoke --lib macro_track::tutorial::sonda -- --ignored --nocapture`
//!
//! As da W6c (§6.3) medem a fracção SOBRE A DA GRAVADA: as variantes que decidem a fracção nos quatro
//! meios, a fatia fina (30 pinceladas), e o custo de medir as referências.

use super::*;
use crate::macro_track::roteiro::{
    CAUDA, DT_MS, ENTRE, Pincelada, SPLASH, TRACO, grava_nos_meios, grava_pinceladas, ponto,
};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};

#[path = "tutorial_sonda_w7_tests.rs"]
mod w7;
#[path = "tutorial_sonda_w8_tests.rs"]
mod w8;

pub(super) fn quadro(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(DT_MS);
}

/// O aluno pinta `p` como o roteiro grava: ponteiro a cada quadro, depois o intervalo.
pub(super) fn pinta(t: &mut PainterTool, p: &Pincelada, lado: f32) {
    traca(t, p, lado);
    for _ in 0..ENTRE {
        quadro(t);
    }
}

/// O traço de `p` até ao quadro do `Up`, inclusive.
pub(super) fn traca(t: &mut PainterTool, p: &Pincelada, lado: f32) {
    t.set_brush_color_srgb8(p.cor);
    t.set_brush_size_px(p.tamanho_px);
    for i in 0..=TRACO {
        #[expect(clippy::cast_precision_loss, reason = "poucos quadros")]
        let f = i as f32 / TRACO as f32;
        let phase = match i {
            0 => PointerPhase::Down,
            i if i == TRACO => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(CanvasPointer {
            pos: ponto(p, f, lado),
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        quadro(t);
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Variante {
    Exacta,
    Desloca(f32),
    Fina,
    Metade,
    Longe,
    Nada,
    MostraMe,
}

/// A pincelada `p` deslocada `d` raios na perpendicular do caminho.
fn deslocada(p: &Pincelada, d: f32, lado: f32) -> Pincelada {
    let (dx, dy) = (p.ate[0] - p.de[0], p.ate[1] - p.de[1]);
    let n = (dx * dx + dy * dy).sqrt().max(1e-6);
    let s = d * p.tamanho_px / lado;
    let (ox, oy) = (-dy / n * s, dx / n * s);
    Pincelada {
        de: [p.de[0] + ox, p.de[1] + oy],
        ate: [p.ate[0] + ox, p.ate[1] + oy],
        ..*p
    }
}

/// Quadros até a água do aluno `t` parar (o tecto da régua): o balão avalia a cada quadro e o
/// «pintado» só cresce, logo o que o aluno alcança é o que ele tem com a água ASSENTADA (W7).
pub(super) fn assenta(t: &mut PainterTool) -> u32 {
    let mut q = 0;
    while q < base::QUADROS_ATE_ASSENTAR && !t.wet_assentada() {
        quadro(t);
        q += 1;
    }
    q
}

/// A variante `v` da pincelada `p` (a `j` da gravação `track`) no aluno `t`, e a água a assentar.
fn faz(t: &mut PainterTool, track: &Track, j: usize, p: &Pincelada, v: Variante, l: f32) {
    faz_sem_assentar(t, track, j, p, v, l);
    assenta(t);
}

/// [`faz`] sem esperar a água.
pub(super) fn faz_sem_assentar(
    t: &mut PainterTool,
    track: &Track,
    j: usize,
    p: &Pincelada,
    v: Variante,
    l: f32,
) {
    match v {
        Variante::Exacta => pinta(t, p, l),
        Variante::Desloca(d) => pinta(t, &deslocada(p, d, l), l),
        Variante::Fina => pinta(
            t,
            &Pincelada {
                tamanho_px: p.tamanho_px * 0.5,
                ..*p
            },
            l,
        ),
        Variante::Metade => pinta(
            t,
            &Pincelada {
                ate: [(p.de[0] + p.ate[0]) * 0.5, (p.de[1] + p.ate[1]) * 0.5],
                curva: p.curva * 0.5,
                ..*p
            },
            l,
        ),
        Variante::Longe => pinta(
            t,
            &Pincelada {
                de: [0.90, 0.96],
                ate: [0.98, 0.90],
                curva: 0.0,
                ..*p
            },
            l,
        ),
        Variante::Nada => {}
        Variante::MostraMe => {
            let (w, h) = t.canvas_size();
            let k = regioes(track, w, h).expect("regiões")[j].k;
            mostra(t, track, k).expect("mostra");
            for _ in 0..ENTRE {
                quadro(t);
            }
        }
    }
}

/// O aluno que fez certas as pinceladas do Splash antes da `k` e a `k` na variante `v`.
pub(super) fn aluno(track: &Track, k: usize, v: Variante, lado: u32) -> PainterTool {
    #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
    let l = lado as f32;
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    for p in &SPLASH[..k] {
        pinta(&mut t, p, l);
    }
    faz(&mut t, track, k, &SPLASH[k], v, l);
    t
}

/// Cada pincelada `k` de `ps` (as gravadas em `track`) em cada variante de `vs`: o aluno com as
/// anteriores certas (pintadas uma vez; cada variante corre num instantâneo dele, em lotes de fios
/// paralelos — a água de cada um assenta sozinha). `f` corre pela ordem, neste fio.
pub(super) fn cada_aluno(
    track: &Track,
    ps: &[Pincelada],
    lado: u32,
    vs: &[Variante],
    mut f: impl FnMut(usize, Variante, &PainterTool),
) {
    #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
    let l = lado as f32;
    let lote = 2 * mede::fios();
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    let mut por_fazer: Vec<(usize, Variante, PainterTool)> = Vec::new();
    let mut corre = |por_fazer: &mut Vec<(usize, Variante, PainterTool)>| {
        std::thread::scope(|sc| {
            for (k, v, a) in por_fazer.iter_mut() {
                let p = &ps[*k];
                sc.spawn(move || faz(a, track, *k, p, *v, l));
            }
        });
        for (k, v, a) in por_fazer.drain(..) {
            f(k, v, &a);
        }
    };
    for (k, p) in ps.iter().enumerate() {
        for &v in vs {
            por_fazer.push((k, v, t.instantaneo().expect("instantâneo do aluno")));
            if por_fazer.len() >= lote {
                corre(&mut por_fazer);
            }
        }
        pinta(&mut t, p, l);
    }
    corre(&mut por_fazer);
}

/// O critério da pincelada `j` da gravação (`None` = sem critério).
pub(super) fn criterio_de<'a>(
    e: &'a EnsinoDoPainter,
    track: &Track,
    j: usize,
) -> Option<&'a CriterioDaPincelada> {
    let k = regioes(track, e.w, e.h).expect("regiões")[j].k;
    e.criterios.iter().find(|c| c.regiao.k == k)
}

/// A fracção de `idx` pintada com «pintado» = algum canal de alguma camada a mais de `limiar`.
fn fracao(e: &EnsinoDoPainter, t: &PainterTool, idx: &[u32], limiar: u8) -> f32 {
    let aluno = t.camadas_rgba();
    let n = idx
        .iter()
        .filter(|&&i| {
            let k = i as usize * 4;
            aluno.iter().any(|(id, px)| {
                e.inicial
                    .iter()
                    .find(|(j, _)| j == id)
                    .map_or(px[k + 3] > limiar, |(_, ini)| {
                        (0..4).any(|c| px[k + c].abs_diff(ini[k + c]) > limiar)
                    })
            })
        })
        .count();
    #[expect(clippy::cast_precision_loss, reason = "contagens")]
    let f = n as f32 / idx.len().max(1) as f32;
    f
}

#[test]
#[ignore = "sonda de medição (W6b): imprime a tabela da fracção"]
fn sonda_a_fracao_pintada_por_variante() {
    use Variante::*;
    for (lado, variantes) in [
        (
            512u32,
            &[
                Exacta,
                Desloca(0.5),
                Desloca(1.0),
                Desloca(2.0),
                Fina,
                Metade,
                Longe,
                Nada,
                MostraMe,
            ][..],
        ),
        (1024, &[Exacta, Desloca(1.0), Metade, Nada, MostraMe][..]),
    ] {
        let (b, _) = grava_pinceladas(lado, 0, &SPLASH, &[0; 4]).expect("grava");
        let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
        let e = EnsinoDoPainter::de(&track).expect("ensino");
        for (k, c) in e.criterios.iter().enumerate() {
            let toda = rasteriza(&c.regiao, lado, lado);
            println!(
                "lado {lado} · pincelada {} (raio {}): região {} px, nova {} px ({:.0} %)",
                k + 1,
                c.regiao.raio,
                toda.len(),
                c.novos.len(),
                100.0 * c.novos.len() as f32 / toda.len().max(1) as f32
            );
            for &v in variantes {
                let t = aluno(&track, k, v, lado);
                let t0 = std::time::Instant::now();
                let f = e.fracao_pintada(&t, c);
                let us = t0.elapsed().as_micros();
                println!(
                    "  {v:<14?} nova {:>5.1} % (>16: {:>5.1} %) · toda {:>5.1} % (>16: {:>5.1} %) · avalia {us} µs",
                    100.0 * f,
                    100.0 * fracao(&e, &t, &c.novos, 16),
                    100.0 * fracao(&e, &t, &toda, 0),
                    100.0 * fracao(&e, &t, &toda, 16),
                );
            }
        }
    }
}

/// As pinceladas `ps` gravadas no meio `m`.
pub(super) fn trilha_no_meio(m: PaintMedia, ps: &[Pincelada], lado: u32) -> Track {
    let (b, _) = grava_nos_meios(lado, ps, &[m]).expect("grava");
    ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0)
}

pub(super) const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// ⭐ **A fracção nos quatro meios, bruta e sobre a da gravada** (W6c, `03_plano_W6.md` §6.3): o
/// Splash a 512² nos quatro meios e a 1024² em Wet Paint.
#[test]
#[ignore = "sonda de medição (W6c): a fracção nos quatro meios, bruta e normalizada"]
fn sonda_a_fracao_por_meio() {
    use Variante::*;
    let vs = [Exacta, Desloca(0.5), Desloca(1.0), Fina, Nada, MostraMe];
    let casos = MEIOS
        .iter()
        .map(|&m| (m, 512))
        .chain([(PaintMedia::WetPaint, 1024)]);
    for (m, lado) in casos {
        let track = trilha_no_meio(m, &SPLASH, lado);
        let e = EnsinoDoPainter::medido(&track).expect("ensino");
        let refs: Vec<String> = e
            .criterios
            .iter()
            .map(|c| {
                let r = c.referencia.as_ref().expect("medida");
                format!(
                    "{:.1} (base {:.1})",
                    100.0 * r.gravada.fim(),
                    100.0 * r.base.fim()
                )
            })
            .collect();
        println!("{m:?} {lado}² · referência {} %", refs.join(" / "));
        let mut linhas: Vec<(String, Vec<String>, Vec<String>)> = vs
            .iter()
            .map(|v| (format!("{v:?}"), Vec::new(), Vec::new()))
            .collect();
        cada_aluno(&track, &SPLASH, lado, &vs, |k, v, t| {
            let i = vs
                .iter()
                .position(|x| format!("{x:?}") == format!("{v:?}"))
                .expect("variante");
            let (b, n) = criterio_de(&e, &track, k).map_or(("—".into(), "—".into()), |c| {
                (
                    format!("{:.1}", 100.0 * e.fracao_pintada(t, c)),
                    format!("{:.1}", 100.0 * e.razao(t, c).unwrap_or(f32::NAN)),
                )
            });
            linhas[i].1.push(b);
            linhas[i].2.push(n);
        });
        for (v, b, n) in linhas {
            println!(
                "  {v:<14} bruta {} % · normalizada {} %",
                b.join(" / "),
                n.join(" / ")
            );
        }
    }
}

/// 30 pinceladas distintas: o Splash deslocado 0,025 da tela a cada volta. Muitas têm por parte nova
/// uma FATIA fina na borda (a orla do raio, onde o pincel pinta menos).
pub(super) fn trinta() -> Vec<Pincelada> {
    (0..30)
        .map(|j: u16| {
            let p = SPLASH[usize::from(j % 4)];
            let d = 0.025 * f32::from(j / 4) - 0.08;
            Pincelada {
                de: [p.de[0] + d, p.de[1]],
                ate: [p.ate[0] + d, p.ate[1]],
                ..p
            }
        })
        .collect()
}

/// ⭐ **A fatia fina** (W6c): as 30 pinceladas nos quatro meios a 512² — a referência e a razão de
/// «Exacta», «Nada» e «Mostra-me» nas pinceladas cuja parte nova é < 10 % da região, e o mínimo do
/// «Mostra-me» e o máximo do «Nada» nas 30.
#[test]
#[ignore = "sonda de medição (W6c): a fatia fina, nos quatro meios"]
fn sonda_a_fatia_fina() {
    use Variante::*;
    let lado = 512;
    let ps = trinta();
    for m in MEIOS {
        let track = trilha_no_meio(m, &ps, lado);
        let e = EnsinoDoPainter::medido(&track).expect("ensino");
        let fina =
            |c: &CriterioDaPincelada| c.novos.len() * 10 < rasteriza(&c.regiao, lado, lado).len();
        let (mut finas, mut rmin, mut rmax) = (0, f32::MAX, 0.0f32);
        for c in e.criterios.iter().filter(|c| fina(c)) {
            finas += 1;
            let r = c.referencia.as_ref().expect("medida").gravada.fim();
            (rmin, rmax) = (rmin.min(r), rmax.max(r));
        }
        let (mut dmin, mut dmax, mut bmax) = (f32::MAX, 0.0f32, 0.0f32);
        for c in &e.criterios {
            let r = c.referencia.as_ref().expect("medida");
            (dmin, dmax, bmax) = (
                dmin.min(r.delta()),
                dmax.max(r.delta()),
                bmax.max(r.base.fim()),
            );
        }
        let mut deltas: Vec<f32> = e
            .criterios
            .iter()
            .map(|c| c.referencia.as_ref().expect("medida").delta())
            .collect();
        deltas.sort_by(f32::total_cmp);
        let menores: Vec<String> = deltas
            .iter()
            .take(5)
            .map(|d| format!("{:.1}", 100.0 * d))
            .collect();
        println!(
            "{m:?}: gravada − base {:.1}–{:.1} % (as 5 menores {}), base até {:.1} %",
            100.0 * dmin,
            100.0 * dmax,
            menores.join(" / "),
            100.0 * bmax
        );
        // (mín, máx) da razão por variante: nas finas e nas 30.
        let mut nas_finas = [(f32::MAX, 0.0f32); 3];
        let mut em_todas = [(f32::MAX, 0.0f32); 3];
        let vs = [Exacta, Nada, MostraMe];
        let mut nada_cumpre = Vec::new();
        cada_aluno(&track, &ps, lado, &vs, |k, v, t| {
            let Some(c) = criterio_de(&e, &track, k) else {
                return;
            };
            let i = match v {
                Exacta => 0,
                Nada => 1,
                _ => 2,
            };
            let r = e.razao(t, c).expect("medida");
            if matches!(v, Nada) && r >= FRACAO_DA_GRAVADA_QUE_CUMPRE {
                let rf = c.referencia.as_ref().expect("medida");
                nada_cumpre.push(format!(
                    "{k} ({} de {} px, ref {:.1} %, base {:.1} %, nada {:.1} %)",
                    c.novos.len(),
                    rasteriza(&c.regiao, lado, lado).len(),
                    100.0 * rf.gravada.fim(),
                    100.0 * rf.base.fim(),
                    100.0 * r
                ));
            }
            let mm = |x: &mut (f32, f32)| *x = (x.0.min(r), x.1.max(r));
            mm(&mut em_todas[i]);
            if fina(c) {
                mm(&mut nas_finas[i]);
            }
        });
        println!(
            "{m:?}: {} critérios, {finas} finos (referência {:.1}–{:.1} %)",
            e.criterios.len(),
            100.0 * rmin,
            100.0 * rmax
        );
        println!(
            "  o «Nada» cumpre {} de {}: {}",
            nada_cumpre.len(),
            e.criterios.len(),
            nada_cumpre.join(", ")
        );
        for (i, v) in vs.iter().enumerate() {
            println!(
                "  {:<10} finas {:.1}–{:.1} % · todas {:.1}–{:.1} %",
                format!("{v:?}"),
                100.0 * nas_finas[i].0,
                100.0 * nas_finas[i].1,
                100.0 * em_todas[i].0,
                100.0 * em_todas[i].1
            );
        }
    }
}

pub(super) fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
}

/// O tempo de [`EnsinoDoPainter::mede_com`] com `n` fios: (até à 1.ª medida, total), em ms.
fn tempo_de_medir(t: &Track, n: usize) -> (u128, u128) {
    let e = EnsinoDoPainter::de(t).expect("de");
    let t0 = std::time::Instant::now();
    let mut primeira = None;
    e.mede_com(t, &AtomicBool::new(false), mede::PASSO_DA_CURVA, n, |_| {
        primeira.get_or_insert_with(|| t0.elapsed().as_millis());
    })
    .expect("mede");
    (primeira.unwrap_or(0), t0.elapsed().as_millis())
}

/// ⭐ **O custo de medir** (W6c, W6d, W7): «uma reprodução» = a trilha que toca do estado inicial;
/// [`EnsinoDoPainter::de`]; o que fica no quadro uma vez por versão (ler a `Macro`, o hash dos bytes, a
/// cópia da trilha para o fio) e por quadro (comparar os bytes) — intercaladas, o MÍNIMO de 3. E
/// «mede» (as curvas da gravada e da base de cada pincelada, W7): até à 1.ª medida e o total, com
/// [`mede::fios`] fios e com um, intercaladas, o mínimo de 2. O Splash em Wet Paint, 512² e 1024².
#[test]
#[ignore = "sonda de medição (W6c/W6d/W7): o custo de medir"]
fn sonda_o_custo_das_referencias() {
    use std::hash::{Hash, Hasher};
    let casos: Vec<(String, Vec<u8>, Track)> = [512u32, 1024]
        .into_iter()
        .map(|lado| {
            let (b, _) = grava_pinceladas(lado, 0, &SPLASH, &[]).expect("grava");
            let t = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
            (format!("4 pinceladas {lado}²"), b, t)
        })
        .collect();
    println!("load antes: {} · fios {}", carga(), mede::fios());
    let mut m = vec![[u128::MAX; 6]; casos.len()];
    let mut medir = vec![[(u128::MAX, u128::MAX); 2]; casos.len()];
    let tempo = |m: &mut u128, f: &mut dyn FnMut()| {
        let t0 = std::time::Instant::now();
        f();
        *m = (*m).min(t0.elapsed().as_micros());
    };
    for volta in 0..3 {
        for (i, (_, b, t)) in casos.iter().enumerate() {
            let mm = &mut m[i];
            tempo(&mut mm[0], &mut || {
                std::hint::black_box(EnsinoDoPainter::de(t).expect("de"));
            });
            tempo(&mut mm[1], &mut || {
                let tocavel = t.tocavel(&LeitorDoPainter).expect("tocável");
                let mut toc = TocadorDoPainter::novo(
                    &tocavel.initial,
                    tocavel.events.iter().map(|e| e.payload.as_slice()),
                )
                .expect("tocador");
                for ev in &tocavel.events {
                    toca(&mut toc, &ev.payload).expect("toca");
                }
                std::hint::black_box(toc.tool().canvas_size());
            });
            tempo(&mut mm[2], &mut || {
                std::hint::black_box(ph2d_macro::codec::read(b).expect("lê"));
            });
            tempo(&mut mm[3], &mut || {
                let mut h = std::hash::DefaultHasher::new();
                b.hash(&mut h);
                std::hint::black_box(h.finish());
            });
            tempo(&mut mm[4], &mut || {
                std::hint::black_box(t.clone());
            });
            let c = b.clone();
            tempo(&mut mm[5], &mut || {
                std::hint::black_box(std::hint::black_box(&c) == b);
            });
            if volta < 2 {
                // Alternadas: muitos fios primeiro numa volta, um fio primeiro na outra.
                let ordem = if volta == 0 { [0, 1] } else { [1, 0] };
                for j in ordem {
                    let n = if j == 0 { mede::fios() } else { 1 };
                    let (a, z) = tempo_de_medir(t, n);
                    let x = &mut medir[i][j];
                    *x = (x.0.min(a), x.1.min(z));
                }
            }
        }
    }
    for ((nome, b, t), (x, md)) in casos.iter().zip(m.iter().zip(&medir)) {
        println!(
            "{nome} ({} eventos, {} KB): de {} µs · uma reprodução {} µs · lê {} µs · hash {} µs · cópia \
             da trilha {} µs · compara os bytes {} µs · mede ({} fios) 1.ª {} ms, total {} ms · mede (1 \
             fio) 1.ª {} ms, total {} ms",
            t.events.len(),
            b.len() / 1024,
            x[0],
            x[1],
            x[2],
            x[3],
            x[4],
            x[5],
            mede::fios(),
            md[0].0,
            md[0].1,
            md[1].0,
            md[1].1
        );
    }
    println!("load depois: {}", carga());
}

/// ⭐ **O custo de medir as 30 pinceladas** (W7): [`EnsinoDoPainter::mede`] em Wet Paint a 512² e
/// 1024² — até à 1.ª medida e o total, uma vez cada (o `loadavg` ao lado).
#[test]
#[ignore = "sonda de medição (W7): o custo de medir as 30 pinceladas"]
fn sonda_o_custo_de_medir_as_trinta() {
    let ps = trinta();
    for lado in [512u32, 1024] {
        let (b, _) = grava_pinceladas(lado, 0, &ps, &[]).expect("grava");
        let t = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
        let antes = carga();
        let (a, z) = tempo_de_medir(&t, mede::fios());
        println!(
            "30 pinceladas {lado}² ({} eventos): mede ({} fios) 1.ª {a} ms, total {z} ms · load {} → {}",
            t.events.len(),
            mede::fios(),
            antes.trim(),
            carga().trim()
        );
    }
}

/// **O custo de uma avaliação** (W6, a auditoria): as 30 pinceladas, o aluno com as 15 primeiras
/// feitas. «Antes» = a régua de antes da cura (as camadas lidas e ordenadas por critério, o inicial
/// procurado por píxel — a [`fracao`] desta sonda); «depois» = [`EnsinoDoPainter::por_cumprir`].
/// Intercaladas, o MÍNIMO de 7.
#[test]
#[ignore = "sonda de medição (W6): o custo da avaliação por quadro"]
fn sonda_o_custo_da_avaliacao() {
    let lado = 512;
    let ps = trinta();
    let (b, _) = grava_pinceladas(lado, 0, &ps, &[]).expect("grava");
    let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
    let e = EnsinoDoPainter::medido(&track).expect("ensino");
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    for c in &e.criterios[..15] {
        mostra(&mut t, &track, c.regiao.k).expect("mostra");
        for _ in 0..ENTRE {
            quadro(&mut t);
        }
    }
    for _ in 0..CAUDA {
        quadro(&mut t);
    }
    let antes = |t: &PainterTool| {
        e.criterios.iter().position(|c| {
            let (b, d) = c.referencia.as_ref().expect("medida").em(idade(t));
            fracao(&e, t, &c.novos, 0) - b < FRACAO_DA_GRAVADA_QUE_CUMPRE * d
        })
    };
    assert_eq!(antes(&t), e.por_cumprir(&t), "as duas réguas concordam");
    for (i, c) in e.criterios.iter().enumerate().take(16) {
        let toda = rasteriza(&c.regiao, lado, lado).len();
        println!(
            "  {i}: nova {} de {toda} px, pintada {:.1} %, referência {:.1} %",
            c.novos.len(),
            100.0 * e.fracao_pintada(&t, c),
            100.0 * c.referencia.as_ref().expect("medida").gravada.fim()
        );
    }
    let (mut a, mut d) = (u128::MAX, u128::MAX);
    for _ in 0..7 {
        let t0 = std::time::Instant::now();
        std::hint::black_box(antes(&t));
        a = a.min(t0.elapsed().as_micros());
        let t0 = std::time::Instant::now();
        std::hint::black_box(e.por_cumprir(&t));
        d = d.min(t0.elapsed().as_micros());
    }
    println!(
        "{} critérios, por cumprir {:?}: antes {a} µs, depois {d} µs por avaliação",
        e.criterios.len(),
        e.por_cumprir(&t)
    );
}
