//! **O papel nas TRÊS saídas do compositor** (2026-10-06): com `LayerCompositor::set_paper`, a saída
//! de `cs_flat` (pilha plana), `cs_grouped` (com grupos) e `cs_encode` (com um ajuste espacial, o
//! grafo segmentado) é a saída SEM papel composta sobre ele pela lei inteira da CPU — ao byte. A
//! paridade do produtor (`the_gpu_producer_shows_what_the_cpu_producer_shows`) só percorre a primeira.
//!
//! `#[ignore]`: precisa de adapter — `-- --ignored`.

use super::layer_compositor_gpu::{MapProvider, try_headless_gpu};
use ph2d_render::{LayerCompositor, LayerOp, Region, SPATIAL_GAUSSIAN};

const PAPEL: [u8; 3] = [196, 170, 120];

/// A lei do Painter (`papel::sobre_o_papel`), transcrita: `(c·a + p·(255−a) + 127)/255`, alfa 255.
fn sobre_o_papel(px: &mut [u8], p: [u8; 3]) {
    let a = u32::from(px[3]);
    if a == 255 {
        return;
    }
    for c in 0..3 {
        px[c] = ((u32::from(px[c]) * a + u32::from(p[c]) * (255 - a) + 127) / 255) as u8;
    }
    px[3] = 255;
}

/// Uma tela variada com MUITO alfa parcial (o papel só aparece onde a pilha é translúcida).
fn tela(w: u32, h: u32, semente: u32) -> Vec<u8> {
    (0..w * h)
        .flat_map(|p| {
            [
                ((p * 37 + semente * 11) % 256) as u8,
                ((p * 53 + semente * 29) % 256) as u8,
                ((p * 97 + semente * 7) % 256) as u8,
                ((p * 13 + semente) % 200) as u8,
            ]
        })
        .collect()
}

fn camada(key: u64, blend_mode: u8, opacity: f32) -> LayerOp {
    LayerOp::Layer {
        mask: None,
        clipping: false,
        key,
        blend_mode,
        opacity,
    }
}

#[test]
#[ignore = "needs a GPU device"]
fn o_papel_compoe_se_sob_as_tres_saidas() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (32u32, 24u32);
    let mut prov = MapProvider::default();
    for k in 0..3u64 {
        prov.insert(k, 1, tela(w, h, k as u32 + 1));
    }
    let pilhas: [(&str, Vec<LayerOp>); 3] = [
        ("cs_flat", vec![camada(0, 0, 1.0), camada(1, 1, 0.6)]),
        (
            "cs_grouped",
            vec![
                camada(0, 0, 1.0),
                LayerOp::PushGroup,
                camada(1, 6, 0.8),
                camada(2, 1, 0.7),
                LayerOp::PopGroup {
                    blend_mode: 0,
                    opacity: 0.9,
                },
            ],
        ),
        (
            "cs_encode",
            vec![
                camada(0, 0, 1.0),
                LayerOp::SpatialAdjustment {
                    kernel: SPATIAL_GAUSSIAN,
                    params: [2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                    blend_mode: 0,
                    opacity: 1.0,
                },
                camada(1, 0, 0.6),
            ],
        ),
    ];
    let regiao = Region::full(w, h);
    for (nome, ops) in pilhas {
        let mut comp = LayerCompositor::new(&gpu);
        comp.set_paper(None);
        comp.composite(&gpu, &ops, &prov, w, h, regiao)
            .expect("composite sem papel");
        let sem = comp.read_output(&gpu).expect("readback sem papel");
        assert!(
            sem.chunks(4).filter(|p| p[3] < 255).count() > 100,
            "controlo: {nome}: a pilha tem alfa parcial (senão o papel não aparece)"
        );
        comp.set_paper(Some(PAPEL));
        comp.composite(&gpu, &ops, &prov, w, h, regiao)
            .expect("composite com papel");
        let com = comp.read_output(&gpu).expect("readback com papel");
        let mut quer = sem.clone();
        quer.chunks_mut(4).for_each(|p| sobre_o_papel(p, PAPEL));
        let difere = com.iter().zip(&quer).filter(|(a, b)| a != b).count();
        assert_eq!(
            difere, 0,
            "{nome}: com papel a saída não é a saída sem papel composta sobre ele ({difere} bytes)"
        );
        comp.set_paper(None);
        comp.composite(&gpu, &ops, &prov, w, h, regiao)
            .expect("composite sem papel outra vez");
        assert_eq!(
            comp.read_output(&gpu).expect("readback"),
            sem,
            "{nome}: `set_paper(None)` não volta ao composite de sempre"
        );
    }
}
