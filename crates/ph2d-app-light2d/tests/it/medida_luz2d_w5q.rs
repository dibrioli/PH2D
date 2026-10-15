//! ⭐ W5q — a `c2` do High por ETAPA da holográfica (traça · sobe · desce · soma · saída, por nível):
//! o fim do [`crate::medida_luz2d_w5o::medida_luz2d_w5o_lastro`]. Com `PH2D_W5Q_FASES=1`, depois da
//! `c2` inteira, o mesmo solver passa ao instrumento ([`ph2d_light2d_gpu::Solver::medir_fases_da_hrc`],
//! um passe por despacho) e imprime o mínimo e a mediana de cada (etapa, nível) em 12 quadros.

use ph2d_light2d_ecs::CenaDeLuz;
use vello::kurbo;

use crate::medida_luz2d_w5o::{Par, mediana, minimo};

const QUADROS: usize = 12;
const FHD: (u32, u32) = (1920, 1080);

pub(crate) fn a_c2_e_as_etapas(
    gpu: &ph2d_gpu::GpuContext,
    mut p: Par,
    cena: &CenaDeLuz,
    rotulo: &str,
) {
    // ⭐ W5u: aquecida antes — na cena cheia o rebote acende a cena nos primeiros quadros e a `c2` do High
    // vai de 9,4 (o 1.º quadro, que o mínimo lia) a ~17 ms (handoff W5u §1). `PH2D_W5U_AQUECE=0` = o antes.
    let aquece = std::env::var("PH2D_W5U_AQUECE").as_deref() != Ok("0");
    for _ in 0..if aquece { QUADROS } else { 0 } {
        p.quadro(gpu, cena);
    }
    let mut c2 = Vec::new();
    // ⭐ W5u: a FATIA de cada passe do quadro (a vista; o entorno inteiro à parte), o mínimo por passe.
    let mut fatias: Vec<Vec<f64>> = vec![Vec::new(); 13];
    for _ in 0..QUADROS {
        p.quadro(gpu, cena);
        let (t, e) = p.tempos(gpu);
        c2.push(t.c2_total());
        let v = [
            t.c0,
            t.c1,
            t.c2_total(),
            t.c3_visibilidade,
            t.c3_fina,
            t.c0_fina,
            t.c3_preenche,
            t.c3_mascaras,
            t.c0_rebote,
            t.c3_rebote,
            t.c3_miolo,
            t.total(),
            e.map_or(0.0, |e| e.total()),
        ];
        fatias.iter_mut().zip(v).for_each(|(f, x)| f.push(x));
    }
    eprintln!(
        "C {rotulo} · c2 High mín {:.3} · med {:.3} ms",
        minimo(&c2),
        mediana(&c2)
    );
    const NOMES: [&str; 13] = [
        "c0", "c1", "c2", "c3_vis", "c3_fina", "c0_fina", "c3_preenche", "c3_masc", "c0_rebote",
        "c3_rebote", "miolo", "total", "entorno",
    ];
    let q: Vec<String> = NOMES
        .iter()
        .zip(&fatias)
        .map(|(n, f)| format!("{n} {:.3}", minimo(f)))
        .collect();
    eprintln!("Q {rotulo} · mín por passe: {}", q.join(" · "));
    if std::env::var("PH2D_W5U_SERIE").is_ok() {
        let s: Vec<String> = c2.iter().map(|x| format!("{x:.2}")).collect();
        eprintln!("Z {rotulo} · c2 por quadro: {}", s.join(" "));
    }
    if std::env::var("PH2D_W5Q_FASES").as_deref() != Ok("1") {
        return;
    }
    assert!(p.vista.medir_fases_da_hrc(gpu), "TIMESTAMP_QUERY");
    let mut por: Vec<(&'static str, u32, Vec<f64>, usize)> = Vec::new();
    for _ in 0..QUADROS {
        p.quadro(gpu, cena);
        for (etapa, nivel, ms, n) in p.vista.fases_da_hrc(gpu).expect("fases") {
            match por.iter_mut().find(|o| o.0 == etapa && o.1 == nivel) {
                Some(o) => o.2.push(ms),
                None => por.push((etapa, nivel, vec![ms], n)),
            }
        }
    }
    let (mut soma_min, mut soma_med) = (0.0, 0.0);
    for (etapa, nivel, ms, n) in &por {
        soma_min += minimo(ms);
        soma_med += mediana(ms);
        eprintln!(
            "F {rotulo} · {etapa} n{nivel} · mín {:.3} · med {:.3} ms · {n} despachos",
            minimo(ms),
            mediana(ms)
        );
    }
    eprintln!("S {rotulo} · soma das etapas mín {soma_min:.3} · med {soma_med:.3} ms");
}

/// ⭐ W5q — o anel só nos ladrilhos com corpo (`Config::anel_por_ladrilhos`) contra a grade inteira,
/// no MESMO processo: a fluência da vista ao BIT (80 quadros) e o relógio da placa da fina (com o
/// anel) e do miolo, o mínimo de 7 rodadas intercaladas. Cenas 1–7 na vista de fábrica, a 3 com a
/// câmera em x = 2, a 4 e a 5 de perto (os corpos atravessam a janela) e a 5 que vira a 4 a meio
/// (ladrilhos que se apagam), e a caixa do `o_interior_do_opaco` de perto, parada e a andar.
/// Controlo: os ladrilhos só da janela ([`ph2d_light2d_gpu::Solver::ladrilhos_so_da_janela`]) mudam
/// bytes em algum caso.
#[test]
#[ignore = "rodada de medição W5q (placa): os ladrilhos do anel ao bit e o custo"]
fn medida_luz2d_w5q_ladrilhos() {
    use crate::medida_luz2d_w5p::Vista;
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    use ph2d_light2d_gpu::Config;
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!(
        "W5q ladrilhos · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    let casos = casos();
    let variante = |ladrilhos: bool, anel: bool| {
        move |c: Config| Config {
            anel_por_ladrilhos: ladrilhos,
            anel_fora_da_regiao: anel,
            ..c
        }
    };
    let mut todos_ao_bit = true;
    let mut vistos = 0;
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
        for (rotulo, cena, cena2, a) in &casos {
            let a = *a;
            let mut par = [
                Vista::nova_com(&gpu, a, nivel, &variante(false, true)),
                Vista::nova_com(&gpu, a, nivel, &variante(true, true)),
            ];
            let correr = |v: &mut Vista| {
                for q in 0..80 {
                    v.quadro(&gpu, if q < 40 { cena } else { cena2 });
                }
            };
            par.iter_mut().for_each(correr);
            let bytes = par
                .each_ref()
                .map(|v| crate::comum::ler(&gpu, v.vista.fluencia(), 8));
            let diferentes = bytes[0]
                .iter()
                .zip(&bytes[1])
                .filter(|(x, y)| x != y)
                .count();
            todos_ao_bit &= diferentes == 0;
            // O controlo: os ladrilhos só da janela (sem os corpos) mudam bytes onde o anel decide.
            let mut m = Vista::nova_com(&gpu, a, nivel, &variante(true, true));
            m.vista.ladrilhos_so_da_janela();
            correr(&mut m);
            let bm = crate::comum::ler(&gpu, m.vista.fluencia(), 8);
            let controlo = bm.iter().zip(&bytes[1]).filter(|(x, y)| x != y).count();
            vistos += usize::from(controlo > 0);
            let mut minimo = [[f64::INFINITY; 2]; 2];
            for rodada in 0..7 {
                for j in 0..2 {
                    let i = (j + rodada) % 2;
                    for _ in 0..3 {
                        par[i].quadro(&gpu, cena2);
                    }
                    let t = par[i].vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
                    minimo[i][0] = minimo[i][0].min(t.c3_fina);
                    minimo[i][1] = minimo[i][1].min(t.c3_miolo);
                }
            }
            eprintln!(
                "{nivel:?} · {rotulo} · bytes diferentes {diferentes} de {} (controlo {controlo}) · fina+anel {:.3} → {:.3} ms · miolo {:.3} → {:.3} ms",
                bytes[0].len(),
                minimo[0][0],
                minimo[1][0],
                minimo[0][1],
                minimo[1][1]
            );
        }
    }
    assert!(
        vistos > 0,
        "o controlo (só a janela) não mudou nenhum caso: a régua não vê o anel"
    );
    eprintln!(
        "W5q ladrilhos · ao bit em todos: {todos_ao_bit} · o controlo mudou {vistos} casos · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
}

/// (rótulo, a cena da 1.ª e da 2.ª metade dos quadros, a câmera).
pub(crate) type Caso = (String, CenaDeLuz, CenaDeLuz, kurbo::Affine);

/// Os casos das réguas W5q: cenas 1–7 na vista de fábrica, a 3 com a câmera em x = 2, a 4 e a 5 de
/// perto, a 5 que vira a 4 a meio, e a caixa do `o_interior_do_opaco` a 2,5 parada e a andar.
pub(crate) fn casos() -> Vec<Caso> {
    let cena_de = |n| {
        let mut w = bevy_ecs::world::World::new();
        assert!(ph2d_app_light2d::smoke::encenar(&mut w, n));
        ph2d_light2d_ecs::coleta::cena_de_luz(&w)
    };
    let camera = |centro: [f32; 2], altura: f32| {
        let c = if altura > 0.0 {
            ph2d_render::Camera2d::new(centro, altura)
        } else {
            ph2d_render::Camera2d::default()
        };
        c.world_to_screen_affine(ph2d_host::WindowSize::new(FHD.0, FHD.1))
    };
    // (rótulo, a cena da 1.ª e da 2.ª metade, a câmera).
    let mut casos: Vec<Caso> = (1..=7)
        .map(|n| {
            (
                format!("cena {n}"),
                cena_de(n),
                cena_de(n),
                camera([0.0; 2], 0.0),
            )
        })
        .collect();
    for (n, centro, altura, depois) in [
        (3, [2.0, 0.0], 2.0, 3),
        (4, [0.0, 0.0], 1.0, 4),
        (5, [0.6, 0.3], 1.5, 5),
        (5, [0.0, 0.0], 0.0, 4),
    ] {
        casos.push((
            format!("cena {n}→{depois} {centro:?} alt {altura}"),
            cena_de(n),
            cena_de(depois),
            camera(centro, altura),
        ));
    }
    // A caixa do `o_interior_do_opaco` no zoom 2,5 (o corpo atravessa a janela: o anel decide), parada
    // e a andar a meio (os ladrilhos dela apagam-se e outros acendem).
    let centro = kurbo::Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let perto = crate::medida_luz2d_w5e::fabrica(FHD).then_scale_about(2.5, centro);
    let caixa = crate::o_interior_do_opaco::cena;
    casos.push(("caixa 2,5".into(), caixa([0.0; 2]), caixa([0.0; 2]), perto));
    casos.push((
        "caixa 2,5 a andar".into(),
        caixa([0.0; 2]),
        caixa([0.4, 0.25]),
        perto,
    ));
    casos
}

/// ⭐ W5q — o BLOCO do miolo (`Miolo::bloco`: K relaxações por despacho) contra uma por despacho, no
/// MESMO processo: a fluência da vista ao BIT (80 quadros), o relógio da placa do miolo (o mínimo de 7
/// rodadas intercaladas) e os despachos (o instrumento das fases, noutro solver). Controlo: o miolo
/// com 1 ciclo a menos muda bytes.
#[test]
#[ignore = "rodada de medição W5q (placa): o bloco do miolo ao bit e o custo"]
fn medida_luz2d_w5q_bloco() {
    use crate::medida_luz2d_w5p::Vista;
    use ph2d_editor_core::luz_edits::NivelDaLuz;
    use ph2d_light2d_gpu::Config;
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!(
        "W5q bloco · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    const BLOCOS: [u32; 3] = [1, 2, 4];
    let variante = |bloco: u32, menos_um: bool| {
        move |c: Config| Config {
            miolo: c.miolo.map(|m| ph2d_light2d_gpu::Miolo {
                bloco,
                ciclos: m.ciclos - u32::from(menos_um),
                ..m
            }),
            ..c
        }
    };
    let mut todos_ao_bit = true;
    let mut vistos = 0;
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
        for (rotulo, cena, cena2, a) in &casos() {
            let a = *a;
            let mut vs = BLOCOS.map(|b| Vista::nova_com(&gpu, a, nivel, &variante(b, false)));
            let correr = |v: &mut Vista| {
                for q in 0..80 {
                    v.quadro(&gpu, if q < 40 { cena } else { cena2 });
                }
            };
            vs.iter_mut().for_each(correr);
            let bytes = vs
                .each_ref()
                .map(|v| crate::comum::ler(&gpu, v.vista.fluencia(), 8));
            let dif = |x: &[u8], y: &[u8]| x.iter().zip(y).filter(|(a, b)| a != b).count();
            let diferentes = [dif(&bytes[0], &bytes[1]), dif(&bytes[0], &bytes[2])];
            todos_ao_bit &= diferentes == [0, 0];
            let mut m = Vista::nova_com(&gpu, a, nivel, &variante(4, true));
            correr(&mut m);
            let controlo = dif(&crate::comum::ler(&gpu, m.vista.fluencia(), 8), &bytes[0]);
            vistos += usize::from(controlo > 0);
            let mut despachos = [0; 3];
            for (i, &b) in BLOCOS.iter().enumerate() {
                let mut f = Vista::nova_com(&gpu, a, nivel, &variante(b, false));
                if f.vista.medir_fases_do_miolo(&gpu) {
                    f.quadro(&gpu, cena2);
                    despachos[i] = f
                        .vista
                        .fases_do_miolo(&gpu)
                        .map_or(0, |v| v.iter().map(|x| x.2).sum());
                }
            }
            let mut minimo = [f64::INFINITY; 3];
            for rodada in 0..7 {
                for j in 0..3 {
                    let i = (j + rodada) % 3;
                    for _ in 0..3 {
                        vs[i].quadro(&gpu, cena2);
                    }
                    let t = vs[i].vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
                    minimo[i] = minimo[i].min(t.c3_miolo);
                }
            }
            eprintln!(
                "{nivel:?} · {rotulo} · bytes diferentes (2, 4) {diferentes:?} (controlo {controlo}) · miolo 1/2/4 {:.3} · {:.3} · {:.3} ms · despachos {despachos:?}",
                minimo[0], minimo[1], minimo[2]
            );
        }
    }
    assert!(
        vistos > 0,
        "o controlo (um ciclo a menos) não mudou nenhum caso"
    );
    eprintln!(
        "W5q bloco · ao bit em todos: {todos_ao_bit} · o controlo mudou {vistos} casos · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
}
