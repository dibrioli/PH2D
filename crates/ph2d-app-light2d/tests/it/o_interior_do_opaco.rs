//! ⭐⭐ **O interior de um opaco não depende de ONDE ele está na grade** (smoke do dono, 10/10: a
//! parede verde da cena 3 esticada e arrastada — a diagonal entre duas faces do interior sai em
//! escada e, num sítio, aparece um risco vertical a mais).
//!
//! A lei (F3, `ph2d_light2d_ref::fluencia`): o ponto de dentro recebe a luz do EXTERIOR mais próximo
//! — as faces encontram-se no eixo medial, num corte SECO. Um texel fino guarda a MÉDIA da lei sobre
//! a área dele: o texel que o eixo corta mistura as duas faces na proporção da área de cada lado (o
//! antisserrilhado de uma aresta). Guardar o valor do CENTRO dava a escada (a aresta amostrada) e,
//! com o eixo sobre uma coluna de centros, uma coluna inteira empatada com a cor de uma face só.
//!
//! A régua: uma caixa livre na sala da cena 3, deslocada por frações do texel fino em dois zooms,
//! pela configuração do produto; cada texel do interior contra o gabarito na MÉDIA sobre o texel
//! (supersamplado junto ao eixo). Sem rebote nas duas pontas (o gabarito é o
//! direto + as cascatas; a lei do interior é a mesma com ele).

use bevy_ecs::world::World;
use ph2d_app_light2d::{
    DEGRAU_PADRAO, captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela, smoke,
};
use ph2d_ecs::{GlobalTransform, Name, Transform};
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};
use ph2d_render::{Camera2d, Sprite};
use vello::kurbo::Point;

use crate::comum::{Fluencia, fluencia_lida, gpu};

const FHD: (u32, u32) = (1920, 1080);
/// A caixa: o centro e o lado (metros).
const CENTRO: [f32; 2] = [1.3, -0.2];
const LADO: [f32; 2] = [1.1, 2.2];
/// Subamostras por eixo na média do gabarito sobre um texel a menos de um texel do eixo medial (a
/// fração da área sai em degraus de `1/SUB`; com 6 o degrau já valia 5 % da diferença das faces).
const SUB: u32 = 12;
/// A faixa medida em volta do eixo medial (texels finos, na folga entre as duas faces mais perto).
const FAIXA: f64 = 4.0;
/// Um texel do interior está ERRADO a mais disto do gabarito, relativo ao máximo dele ali.
const ERRADO: f64 = 0.10;
/// Texels errados por caso no produto — MEDIDO (RTX 5060 Ti, `DEGRAU_PADRAO`): 0 ou 1 (um canto da
/// caixa, onde o eixo chega à borda); sem a média no eixo, 35 a 537 (a escada).
const TETO_ERRADOS: usize = 3;

/// A cena 3 com a parede verde feita uma caixa livre no meio da sala, deslocada `d` metros.
fn cena(d: [f32; 2]) -> CenaDeLuz {
    cena_em([CENTRO[0] + d[0], CENTRO[1] + d[1]], LADO)
}

/// A cena 3 com a parede verde feita uma caixa de `lado` em `centro`.
fn cena_em(centro: [f32; 2], lado: [f32; 2]) -> CenaDeLuz {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let mut q = w.query::<(&Name, &mut Sprite, &mut Transform, &mut GlobalTransform)>();
    let mut achou = false;
    for (n, mut s, mut t, mut g) in q.iter_mut(&mut w) {
        if n.as_str() == "Parede verde" {
            s.size = lado;
            t.translation.x = centro[0];
            t.translation.y = centro[1];
            *g = GlobalTransform::from_transform(*t);
            achou = true;
        }
    }
    assert!(achou, "a cena 3 tem a parede verde");
    cena_de_luz(&w)
}

/// `(a folga entre a face mais próxima de p e a segunda, a distância à borda)` em metros — `None`
/// fora da caixa. A folga é 0 no eixo medial.
fn folgas(p: [f64; 2], d: [f32; 2]) -> Option<(f64, f64)> {
    let cx = f64::from(CENTRO[0] + d[0]);
    let cy = f64::from(CENTRO[1] + d[1]);
    let (hx, hy) = (0.5 * f64::from(LADO[0]), 0.5 * f64::from(LADO[1]));
    let mut f = [
        p[0] - (cx - hx),
        (cx + hx) - p[0],
        p[1] - (cy - hy),
        (cy + hy) - p[1],
    ];
    if f.iter().any(|&x| x <= 0.0) {
        return None;
    }
    f.sort_by(f64::total_cmp);
    Some((f[1] - f[0], f[0]))
}

/// O gabarito nos texels do interior a menos de `FAIXA` texels do eixo medial e a mais de um da borda
/// (longe do eixo a lei é a de sempre, medida noutros gates): `(centro, média sobre o texel)`; e
/// quantos texels o eixo corta.
fn gabarito(
    g: ph2d_light2d_gpu::Grade,
    c: &CenaDeLuz,
    d: [f32; 2],
) -> (Vec<([f64; 2], f64)>, usize) {
    let tx = g.texel_m;
    let mut pts = Vec::new();
    let mut no_eixo = 0;
    for j in 0..g.altura {
        for i in 0..g.largura {
            let p = [
                g.origem_m[0] + (f64::from(i) + 0.5) * tx,
                g.origem_m[1] + (f64::from(j) + 0.5) * tx,
            ];
            let Some((eixo, borda)) = folgas(p, d) else {
                continue;
            };
            if borda <= tx || eixo >= FAIXA * tx {
                continue;
            }
            let v = if eixo < 2.0 * tx {
                no_eixo += usize::from(eixo < 0.5 * tx);
                let sub = if eixo < tx { SUB } else { SUB / 2 };
                let o = |k: u32| (f64::from(k) + 0.5) / f64::from(sub) - 0.5;
                let mut s = 0.0;
                for b in 0..sub {
                    for a in 0..sub {
                        s += ph2d_light2d_ref::fluencia(c, [p[0] + o(a) * tx, p[1] + o(b) * tx])[1];
                    }
                }
                s / f64::from(sub * sub)
            } else {
                ph2d_light2d_ref::fluencia(c, p)[1]
            };
            pts.push((p, v));
        }
    }
    (pts, no_eixo)
}

/// `(o pior erro, quantos texels erram mais que ERRADO)`, relativos ao máximo do gabarito ali.
fn erros(fl: &Fluencia, gab: &[([f64; 2], f64)]) -> (f64, usize) {
    let max = gab.iter().map(|x| x.1).fold(0.0, f64::max);
    gab.iter().fold((0.0, 0), |(pior, n), &(p, g)| {
        let e = (fl.amostra(p) - g).abs() / max;
        (pior.max(e), n + usize::from(e > ERRADO))
    })
}

/// ⭐⭐ **GATE — o texel que o eixo medial corta é a média das faces**, em qualquer posição: o produto
/// deixa no máximo `TETO_ERRADOS` texels errados por caso (os de um canto ou do encontro de três
/// faces); o controlo (o valor do ponto mais perto do centro) tem a escada.
#[test]
fn o_interior_do_opaco_nao_depende_da_posicao_na_grade() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let fabrica =
        Camera2d::default().world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1));
    let centro = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let dg = DEGRAU_PADRAO;
    let (mut falhas, mut controlo) = (Vec::new(), 0);
    for zoom in [1.0, 2.5] {
        let a = fabrica.then_scale_about(zoom, centro);
        let grade = grade_do_quadro(a, FHD, dg).expect("grade");
        let cfg = Config {
            fina: fina_do_quadro(a, dg),
            captura: captura_do_quadro(a, dg),
            holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
            rebote: false,
            ..Config::padrao_para(&gpu.device, grade)
        };
        let tx = grade.texel_m / f64::from(cfg.fina);
        for k in [0u32, 3, 4, 6] {
            let d = [
                (f64::from(k) * tx / 8.0) as f32,
                (f64::from(k) * tx * 3.0 / 8.0) as f32,
            ];
            let c = cena(d);
            let ler = |media: bool| {
                let mut s = Solver::novo(
                    &gpu,
                    Config {
                        media_no_eixo: media,
                        ..cfg
                    },
                )
                .expect("solver");
                let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
                s.regiao_fina(i, f);
                s.quadro(&gpu, &c);
                fluencia_lida(&gpu, &s)
            };
            let (produto, sem) = (ler(true), ler(false));
            let (gab, no_eixo) = gabarito(produto.grade, &c, d);
            assert!(
                no_eixo > 50,
                "a régua não achou o eixo medial ({no_eixo} texels)"
            );
            let (e, n) = erros(&produto, &gab);
            let (e0, n0) = erros(&sem, &gab);
            eprintln!(
                "interior · zoom {zoom} · deslocamento {k}/8 · {no_eixo} texels no eixo: pior {e:.3}, {n} errados (sem a média: {e0:.3}, {n0})"
            );
            controlo = controlo.max(n0);
            if n > TETO_ERRADOS {
                falhas.push(format!("zoom {zoom} · {k}/8: {n} (pior {e:.3})"));
            }
        }
    }
    assert!(
        controlo > 10 * TETO_ERRADOS,
        "o controlo sem a média não mostra a escada ({controlo} texels errados) — a régua não a vê"
    );
    assert!(
        falhas.is_empty(),
        "o interior do opaco muda com a posição na grade (mais de {TETO_ERRADOS} texels errados > {ERRADO}): {}",
        falhas.join(" · ")
    );
}

/// As fotos do interior (a caixa verde como o editor a pinta: a cor × a luz recebida, o tonemap
/// `x/(1+x)` e o gama 2,2 — para OLHAR), no zoom do dono, em quatro deslocamentos, sem e com a média
/// no eixo. Saída: `target/fotos_w5c_interior/*.ppm`.
#[test]
#[ignore = "as fotos do smoke do dono (10/10): corre-se à mão"]
fn fotos_do_interior() {
    use std::io::Write;
    let Some(gpu) = gpu() else {
        return;
    };
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fotos_w5c_interior");
    std::fs::create_dir_all(&dir).expect("pasta");
    let fabrica =
        Camera2d::default().world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1));
    let centro = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let zoom = 2.5;
    let a = fabrica.then_scale_about(zoom, centro);
    let [sx, sy, ..] = a.as_coeffs();
    let ppm = sx.hypot(sy);
    let dg = DEGRAU_PADRAO;
    let grade = grade_do_quadro(a, FHD, dg).expect("grade");
    for media in [false, true] {
        let cfg = Config {
            fina: fina_do_quadro(a, dg),
            captura: captura_do_quadro(a, dg),
            holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
            rebote: false,
            media_no_eixo: media,
            ..Config::padrao_para(&gpu.device, grade)
        };
        let tx = grade.texel_m / f64::from(cfg.fina);
        for k in [0u32, 3, 4, 6] {
            let d = [
                (f64::from(k) * tx / 8.0) as f32,
                (f64::from(k) * tx * 3.0 / 8.0) as f32,
            ];
            let c = cena(d);
            let mut s = Solver::novo(&gpu, cfg).expect("solver");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
            s.regiao_fina(i, f);
            s.quadro(&gpu, &c);
            let fl = fluencia_lida(&gpu, &s);
            let (x0, y1) = (
                f64::from(CENTRO[0] - 0.5 * LADO[0]) - 0.1,
                f64::from(CENTRO[1] + 0.5 * LADO[1]) + 0.1,
            );
            let (w, h) = (
                ((f64::from(LADO[0]) + 0.2) * ppm) as u32,
                ((f64::from(LADO[1]) + 0.2) * ppm) as u32,
            );
            let mut px = Vec::with_capacity((w * h * 3) as usize);
            for j in 0..h {
                for i in 0..w {
                    let p = [
                        x0 + (f64::from(i) + 0.5) / ppm,
                        y1 - (f64::from(j) + 0.5) / ppm,
                    ];
                    let cor = if folgas(p, d).is_some() {
                        [0.08, 0.7, 0.12]
                    } else {
                        [0.8, 0.8, 0.8]
                    };
                    for (canal, alb) in cor.iter().enumerate() {
                        let v = alb * fl.amostra_canal(p, canal) * 0.25;
                        px.push(((v / (1.0 + v)).powf(1.0 / 2.2) * 255.0) as u8);
                    }
                }
            }
            let nome = format!("{}_k{k}.ppm", if media { "depois" } else { "antes" });
            let mut arq = std::fs::File::create(dir.join(&nome)).expect("foto");
            write!(arq, "P6\n{w} {h}\n255\n").expect("cabeçalho");
            arq.write_all(&px).expect("píxeis");
            eprintln!("foto {nome} ({w}×{h})");
        }
    }
}

/// O custo da média no eixo na placa: o `c3_fina` (MÍNIMO de 7 rodadas intercaladas) com e sem ela,
/// nas quatro cenas do smoke na vista de fábrica e na caixa no zoom do dono.
#[test]
#[ignore = "a medida do custo da média no eixo: corre-se à mão"]
fn sonda_custo_da_media_no_eixo() {
    let Some(gpu) = gpu() else {
        return;
    };
    eprintln!("# loadavg antes {}", crate::medida_luz2d_w4b::loadavg());
    let fabrica =
        Camera2d::default().world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1));
    let centro = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let dg = DEGRAU_PADRAO;
    let mut casos: Vec<(String, f64, CenaDeLuz)> = (1..=4)
        .map(|n| {
            let mut w = World::new();
            assert!(smoke::encenar(&mut w, n));
            (format!("cena {n}"), 1.0, cena_de_luz(&w))
        })
        .collect();
    casos.push(("caixa · zoom 2,5".into(), 2.5, cena([0.0; 2])));
    let mut solvers = Vec::new();
    for (_, zoom, _) in &casos {
        let a = fabrica.then_scale_about(*zoom, centro);
        let grade = grade_do_quadro(a, FHD, dg).expect("grade");
        let cfg = Config {
            fina: fina_do_quadro(a, dg),
            captura: captura_do_quadro(a, dg),
            holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
            ..Config::padrao_para(&gpu.device, grade)
        };
        for media in [false, true] {
            let mut s = Solver::novo(
                &gpu,
                Config {
                    media_no_eixo: media,
                    ..cfg
                },
            )
            .expect("solver");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
            s.regiao_fina(i, f);
            solvers.push(s);
        }
    }
    let mut minimo = vec![f64::INFINITY; solvers.len()];
    for rodada in 0..7 {
        for j in 0..solvers.len() {
            let i = (j + rodada) % solvers.len();
            for _ in 0..3 {
                solvers[i].quadro(&gpu, &casos[i / 2].2);
            }
            let t = solvers[i].tempos(&gpu).expect("TIMESTAMP_QUERY");
            minimo[i] = minimo[i].min(t.c3_fina);
        }
    }
    for (c, (nome, ..)) in casos.iter().enumerate() {
        eprintln!(
            "- {nome}: c3_fina sem {:.3} ms · com {:.3} ms",
            minimo[2 * c],
            minimo[2 * c + 1]
        );
    }
    eprintln!("# loadavg depois {}", crate::medida_luz2d_w4b::loadavg());
}

/// ⭐⭐ **GATE — o rebote não depende da oitava** (2.º smoke do dono, 10/10: «até o zoom muda o render»
/// — a face da caixa na sombra e a sombra ao lado clareavam ao afastar o zoom). Medido: com a grade da
/// oitava mais grossa a borda direita da caixa caía DENTRO de uma célula, o ponto logo fora da borda
/// tinha as quatro sondas vizinhas dentro do opaco e a c0 dele voltava às de DENTRO (o lixo do
/// interior): a fonte do rebote ali 0,335 em vez de 0,06, a face 1,65 em vez de 0,27. A régua: a caixa
/// logo à direita e abaixo da lâmpada da cena 3, a vista nela nos dois lados de duas passagens de
/// oitava (zoom 1,6 · 2,2 · 2,5 · 4), pela configuração do produto com o rebote (120 quadros), contra a série do
/// gabarito, na face direita (o interior: o exterior mais próximo), na sombra e no fundo.
#[test]
fn o_rebote_nao_depende_da_oitava() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let fabrica =
        Camera2d::default().world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1));
    let alvo = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let (lado, centro) = ([1.2_f32, 2.0], [0.69_f32 + 0.6, 1.13 - 1.0]);
    let c = cena_em(centro, lado);
    let (x1, cy) = (f64::from(centro[0] + 0.5 * lado[0]), f64::from(centro[1]));
    // (o ponto lido, o ponto do gabarito: na face, o exterior mais próximo)
    let pontos = [
        ("face direita", [x1 - 0.1, cy], [x1 + 0.005, cy]),
        ("sombra", [x1 + 0.15, cy], [x1 + 0.15, cy]),
        ("fundo", [x1 + 1.0, cy + 0.6], [x1 + 1.0, cy + 0.6]),
    ];
    let serie = ph2d_light2d_ref::Rebote::resolver(
        &c,
        ph2d_light2d_ref::OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 200,
        },
    );
    let gab: Vec<f64> = pontos
        .iter()
        .map(|(_, _, q)| 2.0 * std::f64::consts::PI * serie.luz_em(*q)[1])
        .collect();
    let dg = DEGRAU_PADRAO;
    let foco = fabrica * Point::new(f64::from(centro[0]) - 0.3, f64::from(centro[1]) + 0.3);
    let mut falhas = Vec::new();
    for zoom in [1.6, 2.2, 2.5, 4.0] {
        let a = vello::kurbo::Affine::translate(alvo.to_vec2())
            * vello::kurbo::Affine::scale(zoom)
            * vello::kurbo::Affine::translate(-foco.to_vec2())
            * fabrica;
        let grade = grade_do_quadro(a, FHD, dg).expect("grade");
        let cfg = Config {
            fina: fina_do_quadro(a, dg),
            captura: captura_do_quadro(a, dg),
            holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
            rebote: true,
            ..Config::padrao_para(&gpu.device, grade)
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
        s.regiao_fina(i, f);
        for _ in 0..120 {
            s.quadro(&gpu, &c);
        }
        let fl = fluencia_lida(&gpu, &s);
        for ((nome, p, _), g) in pontos.iter().zip(&gab) {
            let v = fl.amostra(*p);
            let e = (v - g).abs() / g;
            eprintln!(
                "rebote · zoom {zoom} (texel {:.4} m) · {nome}: {v:.3} (gabarito {g:.3}, {:+.1} %)",
                grade.texel_m,
                100.0 * (v - g) / g
            );
            if e > TETO_OITAVA {
                falhas.push(format!("zoom {zoom} · {nome}: {v:.3} × {g:.3}"));
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "o rebote muda com a oitava (teto {TETO_OITAVA}): {}",
        falhas.join(" · ")
    );
}

/// O erro relativo máximo de cada ponto do [`o_rebote_nao_depende_da_oitava`] — MEDIDO (RTX 5060 Ti):
/// o pior 2,9 % (o fundo na oitava grossa); antes da cura +514 % (a face) · +204 % (a sombra).
const TETO_OITAVA: f64 = 0.035;
