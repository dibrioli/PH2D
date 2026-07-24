//! **A conversão preserva o que promete e RELATA o que não preserva** (ADR-0141 §5).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Transform, World};

use crate::{PropKind, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// Um objeto animado no modo SEPARADO, com os dois eixos em tempos **diferentes** — o
/// overlap que é a razão de esse modo existir.
fn separate() -> (World, ph2d_ecs::Entity, TimelineDoc) {
    let mut w = World::new();
    let e = w.spawn(Transform::default()).id();
    let mut doc = TimelineDoc::new();
    for (t, v, i) in [(0.0, 0.0_f32, Interp::Linear), (2.0, 20.0, Interp::Linear)] {
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationX,
            s(t),
            AnimValue::Float(v),
            i,
        );
    }
    // O Y chega ATRASADO e com ease — os dois fatos que a trajetória não expressa.
    for (t, v, i) in [
        (
            0.5,
            0.0_f32,
            Interp::Bezier {
                x1: 0.6,
                y1: 0.0,
                x2: 0.4,
                y2: 1.0,
            },
        ),
        (2.0, 10.0, Interp::Linear),
    ] {
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationY,
            s(t),
            AnimValue::Float(v),
            i,
        );
    }
    (w, e, doc)
}

fn pos(w: &World, e: ph2d_ecs::Entity) -> [f32; 2] {
    let xf = w.get::<Transform>(e).unwrap();
    [xf.translation.x, xf.translation.y]
}

/// **Separate → Path: a pose nas keys é a MESMA**, e é isso que faz a conversão ser uma
/// mudança de representação em vez de uma edição.
///
/// O oráculo é onde o objeto ESTÁ nos instantes convertidos, medido pelo apply real
/// antes e depois — não os números do documento, que mudam de unidade por construção.
#[test]
fn converting_to_a_path_keeps_the_pose_at_every_key() {
    let (mut w, e, mut doc) = separate();
    let times = [0.0, 0.5, 2.0];
    let before: Vec<[f32; 2]> = times
        .iter()
        .map(|&t| {
            apply_from_doc(&mut w, &mut doc, t);
            pos(&w, e)
        })
        .collect();

    let r = doc.separate_to_path(e.to_bits()).expect("converteu");
    assert_eq!(r.keys, 3, "a UNIÃO dos instantes: 0, 0.5 e 2");

    for (&t, want) in times.iter().zip(before) {
        apply_from_doc(&mut w, &mut doc, t);
        let got = pos(&w, e);
        let d = ((got[0] - want[0]).powi(2) + (got[1] - want[1]).powi(2)).sqrt();
        assert!(
            d < 1e-3,
            "em t={t} a pose andou {d:.4} na conversão: {want:?} -> {got:?}"
        );
    }
    // E os dois modos não coexistem: as tracks de origem foram embora.
    assert!(
        doc.binding_for(e.to_bits(), PropKind::TranslationX)
            .is_none()
    );
    assert!(
        doc.binding_for(e.to_bits(), PropKind::TranslationY)
            .is_none()
    );
    assert!(doc.position_path(e.to_bits()).is_some());
}

/// **O relatório diz a verdade.** Uma conversão que descarta trabalho em silêncio é a
/// diferença entre uma ferramenta e uma armadilha.
#[test]
fn the_report_counts_what_the_conversion_could_not_carry() {
    let (_, e, mut doc) = separate();
    let r = doc.separate_to_path(e.to_bits()).unwrap();
    println!(
        "MEDIDO  keys {} | instantes desemparelhados {} | eases perdidos {}",
        r.keys, r.unmatched_times, r.dropped_eases
    );
    // X tem keys em {0, 2}, Y em {0.5, 2}: a união é {0, 0.5, 2}, e dois desses
    // instantes existiam num eixo só.
    assert_eq!(r.unmatched_times, 2, "0 só no X e 0.5 só no Y");
    assert_eq!(r.dropped_eases, 1, "o ease do Y em t=0.5");

    // E o CONTROLE: dois eixos alinhados e sem ease não perdem nada, senão os números
    // acima seriam uma propriedade da fixture e não da conversão.
    let mut aligned = TimelineDoc::new();
    for prop in [PropKind::TranslationX, PropKind::TranslationY] {
        for (t, v) in [(0.0, 0.0_f32), (1.0, 5.0)] {
            aligned.insert_key(7, prop, s(t), AnimValue::Float(v), Interp::Linear);
        }
    }
    let r2 = aligned.separate_to_path(7).unwrap();
    assert_eq!((r2.keys, r2.unmatched_times, r2.dropped_eases), (2, 0, 0));
}

/// **Path → Separate: exata nas keys**, e o relatório é honesto sobre o meio do
/// caminho. Ida e volta devolve a pose de cada key ao ponto de partida.
#[test]
fn the_round_trip_returns_the_pose_at_every_key() {
    let (mut w, e, mut doc) = separate();
    let times = [0.0, 0.5, 2.0];
    let before: Vec<[f32; 2]> = times
        .iter()
        .map(|&t| {
            apply_from_doc(&mut w, &mut doc, t);
            pos(&w, e)
        })
        .collect();

    doc.separate_to_path(e.to_bits()).unwrap();
    let back = doc.path_to_separate(e.to_bits()).expect("voltou");
    assert_eq!(back.keys, 3);
    assert_eq!(
        back.unmatched_times, 0,
        "a trajetória não sabe desemparelhar"
    );

    assert!(
        doc.position_path(e.to_bits()).is_none(),
        "o caminho foi embora"
    );
    assert!(
        doc.binding_for(e.to_bits(), PropKind::TranslationX)
            .is_some()
    );

    for (&t, want) in times.iter().zip(before) {
        apply_from_doc(&mut w, &mut doc, t);
        let got = pos(&w, e);
        let d = ((got[0] - want[0]).powi(2) + (got[1] - want[1]).powi(2)).sqrt();
        assert!(d < 1e-3, "em t={t} a ida e volta moveu a pose em {d:.4}");
    }
}

/// ⚠️ **E o que a ida e volta NÃO devolve: a FORMA entre as keys.** É o custo que o
/// modo Path existe para cobrar do modo separado, e enunciá-lo aqui impede alguém de
/// "consertar" a conversão para ser exata onde ela não pode ser.
#[test]
fn the_round_trip_does_not_promise_the_shape_between_the_keys() {
    let (mut w, e, mut doc) = separate();
    doc.separate_to_path(e.to_bits()).unwrap();
    apply_from_doc(&mut w, &mut doc, 1.2);
    let on_path = pos(&w, e);

    doc.path_to_separate(e.to_bits()).unwrap();
    apply_from_doc(&mut w, &mut doc, 1.2);
    let on_axes = pos(&w, e);

    let d = ((on_path[0] - on_axes[0]).powi(2) + (on_path[1] - on_axes[1]).powi(2)).sqrt();
    println!("MEDIDO  entre keys, a ida e volta desvia {d:.4}");
    assert!(
        d > 0.05,
        "o meio do caminho saiu IGUAL ({d:.4}) — ou a trajetória era uma reta (a \
         fixture não contém o fenômeno) ou alguém fez a conversão prometer o que ela \
         não pode prometer"
    );
}

/// Sem nada para converter, a porta **recusa** em vez de criar um binding vazio.
#[test]
fn converting_nothing_is_refused() {
    let mut doc = TimelineDoc::new();
    assert_eq!(doc.separate_to_path(1), None);
    assert_eq!(doc.path_to_separate(1), None);
    // Um binding Position sem keys também não tem o que virar dois eixos.
    doc.bind(1, PropKind::Position);
    assert_eq!(doc.path_to_separate(1), None);
}
