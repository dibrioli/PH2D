//! Gates do [`super`] (`channel_group.rs`) — o grupo que não é de família (a posição do Inspector).

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

const E: u64 = 3;
const POSICAO: [PropKind; 2] = [PropKind::TranslationX, PropKind::TranslationY];

fn em(t: f64) -> Instante {
    Instante {
        t_cut: t,
        solo: true,
    }
}

/// Só o X tem chaves (0 s e 1 s): o losango da posição inteira responde a ele.
fn doc() -> TimelineDoc {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    for t in [0.0, 1.0] {
        d.insert_key(
            E,
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(5.0),
            Interp::Linear,
        );
    }
    d
}

fn na_curva(_: u64, _: PropKind) -> Option<f32> {
    Some(5.0)
}

#[test]
fn o_grupo_da_posicao_responde_a_qualquer_membro() {
    let d = doc();
    assert_eq!(estado_do_grupo(&d, &[E], &POSICAO, em(0.0), &na_curva).0, Forca::Chave);
    assert_eq!(estado_do_grupo(&d, &[E], &POSICAO, em(0.5), &na_curva).0, Forca::Animado);
    let fora = |_, p| Some(if p == PropKind::TranslationX { 9.0 } else { 5.0 });
    assert_eq!(estado_do_grupo(&d, &[E], &POSICAO, em(0.5), &fora).0, Forca::Mudado);
    assert_eq!(
        estado_do_grupo(&d, &[E], &[PropKind::Rotation], em(0.0), &na_curva).0,
        Forca::Livre
    );
    assert_eq!(estado_do_grupo(&d, &[], &POSICAO, em(0.0), &na_curva).0, Forca::Livre);
}

/// Fora de uma chave, o clique grava X E Y pela porta da K, sem pedir o que a família publicou.
/// (Mutação: gravar só o primeiro da lista ⇒ o Y fica sem chave.)
#[test]
fn clicar_fora_de_uma_chave_grava_cada_canal_da_lista() {
    let d = doc();
    let k = |e, p| {
        Some(TimelineIntent::AddKey {
            entity: e,
            prop: p,
            t: RationalTime::from_seconds(0.5),
            value: AnimValue::Float(1.0),
            interp: Interp::Linear,
        })
    };
    let lidos = |_| -> Vec<FieldKey> { panic!("um grupo sem família não pergunta à família") };
    let i = clique_no_grupo(&d, &POSICAO, &[E], em(0.5), lidos, na_curva, k);
    let gravados: Vec<PropKind> = i
        .iter()
        .filter_map(|x| match x {
            TimelineIntent::AddKey { prop, .. } => Some(*prop),
            _ => None,
        })
        .collect();
    assert_eq!(gravados, POSICAO.to_vec());
    let numa_chave = clique_no_grupo(&d, &POSICAO, &[E], em(0.0), |_| Vec::new(), na_curva, k);
    assert!(numa_chave.contains(&TimelineIntent::DeleteSelection));
    assert!(clique_no_grupo(&d, &[], &[E], em(0.5), |_| Vec::new(), na_curva, k).is_empty());
}
