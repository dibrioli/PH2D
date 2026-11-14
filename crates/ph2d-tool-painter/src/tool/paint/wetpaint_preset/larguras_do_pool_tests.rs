//! **B43.4 — o mesmo bit em qualquer número de fios** (lei da missão 4K): um pincel de fábrica por par (meio,
//! ferramenta) pinta a miniatura num pool de 1, 2, 3, 7, 16 e 32 fios, e cada uma é a miniatura APROVADA ao byte.
//! As bandas paralelas do motor só reduzem por `min`/`max`/`||` (censo em `medicoes/N10_build.md` §B43.4); este
//! portão prova que a repartição das linhas nunca muda o pixel. Imprime o FNV de cada pincel (B43.3: o CI compara-o
//! entre sistemas). Mora no `cfg(test)` da crate: a fila do dab é síncrona aqui, e o `install` não esbarra na
//! guarda do Bug #66 (espera pelo depósito dentro de um job do rayon).

use crate::{ferramenta_do_preset, miniatura_do_preset};

#[test]
fn o_mesmo_pincel_pinta_o_mesmo_bit_em_qualquer_largura_do_pool() {
    let fnv = |v: &[u8]| {
        v.iter().fold(0xcbf2_9ce4_8422_2325u64, |a, x| {
            (a ^ u64::from(*x)).wrapping_mul(0x0100_0000_01b3)
        })
    };
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let mut vistos: Vec<(String, String)> = Vec::new();
    let mut escolhidos = Vec::new();
    for e in ph2d_brush_presets::Biblioteca::carregar(&raiz, None).entradas {
        let chave = (
            e.preset.parametros.media.clone(),
            format!("{:?}", ferramenta_do_preset(&e.preset.parametros)),
        );
        if !vistos.contains(&chave) {
            vistos.push(chave);
            escolhidos.push((e.relativo, e.preset));
        }
    }
    assert!(escolhidos.len() >= 4, "a amostra perdeu meios: {vistos:?}");
    for (rel, p) in &escolhidos {
        let aprovada = fnv(&p.miniatura.rgba);
        let cheio = fnv(&miniatura_do_preset(p).rgba);
        let mut fitas = vec![];
        for fios in [1, 2, 3, 7, 16, 32] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(fios)
                .build()
                .expect("o pool");
            let hoje = pool.install(|| miniatura_do_preset(p));
            fitas.push((fios, fnv(&hoje.rgba)));
        }
        println!("pincel-fnv {rel} cheio {cheio:#018x} aprovada {aprovada:#018x} fios {fitas:x?}");
        for (fios, h) in fitas {
            assert_eq!(
                h, cheio,
                "{rel}: {fios} fios pintam outro bit que o pool cheio"
            );
        }
    }
}
