//! **A régua da borda nos níveis de cima** (handoff W6g §41 a / §53 / §54.1 — os +3,62 ms da vista a 2160p Alta).
//! À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_borda_topo -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Cena 5, `[-4.2, -1.8]` h 3,8, a configuração do APP. Num processo: a vista do produto `P` com o céu
//! (`somar_entorno(None)`) e com a borda, e uma 2.ª vista `M` com a borda SEM LER o arquivo (o `de_fora` faz a
//! conta e o índice, a leitura troca-se pelo céu) — as peças céu → conta → leitura. A borda de `P` entra duas vezes
//! (o controlo). [`RODADAS`] em ordem rodada, o MÍNIMO da `c2`; depois as fases da descida por nível (nunca antes:
//! a vida com fases muda o regime, W6g §51 c). `PH2D_BORDA_TOPO_ALVO=<l>x<a>` / `_NIVEL=low|medium|high`.

use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::teto::{Tetos, fina_que_cabe};
use ph2d_app_light2d::{config_da_vista, config_do_entorno, grade_do_quadro, regiao_da_janela};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Solver};
use ph2d_render::Camera2d;

use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5o::{mediana, minimo};
use crate::medida_luz2d_w5v::{App, cena_do_smoke};

const RODADAS: usize = 9;
const AQUECE: usize = 3;
const QUADROS: usize = 6;
const RODADAS_FASES: usize = 4;

/// A leitura do arquivo no `de_fora`; o índice continua vivo (o `select` sobre ele).
const LEITURA: &str = "r = desempacotar(arquivo_dos_r[hrcs[iq()].fora_base[m / 4u][m % 4u] + i]).r;";
const SEM_LER: &str = "r = select(ceu, 0.5 * ceu, i == 7u);";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Var {
    Ceu,
    Borda,
    SemLer,
    BordaControlo,
}

const VARS: [Var; 4] = [Var::Ceu, Var::Borda, Var::SemLer, Var::BordaControlo];

fn sem_ler(c: Config) -> Config {
    let mut fonte =
        ph2d_light2d_gpu::fonte_do_rc(c.aritmetica, c.armazenamento, c.hrc_niveis_em_textura())
            .to_string();
    assert_eq!(fonte.matches(LEITURA).count(), 1, "a agulha da leitura");
    fonte = fonte.replacen(LEITURA, SEM_LER, 1);
    Config {
        modulo_rc: Some(Box::leak(fonte.into_boxed_str())),
        ..c
    }
}

#[test]
#[ignore = "medição de placa: a borda nos níveis de cima (W6g §54.1)"]
fn medida_luz2d_borda_topo() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let alvo = std::env::var("PH2D_BORDA_TOPO_ALVO").map_or((3840, 2160), |v| {
        let (l, a) = v.split_once('x').expect("<l>x<a>");
        (l.parse().expect("largura"), a.parse().expect("altura"))
    });
    let nivel = match std::env::var("PH2D_BORDA_TOPO_NIVEL").as_deref() {
        Ok("low") => NivelDaLuz::Baixa,
        Ok("medium") => NivelDaLuz::Media,
        _ => NivelDaLuz::Alta,
    };
    let a = Camera2d::new([-4.2, -1.8], 3.8)
        .world_to_screen_affine(ph2d_host::WindowSize::new(alvo.0, alvo.1));
    eprintln!(
        "P placa {:?} · lado máx {}",
        gpu.adapter.get_info().name,
        gpu.device.limits().max_texture_dimension_2d
    );
    let cena = cena_do_smoke(5);
    let sem_ceu = CenaDeLuz {
        ceu: [0.0; 3],
        ..cena.clone()
    };
    // Como o app: a fina engrossa até caber no lado da placa (a 2160p, `k ≥ 1`).
    let tetos = Tetos {
        lado: gpu.device.limits().max_texture_dimension_2d,
        recusou: None,
    };
    let k = fina_que_cabe(a, alvo, degrau_do(nivel), tetos).expect("nada cabe na placa");
    let dg = degrau_do(nivel).com_fina_engrossada(k);
    let mut app = {
        let g = grade_do_quadro(a, alvo, dg).expect("a grade");
        let mut vista =
            Solver::novo(&gpu, config_da_vista(&gpu.device, g, a, dg)).expect("solver da vista");
        let (i, f) = regiao_da_janela(vista.grade_da_fluencia(), a, alvo);
        vista.regiao_fina(i, f);
        let mut entorno = config_do_entorno(&gpu.device, a, alvo, dg)
            .map(|c| Solver::novo(&gpu, c).expect("solver do entorno"));
        if let Some(e) = entorno.as_mut() {
            let g = vista.config().grade;
            e.mascarar_miolo(
                &gpu,
                Some([
                    g.origem_m[0],
                    g.origem_m[1],
                    g.origem_m[0] + f64::from(g.largura) * g.texel_m,
                    g.origem_m[1] + f64::from(g.altura) * g.texel_m,
                ]),
            );
        }
        App { vista, entorno }
    };
    let mut m = Solver::novo(&gpu, sem_ler(app.vista.config().clone())).expect("a vista M");
    let (i, f) = regiao_da_janela(m.grade_da_fluencia(), a, alvo);
    m.regiao_fina(i, f);
    let entorno = app.entorno.as_mut().expect("o entorno (com o arquivo da borda)");
    assert!(
        entorno.config().arquivo_da_borda.is_some(),
        "o entorno sem o arquivo: não há borda"
    );
    assert!(m.somar_entorno(&gpu, Some(&*entorno)));
    let c = app.vista.config();
    let rotulo = format!("{}×{} {nivel:?} fina k {k}", alvo.0, alvo.1);
    eprintln!(
        "O {rotulo} · grade {}×{} · load {}",
        c.grade.largura,
        c.grade.altura,
        loadavg().trim()
    );

    // Um quadro da variante `v`: o entorno, depois a vista dela com o estado da borda certo.
    let mut estado_p: Option<bool> = None;
    let mut quadro = |app: &mut App, m: &mut Solver, v: Var| -> f64 {
        let e = app.entorno.as_mut().expect("entorno");
        e.quadro(&gpu, &sem_ceu);
        let vista = if v == Var::SemLer {
            &mut *m
        } else {
            let borda = v != Var::Ceu;
            if estado_p != Some(borda) {
                let ok = app.vista.somar_entorno(&gpu, borda.then_some(&*e));
                assert!(ok, "o estado da borda");
                estado_p = Some(borda);
            }
            &mut app.vista
        };
        vista.quadro(&gpu, &cena);
        vista.tempos(&gpu).map_or(0.0, |t| t.c2_total())
    };

    let mut c2: Vec<Vec<f64>> = vec![Vec::new(); VARS.len()];
    for r in 0..RODADAS {
        for k in 0..VARS.len() {
            let iv = (k + r) % VARS.len();
            for _ in 0..AQUECE {
                quadro(&mut app, &mut m, VARS[iv]);
            }
            for _ in 0..QUADROS {
                let t = quadro(&mut app, &mut m, VARS[iv]);
                c2[iv].push(t);
            }
        }
        eprintln!("R rodada {r} · load {}", loadavg().trim());
    }
    let min: Vec<f64> = c2.iter().map(|v| minimo(v)).collect();
    for (iv, v) in VARS.iter().enumerate() {
        eprintln!(
            "C {rotulo} · {v:?} · c2 mín {:.3} · med {:.3} · − céu {:+.3}",
            min[iv],
            mediana(&c2[iv]),
            min[iv] - min[0]
        );
    }
    eprintln!(
        "C {rotulo} · A×A (controlo − borda) {:+.3} · conta (sem ler − céu) {:+.3} · leitura (borda − sem ler) {:+.3}",
        min[3] - min[1],
        min[2] - min[0],
        min[1] - min[2]
    );

    // As fases da descida por nível (o modo de fases liga em P e M; a `c2` dos tempos fica 0).
    assert!(app.vista.medir_fases_da_hrc(&gpu) && m.medir_fases_da_hrc(&gpu));
    type PorNivel = Vec<(&'static str, u32, Vec<f64>)>;
    let mut fases: Vec<PorNivel> = vec![Vec::new(); 3];
    for r in 0..RODADAS_FASES {
        for k in 0..3 {
            let iv = (k + r) % 3;
            for q in 0..AQUECE + QUADROS {
                quadro(&mut app, &mut m, VARS[iv]);
                if q < AQUECE {
                    continue;
                }
                let s = if VARS[iv] == Var::SemLer { &m } else { &app.vista };
                for (etapa, n, ms, _) in s.fases_da_hrc(&gpu).expect("fases") {
                    match fases[iv].iter_mut().find(|o| o.0 == etapa && o.1 == n) {
                        Some(o) => o.2.push(ms),
                        None => fases[iv].push((etapa, n, vec![ms])),
                    }
                }
            }
        }
    }
    let ler = |iv: usize, etapa: &str, n: u32| {
        fases[iv]
            .iter()
            .find(|o| o.0 == etapa && o.1 == n)
            .map_or(0.0, |o| minimo(&o.2))
    };
    let mut chaves: Vec<(&'static str, u32)> = fases[0].iter().map(|o| (o.0, o.1)).collect();
    chaves.sort_by_key(|k| (k.0, k.1));
    let mut soma = [0.0_f64; 3];
    for (etapa, n) in chaves {
        let v = [ler(0, etapa, n), ler(1, etapa, n), ler(2, etapa, n)];
        for (s, x) in soma.iter_mut().zip(v) {
            *s += x;
        }
        if (v[1] - v[0]).abs() < 0.02 {
            continue;
        }
        eprintln!(
            "F {rotulo} · {etapa} n {n:>2} · céu {:.3} · sem ler {:.3} · borda {:.3} · conta {:+.3} · leitura {:+.3}",
            v[0],
            v[2],
            v[1],
            v[2] - v[0],
            v[1] - v[2]
        );
    }
    eprintln!(
        "F {rotulo} · soma das fases · céu {:.3} · sem ler {:.3} · borda {:.3} · load {}",
        soma[0],
        soma[2],
        soma[1],
        loadavg().trim()
    );
}
