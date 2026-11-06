use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{ChildOf, Transform};

const FILHO: u64 = 99;

fn agora() -> Instante {
    Instante {
        t_cut: 0.0,
        solo: true,
    }
}

fn na_curva(_: u64, _: PropKind) -> Option<f32> {
    Some(2.0)
}

fn grava(e: u64, p: PropKind) -> Option<TimelineIntent> {
    Some(TimelineIntent::AddKey {
        entity: e,
        prop: p,
        t: RationalTime::from_seconds(0.0),
        value: AnimValue::Float(1.0),
        interp: Interp::Linear,
    })
}

fn gravados(i: &[TimelineIntent]) -> Vec<(u64, PropKind)> {
    i.iter()
        .filter_map(|x| match x {
            TimelineIntent::AddKey { entity, prop, .. } => Some((*entity, *prop)),
            _ => None,
        })
        .collect()
}

/// Gravar: a posição do filho entra no passo já aberto, num só modo.
/// *Mutação: devolver `i` à entrada ⇒ o filho fica sem chave e deriva ao tocar a animação.*
#[test]
fn a_bolinha_grava_a_posicao_do_filho_no_mesmo_passo() {
    let abre = vec![TimelineIntent::BeginEdit, TimelineIntent::EndEdit];
    let i = clique_leva_os_filhos(
        abre,
        &TimelineDoc::new(),
        &[FILHO],
        agora(),
        na_curva,
        grava,
    );
    assert_eq!(gravados(&i), vec![(FILHO, PropKind::Position)]);
    assert_eq!(i.first(), Some(&TimelineIntent::BeginEdit), "{i:?}");
    assert_eq!(i.last(), Some(&TimelineIntent::EndEdit), "{i:?}");
}

/// Apagar: a chave do filho neste quadro sai junto, ou ele ficaria com uma chave sem o pivô que a pediu.
/// *Mutação: tirar o ramo do `DeleteSelection` ⇒ apagar a chave passa a gravar outra no filho.*
#[test]
fn apagar_a_chave_do_pivot_apaga_a_do_filho() {
    let mut doc = TimelineDoc::new();
    doc.fps_display = 24.0;
    doc.insert_key(
        FILHO,
        PropKind::Position,
        RationalTime::from_seconds(0.0),
        AnimValue::Vec2([1.0, 2.0].into()),
        Interp::Linear,
    );
    let apaga = vec![
        TimelineIntent::BeginEdit,
        TimelineIntent::ClearSelection,
        TimelineIntent::DeleteSelection,
        TimelineIntent::EndEdit,
    ];
    let i = clique_leva_os_filhos(apaga, &doc, &[FILHO], agora(), na_curva, grava);
    assert!(gravados(&i).is_empty(), "{i:?}");
    let selecionadas = i
        .iter()
        .take_while(|x| **x != TimelineIntent::DeleteSelection)
        .filter(|x| matches!(x, TimelineIntent::AddToSelection(_)))
        .count();
    assert_eq!(selecionadas, 1, "{i:?}");
}

/// Sem o clique da sprite, nada: o filho não ganha chave por conta própria.
#[test]
fn sem_passo_aberto_nao_ha_o_que_levar() {
    let i = clique_leva_os_filhos(
        Vec::new(),
        &TimelineDoc::new(),
        &[FILHO],
        agora(),
        na_curva,
        grava,
    );
    assert!(i.is_empty());
}

/// Só sprite leva filhos, e só os diretos.
#[test]
fn os_filhos_sao_os_diretos_de_quem_e_sprite() {
    let mut w = World::new();
    let imagem = Sprite::individual(1, [1.0, 1.0], [1.0; 4]);
    let sprite = w.spawn((Transform::default(), imagem)).id();
    let vazio = w.spawn(Transform::default()).id();
    let filho = w.spawn((Transform::default(), ChildOf(sprite))).id();
    let neto = w.spawn((Transform::default(), ChildOf(filho))).id();
    let do_vazio = w.spawn((Transform::default(), ChildOf(vazio))).id();
    let achados = filhos_do_pivo(&w, [sprite.to_bits(), vazio.to_bits()].into_iter());
    assert_eq!(achados, vec![filho.to_bits()]);
    assert!(!achados.contains(&neto.to_bits()) && !achados.contains(&do_vazio.to_bits()));
}
