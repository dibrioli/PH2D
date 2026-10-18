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
//!
//! ⭐ W5n — com a DIFUSÃO (W5h) o miolo do produto é outra lei, e o corte no eixo medial é o da lei
//! antiga, que continua no produto com `miolo: None` (a suavidade 0). O portão mede as DUAS: a da
//! média no eixo (a lei antiga contra o gabarito dela, controlo sem a média) e a da difusão (o produto
//! contra o gabarito dela) — esta reprovava no zoom 2,5 até a W5n (837 · 774 · 2 058 texels: a
//! cascata da W5h não convergia — o erro de média frequência ficava; cura: os V-ciclos de correção,
//! `Miolo::ciclos`, controlo na mesma corrida).

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
pub(crate) const CENTRO: [f32; 2] = [1.3, -0.2];
const LADO: [f32; 2] = [1.1, 2.2];
/// Subamostras por eixo na média do gabarito sobre um texel a menos de um texel do eixo medial (a
/// fração da área sai em degraus de `1/SUB`; com 6 o degrau já valia 5 % da diferença das faces).
const SUB: u32 = 12;
/// A faixa medida em volta do eixo medial (texels finos, na folga entre as duas faces mais perto).
const FAIXA: f64 = 4.0;
/// Um texel do interior está ERRADO a mais disto do gabarito, relativo ao máximo dele ali.
const ERRADO: f64 = 0.10;
/// A lei do interior até a W5g (`suavidade = 0`): o exterior mais próximo — a do `miolo: None`.
pub(crate) const EXTERIOR: ph2d_light2d_ref::OpcoesDoMiolo = ph2d_light2d_ref::OpcoesDoMiolo {
    suavidade: 0.0,
    penetracao_m: f64::INFINITY,
};
/// Texels errados por caso no produto — MEDIDO (RTX 5060 Ti, `DEGRAU_PADRAO`): 0 em todo caso desde a
/// borda exata (W5d; antes 0 ou 1 — um canto da caixa, onde o eixo chega à borda); sem a média no
/// eixo, 35 a 537 (a escada).
const TETO_ERRADOS: usize = 1;
/// ⭐ W5n — o mesmo na DIFUSÃO (o produto com 3 V-ciclos): MEDIDO 0 em todo caso salvo o zoom 2,5 ·
/// 3/8, com 2 — os dois no canto de cima-direita da caixa, a < 1 texel das duas faces (placa 1,19 ·
/// 1,04 × gabarito 0,89 · 0,73), iguais com 3, 6, 12 e 24 ciclos: a discretização do canto no nível
/// 1, não a convergência. Sem os ciclos (a W5h) 837 · 774 · 2 058.
const TETO_ERRADOS_DIFUSAO: usize = 2;
/// ⭐ W6e — os texels dos CANTOS da caixa ([`gabarito_dos_cantos`], 4 por caso, 32 ao todo) errados na
/// lei antiga — MEDIDO (`medida_luz2d_w6d_canto`, handoff W6e §2): 6 com a cunha (antes da W6d 4, a lei
/// da W6d sem a cunha 7). Os que restam são o texel do canto que fica com UMA face em vez da média das
/// duas (o `c3_direta` no canto, já antes da W6d — aberto da W6e), não o campo.
const TETO_CANTOS: usize = 6;

/// A cena 3 com a parede verde feita uma caixa livre no meio da sala, deslocada `d` metros.
pub(crate) fn cena(d: [f32; 2]) -> CenaDeLuz {
    cena_em([CENTRO[0] + d[0], CENTRO[1] + d[1]], LADO)
}

/// A cena 3 com a parede verde feita uma caixa de `lado` em `centro`.
fn cena_em(centro: [f32; 2], lado: [f32; 2]) -> CenaDeLuz {
    cena_girada_em(centro, lado, 0.0)
}

/// ⭐ W6e — a [`cena`] com a caixa girada `rot` radianos em volta do centro dela.
pub(crate) fn cena_girada(d: [f32; 2], rot: f32) -> CenaDeLuz {
    cena_girada_em([CENTRO[0] + d[0], CENTRO[1] + d[1]], LADO, rot)
}

fn cena_girada_em(centro: [f32; 2], lado: [f32; 2], rot: f32) -> CenaDeLuz {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 3));
    let mut q = w.query::<(&Name, &mut Sprite, &mut Transform, &mut GlobalTransform)>();
    let mut achou = false;
    for (n, mut s, mut t, mut g) in q.iter_mut(&mut w) {
        if n.as_str() == "Parede verde" {
            s.size = lado;
            t.translation.x = centro[0];
            t.translation.y = centro[1];
            t.rotation = rot;
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
pub(crate) fn gabarito(
    g: ph2d_light2d_gpu::Grade,
    c: &CenaDeLuz,
    d: [f32; 2],
) -> (Vec<([f64; 2], f64)>, usize) {
    gabarito_por(g, d, |p| ph2d_light2d_ref::fluencia(c, p)[1])
}

/// ⭐ W5n — o [`gabarito`] com a lei `lei` (o verde num ponto).
pub(crate) fn gabarito_por(
    g: ph2d_light2d_gpu::Grade,
    d: [f32; 2],
    lei: impl Fn([f64; 2]) -> f64,
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
                        s += lei([p[0] + o(a) * tx, p[1] + o(b) * tx]);
                    }
                }
                s / f64::from(sub * sub)
            } else {
                lei(p)
            };
            pts.push((p, v));
        }
    }
    (pts, no_eixo)
}

/// ⭐ W6e — os texels finos dos CANTOS da caixa (o centro dentro, a menos de um texel das DUAS faces mais
/// perto — fora do [`gabarito_por`]): `(centro, a média da lei sobre a parte do texel dentro da caixa)`.
pub(crate) fn gabarito_dos_cantos(
    g: ph2d_light2d_gpu::Grade,
    d: [f32; 2],
    lei: impl Fn([f64; 2]) -> f64,
) -> Vec<([f64; 2], f64)> {
    let tx = g.texel_m;
    let o = |k: u32| (f64::from(k) + 0.5) / f64::from(SUB) - 0.5;
    let mut pts = Vec::new();
    for j in 0..g.altura {
        for i in 0..g.largura {
            let p = g.centro(i, j);
            match folgas(p, d) {
                Some((eixo, borda)) if eixo + borda < tx => {}
                _ => continue,
            }
            let (mut s, mut n) = (0.0, 0u32);
            for b in 0..SUB {
                for a in 0..SUB {
                    let q = [p[0] + o(a) * tx, p[1] + o(b) * tx];
                    if folgas(q, d).is_some() {
                        s += lei(q);
                        n += 1;
                    }
                }
            }
            pts.push((p, s / f64::from(n)));
        }
    }
    pts
}

/// `(o pior erro, quantos texels erram mais que ERRADO)`, relativos ao máximo do gabarito ali.
pub(crate) fn erros(fl: &Fluencia, gab: &[([f64; 2], f64)]) -> (f64, usize) {
    erros_contra(fl, gab, gab.iter().map(|x| x.1).fold(0.0, f64::max))
}

/// O [`erros`] relativo a `max` (o dos cantos: o máximo do gabarito do interior).
fn erros_contra(fl: &Fluencia, gab: &[([f64; 2], f64)], max: f64) -> (f64, usize) {
    gab.iter().fold((0.0, 0), |(pior, n), &(p, g)| {
        let e = (fl.amostra(p) - g).abs() / max;
        (pior.max(e), n + usize::from(e > ERRADO))
    })
}

/// ⭐⭐ **GATE — o interior não depende da posição na grade**, nas duas leis: o produto (a difusão)
/// contra o gabarito dela, e a lei antiga (`miolo: None`: o texel que o eixo medial corta é a média
/// das faces) contra o gabarito dela — no máximo `TETO_ERRADOS` texels errados por caso (os de um
/// canto ou do encontro de três faces); o controlo (a lei antiga sem a média no eixo) tem a escada.
/// ⭐ W6e — e os CANTOS (o texel a menos de um texel das duas faces): no máximo [`TETO_CANTOS`]
/// errados, nenhum no caso do aberto da W6d, onde o controlo sem a cunha erra.
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
    let (mut falhas, mut controlo, mut controlo_ciclos) = (Vec::new(), 0, 0);
    // ⭐ W6e: os cantos errados (o produto na lei antiga) e o controlo sem a cunha no caso do aberto.
    let (mut cantos, mut cantos_aberto) = (0, None);
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
            let ler = |cfg: Config| {
                let mut s = Solver::novo(&gpu, cfg).expect("solver");
                let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
                s.regiao_fina(i, f);
                s.quadro(&gpu, &c);
                fluencia_lida(&gpu, &s)
            };
            let antiga = Config { miolo: None, ..cfg };
            let sem_ciclos = ler(Config {
                miolo: cfg
                    .miolo
                    .map(|m| ph2d_light2d_gpu::Miolo { ciclos: 0, ..m }),
                ..cfg
            });
            let (produto, media, sem) = (
                ler(cfg),
                ler(antiga),
                ler(Config {
                    media_no_eixo: false,
                    ..antiga
                }),
            );
            let (gab, no_eixo) = gabarito(produto.grade, &c, d);
            let lei = ph2d_light2d_ref::Gabarito::com_miolo(&c, EXTERIOR);
            let (gab_antiga, _) = gabarito_por(produto.grade, d, |p| lei.fluencia(p)[1]);
            assert!(
                no_eixo > 50,
                "a régua não achou o eixo medial ({no_eixo} texels)"
            );
            let (e, n) = erros(&produto, &gab);
            let (em, nm) = erros(&media, &gab_antiga);
            let (e0, n0) = erros(&sem, &gab_antiga);
            let (ea, na) = erros(&sem_ciclos, &gab);
            eprintln!(
                "interior · zoom {zoom} · deslocamento {k}/8 · {no_eixo} texels no eixo: difusão pior {e:.3}, {n} errados (sem os V-ciclos: {ea:.3}, {na}) · lei antiga {em:.3}, {nm} (sem a média: {e0:.3}, {n0})"
            );
            controlo = controlo.max(n0);
            controlo_ciclos = controlo_ciclos.max(na);
            let max_antiga = gab_antiga.iter().map(|x| x.1).fold(0.0, f64::max);
            let gab_cantos = gabarito_dos_cantos(produto.grade, d, |p| lei.fluencia(p)[1]);
            let (ec, nc) = erros_contra(&media, &gab_cantos, max_antiga);
            eprintln!(
                "interior · zoom {zoom} · {k}/8 · os {} cantos na lei antiga: pior {ec:.3}, {nc} errados",
                gab_cantos.len()
            );
            assert_eq!(gab_cantos.len(), 4, "a régua achou os quatro cantos");
            cantos += nc;
            // O texel do canto convexo que a W6d punha no ambiente (o aberto da W6d, zoom 1 · 3/8).
            if zoom == 1.0 && k == 3 {
                let sem_cunha = ler(Config {
                    borda_vertice: None,
                    ..antiga
                });
                cantos_aberto = Some((nc, erros_contra(&sem_cunha, &gab_cantos, max_antiga).1));
            }
            if n > TETO_ERRADOS_DIFUSAO || nm > TETO_ERRADOS {
                falhas.push(format!(
                    "zoom {zoom} · {k}/8: difusão {n} (pior {e:.3}) · lei antiga {nm} (pior {em:.3})"
                ));
            }
        }
    }
    assert!(
        controlo > 10 * TETO_ERRADOS,
        "o controlo sem a média não mostra a escada ({controlo} texels errados) — a régua não a vê"
    );
    let (aberto, aberto_sem) = cantos_aberto.expect("o caso do aberto (zoom 1 · 3/8)");
    eprintln!(
        "interior · cantos errados {cantos} (teto {TETO_CANTOS}) · o caso do aberto: {aberto} (sem a cunha {aberto_sem})"
    );
    assert!(
        aberto_sem > 0,
        "controlo: sem a cunha o canto do aberto (zoom 1 · 3/8) não erra — a régua não o vê"
    );
    assert!(
        aberto == 0 && cantos <= TETO_CANTOS,
        "o canto da caixa: {aberto} errados no caso do aberto (zoom 1 · 3/8) · {cantos} ao todo (teto {TETO_CANTOS})"
    );
    assert!(
        controlo_ciclos > 10 * TETO_ERRADOS,
        "o controlo sem os V-ciclos (a W5h) não mostra o miolo por convergir ({controlo_ciclos} texels errados) — a régua não o vê"
    );
    assert!(
        falhas.is_empty(),
        "o interior do opaco muda com a posição na grade (mais de {TETO_ERRADOS_DIFUSAO} texels errados > {ERRADO} na difusão, {TETO_ERRADOS} na lei antiga): {}",
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
/// gabarito, 10 cm dentro da face direita, na sombra e no fundo. ⚠️ W5n: dentro da caixa a lei é a
/// DIFUSÃO (W5h) — a série resolve o miolo dela no mesmo ponto (até a W5g: o exterior mais próximo, o
/// gabarito lido logo fora da face). O ponto de dentro reprovava até a W5n (+17 % na oitava grossa,
/// +49 % na fina): a cascata da W5h não convergia; cura: os V-ciclos (`Miolo::ciclos`), controlo sem
/// eles na mesma corrida.
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
    // (o ponto lido, o ponto do gabarito)
    let pontos = [
        ("face direita", [x1 - 0.1, cy], [x1 - 0.1, cy]),
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
    // O controlo: a face sem os V-ciclos (a W5h), o pior erro relativo.
    let mut controlo: f64 = 0.0;
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
        let ler = |cfg: Config| {
            let mut s = Solver::novo(&gpu, cfg).expect("solver");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
            s.regiao_fina(i, f);
            for _ in 0..120 {
                s.quadro(&gpu, &c);
            }
            fluencia_lida(&gpu, &s)
        };
        let fl = ler(cfg);
        let face = ler(Config {
            miolo: cfg
                .miolo
                .map(|m| ph2d_light2d_gpu::Miolo { ciclos: 0, ..m }),
            ..cfg
        })
        .amostra(pontos[0].1);
        controlo = controlo.max((face - gab[0]).abs() / gab[0]);
        for ((nome, p, _), g) in pontos.iter().zip(&gab) {
            let v = fl.amostra(*p);
            let e = (v - g).abs() / g;
            eprintln!(
                "rebote · zoom {zoom} (texel {:.4} m) · {nome}: {v:.3} (gabarito {g:.3}, {:+.1} %)",
                grade.texel_m,
                100.0 * (v - g) / g
            );
            let teto = if *nome == "face direita" {
                TETO_MIOLO
            } else {
                TETO_OITAVA
            };
            if e > teto {
                falhas.push(format!("zoom {zoom} · {nome}: {v:.3} × {g:.3}"));
            }
        }
    }
    eprintln!(
        "rebote · controlo sem os V-ciclos: a face erra {:+.1} %",
        100.0 * controlo
    );
    assert!(
        controlo > TETO_MIOLO,
        "controlo: sem os V-ciclos a face não muda com a oitava ({controlo:.3}) — a régua não o vê"
    );
    assert!(
        falhas.is_empty(),
        "o rebote muda com a oitava (teto {TETO_OITAVA}): {}",
        falhas.join(" · ")
    );
}

/// O erro relativo máximo de cada ponto do [`o_rebote_nao_depende_da_oitava`] — MEDIDO (RTX 5060 Ti):
/// o pior 2,9 % (o fundo na oitava grossa); antes da cura +514 % (a face) · +204 % (a sombra).
const TETO_OITAVA: f64 = 0.035;
/// ⭐ W5n — o mesmo para o ponto de DENTRO da face (a difusão): MEDIDO (RTX 5060 Ti, a série do
/// gabarito 0,611) com 3 V-ciclos −3,6 % (oitava grossa) · −0,4 % (fina); convergido (6 ciclos) −3,4 %
/// · −0,7 % — o resto é a discretização do nível 1 (o dobro do texel fino), não a convergência. Sem
/// os ciclos (a W5h) +17,2 % · +49,3 %. O teto é o medido + 10 %.
const TETO_MIOLO: f64 = 0.04;

/// ⭐ W5n — a rodada dos V-ciclos do miolo: (A) a caixa no zoom 2,5 contra o gabarito da difusão, (B)
/// o ponto de dentro da face do rebote em duas oitavas, (C) o custo (`c3_fina` + `c3_miolo`, o MÍNIMO
/// de 7 intercaladas) nas cenas 1–6 e na caixa — por variante (a 1.ª é a W5h). Saída: o handoff W5n.
#[test]
#[ignore = "a rodada dos V-ciclos do miolo: corre-se à mão"]
fn sonda_ciclos_do_miolo() {
    use ph2d_light2d_gpu::{MIOLO_PADRAO, Miolo};
    let Some(gpu) = gpu() else { return };
    eprintln!("# loadavg antes {}", crate::medida_luz2d_w4b::loadavg());
    let fabrica =
        Camera2d::default().world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1));
    let centro = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let dg = DEGRAU_PADRAO;
    // (relaxa, relaxa_max, ciclos, suaviza, anel): a 1.ª é a W5h.
    const V: [(u32, u32, u32, u32, bool); 6] = [
        (4, 32, 0, 2, false),
        (4, 32, 1, 2, true),
        (4, 32, 2, 2, true),
        (4, 32, 3, 2, true),
        (2, 16, 2, 2, true),
        (4, 32, 6, 2, true),
    ];
    let cfg_de = |a: vello::kurbo::Affine, rebote: bool| Config {
        fina: fina_do_quadro(a, dg),
        captura: captura_do_quadro(a, dg),
        holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
        rebote,
        ..Config::padrao_para(&gpu.device, grade_do_quadro(a, FHD, dg).expect("grade"))
    };
    let com =
        |cfg: Config, (relaxa, relaxa_max, ciclos, suaviza, anel): (u32, u32, u32, u32, bool)| {
            Config {
                miolo: Some(Miolo {
                    relaxa,
                    relaxa_max,
                    ciclos,
                    suaviza,
                    ..MIOLO_PADRAO
                }),
                anel_fora_da_regiao: anel,
                ..cfg
            }
        };
    // A. A caixa no zoom 2,5, quatro fases.
    let a = fabrica.then_scale_about(2.5, centro);
    let cfg = cfg_de(a, false);
    let tx = cfg.grade.texel_m / f64::from(cfg.fina);
    for k in [0u32, 4] {
        let d = [
            (f64::from(k) * tx / 8.0) as f32,
            (f64::from(k) * tx * 3.0 / 8.0) as f32,
        ];
        let c = cena(d);
        let mut linha = format!("A caixa 2,5 · {k}/8:");
        let mut gab = None;
        for v in V {
            let mut s = Solver::novo(&gpu, com(cfg, v)).expect("solver");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
            s.regiao_fina(i, f);
            s.quadro(&gpu, &c);
            let fl = fluencia_lida(&gpu, &s);
            let g = gab.get_or_insert_with(|| gabarito(fl.grade, &c, d).0);
            let (e, n) = erros(&fl, g);
            linha += &format!(" {}→{}·{}·{}·{} {n} ({e:.3}) ·", v.0, v.1, v.2, v.3, v.4);
        }
        eprintln!("{linha}");
    }
    // B. O ponto da face do rebote.
    let (lado, cc) = ([1.2_f32, 2.0], [0.69_f32 + 0.6, 1.13 - 1.0]);
    let c = cena_em(cc, lado);
    let p = [f64::from(cc[0] + 0.5 * lado[0]) - 0.1, f64::from(cc[1])];
    let serie = ph2d_light2d_ref::Rebote::resolver(
        &c,
        ph2d_light2d_ref::OpcoesDoRebote {
            passo_m: 0.02,
            rebotes: 200,
        },
    );
    let g = 2.0 * std::f64::consts::PI * serie.luz_em(p)[1];
    let alvo = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let foco = fabrica * Point::new(f64::from(cc[0]) - 0.3, f64::from(cc[1]) + 0.3);
    for zoom in [1.6, 2.5] {
        let a = vello::kurbo::Affine::translate(alvo.to_vec2())
            * vello::kurbo::Affine::scale(zoom)
            * vello::kurbo::Affine::translate(-foco.to_vec2())
            * fabrica;
        let cfg = cfg_de(a, true);
        let mut linha = format!("B rebote face · zoom {zoom} (gabarito {g:.3}):");
        for v in V {
            let mut s = Solver::novo(&gpu, com(cfg, v)).expect("solver");
            let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
            s.regiao_fina(i, f);
            for _ in 0..120 {
                s.quadro(&gpu, &c);
            }
            let x = fluencia_lida(&gpu, &s).amostra(p);
            linha += &format!(
                " {}→{}·{}·{}·{} {x:.3} ({:+.1} %) ·",
                v.0,
                v.1,
                v.2,
                v.3,
                v.4,
                100.0 * (x - g) / g
            );
        }
        eprintln!("{linha}");
    }
    // C. O custo do c3_miolo (o mínimo de 7 intercaladas).
    let mut casos: Vec<(String, f64, CenaDeLuz)> = (1..=6)
        .map(|n| {
            let mut w = World::new();
            assert!(smoke::encenar(&mut w, n));
            (format!("cena {n}"), 1.0, cena_de_luz(&w))
        })
        .collect();
    casos.push(("caixa · zoom 2,5".into(), 2.5, cena([0.0; 2])));
    for (nome, zoom, cena_c) in &casos {
        let a = fabrica.then_scale_about(*zoom, centro);
        let cfg = cfg_de(a, false);
        let mut solvers: Vec<Solver> = V
            .iter()
            .map(|&v| {
                let mut s = Solver::novo(&gpu, com(cfg, v)).expect("solver");
                let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
                s.regiao_fina(i, f);
                s
            })
            .collect();
        let mut minimo = [[f64::INFINITY; 2]; V.len()];
        for rodada in 0..7 {
            for j in 0..V.len() {
                let i = (j + rodada) % V.len();
                for _ in 0..3 {
                    solvers[i].quadro(&gpu, cena_c);
                }
                if let Some(t) = solvers[i].tempos(&gpu) {
                    minimo[i][0] = minimo[i][0].min(t.c3_fina);
                    minimo[i][1] = minimo[i][1].min(t.c3_miolo);
                }
            }
        }
        let mut linha = format!("C {nome} · c3_fina + c3_miolo ms:");
        for (j, v) in V.iter().enumerate() {
            linha += &format!(
                " {}→{}·{}·{}·{} {:.3}+{:.3} ·",
                v.0, v.1, v.2, v.3, v.4, minimo[j][0], minimo[j][1]
            );
        }
        eprintln!("{linha}");
    }
    eprintln!("# loadavg depois {}", crate::medida_luz2d_w4b::loadavg());
}
