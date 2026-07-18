//! Testes do [`crate::flip_erase`] — extraídos para o irmão pelo teto de LOC do
//! HR-18 (o arquivo passou de 600 ao ganhar os gates do §4.C.5, o fix da borracha
//! macia). Mesma convenção do `flip_select_segment_tests.rs`.

use super::*;
use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, Hold, KeyKind, Point, Rgba};

fn doc_with_line() -> (FlipDoc, LayerId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("O");
    let obj = doc.object_mut(oid).unwrap();
    let l = obj.add_layer("L");
    let d = obj
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    let mut s = FlipStroke::new();
    for x in 0..5 {
        s.push_point(Point {
            pos: Vec2::new(x as f32, 0.0),
            width: 0.1,
            opacity: 1.0,
            color: Rgba::WHITE,
        });
    }
    obj.drawing_mut(d).unwrap().strokes.push(s);
    (doc, l)
}

#[test]
fn stroke_mode_removes_the_whole_touched_stroke() {
    let (mut doc, l) = doc_with_line();
    let hit = erase_at(
        &mut doc,
        &Playhead::default(),
        Some(l),
        &mut crate::flip_strip::FlipStrip::default(),
        EraseMode::Stroke,
        Vec2::new(2.0, 0.0),
        0.5,
        1.0,
    );
    assert!(hit);
    let obj = doc.objects().first().unwrap();
    let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
    assert_eq!(obj.drawing(did).unwrap().strokes.len(), 0);
}

#[test]
fn hard_mode_splits_the_stroke_at_the_gap() {
    let (mut doc, l) = doc_with_line();
    // Erase the middle point (x=2) → two runs [0,1] and [3,4].
    let hit = erase_at(
        &mut doc,
        &Playhead::default(),
        Some(l),
        &mut crate::flip_strip::FlipStrip::default(),
        EraseMode::Hard,
        Vec2::new(2.0, 0.0),
        0.5,
        1.0,
    );
    assert!(hit);
    let obj = doc.objects().first().unwrap();
    let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
    assert_eq!(obj.drawing(did).unwrap().strokes.len(), 2, "split into two");
}

#[test]
fn soft_mode_reduces_opacity_then_cleanup_removes_faded() {
    let (mut doc, l) = doc_with_line();
    // **Uma VARREDURA ao longo da linha**, força cheia: cada ponto passa pelo CENTRO
    // da borracha (falloff = 1) em algum dab e vai a zero.
    //
    // O 1º corte deste teste ficava parado em (2,0) e contava com a acumulação por-dab
    // pra zerar também as PONTAS — mas parado, as pontas só veem `falloff ≈ 0,8`, e o
    // certo é que elas sobrem parcialmente (é a borda macia da borracha). A premissa
    // antiga era o bug do §4.C.5 escrito como teste; varrer é o que o artista faz.
    for x in 0..5 {
        erase_at(
            &mut doc,
            &Playhead::default(),
            Some(l),
            &mut crate::flip_strip::FlipStrip::default(),
            EraseMode::Soft,
            Vec2::new(x as f32, 0.0),
            10.0,
            1.0,
        );
    }
    let obj = doc.objects().first().unwrap();
    let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
    let min_op = obj.drawing(did).unwrap().strokes[0]
        .opacities()
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);
    assert!(
        min_op < OPACITY_REMOVE_THRESHOLD,
        "center faded below threshold"
    );
    assert!(cleanup_soft(
        &mut doc,
        &Playhead::default(),
        Some(l),
        &mut crate::flip_strip::FlipStrip::default(),
    ));
}

/// A opacidade do ponto `i` do 1º traço.
fn opacity_at(doc: &FlipDoc, l: LayerId, i: usize) -> f32 {
    let obj = doc.objects().first().unwrap();
    let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
    obj.drawing(did).unwrap().strokes[0].opacities()[i]
}

/// Apaga `dabs` vezes no MESMO lugar, com a força dada, e devolve o doc.
fn soft_erase_n(dabs: usize, strength: f32) -> (FlipDoc, LayerId) {
    let (mut doc, l) = doc_with_line();
    for _ in 0..dabs {
        erase_at(
            &mut doc,
            &Playhead::default(),
            Some(l),
            &mut crate::flip_strip::FlipStrip::default(),
            EraseMode::Soft,
            Vec2::new(2.0, 0.0), // exatamente sobre o ponto de índice 2
            10.0,
            strength,
        );
    }
    (doc, l)
}

/// 🔴 **O apagado é fato do CAMINHO, não de quantos dabs o motor carimbou.**
///
/// Enio 2026-07-17: *"qualquer nível de strength apaga completamente a linha, nunca
/// deixa semitransparente"*. A borracha carimba **um dab por evento de ponteiro**, e a
/// subtração acumulada do 1º corte fazia 12 quadros parados sobre o mesmo ponto com
/// Strength 0,1 zerarem a linha: o resultado era função da taxa de amostragem do mouse.
///
/// O gate passa o MESMO ponto pela borracha 1 vez e 40 vezes e exige o MESMO número.
/// É a forma canônica desta classe de bug no projeto (o irmão no Painter é o
/// `the_trench_is_a_fact_of_the_path_not_of_the_dab_spacing`).
///
/// Mutação que sangra: voltar a `(ops[i] - strength*falloff).max(0.0)` — com 40 dabs a
/// linha vai a zero e a igualdade cai.
#[test]
fn the_soft_erase_is_a_fact_of_the_path_not_of_the_dab_count() {
    let (one, l1) = soft_erase_n(1, 0.5);
    let (many, l2) = soft_erase_n(40, 0.5);
    for i in 0..5 {
        let (a, b) = (opacity_at(&one, l1, i), opacity_at(&many, l2, i));
        assert!(
            (a - b).abs() < 1e-6,
            "ponto {i}: 1 dab deu {a}, 40 dabs deram {b} — o apagado depende da \
             amostragem do ponteiro, nao do caminho"
        );
    }
    // E o centro sobrou SEMITRANSPARENTE, que é a queixa original.
    let center = opacity_at(&many, l2, 2);
    assert!(
        center > OPACITY_REMOVE_THRESHOLD,
        "o centro foi a zero ({center}) mesmo com Strength 0,5: a linha nunca fica \
         semitransparente"
    );
}

/// 🔴 **Strength É a translucidez que sobra**: 0,25 deixa 75 %, 0,5 deixa 50 %, e só
/// 1,0 apaga de vez. Sem isto o knob teria número mas não significado.
#[test]
fn strength_is_the_translucency_that_remains() {
    for strength in [0.25_f32, 0.5, 0.75, 1.0] {
        let (doc, l) = soft_erase_n(12, strength);
        let center = opacity_at(&doc, l, 2); // falloff = 1 no centro
        let want = 1.0 - strength;
        assert!(
            (center - want).abs() < 1e-6,
            "Strength {strength}: o centro ficou em {center}, esperado {want}"
        );
    }
}

/// A borracha **nunca pinta de volta**: um ponto já mais claro que o piso fica onde
/// está. (Sem o `.min(current)` do `soft_erased`, uma passada fraca por cima de uma
/// forte AUMENTARIA a opacidade — a borracha desapagaria.)
#[test]
fn a_weak_pass_never_restores_what_a_strong_one_erased() {
    let (mut doc, l) = soft_erase_n(3, 0.9); // centro → 0.1
    let before = opacity_at(&doc, l, 2);
    erase_at(
        &mut doc,
        &Playhead::default(),
        Some(l),
        &mut crate::flip_strip::FlipStrip::default(),
        EraseMode::Soft,
        Vec2::new(2.0, 0.0),
        10.0,
        0.2, // piso 0.8, MUITO acima do que já está lá
    );
    assert_eq!(
        opacity_at(&doc, l, 2),
        before,
        "a borracha fraca SUBIU a opacidade — ela desapagou"
    );
}

#[test]
fn locked_layer_refuses_erase() {
    let (mut doc, l) = doc_with_line();
    doc.objects().first().unwrap(); // sanity
    let oid = doc.objects().first().unwrap().id;
    doc.object_mut(oid).unwrap().layer_mut(l).unwrap().locked = true;
    let hit = erase_at(
        &mut doc,
        &Playhead::default(),
        Some(l),
        &mut crate::flip_strip::FlipStrip::default(),
        EraseMode::Stroke,
        Vec2::new(2.0, 0.0),
        0.5,
        1.0,
    );
    assert!(!hit, "locked layer preserved");
}

/// Um desenho com line-art + um PREENCHIMENTO (com furo) + um FECHAMENTO de gap.
fn doc_with_fill_and_closure() -> (FlipDoc, LayerId) {
    let (mut doc, l) = doc_with_line();
    let oid = doc.objects().first().unwrap().id;
    let obj = doc.object_mut(oid).unwrap();
    let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
    let dr = obj.drawing_mut(did).unwrap();

    // O preenchimento: contorno invisível, com um furo.
    let mut fill = FlipStroke::new();
    for p in [
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(10.0, 10.0),
        Vec2::new(0.0, 10.0),
    ] {
        fill.push_point(Point {
            pos: p,
            width: 0.0,
            opacity: 1.0,
            color: Rgba::WHITE,
        });
    }
    fill.closed = true;
    fill.hide_stroke = true;
    fill.holes = vec![vec![
        Vec2::new(3.0, 3.0),
        Vec2::new(7.0, 3.0),
        Vec2::new(7.0, 7.0),
        Vec2::new(3.0, 7.0),
    ]];
    fill.fill = Some(ph2d_flip::Fill {
        color: Rgba::WHITE,
        opacity: 1.0,
    });
    dr.strokes.push(fill);

    // O fechamento de gap: invisível, sem cor, opacidade ZERO (é assim que nasce).
    let mut clo = FlipStroke::new();
    for p in [Vec2::new(20.0, 20.0), Vec2::new(22.0, 20.0)] {
        clo.push_point(Point {
            pos: p,
            width: 0.0,
            opacity: 0.0,
            color: Rgba::TRANSPARENT,
        });
    }
    clo.hide_stroke = true;
    dr.strokes.push(clo);
    (doc, l)
}

fn strokes_of(doc: &FlipDoc) -> Vec<FlipStroke> {
    let obj = doc.objects().first().unwrap();
    let l = obj.layers().first().unwrap().id;
    let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
    obj.drawing(did).unwrap().strokes.clone()
}

/// **A borracha macia NÃO pode apagar os fechamentos de gap.**
///
/// Um fechamento nasce com opacidade 0 (é invisível de propósito), e o `cleanup_soft`
/// coletava lixo por opacidade de ponto: `retain(any opacity >= 0.05)` matava TODOS
/// eles — a cada pen-up, em qualquer lugar do canvas. O vão reabria e o balde seguinte
/// vazava. O "fechamento persistente" (o twist do Harmony) era desfeito em silêncio.
#[test]
fn the_soft_eraser_does_not_delete_gap_closures_anywhere_on_the_canvas() {
    let (mut doc, l) = doc_with_fill_and_closure();
    let ph = Playhead::new(1.0 / 12.0);
    let mut strip = crate::flip_strip::FlipStrip::default();
    let before = strokes_of(&doc).len();

    // Um toque de borracha macia LONGE de tudo (em (100, 100)).
    erase_at(
        &mut doc,
        &ph,
        Some(l),
        &mut strip,
        EraseMode::Soft,
        Vec2::new(100.0, 100.0),
        1.0,
        1.0,
    );
    cleanup_soft(&mut doc, &ph, Some(l), &mut strip);

    let after = strokes_of(&doc);
    assert_eq!(
        after.len(),
        before,
        "a borracha apagou tracos do outro lado do canvas (o fechamento de gap sumiu)"
    );
    assert!(
        after.iter().any(|s| s.hide_stroke && s.fill.is_none()),
        "o fechamento de gap invisivel foi coletado como lixo"
    );
}

/// **A borracha de PONTO não morde uma região.** Um preenchimento não tem tinta
/// visível (o contorno não é rasterizado): mordê-lo produzia fragmentos com o furo
/// PERDIDO (o "O" ficava sólido) e sem `hide_stroke` — o fragmento virava fronteira
/// do próximo balde e o Unpaint não o reconhecia mais.
#[test]
fn a_point_eraser_does_not_shred_a_filled_region() {
    for mode in [EraseMode::Soft, EraseMode::Hard] {
        let (mut doc, l) = doc_with_fill_and_closure();
        let ph = Playhead::new(1.0 / 12.0);
        let mut strip = crate::flip_strip::FlipStrip::default();

        // Apaga bem no MEIO da região preenchida.
        erase_at(
            &mut doc,
            &ph,
            Some(l),
            &mut strip,
            mode,
            Vec2::new(5.0, 5.0),
            4.0,
            1.0,
        );
        cleanup_soft(&mut doc, &ph, Some(l), &mut strip);

        let fills: Vec<FlipStroke> = strokes_of(&doc)
            .into_iter()
            .filter(|s| s.hide_stroke && s.fill.is_some())
            .collect();
        assert_eq!(
            fills.len(),
            1,
            "{mode:?}: a regiao foi picada em {} pedacos",
            fills.len()
        );
        assert_eq!(
            fills[0].holes.len(),
            1,
            "{mode:?}: o furo do \"O\" foi perdido (a regiao virou solida)"
        );
        assert!(
            fills[0].hide_stroke,
            "{mode:?}: o fill perdeu o hide_stroke"
        );
    }
}
