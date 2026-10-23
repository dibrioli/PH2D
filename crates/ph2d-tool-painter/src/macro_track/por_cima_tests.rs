//! **Aplicar por cima** (W18, `docs/Macros/16_W18_plano.md` §2.7 e §5; decisões do dono 23/10, 5 e 6): a
//! pintura gravada entra na imagem de baixo NO TAMANHO ORIGINAL, onde o gizmo do objecto Macro está — um
//! [`Encaixe`] afim px-da-gravação → px-do-alvo nas posições e na escala do pincel.

use super::em_cena::MacroEmCena;
use super::encaixe::Encaixe;
use super::etapas::imagens_da_gravacao;
use super::fim::documento_no_fim;
use super::inicial::{sobre_documento, tamanho_gravado};
use super::roteiro::{DT_MS, Pincelada, grava_duas_horizontais_ligado_em, grava_nos_meios};
use crate::tool::PainterTool;
use crate::{PaintMedia, PaintedDocument};
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_macro::{MacroDoc, Track, codec};
use std::sync::atomic::AtomicBool;

const GRAVADO: u32 = 64;
const ALVO: u32 = 128;
/// Um instante depois do fim de qualquer gravação destes gates (s).
const DEPOIS_DO_FIM: f64 = 1.0e6;

/// Uma horizontal preta de Digital, de `x = 16` a `48` em `y = 32` (na tela de 64), raio 3.
// LITERAL-COLOR-OK: a cor de uma pincelada GRAVADA, não da UI.
const TRACO: Pincelada = Pincelada {
    cor: [0, 0, 0],
    tamanho_px: 3.0,
    de: [0.25, 0.5],
    ate: [0.75, 0.5],
    curva: 0.0,
    pressao: [1.0, 1.0],
};

/// A gravação do [`TRACO`] numa tela branca de 64: o `Macro.doc` e a trilha.
fn gravacao() -> (MacroDoc, Track) {
    let (bytes, _) = grava_nos_meios(GRAVADO, &[TRACO], &[PaintMedia::Digital]).expect("grava");
    let doc = codec::read(&bytes).expect("lê");
    let track = doc.tracks[0].clone();
    (doc, track)
}

/// Um documento branco `lado`² (o alvo de baixo).
fn branco(lado: u32) -> PaintedDocument {
    let n = (lado * lado * 4) as usize;
    let mut t = PainterTool::default();
    t.set_source(vec![255; n], lado, lado);
    t.documento_vivo(0)
}

/// A imagem de um documento (straight), como o editor a mostra.
fn imagem(doc: PaintedDocument) -> (Vec<u8>, u32, u32) {
    let (w, h) = doc.size;
    let mut t = PainterTool::default();
    t.install_document(1, doc);
    t.bind_document(1, Vec::new(), w, h);
    t.imagem_aplicada().expect("imagem")
}

/// O texel `(x, y)` tem tinta (não é branco).
fn pintado(rgba: &[u8], w: u32, x: u32, y: u32) -> bool {
    let i = ((y * w + x) * 4) as usize;
    rgba[i..i + 3] != [255, 255, 255]
}

/// A caixa `(x0, y0, x1, y1)` (inclusiva) dos texels pintados.
fn caixa(rgba: &[u8], w: u32, h: u32) -> (u32, u32, u32, u32) {
    let mut c = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if pintado(rgba, w, x, y) {
                c = (c.0.min(x), c.1.min(y), c.2.max(x), c.3.max(y));
            }
        }
    }
    assert!(c.0 <= c.2, "nada pintado");
    c
}

/// A trilha `track` com o estado inicial posto sobre `doc` ([`sobre_documento`]).
fn por_cima(track: &Track, doc: &PaintedDocument) -> Track {
    let mut t = track.clone();
    t.initial = sobre_documento(&track.initial, doc).expect("sobre o documento");
    t
}

fn translacao(dx: f64, dy: f64) -> Encaixe {
    Encaixe {
        afim: [1.0, 0.0, dx, 0.0, 1.0, dy],
    }
}

/// O palco de `doc` (com a trilha `track`) no fim, com o encaixe `e`: a tela.
fn palco_no_fim(doc: &MacroDoc, track: Track, e: Option<Encaixe>) -> (Vec<u8>, u32, u32) {
    let mut d = doc.clone();
    d.tracks = vec![track];
    let mut palco = MacroEmCena::de(&codec::write(&d))
        .expect("lê")
        .expect("trilha");
    palco.poe_encaixe(e);
    palco.segue(DEPOIS_DO_FIM).expect("toca");
    palco.tocador().tool().imagem_aplicada().expect("tela")
}

/// ⭐ **Por cima, no tamanho original, onde o gizmo está** (decisão do dono 5 e 6): o traço gravado em 64² e
/// tocado sobre um branco de 128² com o encaixe `+32 px` cai deslocado 32 px, com a MESMA largura — no fim
/// da trilha e no palco. ⛔ VERMELHO antes: o `Encaixe` não existia, e as posições relativas esticavam ×2.
#[test]
fn por_cima_pinta_no_tamanho_original_onde_o_gizmo_esta() {
    let (doc, track) = gravacao();
    let g = imagens_da_gravacao(&track, &AtomicBool::new(false))
        .expect("toca")
        .expect("não parou");
    let (x0, y0, x1, y1) = caixa(&g.fim, g.w, g.h);
    let trilha = por_cima(&track, &branco(ALVO));
    assert_eq!(tamanho_gravado(&trilha.initial), Some((GRAVADO, GRAVADO)));
    // Reaplicar sobre outro documento nunca esquece o tamanho da gravação.
    let de_novo = sobre_documento(&trilha.initial, &branco(96)).expect("de novo");
    assert_eq!(tamanho_gravado(&de_novo), Some((GRAVADO, GRAVADO)));
    assert_eq!(tamanho_gravado(&track.initial), None);

    let e = translacao(32.0, 0.0);
    let (rgba, w, h) = imagem(documento_no_fim(&trilha, Some(e)).expect("fim"));
    assert_eq!((w, h), (ALVO, ALVO), "o alvo fica com o tamanho dele");
    assert_eq!(
        caixa(&rgba, w, h),
        (x0 + 32, y0, x1 + 32, y1),
        "o traço não caiu deslocado 32 px no tamanho gravado (gravado {:?})",
        (x0, y0, x1, y1)
    );
    let (palco, pw, ph) = palco_no_fim(&doc, trilha, Some(e));
    assert_eq!(caixa(&palco, pw, ph), (x0 + 32, y0, x1 + 32, y1), "o palco");
    // Sem encaixe: a identidade em px, encostada ao canto.
    let (sem, w, h) = imagem(documento_no_fim(&por_cima(&track, &branco(ALVO)), None).expect("fim"));
    assert_eq!(caixa(&sem, w, h), (x0, y0, x1, y1), "sem encaixe");
}

/// **Rodar 90° e escalar ×2**: o traço horizontal fica vertical, duas vezes mais comprido, e a espessura do
/// pincel dobra (medida a meio, na perpendicular ao traço).
#[test]
fn por_cima_com_rotacao_e_escala() {
    let (_, track) = gravacao();
    let g = imagens_da_gravacao(&track, &AtomicBool::new(false))
        .expect("toca")
        .expect("não parou");
    let (x0, _, x1, _) = caixa(&g.fim, g.w, g.h);
    let grossura = (0..g.h).filter(|&y| pintado(&g.fim, g.w, 32, y)).count();
    assert!(grossura >= 4, "controlo: o traço gravado tem espessura ({grossura})");
    // x' = −2·y + 128 · y' = 2·x: (32, 32) → (64, 64).
    let e = Encaixe {
        afim: [0.0, -2.0, 128.0, 2.0, 0.0, 0.0],
    };
    assert!((e.escala_do_pincel() - 2.0).abs() < 1e-12);
    let trilha = por_cima(&track, &branco(ALVO));
    let (rgba, w, h) = imagem(documento_no_fim(&trilha, Some(e)).expect("fim"));
    let (cx0, cy0, cx1, cy1) = caixa(&rgba, w, h);
    let grossa = (0..w).filter(|&x| pintado(&rgba, w, x, 64)).count();
    assert!(
        grossa.abs_diff(2 * grossura) <= 2,
        "a espessura não dobrou: {grossa} contra {grossura} gravados"
    );
    let comprido = (cy1 - cy0 + 1) as usize;
    let gravado = (x1 - x0 + 1) as usize;
    assert!(
        comprido.abs_diff(2 * gravado) <= 6,
        "o comprimento não dobrou na vertical: {comprido} contra {gravado} gravados"
    );
    assert!((cx1 - cx0 + 1) as usize == grossa, "o traço não ficou vertical");
    assert!((cx0 + cx1).abs_diff(128) <= 2, "o traço não caiu em x = 64");
}

/// **Uma trilha sem tamanho gravado toca como antes**: o encaixe não a toca (ela pinta a tela dela), e o fim
/// é, ao byte, a imagem do fim da gravação.
#[test]
fn sem_tamanho_gravado_a_trilha_toca_como_antes() {
    let (_, bytes, _) =
        grava_duas_horizontais_ligado_em(96, 1, PaintMedia::Digital).expect("grava");
    let track = codec::read(&bytes).expect("lê").tracks.remove(0);
    assert_eq!(tamanho_gravado(&track.initial), None);
    let fim = imagens_da_gravacao(&track, &AtomicBool::new(false))
        .expect("toca")
        .expect("não parou")
        .fim;
    let sem = imagem(documento_no_fim(&track, None).expect("fim")).0;
    let com = imagem(documento_no_fim(&track, Some(translacao(32.0, 5.0))).expect("fim")).0;
    assert!(sem == fim, "sem encaixe, o fim mudou");
    assert!(com == fim, "um encaixe mudou uma trilha sem tamanho gravado");
}

/// **O documento do fim é o que o palco mostra no fim** (aplicar «novo»: o editor fica com a pintura
/// acabada), com a água assentada.
#[test]
fn documento_no_fim_e_o_fim_do_palco() {
    let (_, bytes, dur) =
        grava_duas_horizontais_ligado_em(96, 1, PaintMedia::WetPaint).expect("grava");
    let track = codec::read(&bytes).expect("lê").tracks.remove(0);
    let mut palco = MacroEmCena::de(&bytes).expect("lê").expect("trilha");
    palco.segue(dur).expect("toca");
    let mut q = 0;
    while palco.corre_a_cauda(DT_MS, None) {
        q += 1;
        assert!(q < 40_000, "a cauda não parou");
    }
    let tela = palco.tocador().tool().imagem_aplicada().expect("tela").0;
    let fim = imagem(documento_no_fim(&track, None).expect("fim")).0;
    assert!(fim == tela, "o documento do fim não é o fim do palco ({q} quadros de cauda)");
}

/// **Mover o objecto Macro muda onde a reprodução seguinte pinta** (decisão do dono 6): o encaixe lê-se
/// vivo, nunca se assa.
#[test]
fn mover_o_objeto_macro_muda_onde_a_proxima_reproducao_pinta() {
    let (doc, track) = gravacao();
    let trilha = por_cima(&track, &branco(ALVO));
    let mut d = doc.clone();
    d.tracks = vec![trilha];
    let mut palco = MacroEmCena::de(&codec::write(&d))
        .expect("lê")
        .expect("trilha");
    palco.poe_encaixe(Some(translacao(0.0, 0.0)));
    palco.segue(DEPOIS_DO_FIM).expect("toca");
    let (a, w, h) = palco.tocador().tool().imagem_aplicada().expect("tela");
    let antes = caixa(&a, w, h);
    palco.poe_encaixe(Some(translacao(40.0, 50.0)));
    palco.segue(DEPOIS_DO_FIM).expect("toca");
    let (b, w, h) = palco.tocador().tool().imagem_aplicada().expect("tela");
    assert_eq!(
        caixa(&b, w, h),
        (antes.0 + 40, antes.1 + 50, antes.2 + 40, antes.3 + 50),
        "a reprodução seguinte não seguiu o objecto (o traço velho ficou?)"
    );
}
