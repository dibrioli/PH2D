//! ⭐ W5t — a holográfica despachada em 3D (`Config::hrc_grade_3d`): a traça, a subida e a descida leem
//! `(v, xi, k)` do índice do fio, sem as divisões do 1D. A régua, no MESMO processo, sobre os casos da W5s:
//! a fluência ao BIT contra o 1D (80 quadros), um controlo que muda bytes (o texto do módulo com a subida em
//! 3D a ler a linha de baixo) e o relógio da `c2` (o mínimo de 7 rodadas intercaladas). Os cortes de BYTES
//! (o T esparso e o R em 6 bytes) foram medidos e recusados: handoff W5t §1.

use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_gpu::{Config, Precisao};

use crate::medida_luz2d_w5o::{mediana, minimo};
use crate::medida_luz2d_w5p::Vista;
use crate::medida_luz2d_w5s::{camadas, casos, sprites};

fn variante(em_3d: bool) -> impl Fn(Config) -> Config {
    move |c: Config| Config {
        hrc_grade_3d: em_3d,
        ..c
    }
}

const VARIANTES: [(&str, bool); 2] = [("1D", false), ("3D", true)];

/// O módulo do produto com `de` trocado por `para` (uma vez).
fn mutado(de: &str, para: &str) -> &'static str {
    let s = ph2d_light2d_gpu::fonte_do_rc(Precisao::F32, Precisao::F16, true);
    assert_eq!(s.matches(de).count(), 1, "a âncora do controlo: {de}");
    Box::leak(s.replacen(de, para, 1).into_boxed_str())
}

/// ⭐ W5t — o 3D contra o 1D no MESMO processo: a fluência ao BIT, o controlo e o relógio da `c2`.
#[test]
#[ignore = "rodada de medição W5t (placa): o despacho em 3D ao bit e o custo"]
fn medida_luz2d_w5t_3d() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!(
        "W5t 3D · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    let controlo = mutado(
        "empacotar(fusao_de_baixo(gid.z, gid.y, i32(gid.x)));",
        "empacotar(fusao_de_baixo(gid.z, gid.y, i32(gid.x) - 1));",
    );
    let dif = |x: &[u8], y: &[u8]| x.iter().zip(y).filter(|(a, b)| a != b).count();
    let mut todos_ao_bit = true;
    let mut vistos = 0;
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
        for c in &casos() {
            let mut vs = VARIANTES.map(|(_, d)| Vista::nova_com(&gpu, c.a, nivel, &variante(d)));
            let cam = camadas(&gpu, &vs[0], c);
            let sp = sprites(&cam);
            let correr = |v: &mut Vista| {
                for q in 0..80 {
                    v.quadro_com(&gpu, if q < 40 { &c.cena } else { &c.cena2 }, &sp);
                }
            };
            vs.iter_mut().for_each(correr);
            let bytes = vs
                .each_ref()
                .map(|v| crate::comum::ler(&gpu, v.vista.fluencia(), 8));
            let diferentes = dif(&bytes[0], &bytes[1]);
            todos_ao_bit &= diferentes == 0;
            let mut m = Vista::nova_com(&gpu, c.a, nivel, &|cfg| Config {
                modulo_rc: Some(controlo),
                ..variante(true)(cfg)
            });
            correr(&mut m);
            let ctl = dif(&crate::comum::ler(&gpu, m.vista.fluencia(), 8), &bytes[0]);
            vistos += usize::from(ctl > 0);
            drop(m);
            let mut c2: [Vec<f64>; 2] = Default::default();
            for rodada in 0..7 {
                for j in 0..2 {
                    let i = (j + rodada) % 2;
                    for _ in 0..3 {
                        vs[i].quadro_com(&gpu, &c.cena2, &sp);
                    }
                    let t = vs[i].vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
                    c2[i].push(t.c2_total());
                }
            }
            let tempos: Vec<String> = VARIANTES
                .iter()
                .zip(&c2)
                .map(|((r, _), t)| format!("{r} {:.3} ({:.3})", minimo(t), mediana(t)))
                .collect();
            eprintln!(
                "{nivel:?} · {} · bytes diferentes {diferentes} de {} (controlo {ctl}) · c2 mín (med) {} ms",
                c.rotulo,
                bytes[0].len(),
                tempos.join(" · ")
            );
        }
    }
    assert!(
        vistos > 0,
        "o controlo não mudou nenhum caso: a régua não vê o 3D"
    );
    eprintln!(
        "W5t 3D · ao bit em todos: {todos_ao_bit} · o controlo mudou {vistos} casos · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
}
