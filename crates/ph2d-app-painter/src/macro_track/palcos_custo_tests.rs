//! **SONDA — o custo do quadro com N palcos do Painter a tocar** (W4, a prova do bloco; régua
//! `docs/DevOps/MEDIR_VELOCIDADE.md`): N sprites com o Splash da cena `PH2D_MACRO_SMOKE=1` (1024² e
//! 2048²) conduzidas no mesmo instante — `PalcosDoPainter::segue` + `sobe` para um `SpriteRenderer`
//! real, a placa sincronizada no fim do quadro; blocos intercalados por ordem rodada, o MÍNIMO.
//! O trabalho muda de quadro para quadro (a água gravada vem aos lotes), logo cada bloco repete o MESMO
//! troço da macro — um recuo fora da régua — e a régua é a SOMA do bloco, nunca um quadro.
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-painter --profile smoke --lib --
//! --ignored --nocapture macro_track::palcos_custo`

use super::PalcosDoPainter;
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, SimWorld};
use ph2d_editor_core::toast::ToastQueue;
use ph2d_gpu::GpuContext;
use ph2d_render::SpriteRenderer;
use ph2d_tool_painter::macro_track::roteiro::grava_o_splash;

const LADOS: [u32; 2] = [1024, 2048];
const NS: [usize; 3] = [1, 4, 16];
const RODADAS: usize = 7;
const BLOCO: usize = 20;
/// O troço medido começa aqui (a 1.ª pincelada a pousar; a água corre até ao fim).
const INICIO_S: f64 = 0.2;

struct Variante {
    lado: u32,
    n: usize,
    sim: SimWorld,
    palcos: PalcosDoPainter,
    t: f64,
    /// Por bloco: (segue, sobe) em ms por quadro.
    ms: Vec<(f64, f64)>,
}

fn quadro(v: &mut Variante, r: &mut SpriteRenderer, gpu: &GpuContext) -> (f64, f64) {
    let t = v.t;
    let mut q = v.sim.world_mut().query::<&mut MacroTempo>();
    for mut m in q.iter_mut(v.sim.world_mut()) {
        m.segundos = t;
    }
    let t0 = std::time::Instant::now();
    v.palcos.segue(v.sim.world_mut());
    let segue = t0.elapsed().as_secs_f64() * 1e3;
    let trocas = v.palcos.sobe(r, &mut ToastQueue::default());
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let total = t0.elapsed().as_secs_f64() * 1e3;
    assert_eq!(trocas.len(), v.n, "cada palco troca a sua sprite");
    v.t += 1.0 / 60.0;
    (segue, total - segue)
}

fn min_med(v: &[f64]) -> (f64, f64) {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    (s[0], s[s.len() / 2])
}

#[test]
#[ignore = "placa: sonda de custo, precisa de adapter de GPU"]
fn sonda_n_palcos_do_painter() {
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("load antes: {}", carga().trim());
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adaptador de GPU");
    let atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    let mut r = SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let mut vs = Vec::new();
    for lado in LADOS {
        let t0 = std::time::Instant::now();
        let (bytes, d) = grava_o_splash(lado, 0).expect("grava");
        eprintln!(
            "{lado}²: gravar {d:.2} s de macro custou {:.0} ms ({} KiB)",
            t0.elapsed().as_secs_f64() * 1e3,
            bytes.len() / 1024
        );
        for n in NS {
            let mut sim = SimWorld::new();
            for _ in 0..n {
                sim.world_mut().spawn((
                    Macro {
                        doc: bytes.clone(),
                        modo: ModoDaMacro::Fiel,
                    },
                    MacroTempo { segundos: INICIO_S },
                ));
            }
            vs.push(Variante {
                lado,
                n,
                sim,
                palcos: PalcosDoPainter::default(),
                t: INICIO_S,
                ms: Vec::new(),
            });
        }
    }
    // Aquecimento fora da régua: o 1.º quadro toca do zero até ao início e sobe a 1.ª tela.
    for v in &mut vs {
        let t0 = std::time::Instant::now();
        quadro(v, &mut r, &gpu);
        eprintln!(
            "{}² N={:>2}: 1.º quadro {:.1} ms",
            v.lado,
            v.n,
            t0.elapsed().as_secs_f64() * 1e3
        );
        for _ in 0..3 {
            quadro(v, &mut r, &gpu);
        }
    }
    let k = vs.len();
    for rodada in 0..RODADAS {
        for i in 0..k {
            let v = &mut vs[(rodada + i) % k];
            v.t = INICIO_S - 1.0 / 60.0;
            quadro(v, &mut r, &gpu);
            let (mut segue, mut sobe) = (0.0, 0.0);
            for _ in 0..BLOCO {
                let (a, b) = quadro(v, &mut r, &gpu);
                segue += a;
                sobe += b;
            }
            #[expect(clippy::cast_precision_loss, reason = "BLOCO pequeno")]
            let q = BLOCO as f64;
            v.ms.push((segue / q, sobe / q));
        }
    }
    eprintln!(
        "\n   tela | N palcos | mín ms/quadro | mediana | mín por palco | (segue + sobe, no mínimo)"
    );
    for v in &vs {
        let totais: Vec<f64> = v.ms.iter().map(|(a, b)| a + b).collect();
        let (min, med) = min_med(&totais);
        let i = totais.iter().position(|t| *t == min).unwrap_or(0);
        #[expect(clippy::cast_precision_loss, reason = "N pequeno")]
        let por = min / v.n as f64;
        eprintln!(
            "  {:>4}² | {:>8} | {min:>13.3} | {med:>7.3} | {por:>13.3} | {:.3} + {:.3}",
            v.lado, v.n, v.ms[i].0, v.ms[i].1
        );
    }
    eprintln!("load depois: {}", carga().trim());
}

/// O quadro mais caro do Splash a 1024², do início ao fim e de volta ao início (o laço): onde a cena
/// trava.
#[test]
#[ignore = "placa: sonda de custo, precisa de adapter de GPU"]
fn sonda_o_quadro_mais_caro_do_splash() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adaptador de GPU");
    let atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    let mut r = SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let (bytes, d) = grava_o_splash(1024, 0).expect("grava");
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Macro {
            doc: bytes,
            modo: ModoDaMacro::Fiel,
        },
        MacroTempo { segundos: 0.0 },
    ));
    let mut v = Variante {
        lado: 1024,
        n: 1,
        sim,
        palcos: PalcosDoPainter::default(),
        t: 0.0,
        ms: Vec::new(),
    };
    let mut vistos = Vec::new();
    while v.t <= d + 0.1 {
        let t = v.t;
        let (a, b) = quadro(&mut v, &mut r, &gpu);
        vistos.push((a + b, a, t));
    }
    v.t = 0.0;
    let (a, b) = quadro(&mut v, &mut r, &gpu);
    eprintln!("o recuo do laço (fim → 0): {:.1} ms (segue {a:.1})", a + b);
    vistos.sort_by(|x, y| y.0.total_cmp(&x.0));
    for (ms, segue, t) in vistos.iter().take(6) {
        eprintln!("t={t:.3} s: {ms:.1} ms (segue {segue:.1})");
    }
}

/// **W5e: subir a tela de um palco** — premultiplicar + substituir a textura INTEIRA (o `sobe` de hoje)
/// contra só um retângulo de `f` da área (recortar + premultiplicar + `replace_individual_pixels_region`),
/// a placa sincronizada; blocos intercalados, o MÍNIMO.
#[test]
#[ignore = "placa: sonda de custo, precisa de adapter de GPU"]
fn sonda_subir_a_tela() {
    const FRACCOES: [f64; 3] = [1.0, 0.25, 0.05];
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("load antes: {}", carga().trim());
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adaptador de GPU");
    let atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    let mut r = SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let mut vs: Vec<(u32, f64, u32, Vec<u8>, f64)> = Vec::new();
    for lado in LADOS {
        let n = (lado * lado * 4) as usize;
        let tela: Vec<u8> = (0..n).map(|i| (i % 251) as u8).collect();
        let id = r.acquire_individual(lado, lado, &tela).expect("ranhura");
        for f in FRACCOES {
            vs.push((lado, f, id, tela.clone(), f64::MAX));
        }
    }
    let mut premul = Vec::new();
    let k = vs.len();
    for rodada in 0..RODADAS {
        for i in 0..k {
            let (lado, f, id, tela, best) = &mut vs[(rodada + i) % k];
            let t0 = std::time::Instant::now();
            for _ in 0..BLOCO {
                if *f >= 1.0 {
                    premul.clear();
                    premul.extend_from_slice(tela);
                    ph2d_render::premul::premultiply_rgba8(&mut premul);
                    r.replace_individual_pixels(*id, *lado, *lado, &premul)
                        .expect("sobe");
                } else {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "lado"
                    )]
                    let w = (f64::from(*lado) * f.sqrt()) as u32;
                    premul.clear();
                    for y in 0..w {
                        let a = ((y * *lado) * 4) as usize;
                        premul.extend_from_slice(&tela[a..a + (w * 4) as usize]);
                    }
                    ph2d_render::premul::premultiply_rgba8(&mut premul);
                    r.replace_individual_pixels_region(*id, 0, 0, w, w, &premul)
                        .expect("sobe");
                }
                let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            }
            #[expect(clippy::cast_precision_loss, reason = "BLOCO pequeno")]
            let ms = t0.elapsed().as_secs_f64() * 1e3 / BLOCO as f64;
            *best = best.min(ms);
        }
    }
    eprintln!("load depois: {}", carga().trim());
    for (lado, f, _, _, best) in &vs {
        eprintln!(
            "[w5e] subir {lado}², {:>3.0} % da área: {best:.2} ms (mínimo)",
            f * 100.0
        );
    }
}
