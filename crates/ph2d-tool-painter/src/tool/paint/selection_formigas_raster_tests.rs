//! B20.11b: acima do orçamento as formigas são amostradas em ECRÃ — as quatro bordas a qualquer zoom e
//! rodadas, 1 px de ecrã, marcham pelo passo; o orçamento escolhe o caminho certo dos dois lados.

use super::{caixa_no_ecra, formigas_raster};
use crate::tool::PainterTool;
use crate::tool::paint::selection_contorno::{
    ORCAMENTO_DE_LADOS, contornos_da_mascara, medida_da_fronteira,
};
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_vector::{Affine, Point, VectorScene};
use std::sync::Arc;

const N: usize = 400;
const CAIXA: [usize; 4] = [60, 50, 250, 210];
const ECRA: (u32, u32) = (1600, 1400);

fn retangulo() -> Vec<u8> {
    let [x, y, w, h] = CAIXA;
    let mut m = vec![0u8; N * N];
    for yy in y..y + h {
        m[yy * N + x..yy * N + x + w].fill(255);
    }
    m
}

fn afim(zoom: f64, graus: f64) -> Affine {
    Affine::rotate(graus.to_radians())
        .pre_scale(zoom)
        .then_translate((300.3, 200.6).into())
}

/// A imagem do ecrã inteiro (a caixa colada na origem) e a caixa.
fn ecra(m: &[u8], af: Affine, passo: u32) -> (Vec<u8>, [u32; 4]) {
    let cx = caixa_no_ecra(af, (N as u32, N as u32), ECRA).expect("visível");
    (formigas_raster(m, &[], (N, N), af, cx, passo), cx)
}

fn opaco(img: &[u8], cx: [u32; 4], x: i64, y: i64) -> bool {
    let rw = i64::from(cx[2] - cx[0]);
    let (lx, ly) = (x - i64::from(cx[0]), y - i64::from(cx[1]));
    lx >= 0
        && ly >= 0
        && lx < rw
        && ly < i64::from(cx[3] - cx[1])
        && img[((ly * rw + lx) * 4 + 3) as usize] == 255
}

#[test]
fn as_formigas_amostradas_veem_as_quatro_bordas_a_qualquer_zoom_e_rodadas() {
    let m = retangulo();
    let [x, y, w, h] = CAIXA.map(|v| v as f64);
    let lados = [
        ("cima", (x, y), (x + w, y)),
        ("direita", (x + w, y), (x + w, y + h)),
        ("baixo", (x + w, y + h), (x, y + h)),
        ("esquerda", (x, y + h), (x, y)),
    ];
    for (zoom, graus) in [
        (0.37, 0.0),
        (0.5, 0.0),
        (1.0, 0.0),
        (3.3, 0.0),
        (0.5, 30.0),
        (1.0, 30.0),
    ] {
        let af = afim(zoom, graus);
        let (img, cx) = ecra(&m, af, 3);
        for (nome, a, b) in lados {
            for k in 1..20 {
                let t = f64::from(k) / 20.0;
                let p = af * Point::new(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                let (px, py) = (p.x.floor() as i64, p.y.floor() as i64);
                let perto = (-1..=1).any(|dy| (-1..=1).any(|dx| opaco(&img, cx, px + dx, py + dy)));
                assert!(
                    perto,
                    "zoom {zoom} {graus}°: a borda de {nome} some em t = {t}"
                );
            }
        }
        let c = af * Point::new(x + w / 2.0, y + h / 2.0);
        assert!(
            !opaco(&img, cx, c.x as i64, c.y as i64),
            "o interior fica limpo"
        );
        if graus == 0.0 {
            let n = (cx[0]..cx[2])
                .filter(|&sx| opaco(&img, cx, i64::from(sx), c.y as i64))
                .count();
            assert_eq!(
                n, 2,
                "zoom {zoom}: a linha do meio cruza 2 bordas de 1 px de ecrã"
            );
        }
    }
}

#[test]
fn as_formigas_amostradas_marcham_pelo_passo_e_sao_as_mesmas_dentro_dele() {
    let m = retangulo();
    let af = afim(0.5, 30.0);
    assert_eq!(ecra(&m, af, 5).0, ecra(&m, af, 5).0);
    assert_eq!(ecra(&m, af, 5).0, ecra(&m, af, 13).0, "período de 8 passos");
    assert_ne!(
        ecra(&m, af, 5).0,
        ecra(&m, af, 6).0,
        "o passo seguinte marcha"
    );
}

/// Texels isolados num xadrez de passo 2: 4 cantos cada.
fn pontos(n: usize) -> Vec<u8> {
    let mut m = vec![0u8; N * N];
    for i in 0..n {
        let (x, y) = ((i % (N / 2)) * 2, (i / (N / 2)) * 2);
        m[y * N + x] = 255;
    }
    m
}

fn com_mascara(m: Vec<u8>) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; N * N * 4], N as u32, N as u32);
    t.set_rect_selection(0, 0, 4, 4);
    t.paint.selection_mask = Arc::new(m);
    t
}

#[test]
fn os_cantos_medidos_sao_os_pontos_que_o_contorno_emite() {
    let mut s = 0x9e37_79b9_u32;
    let m: Vec<u8> = (0..64 * 64)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            if s.is_multiple_of(3) { 255 } else { 0 }
        })
        .collect();
    for mm in [m, vec![255; 64 * 64], vec![0; 64 * 64]] {
        let c = contornos_da_mascara(&mm, 64, 64);
        assert_eq!(
            medida_da_fronteira(&mm, 64, 64).0,
            c.iter().map(Vec::len).sum::<usize>()
        );
    }
}

#[test]
fn o_orcamento_escolhe_o_caminho_dos_dois_lados() {
    let af = afim(1.0, 0.0);
    for (n, raster) in [
        (ORCAMENTO_DE_LADOS / 4, false),
        (ORCAMENTO_DE_LADOS / 4 + 1, true),
    ] {
        let t = com_mascara(pontos(n));
        let c = t.selection_contornos().expect("viva");
        assert_eq!(c.no_raster, raster, "{} cantos", 4 * n);
        let mut vs = VectorScene::new();
        t.desenha_formigas_da_selecao(&mut vs, af, ECRA, 2);
        let (larguras, _) = vs.probe_strokes();
        assert_eq!(larguras.is_empty(), raster, "{} cantos: traço", 4 * n);
        assert_eq!(
            vs.probe_image_ids().len(),
            usize::from(raster),
            "{} cantos: imagem",
            4 * n
        );
    }
}

#[test]
fn as_formigas_amostradas_guardadas_so_se_refazem_no_passo_ou_na_vista() {
    let t = com_mascara(pontos(ORCAMENTO_DE_LADOS / 4 + 1));
    let n0 = super::construcoes_das_formigas_raster();
    let desenha = |af, passo| {
        let mut vs = VectorScene::new();
        t.desenha_formigas_da_selecao(&mut vs, af, ECRA, passo);
        vs.probe_image_ids()
    };
    let a = desenha(afim(1.0, 0.0), 2);
    assert_eq!(a, desenha(afim(1.0, 0.0), 2), "parado: a mesma imagem");
    assert_eq!(super::construcoes_das_formigas_raster() - n0, 1);
    assert_ne!(a, desenha(afim(1.0, 0.0), 3), "o passo refaz");
    assert_ne!(
        desenha(afim(1.0, 0.0), 3),
        desenha(afim(2.0, 0.0), 3),
        "a vista refaz"
    );
}
