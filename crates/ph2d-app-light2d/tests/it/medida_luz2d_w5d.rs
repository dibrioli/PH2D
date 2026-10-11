//! **A rodada da W5d — a precisão junto à borda** (`docs/Luz2D/01_plano.md` §7; método
//! `docs/DevOps/MEDIR_VELOCIDADE.md`): um lote, as variantes no MESMO processo (configurações do
//! solver, nunca `const` recompilada), intercaladas, vale o MÍNIMO. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5d -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A. as FACETAS das formas curvas (a cena 5): o disco e a elipse feitos POLÍGONOS de [`LADOS`] lados
//!    (o gabarito lê sprites como quads), a luz da placa lida em anéis dentro e logo fora de cada um
//!    contra o gabarito — os pontos errados e o EXCESSO de variação ao longo do anel (uma faceta é um
//!    salto que o gabarito não tem), por variante × zoom × posição na grade;
//! B. o interior da caixa (o gate `o_interior_do_opaco`) por variante: o pior texel e os errados;
//! C. o custo da C3 (o MÍNIMO de [`RODADAS`] intercaladas) nas cinco cenas, por variante.

use bevy_ecs::world::World;
use ph2d_app_light2d::{
    DEGRAU_PADRAO, captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela, smoke,
};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};
use ph2d_render::Camera2d;
use vello::kurbo::{Affine, Point};

use crate::as_formas_redondas::{
    AMOSTRAS, ANEIS, LADOS, Leitura, Redondo, anel, cena_redonda, ler_anel, vista_em,
};
use crate::comum::{Fluencia, fluencia_lida, gpu};
use crate::medida_luz2d_w4b::loadavg;

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 7;

/// Uma variante da C3.
#[derive(Clone, Copy)]
struct Var {
    nome: &'static str,
    /// A borda exata no interior (`Config::borda_exata`, com a cobertura dos sprites pelo meio).
    borda: bool,
    /// A meia-largura da média ao longo da face (`Config::c0_ao_longo_da_face`).
    face: f32,
}

const VARIANTES: [Var; 3] = [
    Var {
        nome: "W5c",
        borda: false,
        face: 0.0,
    },
    Var {
        nome: "borda",
        borda: true,
        face: 0.0,
    },
    Var {
        nome: "borda+face 2",
        borda: true,
        face: 2.0,
    },
];

/// As variantes medidas COM o rebote (contra a série do gabarito).
const COM_REBOTE: [Var; 3] = VARIANTES;

fn fabrica() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

fn vista(zoom: f64) -> Affine {
    let centro = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    fabrica().then_scale_about(zoom, centro)
}

/// A configuração do produto (sem rebote: o gabarito ponto a ponto é o direto + as cascatas) na
/// vista `a`, com a variante.
fn config(gpu: &GpuContext, a: Affine, v: Var) -> Config {
    let dg = DEGRAU_PADRAO;
    Config {
        fina: fina_do_quadro(a, dg),
        captura: captura_do_quadro(a, dg),
        holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
        rebote: false,
        c0_ao_longo_da_face: v.face,
        borda_exata: v.borda,
        ..Config::padrao_para(&gpu.device, grade_do_quadro(a, FHD, dg).expect("grade"))
    }
}

fn solver(gpu: &GpuContext, a: Affine, cfg: Config) -> Solver {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
    s.regiao_fina(i, f);
    s
}

/// B. O rebote (a linha de base e as variantes): a convergência em quadros (K9) e o rebote de FORA
/// da grade — as réguas da W5c, por `PH2D_W5D_REBOTE=1`.
#[test]
#[ignore = "a rodada da W5d, bloco B: corre-se à mão"]
fn medida_luz2d_w5d_rebote() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!("# Rodada W5d · rebote · loadavg antes {}", loadavg());
    crate::medida_luz2d_w5c::convergencia(&gpu);
    crate::medida_luz2d_w5c::fora_da_grade(&gpu);
    entorno(&gpu);
    custo_do_entorno(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// B1. O preço do entorno por nível, na vista de fábrica a 1920×1080 (cenas 3 e 5): o tempo de placa
/// (o MÍNIMO de 7 rodadas intercaladas) da vista e do entorno, e a memória de cada um.
fn custo_do_entorno(gpu: &GpuContext) {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_app_light2d::{ENTORNO_PADRAO, MARGEM_DO_ENTORNO};
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    eprintln!("\n## B1. O preço do entorno (ms, o mínimo de 7; MB alocados)");
    let a = vista(1.0);
    let cenas: Vec<CenaDeLuz> = [3, 5]
        .iter()
        .map(|&n| {
            let mut w = World::new();
            assert!(smoke::encenar(&mut w, n));
            cena_de_luz(&w)
        })
        .collect();
    for (nome, nivel) in [
        ("Low", NivelDaLuz::Baixa),
        ("Medium", NivelDaLuz::Media),
        ("High", NivelDaLuz::Alta),
    ] {
        let d = degrau_do(nivel);
        let g = grade_do_quadro(a, FHD, d).expect("grade");
        let de = ph2d_app_light2d::Degrau {
            c0_px: d.c0_px * ENTORNO_PADRAO,
            margem: MARGEM_DO_ENTORNO,
            ..d
        };
        let ge = grade_do_quadro(a, FHD, de).expect("grade do entorno");
        let mut pares: Vec<(Solver, Solver)> = cenas
            .iter()
            .map(|_| {
                let mut s = solver(
                    gpu,
                    a,
                    Config {
                        fina: fina_do_quadro(a, d),
                        captura: captura_do_quadro(a, d),
                        holografica: Some(HOLOGRAFICA_PADRAO),
                        rebote: true,
                        ..Config::padrao_para(&gpu.device, g)
                    },
                );
                let mut e = Solver::novo(
                    gpu,
                    Config {
                        holografica: Some(HOLOGRAFICA_PADRAO),
                        rebote: true,
                        ..Config::padrao_para(&gpu.device, ge)
                    },
                )
                .expect("o entorno");
                e.regiao_fina([0; 2], [0; 2]);
                let gv = s.config().grade;
                e.mascarar_miolo(
                    gpu,
                    Some([
                        gv.origem_m[0],
                        gv.origem_m[1],
                        gv.origem_m[0] + f64::from(gv.largura) * gv.texel_m,
                        gv.origem_m[1] + f64::from(gv.altura) * gv.texel_m,
                    ]),
                );
                s.somar_entorno(gpu, Some(&e));
                (s, e)
            })
            .collect();
        let mut minimo = vec![[f64::INFINITY; 2]; cenas.len()];
        for rodada in 0..RODADAS {
            for j in 0..cenas.len() {
                let i = (j + rodada) % cenas.len();
                let (s, e) = &mut pares[i];
                let sem_ceu = CenaDeLuz {
                    ceu: [0.0; 3],
                    ..cenas[i].clone()
                };
                for _ in 0..3 {
                    e.quadro(gpu, &sem_ceu);
                    s.quadro(gpu, &cenas[i]);
                }
                let (ts, te) = (
                    s.tempos(gpu).expect("TIMESTAMP_QUERY"),
                    e.tempos(gpu).expect("TIMESTAMP_QUERY"),
                );
                minimo[i][0] = minimo[i][0].min(ts.total());
                minimo[i][1] = minimo[i][1].min(te.total());
            }
        }
        let (s, e) = &pares[0];
        eprintln!(
            "- {nome}: a vista {}×{} · {:.0} MB · o entorno {}×{} · {:.0} MB · cena 3 {:.2} + {:.2} ms · cena 5 {:.2} + {:.2} ms",
            g.largura,
            g.altura,
            s.pass_list().bytes_alocados() as f64 / 1e6,
            ge.largura,
            ge.altura,
            e.pass_list().bytes_alocados() as f64 / 1e6,
            minimo[0][0],
            minimo[0][1],
            minimo[1][0],
            minimo[1][1]
        );
    }
}

/// B1. O ENTORNO: um 2.º solver mais grosso (`fator` × o texel) e 4× mais largo, sem céu e com a
/// emissão do miolo (a grade da vista) apagada — a c0 dele soma-se à da vista. A mesma régua da W5c
/// (o ponto junto à borda esquerda da vista de 2 m, o vermelho contra a série).
fn entorno(gpu: &GpuContext) {
    use ph2d_app_light2d::qualidade::degrau_do;
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    use ph2d_light2d_ref::{OpcoesDoRebote, Rebote};
    eprintln!("\n## B1. O entorno (o vermelho junto à borda esquerda, contra a série; 80 quadros)");
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let c3 = cena_de_luz(&w);
    let sem_ceu = CenaDeLuz {
        ceu: [0.0; 3],
        ..c3.clone()
    };
    let serie = Rebote::resolver(
        &c3,
        OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 200,
        },
    );
    for centro_x in [-2.0_f32, 0.0, 2.0] {
        let a = Camera2d::new([centro_x, 0.0], 2.0)
            .world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        let d = degrau_do(NivelDaLuz::Media);
        let g = grade_do_quadro(a, FHD, d).expect("grade");
        let esquerda = (a.inverse() * Point::new(0.0, 540.0)).x;
        let p = [esquerda.max(-2.7) + 0.3, 0.0];
        let gab = 2.0 * std::f64::consts::PI * serie.luz_em(p)[0];
        let mut linha = format!("- câmera em x = {centro_x} · ponto {:.2}:", p[0]);
        for fator in [0.0_f64, 4.0, 8.0] {
            let cfg = Config {
                fina: fina_do_quadro(a, d),
                captura: captura_do_quadro(a, d),
                holografica: Some(HOLOGRAFICA_PADRAO),
                rebote: true,
                ..Config::padrao_para(&gpu.device, g)
            };
            let mut s = Solver::novo(gpu, cfg).expect("solver");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
            s.regiao_fina(i, f);
            let mut e = (fator > 0.0).then(|| {
                let de = ph2d_app_light2d::Degrau {
                    c0_px: d.c0_px * fator,
                    margem: 1.5,
                    ..d
                };
                let ge = grade_do_quadro(a, FHD, de).expect("grade do entorno");
                let mut e = Solver::novo(
                    gpu,
                    Config {
                        holografica: Some(HOLOGRAFICA_PADRAO),
                        rebote: true,
                        ..Config::padrao_para(&gpu.device, ge)
                    },
                )
                .expect("o entorno");
                e.regiao_fina([0; 2], [0; 2]);
                let gv = s.config().grade;
                assert!(e.mascarar_miolo(
                    gpu,
                    Some([
                        gv.origem_m[0],
                        gv.origem_m[1],
                        gv.origem_m[0] + f64::from(gv.largura) * gv.texel_m,
                        gv.origem_m[1] + f64::from(gv.altura) * gv.texel_m,
                    ])
                ));
                e
            });
            for _ in 0..80 {
                if let Some(e) = e.as_mut() {
                    e.quadro(gpu, &sem_ceu);
                }
                assert!(s.somar_entorno(gpu, e.as_ref()));
                s.quadro(gpu, &c3);
            }
            let fl = fluencia_lida(gpu, &s);
            let nome = if fator > 0.0 {
                format!("entorno {fator}×")
            } else {
                "sem entorno".into()
            };
            linha += &format!(" · {nome} {:+.3}", (fl.amostra_canal(p, 0) - gab) / gab);
        }
        eprintln!("{linha}");
    }
}

#[test]
#[ignore = "a rodada da W5d: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w5d() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W5d · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
        loadavg()
    );
    facetas(&gpu);
    disco_de_imagem(&gpu);
    interior_da_caixa(&gpu);
    custo(&gpu);
    eprintln!("# loadavg depois {}", loadavg());
}

/// A. As facetas do disco e da elipse.
fn facetas(gpu: &GpuContext) {
    eprintln!("\n## A. As facetas das formas curvas (cena 5, polígonos de {LADOS} lados)");
    eprintln!(
        "(por anel: errados · pior · TV = excesso de variação · × = cruzamentos a mais numa passagem luz/sombra · ↔ = maior desvio dela em texels)"
    );
    let mut total = [Leitura::default(); VARIANTES.len()];
    for zoom in [1.0, 2.5] {
        for ri in 0..2 {
            for k in [0u32, 3] {
                let base = cena_redonda([0.0; 2]).1[ri];
                let a = vista_em(zoom, base.centro);
                let tx = {
                    let cfg = config(gpu, a, VARIANTES[0]);
                    cfg.grade.texel_m / f64::from(cfg.fina)
                };
                let d = [f64::from(k) * tx / 8.0, f64::from(k) * tx * 3.0 / 8.0];
                let (c, redondos) = cena_redonda(d);
                let r = redondos[ri];
                let fls: Vec<Fluencia> = VARIANTES
                    .iter()
                    .map(|v| {
                        let mut s = solver(gpu, a, config(gpu, a, *v));
                        s.quadro(gpu, &c);
                        fluencia_lida(gpu, &s)
                    })
                    .collect();
                let aneis: Vec<_> = ANEIS
                    .iter()
                    .map(|(nome, f, fora)| (*nome, anel(&c, &r, *f, fora * tx)))
                    .collect();
                let max = aneis
                    .iter()
                    .flat_map(|(_, (_, v))| v.iter().copied())
                    .fold(0.0, f64::max);
                for (vi, v) in VARIANTES.iter().enumerate() {
                    let mut linha =
                        format!("- zoom {zoom} · desl {k}/8 · {} · {}:", r.nome, v.nome);
                    for (nome, (pts, gab)) in &aneis {
                        let passo = pts
                            .windows(2)
                            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
                            .sum::<f64>()
                            / (pts.len() - 1) as f64
                            / tx;
                        let l =
                            ler_anel(&fls[vi], pts, gab, max, passo, |p| r.ao_eixo(p) < 2.0 * tx);
                        linha += &format!(
                            " | {nome}: {} err ({:.3}) TV {:+.2} ×{} ↔{:.1}",
                            l.errados, l.pior, l.tv, l.cruzes, l.desvio
                        );
                        total[vi].somar(l);
                    }
                    eprintln!("{linha}");
                }
                if zoom > 2.0 && k == 0 {
                    rebote(gpu, &c, &r, a, tx, &fls);
                }
            }
        }
    }
    for (vi, v) in VARIANTES.iter().enumerate() {
        let t = total[vi];
        eprintln!(
            "- TOTAL {}: {} errados · pior {:.3} · TV {:+.2} · cruzamentos a mais {} · desvio {:.1} tx",
            v.nome, t.errados, t.pior, t.tv, t.cruzes, t.desvio
        );
    }
}

/// O corpo com o REBOTE (60 quadros) contra a SÉRIE do gabarito, por variante de [`COM_REBOTE`]:
/// os anéis e as fotos (as de [`VARIANTES`] sem rebote junto).
fn rebote(gpu: &GpuContext, c: &CenaDeLuz, r: &Redondo, a: Affine, tx: f64, sem: &[Fluencia]) {
    let serie = ph2d_light2d_ref::Rebote::resolver(
        c,
        ph2d_light2d_ref::OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 60,
        },
    );
    let aneis: Vec<(&str, Vec<[f64; 2]>, Vec<f64>)> = ANEIS
        .iter()
        .map(|(nome, f, fora)| {
            let pts: Vec<[f64; 2]> = (0..AMOSTRAS)
                .map(|k| {
                    r.no_anel(
                        std::f64::consts::TAU * k as f64 / AMOSTRAS as f64,
                        *f,
                        fora * tx,
                    )
                })
                .collect();
            let v = pts
                .iter()
                .map(|p| std::f64::consts::TAU * serie.luz_em(*p)[1])
                .collect();
            (*nome, pts, v)
        })
        .collect();
    let max = aneis
        .iter()
        .flat_map(|(_, _, v)| v.iter().copied())
        .fold(0.0, f64::max);
    let mut todas: Vec<(String, Fluencia)> = Vec::new();
    for v in COM_REBOTE {
        let cfg = Config {
            rebote: true,
            ..config(gpu, a, v)
        };
        let mut s = solver(gpu, a, cfg);
        for _ in 0..60 {
            s.quadro(gpu, c);
        }
        let fl = fluencia_lida(gpu, &s);
        let mut linha = format!("- REBOTE · {} · {}:", r.nome, v.nome);
        for (nome, pts, gab) in &aneis {
            let passo = pts
                .windows(2)
                .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
                .sum::<f64>()
                / (pts.len() - 1) as f64
                / tx;
            let l = ler_anel(&fl, pts, gab, max, passo, |p| r.ao_eixo(p) < 2.0 * tx);
            linha += &format!(
                " | {nome}: {} err ({:.3}) TV {:+.2} ×{} ↔{:.1}",
                l.errados, l.pior, l.tv, l.cruzes, l.desvio
            );
        }
        eprintln!("{linha}");
        todas.push((format!("r_{}", v.nome), fl));
    }
    let mut fotos_: Vec<(String, &Fluencia)> = VARIANTES
        .iter()
        .zip(sem)
        .enumerate()
        .map(|(i, (v, f))| (format!("{}_{}", i + 1, v.nome), f))
        .collect();
    fotos_.extend(todas.iter().map(|(n, f)| (n.clone(), f)));
    fotos(c, &fotos_, r, tx, &serie);
}

/// As fotos de um corpo redondo — o GABARITO e cada variante (a cor × a luz, tonemap `x/(1+x)`, gama
/// 2,2 — para OLHAR): `target/fotos_w5d/<corpo>_<variante>.ppm`.
fn fotos(
    c: &CenaDeLuz,
    fls: &[(String, &Fluencia)],
    r: &Redondo,
    tx: f64,
    serie: &ph2d_light2d_ref::Rebote,
) {
    use std::io::Write;
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fotos_w5d");
    std::fs::create_dir_all(&dir).expect("pasta");
    let cor = smoke::CORPOS_DO_VOLUME
        .iter()
        .find(|k| k.nome == r.nome)
        .expect("o corpo")
        .cor;
    let (x0, y1) = (r.centro[0] - r.semi[0] - 0.1, r.centro[1] + r.semi[1] + 0.1);
    let (w, h) = (
        ((2.0 * r.semi[0] + 0.2) / tx) as u32,
        ((2.0 * r.semi[1] + 0.2) / tx) as u32,
    );
    let ponto = |i: u32, j: u32| {
        [
            x0 + (f64::from(i) + 0.5) * tx,
            y1 - (f64::from(j) + 0.5) * tx,
        ]
    };
    // O gabarito, por linhas em paralelo.
    let linhas: Vec<u32> = (0..h).collect();
    let gab: Vec<[f64; 3]> = std::thread::scope(|s| {
        let partes: Vec<_> = linhas
            .chunks(h.div_ceil(12) as usize)
            .map(|js| {
                s.spawn(move || {
                    js.iter()
                        .flat_map(|&j| {
                            (0..w).map(move |i| ph2d_light2d_ref::fluencia(c, ponto(i, j)))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        partes
            .into_iter()
            .flat_map(|p| p.join().expect("linha"))
            .collect()
    });
    // O gabarito COM a série do rebote, em blocos de 3 × 3 texels (cada ponto custa a série inteira).
    let (w3, h3) = (w.div_ceil(3), h.div_ceil(3));
    let linhas3: Vec<u32> = (0..h3).collect();
    let gab_r: Vec<[f64; 3]> = std::thread::scope(|s| {
        let partes: Vec<_> = linhas3
            .chunks(h3.div_ceil(12) as usize)
            .map(|js| {
                s.spawn(move || {
                    js.iter()
                        .flat_map(|&j| {
                            (0..w3).map(move |i| {
                                let l = serie.luz_em(ponto(3 * i + 1, 3 * j + 1));
                                l.map(|x| std::f64::consts::TAU * x)
                            })
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        partes
            .into_iter()
            .flat_map(|p| p.join().expect("linha"))
            .collect()
    });
    let gravar = |nome: &str, luz: &dyn Fn(u32, u32, usize) -> f64| {
        let mut px = Vec::with_capacity((w * h * 3) as usize);
        for j in 0..h {
            for i in 0..w {
                let p = ponto(i, j);
                let u = [
                    (p[0] - r.centro[0]) / r.semi[0],
                    (p[1] - r.centro[1]) / r.semi[1],
                ];
                let alb = if u[0] * u[0] + u[1] * u[1] <= 1.0 {
                    [f64::from(cor[0]), f64::from(cor[1]), f64::from(cor[2])]
                } else {
                    [0.6, 0.6, 0.62]
                };
                for (canal, a) in alb.iter().enumerate() {
                    let l = a * luz(i, j, canal) * 0.25;
                    px.push(((l / (1.0 + l)).powf(1.0 / 2.2) * 255.0) as u8);
                }
            }
        }
        let nome = format!("{}_{nome}.ppm", r.nome.to_lowercase());
        let mut arq = std::fs::File::create(dir.join(&nome)).expect("foto");
        write!(arq, "P6\n{w} {h}\n255\n").expect("cabeçalho");
        arq.write_all(&px).expect("píxeis");
    };
    gravar("0_gabarito", &|i, j, canal| {
        gab[(j * w + i) as usize][canal]
    });
    gravar("0_gabarito_rebote", &|i, j, canal| {
        gab_r[((j / 3) * w3 + i / 3) as usize][canal]
    });
    for (nome, fl) in fls {
        let nome = nome.replace([' ', 'í', '+', 'é'], "_");
        gravar(&nome, &|i, j, canal| fl.amostra_canal(ponto(i, j), canal));
    }
    eprintln!("- fotos de {} em target/fotos_w5d/", r.nome);
}

/// B. O interior da caixa (o gate `o_interior_do_opaco`, a mesma régua) por variante.
fn interior_da_caixa(gpu: &GpuContext) {
    use crate::o_interior_do_opaco::{cena, erros, gabarito};
    eprintln!("\n## B. O interior da caixa (a régua do `o_interior_do_opaco`)");
    for zoom in [1.0, 2.5] {
        let a = vista(zoom);
        let tx = {
            let cfg = config(gpu, a, VARIANTES[0]);
            cfg.grade.texel_m / f64::from(cfg.fina)
        };
        for k in [0u32, 3, 4, 6] {
            let d = [
                (f64::from(k) * tx / 8.0) as f32,
                (f64::from(k) * tx * 3.0 / 8.0) as f32,
            ];
            let c = cena(d);
            let mut linha = format!("- zoom {zoom} · desl {k}/8:");
            let mut gab = None;
            for v in VARIANTES {
                let mut s = solver(gpu, a, config(gpu, a, v));
                s.quadro(gpu, &c);
                let fl = fluencia_lida(gpu, &s);
                let g = gab.get_or_insert_with(|| gabarito(fl.grade, &c, d).0);
                let (e, n) = erros(&fl, g);
                linha += &format!(" {} {n} (pior {e:.3}) ·", v.nome);
            }
            eprintln!("{linha}");
        }
    }
}

/// C. O custo da C3 (visibilidade + fina, o MÍNIMO de [`RODADAS`] intercaladas) por cena × variante,
/// na vista de fábrica.
fn custo(gpu: &GpuContext) {
    eprintln!(
        "\n## C. O custo da C3 — preenche (com a borda) + visibilidade + fina (ms, o mínimo de {RODADAS}; vista de fábrica, 1920×1080)"
    );
    let a = vista(1.0);
    let cenas: Vec<CenaDeLuz> = (1..=5)
        .map(|n| {
            let mut w = World::new();
            assert!(smoke::encenar(&mut w, n));
            cena_de_luz(&w)
        })
        .collect();
    let mut solvers: Vec<(usize, usize, Solver)> = Vec::new();
    for ci in 0..cenas.len() {
        for vi in 0..VARIANTES.len() {
            solvers.push((ci, vi, solver(gpu, a, config(gpu, a, VARIANTES[vi]))));
        }
    }
    let mut minimo = vec![f64::INFINITY; solvers.len()];
    for rodada in 0..RODADAS {
        for j in 0..solvers.len() {
            let i = (j + rodada) % solvers.len();
            let (ci, _, s) = &mut solvers[i];
            for _ in 0..3 {
                s.quadro(gpu, &cenas[*ci]);
            }
            let t = s.tempos(gpu).expect("TIMESTAMP_QUERY");
            minimo[i] = minimo[i].min(t.c3_preenche + t.c3_visibilidade + t.c3_fina);
        }
    }
    for ci in 0..cenas.len() {
        let mut linha = format!("- cena {}:", ci + 1);
        for vi in 0..VARIANTES.len() {
            linha += &format!(
                " {} {:.3} ·",
                VARIANTES[vi].nome,
                minimo[ci * VARIANTES.len() + vi]
            );
        }
        eprintln!("{linha}");
    }
}

/// As variantes do disco de IMAGEM: `(nome, borda exata, média ao longo da face)`.
const DO_SPRITE: [(&str, bool, f32); 2] = [("W5c", false, 0.0), ("borda+face 2", true, 2.0)];

/// A2. O disco da cena 5 como o PRODUTO o captura (o sprite do disco do atlas, por texel), com o rebote
/// desligado, contra o polígono do gabarito do mesmo raio.
fn disco_de_imagem(gpu: &GpuContext) {
    eprintln!("\n## A2. O disco de IMAGEM (o sprite do atlas, pelo caminho do produto)");
    let r = cena_redonda([0.0; 2]).1[0];
    for zoom in [1.0, 2.5] {
        let a = vista_em(zoom, r.centro);
        let base = config(gpu, a, VARIANTES[0]);
        let tx = base.grade.texel_m / f64::from(base.fina);
        let (_, r, gab_cena) = crate::as_formas_redondas::disco_de_imagem(gpu, a, base);
        let aneis: Vec<_> = ANEIS
            .iter()
            .map(|(nome, f, fora)| (*nome, anel(&gab_cena, &r, *f, fora * tx)))
            .collect();
        let max = aneis
            .iter()
            .flat_map(|(_, (_, v))| v.iter().copied())
            .fold(0.0, f64::max);
        let mut fls = Vec::new();
        for (nome, borda, face) in DO_SPRITE {
            let cfg = Config {
                borda_exata: borda,
                c0_ao_longo_da_face: face,
                ..base
            };
            let (fl, ..) = crate::as_formas_redondas::disco_de_imagem(gpu, a, cfg);
            let mut linha = format!("- zoom {zoom} · disco de imagem · {nome}:");
            for (anel_nome, (pts, gab)) in &aneis {
                let passo = crate::as_formas_redondas::passo_tx(pts, tx);
                let l = ler_anel(&fl, pts, gab, max, passo, |p| r.ao_eixo(p) < 2.0 * tx);
                linha += &format!(
                    " | {anel_nome}: {} err ({:.3}) TV {:+.2} ×{} ↔{:.1}",
                    l.errados, l.pior, l.tv, l.cruzes, l.desvio
                );
            }
            eprintln!("{linha}");
            fls.push((format!("s_{nome}"), fl));
        }
        if zoom > 2.0 {
            let lista: Vec<(String, &Fluencia)> = fls.iter().map(|(n, f)| (n.clone(), f)).collect();
            let serie = ph2d_light2d_ref::Rebote::resolver(
                &gab_cena,
                ph2d_light2d_ref::OpcoesDoRebote {
                    passo_m: 0.05,
                    rebotes: 0,
                },
            );
            fotos(&gab_cena, &lista, &r, tx, &serie);
        }
    }
}
