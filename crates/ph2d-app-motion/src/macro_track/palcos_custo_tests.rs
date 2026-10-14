//! **SONDA — o custo do quadro com N palcos do Motion a tocar** (W4, a prova do bloco; régua
//! `docs/DevOps/MEDIR_VELOCIDADE.md`): N objectos com a macro da cena `PH2D_MACRO_SMOKE=2` conduzidos
//! no mesmo instante, cozendo na placa — `PalcosDasMacros::segue` com a placa sincronizada no fim do
//! quadro, em blocos intercalados por ordem rodada; o resumo é o MÍNIMO (mediana ao lado).
//! Mede o cozimento; o desenho de cada palco (camada, filtros) é do presente da shell.
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib --
//! --ignored --nocapture macro_track::palcos_custo`

use super::palcos::PalcosDasMacros;
use super::smoke::grava_o_tutorial;
use crate::motion_mundo::MundoDaCena;
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, SimWorld};
use ph2d_gpu::GpuContext;

const NS: [usize; 3] = [1, 4, 16];
const RODADAS: usize = 7;
const BLOCO: usize = 20;
/// O troço medido começa aqui (a grelha inteira, o vento a soprar).
const INICIO_S: f64 = 11.0;

struct Variante {
    n: usize,
    sim: SimWorld,
    palcos: PalcosDasMacros,
    t: f64,
    ms: Vec<f64>,
}

fn quadro(v: &mut Variante, mundo: &MundoDaCena, gpu: &GpuContext) -> f64 {
    let t = v.t;
    let mut q = v.sim.world_mut().query::<&mut MacroTempo>();
    for mut m in q.iter_mut(v.sim.world_mut()) {
        m.segundos = t;
    }
    let t0 = std::time::Instant::now();
    let tocados = v.palcos.segue(v.sim.world_mut(), mundo, Some(gpu));
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    assert_eq!(tocados.len(), 1, "todos tocam o mesmo dono");
    v.t += 1.0 / 60.0;
    ms
}

fn min_med(v: &[f64]) -> (f64, f64) {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    (s[0], s[s.len() / 2])
}

#[test]
#[ignore = "placa: sonda de custo, precisa de adapter de GPU"]
fn sonda_n_palcos_do_motion() {
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("load antes: {}", carga().trim());
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adaptador de GPU");
    let (bytes, _) = grava_o_tutorial(7).expect("grava");
    let mundo = MundoDaCena::default();
    let mut vs: Vec<Variante> = NS
        .iter()
        .map(|&n| {
            let mut sim = SimWorld::new();
            for _ in 0..n {
                sim.world_mut().spawn((
                    Macro {
                        doc: bytes.clone(),
                        modo: ModoDaMacro::Explicativo,
                    },
                    MacroTempo {
                        segundos: INICIO_S,
                    },
                ));
            }
            Variante {
                n,
                sim,
                palcos: PalcosDasMacros::default(),
                t: INICIO_S,
                ms: Vec::new(),
            }
        })
        .collect();
    // Aquecimento fora da régua: o 1.º quadro toca do zero até ao início (e compila os pipelines).
    for v in &mut vs {
        let t0 = std::time::Instant::now();
        quadro(v, &mundo, &gpu);
        eprintln!(
            "N={:>2}: 1.º quadro (0 → {INICIO_S} s) {:.1} ms",
            v.n,
            t0.elapsed().as_secs_f64() * 1e3
        );
        for _ in 0..5 {
            quadro(v, &mundo, &gpu);
        }
    }
    for r in 0..RODADAS {
        for k in 0..vs.len() {
            let v = &mut vs[(r + k) % NS.len()];
            for _ in 0..BLOCO {
                let ms = quadro(v, &mundo, &gpu);
                v.ms.push(ms);
            }
        }
    }
    eprintln!("\n  N palcos | mín ms/quadro | mediana | mín por palco");
    for v in &vs {
        let (min, med) = min_med(&v.ms);
        #[expect(clippy::cast_precision_loss, reason = "N pequeno")]
        let por = min / v.n as f64;
        eprintln!("  {:>8} | {min:>13.3} | {med:>7.3} | {por:>13.3}", v.n);
    }
    eprintln!("load depois: {}", carga().trim());
}
