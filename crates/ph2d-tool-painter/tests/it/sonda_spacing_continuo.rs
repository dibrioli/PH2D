//! **SONDA (prompt 51): o Spacing 0,1 + Continuous em TODOS os pincéis de fábrica** — a régua num lote só, cada
//! preset em variantes no MESMO processo (`PH2D_SONDA_VARIANTES="nome:k=v,e.k=v,-e.k|…"`: `k` é um campo do
//! pincel em JSON, `e.k` um knob do motor; a vazia é a fábrica). `PH2D_SONDA_PRESETS="A,B"` filtra.
//! - `sonda_a_regua_do_spacing`: a cena seca das mídias ao raio da foto (`midias_de_fabrica`) e a 103 px, a escada
//!   dos óleos (`regua_escada`) a 103 e a 41 px, e a largura (`a_largura_e_a_prometida`) a 4 raios.
//! - `sonda_o_custo_do_spacing`: o traço rápido (`sonda_traco_rapido::mede`) a 40 px e a 103 px, intercalado, o
//!   MÍNIMO de 7 (`--release`, `PH2D_RELEASE="mede velocidade"`).

use super::midias_de_fabrica::{cena_seca, de_fabrica, raio_da_midia};
use super::receitas_de_fabrica::RECEITAS;
use ph2d_brush_presets::Preset;

/// `antes` é o traço de antes da regra (o Spacing de antes, sem Continuous): a referência da diferença de imagem.
const VARIANTES: &str = "antes:spacing=antes,continuous_spacing=false|fabrica:|s001:spacing=0.01,continuous_spacing=false|s0025:spacing=0.025,continuous_spacing=false|s005C:spacing=0.05|sem_lei:e.pigmentSpacing=0";
/// O raio da cena das quatro aquarelas sem foto própria (o controlo do portão das mídias pinta a esta largura).
const RAIO_SEM_FOTO: f32 = 20.0;
const RAIO_GRANDE: f32 = 51.5;
const RAIOS_DA_LARGURA: [f32; 4] = [7.0, 26.0, 55.0, 120.0];

fn env(k: &str, d: &str) -> String {
    std::env::var(k).unwrap_or_else(|_| d.to_owned())
}

fn presets() -> Vec<&'static str> {
    let so = std::env::var("PH2D_SONDA_PRESETS").ok();
    RECEITAS
        .iter()
        .map(|r| r.nome)
        .filter(|n| so.as_ref().is_none_or(|s| s.split(',').any(|x| x == *n)))
        .collect()
}

/// O Spacing de cada preset ANTES da regra (prompt 51): 0,01 nos três óleos, 0,025 no resto.
fn spacing_de_antes(nome: &str) -> &'static str {
    if e_oleo(nome) { "0.01" } else { "0.025" }
}

pub(super) fn com(mut p: Preset, kv: &str) -> Preset {
    let antes = format!("spacing={}", spacing_de_antes(&p.parametros.name));
    let kv = kv.replace("spacing=antes", &antes);
    for x in kv
        .split(',')
        .filter(|x| !x.is_empty() && !x.starts_with("sub="))
    {
        if let Some(k) = x.strip_prefix("-e.") {
            p.parametros.engine.remove(k);
        } else if let Some((k, v)) = x.split_once('=') {
            if let Some(k) = k.strip_prefix("e.") {
                p.parametros.engine.insert(k.to_owned(), v.parse().unwrap());
            } else {
                let v: serde_json::Value = serde_json::from_str(v).unwrap();
                assert!(
                    p.parametros.brush.insert(k.to_owned(), v).is_some()
                        || k == "continuous_spacing",
                    "campo {k} do pincel"
                );
            }
        }
    }
    p
}

/// `sub=F` numa variante: a fracção do sub-passo da cápsula (`capsula::SUB_SPACING` por omissão), nesta thread.
pub(super) fn com_sub<R>(kv: &str, f: impl FnOnce() -> R) -> R {
    let sub = kv
        .split(',')
        .find_map(|x| x.strip_prefix("sub="))
        .map(|v| v.parse().unwrap());
    ph2d_painter_brush::capsula::com_lei(ph2d_painter_brush::capsula::Lei::Uniao, sub, f)
}

pub(super) fn variantes() -> Vec<(String, String)> {
    env("PH2D_SONDA_VARIANTES", VARIANTES)
        .split('|')
        .map(|s| {
            let (n, kv) = s.split_once(':').unwrap_or((s, ""));
            (n.to_owned(), kv.to_owned())
        })
        .collect()
}

fn e_oleo(nome: &str) -> bool {
    matches!(nome, "Oil Bristle" | "Acrylic Round" | "Express Oil")
}

#[test]
#[ignore = "sonda: a régua do Spacing 0,1 + Continuous nos 20 pincéis (prompt 51); corre-se à mão"]
fn sonda_a_regua_do_spacing() {
    let vs = variantes();
    for nome in presets() {
        let raio = raio_da_midia(nome).unwrap_or(RAIO_SEM_FOTO);
        let linhas: Vec<String> = std::thread::scope(|s| {
            let hs: Vec<_> = vs
                .iter()
                .flat_map(|(vn, kv)| {
                    let p = com(de_fabrica(nome), kv);
                    let tag = format!("{nome}@{vn}");
                    let mut v: Vec<std::thread::ScopedJoinHandle<'_, String>> = Vec::new();
                    let (p1, t1) = (p.clone(), tag.clone());
                    v.push(s.spawn(move || {
                        let (m, c) =
                            com_sub(kv, || cena_seca(&p1, raio, std::slice::from_ref(&t1)));
                        format!("[regua] {t1} cena r {raio}: {m:?} {c:?}")
                    }));
                    if e_oleo(nome) {
                        for r in [RAIO_GRANDE, 20.5] {
                            let (p3, t3) = (p.clone(), tag.clone());
                            v.push(s.spawn(move || {
                                let e = com_sub(kv, || super::o_relevo_dos_fios::escada_de(&p3, r));
                                format!("[regua] {t3} escada r {r}: {e:?}")
                            }));
                        }
                    }
                    let (p4, t4) = (p, tag);
                    v.push(s.spawn(move || {
                        let l: Vec<f32> = com_sub(kv, || {
                            RAIOS_DA_LARGURA
                                .iter()
                                .map(|&r| super::presets_de_fabrica::largura_de(&p4, r, false))
                                .collect()
                        });
                        format!("[regua] {t4} largura {RAIOS_DA_LARGURA:?}: {l:?}")
                    }));
                    v
                })
                .collect();
            let grandes: Vec<_> = vs
                .iter()
                .map(|(vn, kv)| {
                    let p = com(de_fabrica(nome), kv);
                    (
                        vn,
                        s.spawn(move || {
                            com_sub(kv, || {
                                super::o_relevo_dos_fios::cena_seca_de(&p, RAIO_GRANDE, false).0
                            })
                        }),
                    )
                })
                .collect();
            let mut l: Vec<String> = hs
                .into_iter()
                .map(|h| {
                    h.join()
                        .unwrap_or_else(|_| format!("[regua] {nome} PANICO numa cena"))
                })
                .collect();
            let grandes: Vec<_> = grandes
                .into_iter()
                .map(|(vn, h)| (vn, h.join().ok()))
                .collect();
            if let Some((_, Some((base, w, _)))) = grandes.first() {
                for (vn, img) in &grandes {
                    if let Some((px, _, _)) = img {
                        l.push(format!(
                            "[regua] {nome}@{vn} grande r {RAIO_GRANDE}: {}",
                            diferenca(base, px, *w as usize)
                        ));
                    }
                }
            }
            l
        });
        for l in linhas {
            println!("{l}");
        }
    }
}

/// A diferença de `b` contra `a` (a fábrica) onde há tinta em algum dos dois: a média de |ΔY| ÷ a tinta média
/// de `a` (a régua normalizada do 34), a tinta de `b` ÷ a de `a`, e a fracção dos px com |ΔY| > 8.
fn diferenca(a: &[u8], b: &[u8], _w: usize) -> String {
    let [dif, tinta, fora, n] = diferenca_de(a, b);
    format!("dif {dif:.4} · tinta {tinta:.4} · fora {fora:.4} (n {n})")
}

/// `[dif, tinta, fora, n]` de [`diferenca`].
pub(super) fn diferenca_de(a: &[u8], b: &[u8]) -> [f64; 4] {
    let y = |q: &[u8]| 0.299 * f64::from(q[0]) + 0.587 * f64::from(q[1]) + 0.114 * f64::from(q[2]);
    let (mut dif, mut ta, mut tb, mut n, mut fora) = (0.0, 0.0, 0.0, 0usize, 0usize);
    for (p, q) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
        let (ya, yb) = (255.0 - y(p), 255.0 - y(q));
        if ya > 4.0 || yb > 4.0 {
            n += 1;
            dif += (ya - yb).abs();
            ta += ya;
            tb += yb;
            fora += usize::from((ya - yb).abs() > 8.0);
        }
    }
    [
        dif / ta.max(1e-9),
        tb / ta.max(1e-9),
        fora as f64 / n.max(1) as f64,
        n as f64,
    ]
}

#[test]
#[ignore = "sonda: o custo do traço rápido por pincel e Spacing (prompt 51); corre-se à mão em --release"]
fn sonda_o_custo_do_spacing() {
    let raios: Vec<f32> = env("PH2D_SONDA_RAIOS", "20,51.5")
        .split(',')
        .map(|r| r.parse().unwrap())
        .collect();
    let passo: f32 = env("PH2D_SONDA_PASSO", "25").parse().unwrap();
    let vs = variantes();
    let casos: Vec<(String, f32, Preset, String)> = presets()
        .into_iter()
        .flat_map(|nome| {
            let vs = &vs;
            raios.iter().flat_map(move |&r| {
                vs.iter().map(move |(vn, kv)| {
                    (
                        format!("{nome}@{vn}"),
                        r,
                        com(de_fabrica(nome), kv),
                        kv.clone(),
                    )
                })
            })
        })
        .collect();
    let mut melhor = vec![[f64::MAX; 4]; casos.len()];
    for rodada in 0..7 {
        for j in 0..casos.len() {
            let i = (rodada + j) % casos.len();
            let m = com_sub(&casos[i].3, || {
                super::sonda_traco_rapido::mede(&casos[i].2, casos[i].1, passo, 3)
            });
            for (a, b) in melhor[i].iter_mut().zip(m) {
                *a = a.min(b);
            }
        }
    }
    println!(
        "[custo] load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for ((n, r, p, kv), [a, _, c, d]) in casos.iter().zip(melhor) {
        let k = com_sub(kv, || conta_dabs(p, *r, passo));
        println!(
            "[custo] {n} r {r}: evento {a:.3} · quadro {c:.3} · pior {d:.3} ms · {k} dabs no motor"
        );
    }
}

/// Quantos dabs o traço da sonda do traço rápido entrega ao MOTOR (os sub-dabs da cápsula contam um a um).
fn conta_dabs(p: &Preset, raio: f32, passo: f32) -> usize {
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(raio);
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 500.0 - 0.14 * (x - 100.0)],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let n = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    t.on_canvas_pointer(cp(100.0, PointerPhase::Down));
    let n2 = n.clone();
    t.wet_sonda_motor(move |e| {
        e.on_dab = Some(Box::new(move |_, _| {
            n2.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }));
    });
    for k in 1..=(800.0 / passo) as usize {
        t.on_canvas_pointer(cp(100.0 + passo * k as f32, PointerPhase::Move));
        if k % 3 == 0 {
            t.on_tick(1000.0 / 60.0);
        }
    }
    t.on_canvas_pointer(cp(900.0, PointerPhase::Up));
    n.load(std::sync::atomic::Ordering::Relaxed)
}
