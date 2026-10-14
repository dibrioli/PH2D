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
    let mut c2 = Vec::new();
    for _ in 0..QUADROS {
        p.quadro(gpu, cena);
        c2.push(p.tempos(gpu).0.c2_total());
    }
    eprintln!(
        "C {rotulo} · c2 High mín {:.3} · med {:.3} ms",
        minimo(&c2),
        mediana(&c2)
    );
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
    let mut casos: Vec<(String, CenaDeLuz, CenaDeLuz, kurbo::Affine)> = (1..=7)
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
