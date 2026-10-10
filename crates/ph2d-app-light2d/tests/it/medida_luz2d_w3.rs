//! **A rodada da W3** (`docs/DevOps/MEDIR_VELOCIDADE.md`): TODAS as perguntas num lote, variantes no
//! MESMO processo, intercaladas com a ordem rodada, vale o MÍNIMO (mediana ao lado). Corre-se à mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w3 -- --ignored --nocapture
//! ```
//!
//! A. o DEGRAU (texel em px × margem) a 1920×1080: dimensões, ms do solver na placa, bytes/quadro e
//!    GB/s a 60 Hz contra o K7 (≤ 3 GB/s);
//! B. o QUADRO com e sem luz (relógio de parede até a placa acabar): a cadeia inteira em miniatura;
//! C. a CAPTURA de sprites por texel: ms da C0 com 0 · 100 · 1 000 sprites texturizados;
//! D. a CAMADA do vetor: ms e erro com 1 · 2 · 4 px por texel;
//! E. a MARGEM: o erro da luz que chega à borda da vista quando a lâmpada está FORA dela.

use std::time::Instant;

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, Degrau, LuzDoQuadro, grade_do_quadro};
use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef, SpriteEmissive, Transform, VecPathRef};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, TipoDeLuz};
use ph2d_light2d_gpu::{Config, PassList};
use ph2d_render::{Camera2d, LiftedInstances, Sprite, SpriteRenderer};
use ph2d_vec_scene::{VecScene, VecXforms, rectangle};
use vello::kurbo::Affine;

use crate::comum::{Cadeia, fluencia_lida, gpu, quad, quadrado_vetorial, renderer};

const FHD: (u32, u32) = (1920, 1080);
/// 100 px/m: a vista tem 19,2 × 10,8 m.
const ALTURA_M: f32 = 10.8;
const RODADAS: usize = 15;

fn afim_fhd() -> Affine {
    camera_fhd().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

fn camera_fhd() -> Camera2d {
    Camera2d::new([0.0, 0.0], ALTURA_M)
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

fn pose(x: f32, y: f32, rot: f32) -> (Transform, GlobalTransform) {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t.rotation = rot;
    (t, GlobalTransform::from_transform(t))
}

fn luz(w: &mut World, x: f32, y: f32, intensidade: f32, raio: f32) {
    let (t, g) = pose(x, y, 0.0);
    w.spawn((
        Luz {
            cor: [1.0, 0.9, 0.75],
            intensidade,
            tamanho_m: raio,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        g,
    ));
}

/// Uma textura 64×64 com um disco opaco (o alfa é o que a C0 lê).
fn disco_64(r: &mut SpriteRenderer) -> u32 {
    let mut px = Vec::with_capacity(64 * 64 * 4);
    for j in 0..64 {
        for i in 0..64 {
            let d = ((f64::from(i) - 31.5).powi(2) + (f64::from(j) - 31.5).powi(2)).sqrt();
            px.extend_from_slice(&[200, 180, 160, if d < 30.0 { 255 } else { 0 }]);
        }
    }
    r.acquire_individual(64, 64, &px).expect("disco")
}

/// A cena do editor em miniatura: `n` sprites (discos de 0,6 m numa treliça), dos quais `bloqueiam`
/// têm `BloqueiaLuz` e `brilham` têm `SpriteEmissive`; `com_luz` acrescenta uma `Luz`.
fn cena_do_editor(
    r: &mut SpriteRenderer,
    n: usize,
    bloqueiam: usize,
    brilham: usize,
    com_luz: bool,
) -> (World, PresentWorld) {
    let mut w = World::new();
    if com_luz {
        luz(&mut w, -3.0, 1.0, 40.0, 0.3);
    }
    let tex = disco_64(r);
    let mut present = PresentWorld::new();
    let lado = (n as f64).sqrt().ceil() as usize;
    for k in 0..n {
        let x = -8.5 + 17.0 * (k % lado) as f32 / lado as f32;
        let y = -4.5 + 9.0 * (k / lado) as f32 / lado as f32;
        let (t, g) = pose(x, y, 0.3 * k as f32);
        let sprite = Sprite::atlas(0, [0.6, 0.6], [1.0; 4]);
        let e = if k < bloqueiam {
            w.spawn((sprite, BloqueiaLuz { opacidade: 1.0 }, t, g)).id()
        } else if k < bloqueiam + brilham {
            w.spawn((sprite, SpriteEmissive(3.0), t, g)).id()
        } else {
            w.spawn((sprite, t, g)).id()
        };
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = quad(tex, [tx, ty]);
        i.size = sprite.size;
        i.basis = [ax, ay, bx, by];
        i.z_order = k as u32;
        present.world_mut().spawn((i, SimRef(e)));
    }
    (w, present)
}

#[test]
#[ignore = "a rodada da W3: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w3() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W3 · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
        loadavg()
    );
    degraus(&gpu);
    quadro(&gpu);
    captura(&gpu);
    camada_vetorial(&gpu);
    margem(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// A. O degrau.
fn degraus(gpu: &GpuContext) {
    let mut r = renderer(gpu);
    let (w, mut present) = cena_do_editor(&mut r, 120, 30, 5, true);
    let variantes: Vec<Degrau> = [2.0, 3.0, 4.0]
        .into_iter()
        .flat_map(|c0_px| {
            [0.0, 0.125, 0.25, 0.5].map(|margem| Degrau {
                c0_px,
                margem,
                ..DEGRAU_PADRAO
            })
        })
        .collect();
    let mut luzes: Vec<LuzDoQuadro> = variantes
        .iter()
        .map(|d| LuzDoQuadro::com_degrau(*d))
        .collect();
    let mut ms: Vec<Vec<f64>> = vec![Vec::new(); luzes.len()];
    for rodada in 0..(RODADAS + 2) {
        for o in 0..luzes.len() {
            let k = (o + rodada) % luzes.len();
            let l = &mut luzes[k];
            assert!(l.preparar(&w, afim_fhd(), FHD));
            l.gravar(gpu, &mut present, &r);
            let t = l
                .solver()
                .and_then(|s| s.tempos(gpu))
                .expect("TIMESTAMP_QUERY");
            if rodada >= 2 {
                ms[k].push(t.c0 + t.rc());
            }
        }
    }
    eprintln!("\n## A. Degrau a 1920×1080 (C0+C1+C2 na placa, ms) — K7: ≤ 3 GB/s a 60 Hz");
    eprintln!(
        "| c0 px | margem | grade | cascatas | ms (mín · mediana) | MB/quadro | GB/s a 60 Hz | K7 |"
    );
    eprintln!("|---|---|---|---|---|---|---|---|");
    for (d, v) in variantes.iter().zip(ms) {
        let g = grade_do_quadro(afim_fhd(), FHD, *d).expect("grade");
        let cfg = Config::padrao_para(&gpu.device, g);
        let bytes = PassList::bytes_por_quadro(&cfg) as f64;
        let gbs = bytes * 60.0 / 1e9;
        let (mn, md) = min_med(v);
        eprintln!(
            "| {} | {} | {}×{} | {} | {mn:.2} · {md:.2} | {:.1} | {gbs:.2} | {} |",
            d.c0_px,
            d.margem,
            cfg.grade_efetiva().largura,
            cfg.grade_efetiva().altura,
            cfg.n_cascatas(),
            bytes / 1e6,
            if gbs <= 3.0 { "✔" } else { "✗" }
        );
    }
}

/// B. O quadro com e sem luz (parede, até a placa acabar).
fn quadro(gpu: &GpuContext) {
    let mut cadeias = [Cadeia::nova_com(gpu, FHD), Cadeia::nova_com(gpu, FHD)];
    let mut cenas = Vec::new();
    for (k, c) in cadeias.iter_mut().enumerate() {
        cenas.push(cena_do_editor(&mut c.renderer, 300, 30, 5, k == 1));
    }
    let mut doc = VecScene::new();
    let mut xforms = VecXforms::new();
    let mut vetor = vello::Scene::new();
    let mut w_vec = Vec::new();
    for k in 0..20 {
        let x = -8.0 + 0.8 * f64::from(k);
        let id = doc.push_path(rectangle([x, -4.8], [x + 0.4, -4.2]));
        xforms.insert(id, ph2d_vec_scene::Xform::IDENTITY);
        quadrado_vetorial(&mut vetor, [90, 120, 200, 255], [x + 0.2, -4.5]);
        w_vec.push(id);
    }
    for id in w_vec.iter().take(5) {
        cenas[1]
            .0
            .spawn((VecPathRef(*id), BloqueiaLuz { opacidade: 1.0 }));
    }
    let mut luzes = [LuzDoQuadro::new(), LuzDoQuadro::new()];
    let mut ms: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    for rodada in 0..(RODADAS + 2) {
        for o in 0..2 {
            let k = (o + rodada) % 2;
            let (w, present) = &mut cenas[k];
            let c = &mut cadeias[k];
            let l = &mut luzes[k];
            let t0 = Instant::now();
            if l.preparar(w, afim_fhd(), FHD) {
                l.desenhar_vetores(gpu, &doc, &xforms);
            }
            l.gravar(gpu, present, &c.renderer);
            l.ligar(gpu, &mut c.renderer, &mut c.colagem, &c.tonemap);
            c.desenhar(
                gpu,
                present,
                &LiftedInstances::default(),
                &vetor,
                &camera_fhd(),
            );
            gpu.device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("poll");
            if rodada >= 2 {
                ms[k].push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
    }
    let (s0, s1) = (min_med(ms[0].clone()), min_med(ms[1].clone()));
    eprintln!(
        "\n## B. Quadro a 1920×1080 (300 sprites, 20 retângulos vetoriais; parede até a placa acabar)"
    );
    eprintln!("| variante | ms (mín · mediana) |\n|---|---|");
    eprintln!("| sem `Luz` (ponto neutro) | {:.2} · {:.2} |", s0.0, s0.1);
    eprintln!(
        "| com `Luz` (30 bloqueiam, 5 brilham, 5 caminhos) | {:.2} · {:.2} |",
        s1.0, s1.1
    );
    eprintln!("| sprites por texel | {:?} |", luzes[1].sprites_por_texel());
}

/// C. A captura de sprites por texel: a C0 na placa com N sprites texturizados.
fn captura(gpu: &GpuContext) {
    let mut r = renderer(gpu);
    let casos: Vec<(usize, World, PresentWorld)> = [0usize, 100, 1000]
        .into_iter()
        .map(|n| {
            let (w, p) = cena_do_editor(&mut r, n.max(1), n, 0, true);
            (n, w, p)
        })
        .collect();
    let mut casos = casos;
    let mut luzes: Vec<LuzDoQuadro> = casos.iter().map(|_| LuzDoQuadro::new()).collect();
    let mut ms: Vec<Vec<f64>> = vec![Vec::new(); casos.len()];
    for rodada in 0..(RODADAS + 2) {
        for o in 0..casos.len() {
            let k = (o + rodada) % casos.len();
            let (_, w, p) = &mut casos[k];
            let l = &mut luzes[k];
            assert!(l.preparar(w, afim_fhd(), FHD));
            l.gravar(gpu, p, &r);
            let t = l.solver().and_then(|s| s.tempos(gpu)).expect("tempos");
            if rodada >= 2 {
                ms[k].push(t.c0);
            }
        }
    }
    eprintln!("\n## C. Captura de sprites por texel (C0 na placa, degrau padrão, 1920×1080)");
    eprintln!("| sprites (64×64, 0,6 m) | C0 ms (mín · mediana) |\n|---|---|");
    for ((n, _, _), v) in casos.iter().zip(ms) {
        let (mn, md) = min_med(v);
        eprintln!("| {n} | {mn:.3} · {md:.3} |");
    }
}

/// D. A camada do vetor: custo (parede) e erro contra o polígono.
fn camada_vetorial(gpu: &GpuContext) {
    eprintln!("\n## D. Camada do vetor (px por texel)");
    eprintln!(
        "| px por texel | erro × polígono, RMSE rel. (padrão · Alto) | desenhar_vetores ms (mín, 20 caminhos, 1920×1080) |"
    );
    eprintln!("|---|---|---|");
    let mut doc = VecScene::new();
    let mut xforms = VecXforms::new();
    let mut w = World::new();
    luz(&mut w, -3.0, 1.0, 40.0, 0.3);
    for k in 0..20 {
        let x = -8.0 + 0.8 * f64::from(k);
        let id = doc.push_path(rectangle([x, -1.0], [x + 0.4, 1.0]));
        xforms.insert(id, ph2d_vec_scene::Xform::IDENTITY);
        w.spawn((VecPathRef(id), BloqueiaLuz { opacidade: 1.0 }));
    }
    for fator in [1u32, 2, 4] {
        let d = Degrau {
            vetor_px_por_texel: fator,
            ..DEGRAU_PADRAO
        };
        let (erro, _) = crate::captura_do_vetor::medir(gpu, d);
        let (erro_alto, _) = crate::captura_do_vetor::medir(
            gpu,
            Degrau {
                vetor_px_por_texel: fator,
                ..crate::comum::ALTO
            },
        );
        let mut l = LuzDoQuadro::com_degrau(d);
        let mut v = Vec::new();
        for rodada in 0..(RODADAS + 2) {
            assert!(l.preparar(&w, afim_fhd(), FHD));
            let t0 = Instant::now();
            l.desenhar_vetores(gpu, &doc, &xforms);
            gpu.device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("poll");
            if rodada >= 2 {
                v.push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        eprintln!(
            "| {fator} | {erro:.4} · {erro_alto:.4} | {:.2} |",
            min_med(v).0
        );
    }
}

/// E. A margem: a luz FORA da vista, a `d` metros da borda esquerda; o erro na coluna visível junto
/// à borda, contra a mesma cena com margem 1 (a vista inteira de cada lado).
fn margem(gpu: &GpuContext) {
    let r = renderer(gpu);
    let borda = -9.6_f64;
    let distancias = [0.5_f64, 2.0, 5.0, 9.0];
    let margens = [0.0_f64, 0.125, 0.25, 0.5];
    eprintln!(
        "\n## E. Margem — erro relativo na coluna visível a 0,2 m da borda (lâmpada a d m FORA)"
    );
    eprintln!(
        "| margem (m de cada lado) | {} |",
        distancias.map(|d| format!("d = {d} m")).join(" | ")
    );
    eprintln!("|---|{}", "---|".repeat(distancias.len()));
    let coluna = |l: &LuzDoQuadro| -> Vec<f64> {
        let crate::comum::Fluencia { v: f, grade: g } =
            fluencia_lida(gpu, l.solver().expect("solver"));
        let x = borda + 0.2;
        let i = (((x - g.origem_m[0]) / g.texel_m) as u32).min(g.largura - 1);
        (0..g.altura)
            .filter(|j| {
                let y = g.origem_m[1] + (f64::from(*j) + 0.5) * g.texel_m;
                y.abs() < 5.0
            })
            .map(|j| f64::from(f[(j * g.largura + i) as usize][1]))
            .collect()
    };
    let mut linhas = vec![String::new(); margens.len()];
    for d in distancias {
        let mut w = World::new();
        luz(&mut w, (borda - d) as f32, 0.0, 40.0, 0.3);
        // Uma parede DENTRO da vista, junto à borda: a sombra também tem de vir.
        let (t, g) = pose(-8.0, 2.0, 0.2);
        w.spawn((
            Sprite::atlas(0, [0.3, 3.0], [1.0; 4]),
            BloqueiaLuz { opacidade: 1.0 },
            t,
            g,
        ));
        let mut refe = LuzDoQuadro::com_degrau(Degrau {
            margem: 1.0,
            ..DEGRAU_PADRAO
        });
        assert!(refe.preparar(&w, afim_fhd(), FHD));
        refe.gravar(gpu, &mut PresentWorld::new(), &r);
        let alvo = coluna(&refe);
        let pico = alvo.iter().copied().fold(0.0, f64::max).max(1e-9);
        for (k, m) in margens.iter().enumerate() {
            let mut l = LuzDoQuadro::com_degrau(Degrau {
                margem: *m,
                ..DEGRAU_PADRAO
            });
            assert!(l.preparar(&w, afim_fhd(), FHD));
            l.gravar(gpu, &mut PresentWorld::new(), &r);
            let v = coluna(&l);
            let erro = v
                .iter()
                .zip(&alvo)
                .map(|(a, b)| (a - b).abs() / pico)
                .fold(0.0, f64::max);
            linhas[k].push_str(&format!(" {erro:.3} |"));
        }
    }
    for (m, l) in margens.iter().zip(linhas) {
        eprintln!("| {m} ({:.1} m) |{l}", m * 19.2);
    }
}
