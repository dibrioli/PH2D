//! ⭐ W5r — a cascata holográfica por COLUNAS (`Config::hrc_por_colunas`) e com os arcos numa tabela
//! (`Config::hrc_arcos_em_tabela`) contra o antes da onda, no MESMO processo: a fluência da vista ao BIT
//! (80 quadros, os casos das réguas W5q), o relógio da placa da `c2` (o mínimo de 7 rodadas
//! intercaladas) e as etapas por nível (o instrumento das fases).

use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_gpu::Config;

use crate::medida_luz2d_w5o::{mediana, minimo};
use crate::medida_luz2d_w5p::Vista;

/// `(rótulo, por colunas, arcos em tabela)`: o antes da onda, o 1.º corte, os dois.
const VARIANTES: [(&str, bool, bool); 3] = [
    ("linhas", false, false),
    ("colunas", true, false),
    ("colunas+tabela", true, true),
];

/// `(por colunas, arcos em tabela, o «+1»)`.
fn variante(colunas: bool, tabela: bool, mais_um: bool) -> impl Fn(Config) -> Config {
    move |c: Config| Config {
        hrc_por_colunas: colunas,
        hrc_arcos_em_tabela: tabela,
        holografica: c
            .holografica
            .map(|h| ph2d_light2d_gpu::Holografica { mais_um, ..h }),
        ..c
    }
}

#[test]
#[ignore = "rodada de medição W5r (placa): a holográfica por colunas e com a tabela, ao bit e o custo"]
fn medida_luz2d_w5r_colunas() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!(
        "W5r colunas · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    let casos = crate::medida_luz2d_w5q::casos();
    let dif = |x: &[u8], y: &[u8]| x.iter().zip(y).filter(|(a, b)| a != b).count();
    let mut todos_ao_bit = true;
    let mut vistos = 0;
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
        for (rotulo, cena, cena2, a) in &casos {
            let a = *a;
            let correr = |v: &mut Vista| {
                for q in 0..80 {
                    v.quadro(&gpu, if q < 40 { cena } else { cena2 });
                }
            };
            let mut vs = VARIANTES.map(|(_, c, t)| {
                let mut v = Vista::nova_com(&gpu, a, nivel, &variante(c, t, true));
                correr(&mut v);
                v
            });
            let bytes = vs
                .each_ref()
                .map(|v| crate::comum::ler(&gpu, v.vista.fluencia(), 8));
            let diferentes = [dif(&bytes[0], &bytes[1]), dif(&bytes[0], &bytes[2])];
            todos_ao_bit &= diferentes == [0, 0];
            // O controlo: sem o «+1» (a soma, que esta onda também reescreve) muda bytes.
            let mut m = Vista::nova_com(&gpu, a, nivel, &variante(true, true, false));
            correr(&mut m);
            let controlo = dif(&crate::comum::ler(&gpu, m.vista.fluencia(), 8), &bytes[2]);
            vistos += usize::from(controlo > 0);
            let mut c2 = [f64::INFINITY; 3];
            for rodada in 0..7 {
                for j in 0..3 {
                    let i = (j + rodada) % 3;
                    for _ in 0..3 {
                        vs[i].quadro(&gpu, cena2);
                    }
                    let t = vs[i].vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
                    c2[i] = c2[i].min(t.c2_total());
                }
            }
            eprintln!(
                "{nivel:?} · {rotulo} · bytes diferentes {diferentes:?} de {} (controlo {controlo}) · c2 linhas {:.3} → colunas {:.3} → +tabela {:.3} ms",
                bytes[0].len(),
                c2[0],
                c2[1],
                c2[2]
            );
        }
    }
    assert!(
        vistos > 0,
        "o controlo (sem o «+1») não mudou nenhum caso: a régua não vê a soma"
    );
    // As etapas por nível, as três variantes alternadas (o High, a cena 5 de fábrica).
    let (_, cena, _, a) = &casos[4];
    let mut fs = VARIANTES.map(|(_, c, t)| {
        let mut v = Vista::nova_com(&gpu, *a, NivelDaLuz::Alta, &variante(c, t, true));
        assert!(v.vista.medir_fases_da_hrc(&gpu), "TIMESTAMP_QUERY");
        v
    });
    let mut por: Vec<(&'static str, u32, [Vec<f64>; 3])> = Vec::new();
    for q in 0..36 {
        let i = q % 3;
        fs[i].quadro(&gpu, cena);
        for (etapa, nivel, ms, _) in fs[i].vista.fases_da_hrc(&gpu).expect("fases") {
            match por.iter_mut().find(|o| o.0 == etapa && o.1 == nivel) {
                Some(o) => o.2[i].push(ms),
                None => {
                    let mut v = [Vec::new(), Vec::new(), Vec::new()];
                    v[i].push(ms);
                    por.push((etapa, nivel, v));
                }
            }
        }
    }
    let mut soma = [0.0; 3];
    for (etapa, nivel, ms) in &por {
        for (s, m) in soma.iter_mut().zip(ms) {
            *s += minimo(m);
        }
        eprintln!(
            "F {etapa} n{nivel} · mín (med) {:.3} ({:.3}) → {:.3} ({:.3}) → {:.3} ({:.3}) ms",
            minimo(&ms[0]),
            mediana(&ms[0]),
            minimo(&ms[1]),
            mediana(&ms[1]),
            minimo(&ms[2]),
            mediana(&ms[2])
        );
    }
    eprintln!(
        "S soma das etapas · linhas {:.3} → colunas {:.3} → +tabela {:.3} ms",
        soma[0], soma[1], soma[2]
    );
    eprintln!(
        "W5r colunas · ao bit em todos: {todos_ao_bit} · o controlo mudou {vistos} casos · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
}
