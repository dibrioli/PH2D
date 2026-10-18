//! W9 — **arrastar a cabeça da Timeline** (o smoke do dono de 18/10: buracos retangulares nos traços,
//! `docs/Macros/handoffs/HANDOFF_CONTINUACAO_W9_ARRASTAR_A_TIMELINE_2026-10-18.md`). Cada salto — para
//! trás, para a frente, a meio de uma pincelada, ou o arrasto contínuo de uma mão — deixa a ranhura
//! igual, ao byte, à tela INTEIRA da macro nesse instante, em cada um dos quatro meios.

use super::palcos_tests::Placa;
use super::{Atuais, PalcosDoPainter};
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, World};
use ph2d_editor_core::toast::ToastQueue;
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;

/// Texels (RGBA) em que `a` e `b` diferem.
fn diferentes(a: &[u8], b: &[u8]) -> usize {
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .filter(|(x, y)| x != y)
        .count()
}

/// A tela de uma reprodução de `doc` lida UMA vez em `t` (do zero), premultiplicada.
fn fresca(doc: &[u8], t: f64) -> Vec<u8> {
    let mut m = MacroEmCena::de(doc).expect("lê-se").expect("trilha");
    m.segue(t).expect("toca");
    let mut tela = m.toma_a_tela().expect("a 1.ª leitura mostra").0.to_vec();
    ph2d_render::premul::premultiply_rgba8(&mut tela);
    tela
}

/// Os instantes: tocar até ao fim quadro a quadro (os instantâneos de cada etapa), os saltos do dono, e
/// um arrasto contínuo para trás e outra vez para a frente.
fn instantes(d: f64) -> Vec<f64> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "segundos"
    )]
    let mut ts: Vec<f64> = (0..=(d * 60.0).ceil() as u32)
        .map(|q| f64::from(q) / 60.0)
        .collect();
    ts.extend([0.88, 0.41, 0.82, 0.26, 1.0, 0.59, 0.07, 0.95].map(|f| f * d));
    ts.extend((0..=30).map(|k| d * (1.0 - f64::from(k) / 30.0)));
    ts.extend((0..=30).map(|k| d * f64::from(k) / 30.0));
    ts
}

/// `(quadro, t, texels diferentes da tela inteira, da reprodução fresca)` dos quadros errados, e as
/// subidas parciais.
fn arrasta(meio: PaintMedia) -> (Vec<(usize, f64, usize, usize)>, usize) {
    let (doc, d) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_nos_meios(64, 0, &[meio; 4])
            .expect("arma");
    let mut world = World::new();
    let e = world
        .spawn((
            Macro {
                doc: doc.clone(),
                modo: ModoDaMacro::Fiel,
            },
            MacroTempo { segundos: 0.0 },
        ))
        .id();
    let mut palcos = PalcosDoPainter::default();
    let mut placa = Placa::default();
    // A mesma macro, os mesmos instantes, a tela lida INTEIRA (sem caixas).
    let mut verdade = MacroEmCena::de(&doc).expect("lê-se").expect("trilha");
    let mut tela = Vec::new();
    let mut errados = Vec::new();
    let mut antes = 0.0;
    for (q, t) in instantes(d).into_iter().enumerate() {
        world.get_mut::<MacroTempo>(e).expect("tempo").segundos = t;
        palcos.segue(&mut world);
        let trocas = palcos.sobe(
            &mut placa,
            &mut ToastQueue::default(),
            None,
            Atuais::default(),
        );
        verdade.segue(t).expect("toca");
        if let Some((a, _, _)) = verdade.toma_a_tela() {
            tela = a.to_vec();
            ph2d_render::premul::premultiply_rgba8(&mut tela);
        }
        let ranhura = &placa.texturas[&trocas[0].texture_id].1;
        let inteira = diferentes(ranhura, &tela);
        // Depois de um salto para trás a tela lê-se uma vez, como a reprodução fresca.
        let fresca = if t < antes {
            diferentes(ranhura, &fresca(&doc, t))
        } else {
            0
        };
        if inteira + fresca > 0 {
            errados.push((q, (t * 1000.0).round() / 1000.0, inteira, fresca));
        }
        antes = t;
    }
    let p = palcos.lidos[&e.to_bits()]
        .palco
        .as_ref()
        .ok()
        .and_then(Option::as_ref)
        .expect("palco");
    assert!(
        p.em_cena.instantaneos().0.len() > 1,
        "{meio:?} — controlo: sem instantâneos o arrasto não passa pela cópia"
    );
    (errados, placa.regioes)
}

#[test]
fn arrastar_a_timeline_deixa_a_ranhura_igual_a_tela_inteira_em_cada_meio() {
    let mut falhas = Vec::new();
    let mut parciais = 0;
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let (errados, regioes) = arrasta(meio);
        parciais += regioes;
        if !errados.is_empty() {
            falhas.push((meio, errados));
        }
    }
    assert!(
        falhas.is_empty(),
        "(quadro, t, texels ≠ da tela inteira, ≠ da reprodução fresca): {falhas:?}"
    );
    assert!(
        parciais > 0,
        "controlo: sem subidas parciais o espelho nunca fica de outro instante"
    );
}
