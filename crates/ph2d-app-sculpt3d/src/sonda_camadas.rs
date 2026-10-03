//! 🔎 **SONDA (não é gate) — o compositor do Painter sobre um plano da peça**
//! (`docs/3D/30` §6, o plano das camadas e efeitos na peça).
//!
//! Um plano de tinta fina é uma lista de amostras: o compositor de camadas do
//! Painter (`ph2d_tool_painter::composite`) lê-o como uma imagem de `N × 1`
//! sem uma linha nova. A sonda mede o preço disso nos quatro degraus da peça
//! de fábrica — a peça inteira (abrir, mudar uma camada de modo) e uma faixa de
//! amostras do tamanho do que um quadro de traço suja.

use ph2d_tool_painter::{
    AdjustmentParams, BlendMode, HsbParams, LayerImage, LayerStack, MapPixelSource, Region,
    composite, composite_region,
};
use std::time::Instant;

fn camada(n: usize, semente: u32) -> LayerImage {
    let mut rgba8 = vec![0u8; n * 4];
    for (i, px) in rgba8.chunks_exact_mut(4).enumerate() {
        let h = (i as u32).wrapping_mul(2_654_435_761) ^ semente;
        px.copy_from_slice(&[
            (h >> 8) as u8,
            (h >> 16) as u8,
            (h >> 24) as u8,
            (h & 0xff) as u8,
        ]);
    }
    LayerImage {
        width: n as u32,
        height: 1,
        rgba8,
    }
}

#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_o_compositor_do_painter_sobre_o_plano_da_peca() {
    for n in [47_106usize, 188_418, 753_666, 3_014_658] {
        let mut pilha = LayerStack::new();
        let mut fonte = MapPixelSource::default();
        for (k, modo) in [BlendMode::Normal, BlendMode::Multiply, BlendMode::Overlay]
            .into_iter()
            .enumerate()
        {
            let id = pilha
                .add_raster(format!("c{k}"), n as u32, 1)
                .expect("camada");
            pilha.get_mut(id).expect("camada").blend_mode = modo;
            fonte.images.insert(id, camada(n, k as u32 * 77));
        }
        let ajuste = pilha
            .add_adjustment(ph2d_tool_painter::AdjustmentKind::HueSaturationBrightness)
            .expect("ajuste");
        pilha.adjustment_mut(ajuste).expect("ajuste").params =
            AdjustmentParams::HueSaturationBrightness(HsbParams {
                h: 30.0,
                s: 0.2,
                b: 0.1,
            });
        let mut pior = 0.0f64;
        let mut soma = 0.0f64;
        for _ in 0..5 {
            let t = Instant::now();
            std::hint::black_box(composite(&pilha, &fonte, n as u32, 1));
            let ms = t.elapsed().as_secs_f64() * 1e3;
            pior = pior.max(ms);
            soma += ms;
        }
        // Uma faixa de 1 024 amostras contíguas — a ordem do que um quadro de
        // traço suja (mediana 137, máx 323 sujas; as de uma face são contíguas).
        let faixa = Region {
            x: (n / 2) as u32,
            y: 0,
            w: 1024,
            h: 1,
        };
        let t = Instant::now();
        for _ in 0..100 {
            std::hint::black_box(composite_region(&pilha, &fonte, n as u32, 1, faixa));
        }
        let faixa_ms = t.elapsed().as_secs_f64() * 1e3 / 100.0;
        // A MESMA peça dobrada em linhas de 1 024 (a composição ponto a ponto
        // não depende da forma): o compositor reparte as linhas pelos núcleos.
        let largura = 1024u32;
        let altura = (n as u32).div_ceil(largura);
        let mut dobrada = MapPixelSource::default();
        for (id, img) in &fonte.images {
            let mut rgba8 = img.rgba8.clone();
            rgba8.resize((largura * altura * 4) as usize, 0);
            dobrada.images.insert(
                *id,
                LayerImage {
                    width: largura,
                    height: altura,
                    rgba8,
                },
            );
        }
        let mut pilha_d = pilha.clone();
        let ids: Vec<_> = pilha_d.all_ids().collect();
        for id in ids {
            if let Some(ph2d_tool_painter::LayerKind::Raster(r)) =
                pilha_d.get_mut(id).map(|l| &mut l.kind)
            {
                r.width = largura;
                r.height = altura;
            }
        }
        let t = Instant::now();
        for _ in 0..5 {
            std::hint::black_box(composite(&pilha_d, &dobrada, largura, altura));
        }
        let dobrada_ms = t.elapsed().as_secs_f64() * 1e3 / 5.0;
        eprintln!(
            "{n} amostras · 3 camadas + HSB · a peça inteira N×1 média {:.2} ms pior {pior:.2} ms · \
             dobrada 1024×{altura} {dobrada_ms:.2} ms · uma faixa de 1 024 amostras {faixa_ms:.4} ms · \
             RGBA8 por camada {:.1} MB",
            soma / 5.0,
            (n * 4) as f64 / 1e6
        );
    }
}
