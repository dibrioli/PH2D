//! Os gates do tutorial «fazer» do Painter (W6b, `docs/Macros/03_plano_W6.md` §6): o Splash gravado
//! a 512², um aluno no estado inicial.

use super::sonda::{Variante, aluno, quadro};
use super::*;
use crate::macro_track::roteiro::{CAUDA, SPLASH, grava_pinceladas};
use ph2d_editor_core::tool::RasterEditTool;

const LADO: u32 = 512;
/// A entidade a que o tool no estado inicial liga o documento.
const ALVO: u64 = inicial::ENTIDADE_DO_TOCADOR;

fn trilha() -> Track {
    let (b, _) = grava_pinceladas(LADO, 0, &SPLASH, &[0; 4]).expect("grava");
    ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0)
}

fn assenta(t: &mut PainterTool) {
    for _ in 0..CAUDA {
        quadro(t);
    }
}

/// Cada pincelada cumpre-se só quando feita («Mostra-me» = a gravada), e o Ctrl+Z volta o passo.
#[test]
fn cada_pincelada_so_se_cumpre_feita_e_o_desfazer_a_volta() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track, Some((LADO, LADO))).expect("ensino");
    assert_eq!(e.criterios.len(), 4);
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    assert_eq!(e.por_cumprir(&t), Some(0), "nada feito");
    for (i, c) in e.criterios.iter().enumerate() {
        mostra(&mut t, &track, c.regiao.k).expect("mostra");
        assenta(&mut t);
        assert_eq!(
            e.por_cumprir(&t),
            (i + 1 < 4).then_some(i + 1),
            "depois da {i}"
        );
    }
    assert!(t.undo_last(), "a última pincelada desfaz-se");
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), Some(3), "o desfazer volta o passo");
    assert!(t.redo_last());
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), None, "o refazer avança");
}

/// A fracção medida separa a mão torta (cumpre) do pincel fino e do nada (não cumprem).
#[test]
fn a_mao_torta_cumpre_o_pincel_fino_e_o_nada_nao() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track, Some((LADO, LADO))).expect("ensino");
    let c = &e.criterios[0];
    let f = |v| e.fracao_pintada(&aluno(&track, 0, v, LADO), c);
    let (torta, fina, nada) = (
        f(Variante::Desloca(0.5)),
        f(Variante::Fina),
        f(Variante::Nada),
    );
    assert!(torta >= FRACAO_QUE_CUMPRE, "torta meio raio: {torta}");
    assert!(fina < FRACAO_QUE_CUMPRE, "fina: {fina}");
    assert!(nada < FRACAO_QUE_CUMPRE, "nada: {nada}");
}

/// A parte nova de uma pincelada não tem píxeis de uma anterior.
#[test]
fn a_parte_nova_nao_conta_o_que_a_anterior_cobre() {
    let e = EnsinoDoPainter::de(&trilha(), Some((LADO, LADO))).expect("ensino");
    let antes: std::collections::BTreeSet<u32> = rasteriza(&e.criterios[0].regiao, LADO, LADO)
        .into_iter()
        .collect();
    let novos = &e.criterios[1].novos;
    assert!(!novos.is_empty());
    assert!(novos.iter().all(|i| !antes.contains(i)));
    let toda = rasteriza(&e.criterios[1].regiao, LADO, LADO);
    assert!(novos.len() < toda.len(), "as duas cruzam-se no Splash");
}

/// Antes das pinceladas: o Painter na imagem do tutorial, depois o meio gravado.
#[test]
fn o_preparo_pede_a_imagem_e_depois_o_meio() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track, Some((LADO, LADO))).expect("ensino");
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    assert_eq!(e.preparo(None, true, ALVO), Some(Preparo::AbreParaPintar));
    assert_eq!(
        e.preparo(Some(&t), false, ALVO),
        Some(Preparo::AbreParaPintar),
        "outra ferramenta na mão"
    );
    assert_eq!(
        e.preparo(Some(&t), true, ALVO + 1),
        Some(Preparo::AbreParaPintar),
        "outra imagem"
    );
    assert_eq!(e.preparo(Some(&t), true, ALVO), None);
    t.set_paint_media(PaintMedia::Digital);
    assert_eq!(
        e.preparo(Some(&t), true, ALVO),
        Some(Preparo::Meio(PaintMedia::WetPaint))
    );
}

/// «Recomeçar» volta a tela ao início num passo de undo, e o desfazer volta ao feito.
#[test]
fn recomecar_volta_ao_inicio_num_passo_de_undo() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track, Some((LADO, LADO))).expect("ensino");
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    mostra(&mut t, &track, e.criterios[0].regiao.k).expect("mostra");
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), Some(1));
    recomeca(&mut t, &track).expect("recomeça");
    assert_eq!(e.por_cumprir(&t), Some(0), "de volta ao início");
    assert!(t.undo_last(), "o recomeçar é um passo");
    assert_eq!(e.por_cumprir(&t), Some(1), "o desfazer volta ao feito");
}

/// «Mostra-me» pinta com o pincel da gravação e devolve o do aluno.
#[test]
fn mostra_me_devolve_a_cor_e_o_tamanho_do_aluno() {
    let track = trilha();
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    t.set_brush_color_srgb8([1, 2, 3]);
    t.set_brush_size_px(9.0);
    let k = EnsinoDoPainter::de(&track, Some((LADO, LADO)))
        .expect("ensino")
        .criterios[2]
        .regiao
        .k;
    mostra(&mut t, &track, k).expect("mostra");
    assert_eq!((t.brush_color_srgb8(), t.brush_size_px()), ([1, 2, 3], 9.0));
    assert!(
        mostra(&mut t, &track, 999).is_err(),
        "uma etapa que não é pincelada recusa-se"
    );
}
