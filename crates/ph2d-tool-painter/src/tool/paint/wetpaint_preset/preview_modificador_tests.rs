//! ⭐ **O preview de uma ferramenta que mexe na tinta cruza a tinta de referência** (dono 23/10): o cruzamento
//! muda a faixa vertical (a ferramenta agiu), fora da passagem tudo fica igual à referência sozinha, e mexer
//! num ajuste da ferramenta muda o preview.

use super::super::preview::{
    ALTURA_DO_PREVIEW, LARGURA_DO_PREVIEW, pintar_preview, reduz, tela_do_preview,
};
use super::{Modificador, faixa, pinta_a_referencia};
use crate::{PaintMedia, PainterTool};
use ph2d_editor_core::tool::RasterEditTool;

const RAIO: f32 = 10.0;

/// A ferramenta `m` na mão, com o pincel de fábrica dela (o slot do trilho), no meio `meio`.
fn na_mao(meio: PaintMedia, m: Modificador) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255; 64 * 64 * 4], 64, 64);
    t.set_paint_media(meio);
    t.set_paint_tool_mode(m.fio());
    t.paint.brush.radius_px = RAIO;
    t
}

fn ultimo_quadro(t: &PainterTool) -> Vec<u8> {
    let cena = t.wet_preview_cena();
    let mut ultimo = Vec::new();
    assert!(pintar_preview(&cena, &|| false, &mut |px, _, _| {
        ultimo = px.to_vec();
    }));
    ultimo
}

/// A faixa de referência SOZINHA, reduzida como o preview.
fn so_a_referencia() -> Vec<u8> {
    let (lw, lh) = tela_do_preview(RAIO);
    let mut t = PainterTool::default();
    t.set_source(vec![255; (lw * lh * 4) as usize], lw, lh);
    pinta_a_referencia(&mut t, lw, lh);
    let (px, w, h) = t.take_preview_arc().expect("a faixa sujou a tela");
    reduz(&px, w, h, LARGURA_DO_PREVIEW, ALTURA_DO_PREVIEW)
}

/// `(texels diferentes DENTRO da faixa e da passagem, texels diferentes FORA da passagem)`.
fn onde_mudou(referencia: &[u8], depois: &[u8]) -> (usize, usize) {
    let (lw, lh) = tela_do_preview(RAIO);
    let k = LARGURA_DO_PREVIEW as f32 / lw as f32;
    let (cx, meia) = faixa(lw, lh);
    let (cx, meia) = (cx * k, meia * k);
    let cy = ALTURA_DO_PREVIEW as f32 * 0.5;
    // A passagem: o raio da ferramenta com folga para a orla macia, o arrasto e o desfoque.
    let banda = (RAIO * 3.0 + 6.0) * k;
    let (mut dentro, mut fora) = (0, 0);
    for (i, (a, b)) in referencia
        .as_chunks::<4>()
        .0
        .iter()
        .zip(depois.as_chunks::<4>().0)
        .enumerate()
    {
        let (x, y) = (
            (i as u32 % LARGURA_DO_PREVIEW) as f32 + 0.5,
            (i as u32 / LARGURA_DO_PREVIEW) as f32 + 0.5,
        );
        if a == b {
            continue;
        }
        if (y - cy).abs() > banda {
            fora += 1;
        } else if (x - cx).abs() < meia {
            dentro += 1;
        }
    }
    (dentro, fora)
}

#[test]
fn a_ferramenta_que_mexe_na_tinta_cruza_a_faixa_de_referencia() {
    let referencia = so_a_referencia();
    let mut falhas = Vec::new();
    for (meio, m) in [
        (PaintMedia::Digital, Modificador::Borracha),
        (PaintMedia::Digital, Modificador::Blur),
        (PaintMedia::Digital, Modificador::Smear),
        (PaintMedia::Digital, Modificador::Clone),
        (PaintMedia::Impasto, Modificador::Faca),
    ] {
        let t = na_mao(meio, m);
        assert_eq!(
            Modificador::de(&t),
            Some(m),
            "{m:?}: a ferramenta não ficou na mão"
        );
        let (dentro, fora) = onde_mudou(&referencia, &ultimo_quadro(&t));
        eprintln!("{m:?}: {dentro} texels mudaram no cruzamento, {fora} fora da passagem");
        if dentro < 50 || fora > 0 {
            falhas.push(format!("{m:?}: cruzamento {dentro}, fora {fora}"));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// A borracha do preview APAGA a faixa (não pinta por cima): no centro do cruzamento a tinta de referência sai.
#[test]
fn a_borracha_do_preview_apaga_a_faixa() {
    let px = ultimo_quadro(&na_mao(PaintMedia::Digital, Modificador::Borracha));
    let (lw, lh) = tela_do_preview(RAIO);
    let k = LARGURA_DO_PREVIEW as f32 / lw as f32;
    let (cx, _) = faixa(lw, lh);
    let (x, y) = ((cx * k) as u32, ALTURA_DO_PREVIEW / 2);
    let i = ((y * LARGURA_DO_PREVIEW + x) * 4) as usize;
    let referencia = so_a_referencia();
    assert_eq!(
        &referencia[i..i + 3],
        &super::COR_DA_REFERENCIA,
        "controlo: a faixa passa ali"
    );
    // Sem papel, o apagado mostra o branco da folha (nunca o fundo escuro do painel).
    assert!(
        px[i..i + 3].iter().all(|&c| c > 230),
        "a borracha não apagou a faixa: {:?}",
        &px[i..i + 4]
    );
}

/// O controlo de não-vácuo: a força da ferramenta muda o preview dela (o preview lê o pincel da ferramenta).
#[test]
fn mexer_na_forca_da_ferramenta_muda_o_preview_dela() {
    for m in [Modificador::Borracha, Modificador::Blur, Modificador::Smear] {
        let mut t = na_mao(PaintMedia::Digital, m);
        t.paint.brush.strength = 1.0;
        let forte = ultimo_quadro(&t);
        t.paint.brush.strength = 0.3;
        assert_ne!(
            forte,
            ultimo_quadro(&t),
            "{m:?}: a força não chegou ao preview"
        );
    }
}

/// O pincel que PINTA continua com o S de sempre (nada de faixa de referência).
#[test]
fn o_pincel_continua_sem_a_faixa() {
    let mut t = na_mao(PaintMedia::Digital, Modificador::Borracha);
    t.set_paint_tool_mode("brush");
    assert_eq!(Modificador::de(&t), None);
    let (lw, _) = tela_do_preview(RAIO);
    let k = LARGURA_DO_PREVIEW as f32 / lw as f32;
    let (cx, _) = faixa(lw, lw / 2);
    // Na base da faixa (longe do S, que vive no terço de cima) o pincel não pôs tinta nenhuma.
    let px = ultimo_quadro(&t);
    let (x, y) = ((cx * k) as u32, ALTURA_DO_PREVIEW - 4);
    let i = ((y * LARGURA_DO_PREVIEW + x) * 4) as usize;
    assert_eq!(&px[i..i + 3], &[255, 255, 255], "o pincel ganhou a faixa");
}
