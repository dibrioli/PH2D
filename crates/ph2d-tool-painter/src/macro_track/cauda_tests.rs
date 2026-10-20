//! **A água depois do fim do clipe** (W14, `docs/Macros/12_W14_a_agua_depois_do_parar.md`; decisão do dono
//! 20/10): com o clipe no fim, a água da gravação continua no palco até à imagem do artista — assentada,
//! ou parada onde o 1.º toque dele a assou (W12). O oráculo é o documento do editor, nunca o palco.

use super::MarcaDaAgua;
use super::em_cena::MacroEmCena;
use super::etapas::imagens_da_gravacao;
use super::marca::{MarcaDoArtista, epoca_da_gravacao};
use super::roteiro::{DT_MS, grava_duas_horizontais_ligado_em, grava_no_papel_ligado_com_o_worker};
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};
use std::sync::atomic::AtomicBool;

const L: u32 = 96;

fn tique(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(DT_MS);
}

fn assenta(t: &mut PainterTool) -> usize {
    for q in 0..40_000 {
        if t.agua_parada() {
            return q;
        }
        tique(t);
    }
    panic!("a água não assentou");
}

/// `(texels que diferem, o maior desvio em níveis)`.
fn diferem(a: &[u8], b: &[u8]) -> (usize, u8) {
    assert_eq!(a.len(), b.len(), "tamanhos");
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .map(|(x, y)| x.iter().zip(y).map(|(p, q)| p.abs_diff(*q)).max().unwrap_or(0))
        .filter(|&d| d > 0)
        .fold((0, 0), |(n, m), d| (n + 1, m.max(d)))
}

/// O palco no fim do clipe de `bytes` (`dur` s): a tela no Parar e o palco.
fn no_fim(bytes: &[u8], dur: f64) -> (MacroEmCena, Vec<u8>) {
    let mut palco = MacroEmCena::de(bytes).expect("lê").expect("trilha");
    palco.segue(dur).expect("toca");
    let tela = palco.tocador().tool().imagem_aplicada().expect("tela").0;
    (palco, tela)
}

/// A cauda quadro a quadro até parar: a imagem do palco depois de cada quadro que andou.
fn cauda(palco: &mut MacroEmCena, ate: Option<MarcaDaAgua>) -> Vec<Vec<u8>> {
    let mut telas = Vec::new();
    for _ in 0..40_000 {
        if !palco.corre_a_cauda(DT_MS, ate) {
            return telas;
        }
        telas.push(palco.tocador().tool().imagem_aplicada().expect("tela").0);
    }
    panic!("a cauda não parou");
}

/// ⭐ **Sem o artista tocar, a cauda é a água que o editor correu depois do Parar**: anda aos poucos (nenhum
/// quadro leva mais de um quarto do salto) e assenta na imagem do fim da gravação, ao byte. ⛔ VERMELHO
/// antes da W14: o palco fica na tela do Parar e a imagem assentada aparecia de uma vez ao sair do clipe
/// (sonda W14: 93 % da área pintada a 1024²).
#[test]
fn no_fim_do_clipe_a_agua_continua_e_assenta_na_imagem_do_fim() {
    let (_, bytes, dur) =
        grava_duas_horizontais_ligado_em(L, 1, PaintMedia::WetPaint).expect("grava");
    let track = ph2d_macro::codec::read(&bytes)
        .expect("lê")
        .tracks
        .remove(0);
    let fim = imagens_da_gravacao(&track, &AtomicBool::new(false))
        .expect("toca")
        .expect("não parou")
        .fim;
    let (mut palco, no_parar) = no_fim(&bytes, dur);
    let (salto, _) = diferem(&no_parar, &fim);
    assert!(salto > 0, "controlo: a água corre depois do Parar");
    let telas = cauda(&mut palco, None);
    let ultima = telas.last().unwrap_or(&no_parar);
    assert_eq!(
        diferem(ultima, &fim),
        (0, 0),
        "a cauda não assenta na imagem do fim ({} quadros; no Parar diferiam {salto})",
        telas.len()
    );
    let maior = std::iter::once(&no_parar)
        .chain(&telas)
        .zip(&telas)
        .map(|(a, b)| diferem(a, b).0)
        .max()
        .unwrap_or(0);
    assert!(
        telas.len() > 60 && maior * 4 < salto,
        "a cauda salta: {} quadros, o maior muda {maior} de {salto} texels",
        telas.len()
    );
    assert!(
        !palco.corre_a_cauda(DT_MS, None),
        "a água assentada não anda mais"
    );
}

/// ⭐ **A água da gravação que parou no 1.º toque do artista pára no mesmo passo na cauda** (W12: a
/// pincelada depois do Parar assa a sessão da gravação): a imagem guarda onde ela parou e, fora do que o
/// artista pintou, o palco no fim da cauda é o documento ao byte — no relógio fixo e no worker do editor
/// (o do produto, ao ritmo da parede). ⛔ VERMELHO antes da W14: a marca não guardava o passo (e a cauda
/// sem ele corre até assentar — o controlo).
#[test]
fn a_agua_que_parou_no_toque_do_artista_para_no_mesmo_passo_na_cauda() {
    let fixo = grava_duas_horizontais_ligado_em(L, 1, PaintMedia::WetPaint).expect("grava");
    toca_e_confere("relógio fixo", fixo, false);
    let worker = grava_no_papel_ligado_com_o_worker(L, 1, 2).expect("grava");
    toca_e_confere("worker", worker, true);
}

/// O artista pinta meio segundo depois do Parar; a cauda até onde a imagem guardou a água é o documento.
fn toca_e_confere(nome: &str, (mut t, bytes, dur): (PainterTool, Vec<u8>, f64), parede: bool) {
    let track = ph2d_macro::codec::read(&bytes)
        .expect("lê")
        .tracks
        .remove(0);
    t.poe_a_marca(Some(MarcaDoArtista::nova(L, L, epoca_da_gravacao(&track))));
    for _ in 0..30 {
        tique(&mut t);
        if parede {
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
    assert!(!t.agua_parada(), "controlo {nome}: a água da gravação ainda corre");
    // LITERAL-COLOR-OK: a tinta do artista na fixtura.
    t.set_brush_color_srgb8([200, 40, 40]);
    t.set_brush_size_px(6.0);
    for (i, phase) in [PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
        .into_iter()
        .enumerate()
    {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let x = 8.0 + 6.0 * i as f32;
        t.on_canvas_pointer(CanvasPointer {
            pos: [x, 88.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        tique(&mut t);
    }
    assenta(&mut t);
    let doc = t.imagem_aplicada().expect("documento").0;
    let m = t.marca_do_artista().cloned().expect("a marca");
    let ate = m.agua_da_gravacao_parou();
    assert!(
        ate.is_some(),
        "{nome}: a imagem não guardou onde a água da gravação parou"
    );
    let fora = |a: &[u8], b: &[u8]| {
        a.as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0)
            .enumerate()
            .filter(|(k, (x, y))| !m.marcado(*k) && x != y)
            .count()
    };
    let (mut sem, _) = no_fim(&bytes, dur);
    let assentada = cauda(&mut sem, None).pop().expect("a cauda andou");
    assert!(
        fora(&assentada, &doc) > 0,
        "controlo {nome}: a água assentada difere da que o toque parou"
    );
    let (mut palco, _) = no_fim(&bytes, dur);
    let parada = cauda(&mut palco, ate).pop().expect("a cauda andou");
    assert_eq!(
        fora(&parada, &doc),
        0,
        "{nome}: fora da tinta do artista, a cauda não é o documento"
    );
}
