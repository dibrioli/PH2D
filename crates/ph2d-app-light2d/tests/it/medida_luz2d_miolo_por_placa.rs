//! ⭐ Item 4 (linha Luz2D, 24/10) — **onde vai o tempo do miolo em cada placa**: as 8 fases do
//! instrumento W5o ([`Solver::medir_fases_do_miolo`]) nas duas cenas de referência — a do bug 7 (o
//! portão `o_custo_do_miolo`, disco opaco na grade 2560×1536, região a metade) e a vista vazia Low a
//! 1080p da W5o —, o mínimo de [`RODADAS`] rodadas intercaladas, ao lado do `c3_miolo` inteiro (sem o
//! instrumento) do mesmo processo. Resultados: `docs/Luz2D/BUGS_luz2d.md` bug 7.

use bevy_ecs::world::World;
use ph2d_app_light2d::qualidade::degrau_do;
use ph2d_app_light2d::regiao_da_janela;
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_gpu::{Config, Grade, HOLOGRAFICA_PADRAO, MIOLO_PADRAO, Solver};

use crate::medida_luz2d_w5e::{config_do_produto, fabrica};

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 9;
const QUADROS: usize = 3;

/// A cena e o solver do portão `o_custo_do_miolo` (crate `ph2d-light2d-gpu`), com a base e o bloco de
/// fábrica.
pub(crate) fn caso_do_bug7(
    gpu: &ph2d_gpu::GpuContext,
    com: &dyn Fn(Config) -> Config,
) -> (Solver, CenaDeLuz) {
    const L: u32 = 2560;
    const A: u32 = 1536;
    const T: f64 = 0.004;
    let c = [
        (f64::from(L) * 0.5 * T) as f32,
        (f64::from(A) * 0.5 * T) as f32,
    ];
    let vertices = (0..96)
        .map(|k| {
            let a = 0.1 + std::f32::consts::TAU * k as f32 / 96.0;
            [c[0] + 0.9 * a.cos(), c[1] + 0.9 * a.sin()]
        })
        .collect();
    let cena = CenaDeLuz {
        emissores: vec![Emissor::Disco {
            centro: [c[0] - 1.6, c[1] + 0.4],
            raio: 0.15,
            radiancia: [1.0; 3],
            alcance_m: f32::INFINITY,
        }],
        oclusores: vec![Oclusor {
            vertices,
            opacidade: 1.0,
            albedo: [0.0; 3],
        }],
        luzes: 1,
        ..Default::default()
    };
    let g = Grade {
        origem_m: [0.0, 0.0],
        texel_m: T,
        largura: L,
        altura: A,
    };
    let cfg = com(Config {
        holografica: Some(HOLOGRAFICA_PADRAO),
        miolo: Some(MIOLO_PADRAO),
        ..Config::padrao_para(&gpu.device, g)
    });
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    s.regiao_fina([L / 4, A / 4], [L * 3 / 4, A * 3 / 4]);
    (s, cena)
}

/// A vista vazia Low a 1080p da W5o (a do portão `a_nitidez_da_luz`), sem o entorno.
pub(crate) fn caso_vazio_low(
    gpu: &ph2d_gpu::GpuContext,
    com: &dyn Fn(Config) -> Config,
) -> (Solver, CenaDeLuz) {
    caso_da_vista(gpu, com, None)
}

/// A vista Low a 1080p com a cena `n` do smoke (os corpos: o miolo com conteúdo).
pub(crate) fn caso_cena5_low(
    gpu: &ph2d_gpu::GpuContext,
    com: &dyn Fn(Config) -> Config,
) -> (Solver, CenaDeLuz) {
    caso_da_vista(gpu, com, Some(5))
}

fn caso_da_vista(
    gpu: &ph2d_gpu::GpuContext,
    com: &dyn Fn(Config) -> Config,
    n: Option<u32>,
) -> (Solver, CenaDeLuz) {
    let mut w = World::new();
    match n {
        Some(n) => assert!(ph2d_app_light2d::smoke::encenar(&mut w, n)),
        None => {
            w.spawn((
                ph2d_light2d_ecs::Luz::default(),
                ph2d_ecs::Transform::IDENTITY,
            ));
        }
    }
    let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
    let a = fabrica(FHD);
    let produto = config_do_produto(a, FHD, degrau_do(NivelDaLuz::Baixa)).expect("grade");
    let aritmetica = Config::padrao_para(&gpu.device, produto.grade).aritmetica;
    let mut v = Solver::novo(
        gpu,
        com(Config {
            aritmetica,
            ..produto
        }),
    )
    .expect("solver da vista");
    let (i, f) = regiao_da_janela(v.grade_da_fluencia(), a, FHD);
    v.regiao_fina(i, f);
    (v, cena)
}

#[test]
#[ignore = "medição (placa): as fases do miolo nesta placa"]
fn medida_luz2d_miolo_por_placa() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    if !gpu
        .device
        .features()
        .contains(wgpu::Features::TIMESTAMP_QUERY)
    {
        eprintln!("medida: sem TIMESTAMP_QUERY");
        return;
    }
    let info = gpu.adapter.get_info();
    eprintln!(
        "# miolo por placa · {} · {} {} · load {}",
        info.name,
        info.driver,
        info.driver_info,
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    let id = |c: Config| c;
    for (rotulo, faz) in [
        (
            "bug 7 (o_custo_do_miolo)",
            caso_do_bug7 as fn(&_, &dyn Fn(Config) -> Config) -> _,
        ),
        ("vazia Low 1080p (W5o)", caso_vazio_low),
    ] {
        let (mut total, cena) = faz(&gpu, &id);
        let (mut fases, _) = faz(&gpu, &id);
        assert!(fases.medir_fases_do_miolo(&gpu));
        for _ in 0..QUADROS {
            total.quadro(&gpu, &cena);
            fases.quadro(&gpu, &cena);
        }
        let _ = total.tempos(&gpu);
        let mut min_total = f64::INFINITY;
        let mut min_fases: Vec<(&'static str, f64, usize)> = Vec::new();
        for r in 0..RODADAS {
            for o in 0..2 {
                if (o + r) % 2 == 0 {
                    for _ in 0..QUADROS {
                        total.quadro(&gpu, &cena);
                    }
                    min_total = min_total.min(total.tempos(&gpu).expect("tempos").c3_miolo);
                } else {
                    for _ in 0..QUADROS {
                        fases.quadro(&gpu, &cena);
                    }
                    let f = fases.fases_do_miolo(&gpu).expect("fases");
                    if min_fases.is_empty() {
                        min_fases = f;
                    } else {
                        for (m, x) in min_fases.iter_mut().zip(f) {
                            m.1 = m.1.min(x.1);
                        }
                    }
                }
            }
        }
        let g = total.grade_da_fluencia();
        let soma: f64 = min_fases.iter().map(|x| x.1).sum();
        eprintln!(
            "\n## {rotulo} · fluência {}×{} · níveis {:?} · c3_miolo inteiro (mín {RODADAS}) {min_total:.3} ms · soma das fases {soma:.3} ms",
            g.largura,
            g.altura,
            total.config().niveis_do_miolo()
        );
        for (n, ms, d) in &min_fases {
            eprintln!(
                "  {n:<26} {ms:>8.3} ms  {:>5.1} %  {d:>4} despachos",
                100.0 * ms / soma
            );
        }
    }
    eprintln!(
        "# load depois {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
}

type Caso = fn(&ph2d_gpu::GpuContext, &dyn Fn(Config) -> Config) -> (Solver, CenaDeLuz);

/// ⭐ Item 4 — as FORMAS do miolo (`Miolo::salta_o_fora`; a do ladrilho, recusada, em
/// `docs/Luz2D/ferramentas/item4_recusado/`) contra o produto no MESMO processo: o `c3_miolo` (o
/// mínimo de [`RODADAS`] intercaladas, o produto duas vezes: o controlo A × A) e a fluência pela lei do
/// mesmo bit por placa (`comum::mesmo_bit_por_placa`), cada solver com o mesmo número de quadros.
#[test]
#[ignore = "medição (placa): as formas do miolo × o produto, relógio e bits"]
fn medida_luz2d_miolo_formas() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let info = gpu.adapter.get_info();
    eprintln!(
        "# formas do miolo · {} · {} {} · load {}",
        info.name,
        info.driver,
        info.driver_info,
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    // (nome, salta o fora, um ciclo a menos — o controlo de que a régua dos bits VÊ o miolo).
    const FORMAS: [(&str, bool, bool); 4] = [
        ("antes (grava o fora)", false, false),
        ("salta o fora (o produto)", true, false),
        ("antes (controlo A × A)", false, false),
        ("um ciclo a menos (controlo dos bits)", false, true),
    ];
    let casos: [(&str, Caso); 3] = [
        ("bug 7 (o_custo_do_miolo)", caso_do_bug7),
        ("vazia Low 1080p (W5o)", caso_vazio_low),
        ("cena 5 Low 1080p", caso_cena5_low),
    ];
    let mut bits = Vec::new();
    let mut controlos = 0;
    for (rotulo, faz) in casos {
        let mut s: Vec<(Solver, CenaDeLuz)> = FORMAS
            .iter()
            .map(|&(_, salta, menos)| {
                faz(&gpu, &|c: Config| Config {
                    miolo: c.miolo.map(|m| ph2d_light2d_gpu::Miolo {
                        salta_o_fora: salta,
                        ciclos: m.ciclos - u32::from(menos),
                        ..m
                    }),
                    ..c
                })
            })
            .collect();
        for (x, c) in &mut s {
            for _ in 0..QUADROS {
                x.quadro(&gpu, c);
            }
            let _ = x.tempos(&gpu);
        }
        let mut minimo = [f64::INFINITY; FORMAS.len()];
        for r in 0..RODADAS {
            for o in 0..s.len() {
                let k = (o + r) % s.len();
                let (x, c) = &mut s[k];
                for _ in 0..QUADROS {
                    x.quadro(&gpu, c);
                }
                minimo[k] = minimo[k].min(x.tempos(&gpu).expect("tempos").c3_miolo);
            }
        }
        let n = FORMAS.len() - 1;
        eprintln!("\n## {rotulo} · c3_miolo (mín {RODADAS}) ms");
        for (k, (nome, _, _)) in FORMAS.iter().enumerate() {
            eprintln!(
                "  {nome:<28} {:>8.3}  razão {:.3}",
                minimo[k],
                minimo[k] / minimo[0]
            );
        }
        let fl: Vec<_> = s
            .iter()
            .map(|(x, _)| crate::comum::fluencia_lida(&gpu, x).v)
            .collect();
        for k in 1..FORMAS.len() - 1 {
            bits.push((
                format!("{rotulo} · {} × antes", FORMAS[k].0),
                fl[0].clone(),
                fl[k].clone(),
            ));
        }
        let vistos = fl[0].iter().zip(&fl[n]).filter(|(x, y)| x != y).count();
        eprintln!(
            "CONTROLO · {rotulo} · {}: {vistos} texels diferentes do antes",
            FORMAS[n].0
        );
        controlos += usize::from(vistos > 0);
    }
    for (onde, a, b) in &bits {
        let d = a.iter().zip(b).filter(|(x, y)| x != y).count();
        eprintln!("BITS · {onde}: {d} texels diferentes de {}", a.len());
    }
    eprintln!(
        "# load depois {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    assert!(
        controlos > 0,
        "o controlo (um ciclo a menos) não mudou a fluência em caso nenhum: a régua dos bits é cega"
    );
    for (onde, a, b) in &bits {
        crate::comum::mesmo_bit_por_placa(&gpu, a, b, onde);
    }
}
