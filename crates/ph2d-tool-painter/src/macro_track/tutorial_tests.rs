//! Os gates do tutorial «fazer» do Painter (W6b, `docs/Macros/03_plano_W6.md` §6): o Splash gravado
//! a 512², um aluno no estado inicial.

use super::sonda::{Variante, aluno, quadro};
use super::*;
use crate::macro_track::roteiro::{CAUDA, SPLASH, grava_nos_meios, grava_pinceladas};
use ph2d_editor_core::tool::{PanelEvent, Tool};

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
    let e = EnsinoDoPainter::de(&track).expect("ensino");
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
    let e = EnsinoDoPainter::de(&track).expect("ensino");
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
    let e = EnsinoDoPainter::de(&trilha()).expect("ensino");
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
    let e = EnsinoDoPainter::de(&track).expect("ensino");
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

/// Uma imagem de outro tamanho que a gravada não se avalia: o preparo di-lo (com o tamanho gravado), e
/// o «Recomeçar» recusa-a.
#[test]
fn outra_tela_recusa_se_e_diz_o_tamanho_gravado() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track).expect("ensino");
    let n = 256 * 256 * 4;
    let mut t = PainterTool::default();
    t.bind_document(ALVO, vec![255; n], 256, 256);
    assert_eq!(
        e.preparo(Some(&t), true, ALVO),
        Some(Preparo::OutroTamanho { w: LADO, h: LADO })
    );
    assert!(recomeca(&mut t, &track).is_err());
}

/// ⭐ **O meio é o da PINCELADA** (W6c): a 1.ª gravada em Wet Paint, a troca pelo seletor do painel
/// (um passo gravado), a 2.ª em Watercolor. O aluno que fez a 1.ª e trocou para Watercolor pinta a 2.ª;
/// em Wet Paint, o preparo pede o Watercolor.
#[test]
fn o_meio_pedido_e_o_da_pincelada_corrente() {
    let (b, _) = grava_nos_meios(
        LADO,
        &SPLASH[..2],
        &[PaintMedia::WetPaint, PaintMedia::Watercolor],
    )
    .expect("grava");
    let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
    let e = EnsinoDoPainter::de(&track).expect("ensino");
    let meios: Vec<PaintMedia> = e.criterios.iter().map(|c| c.regiao.meio).collect();
    assert_eq!(meios, [PaintMedia::WetPaint, PaintMedia::Watercolor]);
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    assert_eq!(e.preparo(Some(&t), true, ALVO), None, "a 1.ª em Wet Paint");
    mostra(&mut t, &track, e.criterios[0].regiao.k).expect("mostra");
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), Some(1));
    assert_eq!(
        e.preparo(Some(&t), true, ALVO),
        Some(Preparo::Meio(PaintMedia::Watercolor)),
        "a 2.ª pede o meio dela"
    );
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_MEDIA,
        PaintMedia::Watercolor.to_u8().to_string(),
    ));
    assert_eq!(e.preparo(Some(&t), true, ALVO), None);
    mostra(&mut t, &track, e.criterios[1].regiao.k).expect("mostra");
    assenta(&mut t);
    assert_eq!(e.por_cumprir(&t), None, "a 2.ª pintada em Watercolor");
}

/// ⭐ Uma pincelada que as anteriores cobrem inteira não tem parte nova: não tem critério (a lei da
/// W6a: etapa com Δ vazio não espera nem mostra balão). O Splash + uma 5.ª igual à 1.ª = 4 critérios.
#[test]
fn uma_pincelada_toda_coberta_nao_tem_criterio() {
    let ps = [SPLASH[0], SPLASH[1], SPLASH[2], SPLASH[3], SPLASH[0]];
    let (b, _) = grava_pinceladas(LADO, 0, &ps, &[0; 5]).expect("grava");
    let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
    assert_eq!(regioes(&track, LADO, LADO).expect("regiões").len(), 5);
    let e = EnsinoDoPainter::de(&track).expect("ensino");
    assert_eq!(e.criterios.len(), 4);
    assert!(e.criterios.iter().all(|c| !c.novos.is_empty()));
    let ultima = regioes(&track, LADO, LADO).expect("regiões")[4].k;
    assert!(e.criterios.iter().all(|c| c.regiao.k != ultima));
}

/// O «Recomeçar» com uma macro a gravar grava uma recusa (a reprodução pára aí em voz alta).
#[test]
fn recomecar_a_gravar_grava_uma_recusa() {
    let track = trilha();
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
    let l = LADO as f32;
    let mut rec = ph2d_macro::MacroRecorder::new(ph2d_macro::RefRect { w: l, h: l }, 0);
    t.macro_abre(&mut rec, 0).expect("arma");
    drop(t.macro_passos());
    recomeca(&mut t, &track).expect("recomeça");
    let passos: Vec<Passo> = t
        .macro_passos()
        .iter()
        .map(|c| Passo::descodifica(c).expect("passo"))
        .collect();
    assert!(
        passos.contains(&Passo::Recusada("tutorial_recomeca".into())),
        "{passos:?}"
    );
}

/// «Recomeçar» volta a tela ao início num passo de undo, e o desfazer volta ao feito.
#[test]
fn recomecar_volta_ao_inicio_num_passo_de_undo() {
    let track = trilha();
    let e = EnsinoDoPainter::de(&track).expect("ensino");
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
    let k = EnsinoDoPainter::de(&track).expect("ensino").criterios[2]
        .regiao
        .k;
    mostra(&mut t, &track, k).expect("mostra");
    assert_eq!((t.brush_color_srgb8(), t.brush_size_px()), ([1, 2, 3], 9.0));
    assert!(
        mostra(&mut t, &track, 999).is_err(),
        "uma etapa que não é pincelada recusa-se"
    );
}
