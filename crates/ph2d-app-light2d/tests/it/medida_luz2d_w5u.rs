//! ⭐ W5u — o custo por fio da holográfica nos TRÊS níveis: cones vizinhos por fio na descida
//! (`Config::hrc_cones_por_fio`), pares por fio na subida (`Config::hrc_pares_na_subida`) e o acumulado em
//! rgb (`Config::hrc_acumulado_rgb`). A régua, no MESMO processo, sobre os casos da W5s × Low/Medium/High: a
//! fluência ao BIT contra a base (80 quadros), um controlo por corte (o texto do módulo mutado no caminho
//! novo) e o relógio da `c2` (o mínimo de 7 rodadas intercaladas).

use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_gpu::{Config, Precisao};

use crate::medida_luz2d_w5o::{mediana, minimo};
use crate::medida_luz2d_w5p::Vista;
use crate::medida_luz2d_w5s::{camadas, casos, sprites};

/// `(rótulo, cones por fio, pares na subida, acumulado rgb)`.
type Variante = (&'static str, u32, u32, bool);

fn com(v: Variante) -> impl Fn(Config) -> Config {
    move |c: Config| Config {
        hrc_cones_por_fio: v.1,
        hrc_pares_na_subida: v.2,
        hrc_acumulado_rgb: v.3,
        ..c
    }
}

/// As variantes da régua (`PH2D_W5U_VARIANTES=todas` acrescenta as largas).
fn variantes() -> Vec<Variante> {
    let mut v = vec![
        ("base", 1, 0, false),
        ("cones4", 4, 0, false),
        ("pares2", 1, 2, false),
        ("c4p1", 4, 1, false),
        ("c4p2", 4, 2, false),
        ("c8p2", 8, 2, false),
        ("c4p4", 4, 4, false),
    ];
    if std::env::var("PH2D_W5U_VARIANTES").as_deref() == Ok("todas") {
        v.extend([
            ("cones2", 2, 0, false),
            ("cones8", 8, 0, false),
            ("pares1", 1, 1, false),
            ("rgb", 1, 0, true),
        ]);
    }
    v
}

/// O módulo do produto com `de` trocado por `para` (uma vez).
fn mutado(de: &str, para: &str) -> &'static str {
    let s = ph2d_light2d_gpu::fonte_do_rc(Precisao::F32, Precisao::F16);
    assert_eq!(s.matches(de).count(), 1, "a âncora do controlo: {de}");
    Box::leak(s.replacen(de, para, 1).into_boxed_str())
}

/// ⭐ W5u — os cortes contra a base no MESMO processo: a fluência ao BIT, os controlos e o relógio da `c2`.
#[test]
#[ignore = "rodada de medição W5u (placa): os cortes por fio ao bit e o custo, nos três níveis"]
fn medida_luz2d_w5u_por_fio() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!(
        "W5u por fio · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    let controlos: [(&str, &str, &str, Variante); 3] = [
        (
            "cones",
            "soma += metade_da_descida(xi, v, m + 1u, 2u * m + 1u, t_prox);",
            "soma += metade_da_descida(xi, v, m + 1u, 2u * m + 1u, t_ant);",
            ("", 2, 0, false),
        ),
        (
            "pares",
            "let menos = fundir(b_ant, t_de_baixo(2u * xi + 1u, v + 2 * i32(c) - h, c + 1u));",
            "let menos = fundir(b_prox, t_de_baixo(2u * xi + 1u, v + 2 * i32(c) - h, c + 1u));",
            ("", 1, 1, false),
        ),
        (
            "rgb",
            "acumulado_rgb[j + 2u] += f.z;",
            "acumulado_rgb[j + 2u] += f.y;",
            ("", 1, 0, true),
        ),
    ];
    let controlos = controlos.map(|(r, de, para, v)| (r, mutado(de, para), v));
    let vars = variantes();
    let dif = |x: &[u8], y: &[u8]| x.iter().zip(y).filter(|(a, b)| a != b).count();
    let mut todos_ao_bit = true;
    let mut vistos = [0; 3];
    let niveis = match std::env::var("PH2D_W5U_NIVEL").as_deref() {
        Ok("low") => vec![NivelDaLuz::Baixa],
        Ok("medium") => vec![NivelDaLuz::Media],
        Ok("high") => vec![NivelDaLuz::Alta],
        _ => NivelDaLuz::ALL.to_vec(),
    };
    for nivel in niveis {
        for c in &casos() {
            let mut vs: Vec<Vista> = vars
                .iter()
                .map(|&v| Vista::nova_com(&gpu, c.a, nivel, &com(v)))
                .collect();
            let cam = camadas(&gpu, &vs[0], c);
            let sp = sprites(&cam);
            let correr = |v: &mut Vista| {
                for q in 0..80 {
                    v.quadro_com(&gpu, if q < 40 { &c.cena } else { &c.cena2 }, &sp);
                }
            };
            vs.iter_mut().for_each(correr);
            let bytes: Vec<Vec<u8>> = vs
                .iter()
                .map(|v| crate::comum::ler(&gpu, v.vista.fluencia(), 8))
                .collect();
            let diferentes: Vec<usize> = bytes[1..].iter().map(|b| dif(&bytes[0], b)).collect();
            todos_ao_bit &= diferentes.iter().all(|&d| d == 0);
            let mut ctl = [0; 3];
            for (i, (_, modulo, v)) in controlos.iter().enumerate() {
                let mut m = Vista::nova_com(&gpu, c.a, nivel, &|cfg| Config {
                    modulo_rc: Some(*modulo),
                    ..com(*v)(cfg)
                });
                correr(&mut m);
                ctl[i] = dif(&crate::comum::ler(&gpu, m.vista.fluencia(), 8), &bytes[0]);
                vistos[i] += usize::from(ctl[i] > 0);
            }
            let k = vs.len();
            let mut c2: Vec<Vec<f64>> = vec![Vec::new(); k];
            for rodada in 0..7 {
                for j in 0..k {
                    let i = (j + rodada) % k;
                    for _ in 0..3 {
                        vs[i].quadro_com(&gpu, &c.cena2, &sp);
                    }
                    let t = vs[i].vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
                    c2[i].push(t.c2_total());
                }
            }
            let tempos: Vec<String> = vars
                .iter()
                .zip(&c2)
                .map(|(v, t)| format!("{} {:.3} ({:.3})", v.0, minimo(t), mediana(t)))
                .collect();
            eprintln!(
                "U {nivel:?} · {} · bytes diferentes {diferentes:?} de {} (controlos {ctl:?}) · c2 mín (med) {} ms",
                c.rotulo,
                bytes[0].len(),
                tempos.join(" · ")
            );
        }
    }
    assert!(
        vistos.iter().all(|&v| v > 0),
        "um controlo não mudou nenhum caso: a régua não vê esse corte ({vistos:?})"
    );
    eprintln!(
        "W5u por fio · ao bit em todos: {todos_ao_bit} · os controlos mudaram {vistos:?} casos · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
}
