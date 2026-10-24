//! ⭐ **O preview da Borracha** (dono 24/10): a tela cheia de [`COR_DO_FUNDO_DA_BORRACHA`] e o S do Pincel apagado
//! nela; mexer num ajuste da borracha muda o preview; o Pincel continua com o S sobre o branco.

use super::super::preview::{ALTURA_DO_PREVIEW, LARGURA_DO_PREVIEW, pintar_preview};
use super::{COR_DO_FUNDO_DA_BORRACHA, e_a_borracha};
use crate::{PaintMedia, PainterTool};
use ph2d_editor_core::tool::RasterEditTool;

const RAIO: f32 = 10.0;

/// A ferramenta `fio` na mão, com o pincel de fábrica dela (o slot do trilho), no meio `meio`.
fn na_mao(meio: PaintMedia, fio: &str) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255; 64 * 64 * 4], 64, 64);
    t.set_paint_media(meio);
    t.set_paint_tool_mode(fio);
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

/// `true` onde o texel se afasta de `cor` (algum canal por mais de 48).
fn longe_de(px: &[u8], cor: [u8; 3]) -> Vec<bool> {
    px.as_chunks::<4>()
        .0
        .iter()
        .map(|p| (0..3).any(|c| p[c].abs_diff(cor[c]) > 48))
        .collect()
}

/// Só a borracha do modo Paint leva a cena dela; as ferramentas sem preview e o Pincel não.
#[test]
fn so_a_borracha_e_a_borracha() {
    assert!(e_a_borracha(&na_mao(PaintMedia::Digital, "eraser")));
    for (meio, fio) in [
        (PaintMedia::Digital, "brush"),
        (PaintMedia::Digital, "blur"),
        (PaintMedia::Digital, "smear"),
        (PaintMedia::Digital, "clone"),
        (PaintMedia::Digital, "mask"),
        (PaintMedia::Impasto, "knife"),
    ] {
        assert!(!e_a_borracha(&na_mao(meio, fio)), "{fio}");
    }
}

/// ⭐ **A tela cheia e o S do Pincel apagado** (dono 24/10). As margens são a cor cheia, o S mostra o branco da
/// folha, e a pegada dele é a do S que o Pincel pinta no branco.
#[test]
fn a_borracha_apaga_o_s_do_pincel_na_tela_cheia() {
    let borracha = ultimo_quadro(&na_mao(PaintMedia::Digital, "eraser"));
    let (w, h) = (LARGURA_DO_PREVIEW as usize, ALTURA_DO_PREVIEW as usize);
    let em = |x: usize, y: usize| &borracha[(y * w + x) * 4..(y * w + x) * 4 + 3];
    for (x, y) in [
        (0, 0),
        (w - 1, 0),
        (0, h - 1),
        (w - 1, h - 1),
        (w / 2, h - 2),
    ] {
        assert_eq!(
            em(x, y),
            &COR_DO_FUNDO_DA_BORRACHA,
            "a margem ({x},{y}) não é a cor cheia"
        );
    }
    // O Pincel com a PONTA da Borracha (ela tem o seu slot desde 24/10): a sobreposição mede o caminho do S.
    let mut p = na_mao(PaintMedia::Digital, "brush");
    p.paint.brush.hardness = na_mao(PaintMedia::Digital, "eraser").paint.brush.hardness;
    let pincel = ultimo_quadro(&p);
    let apagado = longe_de(&borracha, COR_DO_FUNDO_DA_BORRACHA);
    let pintado = longe_de(&pincel, [255, 255, 255]);
    let branco = borracha
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..3].iter().all(|&c| c > 230))
        .count();
    let conta = |f: &dyn Fn(bool, bool) -> bool| {
        apagado
            .iter()
            .zip(&pintado)
            .filter(|(a, p)| f(**a, **p))
            .count()
    };
    let (a, p) = (conta(&|a, _| a), conta(&|_, p| p));
    let sobre = conta(&|a, p| a && p) as f32 / conta(&|a, p| a || p).max(1) as f32;
    eprintln!(
        "borracha: {a} apagados ({branco} brancos), pincel: {p} pintados, sobreposição {sobre:.3}"
    );
    assert!(
        branco >= 500,
        "o S não mostra a folha: {branco} texels brancos"
    );
    assert!(
        sobre >= 0.9,
        "a pegada da borracha não é o S do pincel: {sobre:.3}"
    );
}

/// O controlo de não-vácuo: a força da borracha muda o preview dela (o preview lê o pincel da ferramenta).
#[test]
fn mexer_na_forca_da_borracha_muda_o_preview_dela() {
    let mut t = na_mao(PaintMedia::Digital, "eraser");
    t.paint.brush.strength = 1.0;
    let forte = ultimo_quadro(&t);
    t.paint.brush.strength = 0.3;
    assert_ne!(forte, ultimo_quadro(&t), "a força não chegou ao preview");
}

/// O pincel que PINTA continua com o S sobre o branco (nada de fundo cheio).
#[test]
fn o_pincel_continua_no_branco() {
    let px = ultimo_quadro(&na_mao(PaintMedia::Digital, "brush"));
    let i = ((ALTURA_DO_PREVIEW - 4) * LARGURA_DO_PREVIEW * 4) as usize;
    assert_eq!(&px[i..i + 3], &[255, 255, 255], "o pincel ganhou o fundo");
}
