//! **A app real cai no regime da vista `P`?** (handoff W6h §13 b; o item 2 da W6i). À mão, um processo por modo:
//!
//! ```text
//! PH2D_REGIME_MODO=app|controlo PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d \
//!     --profile smoke --test it medida_luz2d_regime_app -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Cena 5, `[-4.2, -1.8]` h 3,8, Alta (`PH2D_REGIME_ALVO=<l>x<a>`, por omissão 3840×2160). Só a `c2` total (sem
//! fases) decide; as fases correm por último, em vistas que nada mais mede depois (W6g §51 c).
//! - `controlo`: a vista `P` e a `identidade` da régua `medida_luz2d_borda_topo` (as mesmas idades), intercaladas.
//! - `app`: o `LuzDoQuadro` (nasce, [`LuzDoQuadro::renascer_a_vista`]), cada vida sozinha; depois intercalado com
//!   uma `identidade` com o entorno próprio, nascida com a app viva.

use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::teto::{Tetos, fina_que_cabe};
use ph2d_app_light2d::{
    LuzDoQuadro, config_da_vista, config_do_entorno, grade_do_quadro, regiao_da_janela,
};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_gpu::GpuContext;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Solver};
use ph2d_render::Camera2d;
use vello::kurbo::Affine;

use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5o::{mediana, minimo};
use crate::medida_luz2d_w5v::cena_do_smoke;

const VIDA: usize = 60;
const RODADAS: usize = 9;
const AQUECE: usize = 3;
const QUADROS: usize = 6;
const RODADAS_FASES: usize = 4;

/// A memória usada na placa inteira (MiB, `nvidia-smi`): a vista de ~4 GB ao lado de outra.
fn vram() -> String {
    std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.used", "--format=csv,noheader"])
        .output()
        .map_or_else(
            |_| "?".into(),
            |o| String::from_utf8_lossy(&o.stdout).trim().to_string(),
        )
}

fn espera(gpu: &GpuContext) {
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
}

/// O entorno de `cfg_vista` com o miolo mascarado (como o app e a régua).
fn entorno_de(
    gpu: &GpuContext,
    a: Affine,
    alvo: (u32, u32),
    dg: ph2d_app_light2d::Degrau,
    vista: &Config,
) -> Solver {
    let c = config_do_entorno(&gpu.device, a, alvo, dg).expect("o entorno");
    let mut e = Solver::novo(gpu, c).expect("solver do entorno");
    let g = vista.grade;
    e.mascarar_miolo(
        gpu,
        Some([
            g.origem_m[0],
            g.origem_m[1],
            g.origem_m[0] + f64::from(g.largura) * g.texel_m,
            g.origem_m[1] + f64::from(g.altura) * g.texel_m,
        ]),
    );
    assert!(
        e.config().arquivo_da_borda.is_some(),
        "o entorno sem o arquivo: não há borda"
    );
    e
}

/// A `identidade` da régua: o módulo do produto por `modulo_rc`, nascida, solta e renascida.
fn identidade(
    gpu: &GpuContext,
    c: Config,
    a: Affine,
    alvo: (u32, u32),
    entorno: &Solver,
) -> Solver {
    let m = ph2d_light2d_gpu::fonte_do_rc(c.aritmetica, c.armazenamento, c.hrc_niveis_em_textura())
        .to_string();
    let c = Config {
        modulo_rc: Some(Box::leak(m.into_boxed_str())),
        ..c
    };
    drop(Solver::novo(gpu, c.clone()).expect("identidade"));
    espera(gpu);
    let mut s = Solver::novo(gpu, c).expect("identidade");
    let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, alvo);
    s.regiao_fina(i, f);
    assert!(s.somar_entorno(gpu, Some(entorno)));
    s
}

fn resumo(rotulo: &str, nome: &str, v: &[f64]) {
    eprintln!(
        "C {rotulo} · {nome} · c2 mín {:.3} · med {:.3} · máx {:.3} · n {}",
        minimo(v),
        mediana(v),
        v.iter().copied().fold(f64::MIN, f64::max),
        v.len()
    );
}

/// As fases (a soma n 0 e a soma de todas) das vistas `vs`, intercaladas; a última coisa do processo.
fn fases(
    gpu: &GpuContext,
    rotulo: &str,
    e: &mut Solver,
    vs: &mut [(&str, &mut Solver)],
    cena: &CenaDeLuz,
    sem_ceu: &CenaDeLuz,
) {
    for (_, v) in vs.iter_mut() {
        assert!(v.medir_fases_da_hrc(gpu));
    }
    let n = vs.len();
    let mut soma0 = vec![Vec::new(); n];
    let mut todas = vec![Vec::new(); n];
    for r in 0..RODADAS_FASES {
        for k in 0..n {
            let iv = (k + r) % n;
            let (_, v) = &mut vs[iv];
            for q in 0..AQUECE + QUADROS {
                e.quadro(gpu, sem_ceu);
                v.quadro(gpu, cena);
                if q < AQUECE {
                    continue;
                }
                let f = v.fases_da_hrc(gpu).expect("fases");
                soma0[iv].extend(
                    f.iter()
                        .filter(|x| x.0 == "hrc_soma" && x.1 == 0)
                        .map(|x| x.2),
                );
                todas[iv].push(f.iter().map(|x| x.2).sum::<f64>());
            }
        }
    }
    for (iv, (nome, _)) in vs.iter().enumerate() {
        eprintln!(
            "F {rotulo} · {nome} · hrc_soma n 0 mín {:.3} med {:.3} · soma das fases mín {:.3}",
            minimo(&soma0[iv]),
            mediana(&soma0[iv]),
            minimo(&todas[iv])
        );
    }
}

#[test]
#[ignore = "medição de placa: a app real no regime da vista P (W6h §13 b)"]
fn medida_luz2d_regime_app() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let alvo = std::env::var("PH2D_REGIME_ALVO").map_or((3840, 2160), |v| {
        let (l, a) = v.split_once('x').expect("<l>x<a>");
        (l.parse().expect("largura"), a.parse().expect("altura"))
    });
    let modo = std::env::var("PH2D_REGIME_MODO").unwrap_or_else(|_| "controlo".into());
    let info = gpu.adapter.get_info();
    eprintln!(
        "P placa {:?} · {:?} · modo {modo} · vram à partida {}",
        info.name,
        info.backend,
        vram()
    );
    let a = Camera2d::new([-4.2, -1.8], 3.8)
        .world_to_screen_affine(ph2d_host::WindowSize::new(alvo.0, alvo.1));
    let cena = cena_do_smoke(5);
    let sem_ceu = CenaDeLuz {
        ceu: [0.0; 3],
        ..cena.clone()
    };
    let nivel = NivelDaLuz::Alta;
    let tetos = Tetos {
        lado: gpu.device.limits().max_texture_dimension_2d,
        recusou: None,
    };
    let k = fina_que_cabe(a, alvo, degrau_do(nivel), tetos).expect("nada cabe na placa");
    let dg = degrau_do(nivel).com_fina_engrossada(k);
    let rotulo = format!("{}×{} {nivel:?} fina k {k}", alvo.0, alvo.1);
    match modo.as_str() {
        "controlo" => controlo(&gpu, a, alvo, dg, &rotulo, &cena, &sem_ceu),
        "app" => app(&gpu, a, alvo, dg, &rotulo, &cena, &sem_ceu),
        m => panic!("modo {m}: app | controlo"),
    }
}

/// A vista `P` e a `identidade` como a régua `medida_luz2d_borda_topo` as nasce.
fn controlo(
    gpu: &GpuContext,
    a: Affine,
    alvo: (u32, u32),
    dg: ph2d_app_light2d::Degrau,
    rotulo: &str,
    cena: &CenaDeLuz,
    sem_ceu: &CenaDeLuz,
) {
    let g = grade_do_quadro(a, alvo, dg).expect("a grade");
    let cv = config_da_vista(&gpu.device, g, a, dg);
    let vista = Solver::novo(gpu, cv.clone()).expect("solver da vista");
    let mut entorno = entorno_de(gpu, a, alvo, dg, vista.config());
    let (i, f) = regiao_da_janela(vista.grade_da_fluencia(), a, alvo);
    drop(vista);
    espera(gpu);
    let mut p = Solver::novo(gpu, cv).expect("solver da vista");
    p.regiao_fina(i, f);
    assert!(p.somar_entorno(gpu, Some(&entorno)));
    // `P` sozinha, antes de a `identidade` nascer ao lado (o controlo da co-residência).
    let so: Vec<f64> = (0..AQUECE + VIDA)
        .map(|_| {
            entorno.quadro(gpu, sem_ceu);
            p.quadro(gpu, cena);
            p.tempos(gpu).map_or(0.0, |t| t.c2_total())
        })
        .skip(AQUECE)
        .collect();
    eprintln!(
        "O {rotulo} · P sozinha · vram {} · load {}",
        vram(),
        loadavg().trim()
    );
    resumo(rotulo, "P sozinha", &so);
    let mut id = identidade(gpu, p.config().clone(), a, alvo, &entorno);
    eprintln!(
        "O {rotulo} · controlo · grade {}×{} · vram {} · load {}",
        p.config().grade.largura,
        p.config().grade.altura,
        vram(),
        loadavg().trim()
    );
    let mut c2 = [Vec::new(), Vec::new()];
    for r in 0..RODADAS {
        for k in 0..2 {
            let iv = (k + r) % 2;
            for q in 0..AQUECE + QUADROS {
                entorno.quadro(gpu, sem_ceu);
                let v = if iv == 0 { &mut p } else { &mut id };
                v.quadro(gpu, cena);
                let t = v.tempos(gpu).map_or(0.0, |t| t.c2_total());
                if q >= AQUECE {
                    c2[iv].push(t);
                }
            }
        }
    }
    resumo(rotulo, "P", &c2[0]);
    resumo(rotulo, "identidade", &c2[1]);
    eprintln!(
        "D {rotulo} · controlo · P − identidade (mín) {:+.3} · load {}",
        minimo(&c2[0]) - minimo(&c2[1]),
        loadavg().trim()
    );
    fases(
        gpu,
        rotulo,
        &mut entorno,
        &mut [("P", &mut p), ("identidade", &mut id)],
        cena,
        sem_ceu,
    );
}

/// O `LuzDoQuadro` como o editor o corre (sem preferência: o degrau fica na Alta; a `c2` lê-se esperando).
fn app(
    gpu: &GpuContext,
    a: Affine,
    alvo: (u32, u32),
    dg: ph2d_app_light2d::Degrau,
    rotulo: &str,
    cena: &CenaDeLuz,
    sem_ceu: &CenaDeLuz,
) {
    let mut w = bevy_ecs::world::World::new();
    assert!(ph2d_app_light2d::smoke::encenar(&mut w, 5));
    let r = crate::comum::renderer(gpu);
    let mut present = ph2d_ecs::PresentWorld::new();
    let mut l = LuzDoQuadro::com_degrau(degrau_do(NivelDaLuz::Alta));
    let mut quadro_app = |l: &mut LuzDoQuadro| {
        assert!(l.preparar(&w, a, alvo));
        l.gravar(gpu, &mut present, &r);
        espera(gpu);
        assert!(l.erro().is_none(), "a app recusou: {:?}", l.erro());
        l.solver()
            .and_then(|s| s.tempos(gpu))
            .map_or(0.0, |t| t.c2_total())
    };
    for vida in ["nasce", "renasce"] {
        if vida == "renasce" {
            l.renascer_a_vista();
        }
        let v: Vec<f64> = (0..AQUECE + VIDA)
            .map(|_| quadro_app(&mut l))
            .skip(AQUECE)
            .collect();
        let c = l.solver().expect("a vista").config();
        eprintln!(
            "O {rotulo} · app {vida} · grade {}×{} · fina {:?} · montagens {} · vram {} · load {}",
            c.grade.largura,
            c.grade.altura,
            l.fina_engrossada(),
            l.montagens(),
            vram(),
            loadavg().trim()
        );
        resumo(rotulo, &format!("app {vida} sozinha"), &v);
    }
    // A `identidade` nascida com a app viva (como na régua com `P` viva), com o entorno próprio.
    let cfg = l.solver().expect("a vista").config().clone();
    let mut entorno = entorno_de(gpu, a, alvo, dg, &cfg);
    let mut id = identidade(gpu, cfg, a, alvo, &entorno);
    eprintln!("O {rotulo} · app + identidade · vram {}", vram());
    let mut c2 = [Vec::new(), Vec::new()];
    for rr in 0..RODADAS {
        for k in 0..2 {
            let iv = (k + rr) % 2;
            for q in 0..AQUECE + QUADROS {
                let t = if iv == 0 {
                    quadro_app(&mut l)
                } else {
                    entorno.quadro(gpu, sem_ceu);
                    id.quadro(gpu, cena);
                    id.tempos(gpu).map_or(0.0, |t| t.c2_total())
                };
                if q >= AQUECE {
                    c2[iv].push(t);
                }
            }
        }
    }
    resumo(rotulo, "app intercalada", &c2[0]);
    resumo(rotulo, "identidade", &c2[1]);
    eprintln!(
        "D {rotulo} · app · app − identidade (mín) {:+.3} · load {}",
        minimo(&c2[0]) - minimo(&c2[1]),
        loadavg().trim()
    );
    // A soma n 0 só da `identidade` (as fases da app ligam-se só por ambiente, desde o nascimento).
    fases(
        gpu,
        rotulo,
        &mut entorno,
        &mut [("identidade", &mut id)],
        cena,
        sem_ceu,
    );
}
