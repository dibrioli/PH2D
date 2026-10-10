//! **A rodada da W4** (`docs/DevOps/MEDIR_VELOCIDADE.md`): as cenas de smoke na placa, num lote,
//! variantes no MESMO processo, intercaladas com a ordem rodada, vale o MÍNIMO. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w4 -- --ignored --nocapture
//! ```
//!
//! A. ms do solver (C0 + C1 + C2, relógio da PLACA) e da CPU (`preparar` + `gravar`, parede) a
//!    1920×1080 na vista de fábrica: cena 1 · cena 2 com 1 e com 1 000 vaga-lumes · cena 4;
//! B. a penumbra 10–90 % da cena 4 NA PLACA × a prevista pela geometria.

use std::time::Instant;

use bevy_ecs::world::World;
use ph2d_app_light2d::{LuzDoQuadro, smoke};
use ph2d_ecs::{GlobalTransform, Name, PresentWorld, SimRef};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
use vello::kurbo::Affine;

use crate::comum::{gpu, quad, renderer};

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 15;

fn afim() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

fn min_med(mut v: Vec<f64>) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

/// As instâncias que o passe de sprite desenharia (a C0 captura os sprites por texel a partir delas).
fn instancias(w: &mut World, r: &mut SpriteRenderer) -> PresentWorld {
    let tex = r
        .acquire_individual(4, 4, &[255; 64])
        .expect("textura branca");
    let mut present = PresentWorld::new();
    let mut q = w.query::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform)>();
    for (k, (e, sprite, g)) in q.iter(w).enumerate() {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = quad(tex, [tx, ty]);
        i.size = sprite.size;
        i.basis = [ax, ay, bx, by];
        i.z_order = k as u32;
        present.world_mut().spawn((i, SimRef(e)));
    }
    present
}

/// A cena 2 só com `n` vaga-lumes (os outros saem).
fn vaga_lumes(n: usize) -> World {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 2));
    let fora: Vec<_> = w
        .query::<(bevy_ecs::entity::Entity, &Name)>()
        .iter(&w)
        .filter(|(_, nome)| {
            nome.0
                .strip_prefix("Vaga-lume ")
                .and_then(|k| k.parse::<usize>().ok())
                .is_some_and(|k| k > n)
        })
        .map(|(e, _)| e)
        .collect();
    for e in fora {
        w.despawn(e);
    }
    w
}

fn cena(n: u32) -> World {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, n));
    w
}

#[test]
#[ignore = "a rodada da W4: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w4() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W4 · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
        loadavg()
    );
    custo(&gpu);
    penumbra(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// A. O custo de cada cena.
fn custo(gpu: &GpuContext) {
    let mut r = renderer(gpu);
    let mut variantes: Vec<(&str, World)> = vec![
        ("cena 1 (Uma luz)", cena(1)),
        ("cena 2, 1 vaga-lume", vaga_lumes(1)),
        ("cena 2, 1 000 vaga-lumes", vaga_lumes(smoke::VAGA_LUMES)),
        ("cena 4 (Tamanho da lâmpada)", cena(4)),
    ];
    let mut presents: Vec<PresentWorld> = variantes
        .iter_mut()
        .map(|(_, w)| instancias(w, &mut r))
        .collect();
    let mut luzes: Vec<LuzDoQuadro> = variantes.iter().map(|_| LuzDoQuadro::new()).collect();
    let n = variantes.len();
    let (mut placa, mut cpu) = (vec![Vec::new(); n], vec![Vec::new(); n]);
    let (mut c0, mut rc) = (vec![Vec::new(); n], vec![Vec::new(); n]);
    for rodada in 0..(RODADAS + 2) {
        for o in 0..n {
            let k = (o + rodada) % n;
            let t0 = Instant::now();
            assert!(luzes[k].preparar(&variantes[k].1, afim(), FHD));
            luzes[k].gravar(gpu, &mut presents[k], &r);
            let parede = t0.elapsed().as_secs_f64() * 1e3;
            let t = luzes[k]
                .solver()
                .and_then(|s| s.tempos(gpu))
                .expect("TIMESTAMP_QUERY");
            if rodada >= 2 {
                placa[k].push(t.c0 + t.rc());
                c0[k].push(t.c0);
                rc[k].push(t.rc());
                cpu[k].push(parede);
            }
        }
    }
    eprintln!("\n## A. Custo a 1920×1080, vista de fábrica (ms)");
    eprintln!(
        "| cena | emissores | por texel / sprites | placa C0+C1+C2 (mín · med) | C0 mín | C1+C2 mín | CPU preparar+gravar (mín · med) |"
    );
    eprintln!("|---|---|---|---|---|---|---|");
    for k in 0..n {
        let (pm, pd) = min_med(placa[k].clone());
        let (cm, cd) = min_med(cpu[k].clone());
        let (c0m, rcm) = (min_med(c0[k].clone()).0, min_med(rc[k].clone()).0);
        let (por_texel, total) = luzes[k].sprites_por_texel();
        eprintln!(
            "| {} | {} | {por_texel} / {total} | {pm:.3} · {pd:.3} | {c0m:.3} | {rcm:.3} | {cm:.3} · {cd:.3} |",
            variantes[k].0,
            luzes[k].cena().emissores.len(),
        );
    }
}

/// B. A penumbra da cena 4 na placa: a fluência com a parede ÷ sem ela, ao longo da linha de medida.
fn penumbra(gpu: &GpuContext) {
    eprintln!("\n## B. Penumbra 10–90 % da cena 4 na placa × geometria");
    eprintln!("| lâmpada | raio m | texel m | placa m | geometria m | erro |");
    eprintln!("|---|---|---|---|---|---|");
    let fino = ph2d_app_light2d::Degrau {
        c0_px: 2.0,
        ..ph2d_app_light2d::DEGRAU_PADRAO
    };
    for d in [ph2d_app_light2d::DEGRAU_PADRAO, fino] {
        for (nome, raio, texel, m, p) in crate::smoke_cenas_w4::penumbras_na_placa(gpu, d) {
            let m = m.map_or_else(|| "(sem cruzamento)".into(), |m| format!("{m:.4}"));
            eprintln!("| {nome} | {raio} | {texel:.4} | {m} | {p:.4} | — |");
        }
    }
}

/// O ponto onde a fração da luz cruza `nivel` ao longo de `ts` (curva crescente ou decrescente).
fn cruza(ts: &[f64], f: &[f64], nivel: f64) -> Option<f64> {
    let i = f
        .windows(2)
        .position(|w| (w[0] - nivel) * (w[1] - nivel) <= 0.0 && w[0] != w[1])?;
    let t = (nivel - f[i]) / (f[i + 1] - f[i]);
    Some(ts[i] + t * (ts[i + 1] - ts[i]))
}

/// A ondulação de uma borda: os desvios `d(s)` depois de tirar a reta de mínimos quadrados —
/// `(rms, máx)` em metros.
fn ondulacao(pts: &[(f64, f64)]) -> (f64, f64) {
    let n = pts.len() as f64;
    let (sx, sy) = pts.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
    let (mx, my) = (sx / n, sy / n);
    let (sxy, sxx) = pts.iter().fold((0.0, 0.0), |a, p| {
        (a.0 + (p.0 - mx) * (p.1 - my), a.1 + (p.0 - mx).powi(2))
    });
    let b = sxy / sxx;
    let r: Vec<f64> = pts.iter().map(|p| p.1 - (my + b * (p.0 - mx))).collect();
    let rms = (r.iter().map(|x| x * x).sum::<f64>() / n).sqrt();
    (rms, r.iter().fold(0.0_f64, |m, x| m.max(x.abs())))
}

/// A cena 1 sem a ambiente (a fração é só da lâmpada) — com e sem a parede.
fn cena1_com_e_sem() -> (World, World) {
    let tira = |w: &mut World, nome: &str| {
        let e = w
            .query::<(bevy_ecs::entity::Entity, &Name)>()
            .iter(w)
            .find(|(_, n)| n.0 == nome)
            .map(|(e, _)| e)
            .expect("existe");
        w.despawn(e);
    };
    let (mut com, mut sem) = (cena(1), cena(1));
    tira(&mut com, "Luz ambiente");
    tira(&mut sem, "Luz ambiente");
    tira(&mut sem, "Parede");
    (com, sem)
}

/// A borda de CIMA da sombra da cena 1: a lâmpada `(−3, 0,4)` e o canto de cima-esquerda da parede
/// `(−0,95, 1,4)`. Devolve, para cada `s` (m depois do canto, ao longo da reta), o desvio
/// perpendicular do ponto de 50 % de luz — medido por `frac(p)`.
fn borda_de_cima(frac: &dyn Fn([f64; 2]) -> f64) -> Vec<(f64, f64)> {
    let (l, c) = ([-3.0_f64, 0.4], [-0.95_f64, 1.4]);
    let d = [c[0] - l[0], c[1] - l[1]];
    let n_ = d[0].hypot(d[1]);
    let u = [d[0] / n_, d[1] / n_];
    let nrm = [-u[1], u[0]];
    let ts: Vec<f64> = (0..=240).map(|k| -0.6 + 0.005 * f64::from(k)).collect();
    (5..=60)
        .filter_map(|k| {
            let s = 0.1 * f64::from(k);
            let p0 = [c[0] + s * u[0], c[1] + s * u[1]];
            let f: Vec<f64> = ts
                .iter()
                .map(|&t| frac([p0[0] + t * nrm[0], p0[1] + t * nrm[1]]))
                .collect();
            cruza(&ts, &f, 0.5).map(|t| (s, t))
        })
        .collect()
}

/// C. **A borda da sombra é RETA?** (smoke do dono, 2026-10-09: *«mesmo com alta qualidade, a
/// sombra não fica lisa/reta»*.) A ondulação do ponto de 50 % ao longo da borda, no gabarito e na
/// placa, por degrau × fusão × intervalo da c0.
#[test]
#[ignore = "sonda da borda: corre-se à mão"]
fn medida_borda_reta() {
    let Some(gpu) = gpu() else {
        return;
    };
    let perfil = if cfg!(debug_assertions) {
        "DEBUG"
    } else {
        "smoke"
    };
    eprintln!("# Borda reta · perfil {perfil} · loadavg {}", loadavg());
    let (com, sem) = cena1_com_e_sem();
    let (cc, cs) = (
        ph2d_light2d_ecs::coleta::cena_de_luz(&com),
        ph2d_light2d_ecs::coleta::cena_de_luz(&sem),
    );
    // A escala da tela: px por metro na vista de fábrica a 1920×1080.
    let [a, b, ..] = afim().as_coeffs();
    let ppm = a.hypot(b);
    let gab = borda_de_cima(&|p| {
        ph2d_light2d_ref::fluencia(&cc, p)[1] / ph2d_light2d_ref::fluencia(&cs, p)[1]
    });
    let (r, m) = ondulacao(&gab);
    eprintln!("| variante | texel na tela (px) | ondulação rms (px) | máx (px) | placa ms |");
    eprintln!("|---|---|---|---|---|");
    eprintln!("| gabarito | — | {:.2} | {:.2} | — |", r * ppm, m * ppm);
    use ph2d_light2d_gpu::{Config, Fusao, Solver};
    for c0_px in [4.0, 2.0, 1.0] {
        for (fusao, intervalo) in [(Fusao::CorrecaoBilinear, 1.0), (Fusao::Classica, 1.0)] {
            let degrau = ph2d_app_light2d::Degrau {
                c0_px,
                ..ph2d_app_light2d::DEGRAU_PADRAO
            };
            let g = ph2d_app_light2d::grade_do_quadro(afim(), FHD, degrau).expect("grade");
            let cfg = Config {
                fusao,
                intervalo_c0: intervalo,
                ..Config::padrao_para(&gpu.device, g)
            };
            let flu = |cena: &ph2d_light2d_ecs::CenaDeLuz| {
                let mut s = Solver::novo(&gpu, cfg).expect("solver");
                let mut ms = f64::INFINITY;
                for _ in 0..5 {
                    s.quadro(&gpu, cena);
                    ms = ms.min(s.tempos(&gpu).map_or(f64::NAN, |t| t.total()));
                }
                (crate::comum::fluencia_lida(&gpu, &s), ms)
            };
            let ((fc, ms), (fs, _)) = (flu(&cc), flu(&cs));
            let pts = borda_de_cima(&|p| {
                crate::comum::amostra(&fc, g, p) / crate::comum::amostra(&fs, g, p).max(1e-9)
            });
            let (r, m) = ondulacao(&pts);
            eprintln!(
                "| {c0_px} px · {fusao:?} | {:.2} | {:.2} | {:.2} | {ms:.2} |",
                g.texel_m * ppm,
                r * ppm,
                m * ppm
            );
        }
    }
}
