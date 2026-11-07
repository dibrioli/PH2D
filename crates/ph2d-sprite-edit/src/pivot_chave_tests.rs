use super::*;
use ph2d_anim::{AnimValue, RationalTime};
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

/// O vai e vem do dono: numa trajetória, o pivô e a colisão de quem grava o pivô andam em ritmo constante.
/// *Mutação: devolver sempre `padrao` ⇒ o desenho desloca 0,28 m em t = 1,25 (cena `sprite_pivot_chave`).*
#[test]
fn na_trajetoria_o_pivo_e_a_colisao_andam_no_ritmo_da_posicao() {
    let doc = TimelineDoc::new();
    let ancora =
        PropKind::Field(ph2d_ecs::canal_componente::chave("ph2d::render::Sprite", 4, 0).unwrap());
    let colisao = PropKind::Field(ph2d_ecs::FieldKey::new(
        ph2d_ecs::canal::familia::FISICA,
        ph2d_ecs::canal_fisica::DESLOC_X,
        0,
        0,
    ));
    let suave = Interp::Hold;
    let quem = com_pivo([(FILHO, ancora)].into_iter());
    assert_eq!(
        ritmo_da_chave(&doc, FILHO, ancora, &quem, suave),
        Interp::Linear
    );
    assert_eq!(
        ritmo_da_chave(&doc, FILHO, colisao, &quem, suave),
        Interp::Linear
    );
    assert_eq!(
        ritmo_da_chave(&doc, FILHO, colisao, &[], suave),
        suave,
        "quem só anima a colisão fica com o ritmo de sempre"
    );
}

fn ancora(i: u16) -> PropKind {
    PropKind::Field(ph2d_ecs::canal_componente::chave("ph2d::render::Sprite", 4, i).unwrap())
}

/// ⭐ «Pivot animado faz a imagem tremer» (dono 06/11): com três chaves o desenho fica parado, e a curva lê-se por
/// `apply_from_doc`, a mesma porta do transporte.
/// *Mutação: `path_key_corner` não faz nada ⇒ a trajetória curva e o desenho desvia até 0,2 m.*
#[test]
fn com_tres_chaves_de_pivo_o_desenho_nao_anda() {
    let mut w = World::new();
    let imagem = Sprite::individual(1, [3.0, 0.5], [1.0; 4]);
    let e = w.spawn((Transform::default(), imagem)).id();
    let bits = e.to_bits();
    let mut doc = TimelineDoc::new();
    doc.fps_display = 60.0;
    let chaves = [
        (1.0, [8.5, 1.75], [0.0, 0.25]),
        (2.0, [7.0, 2.0], [1.5, 0.0]),
        (3.0, [10.0, 2.25], [-1.5, -0.25]),
    ];
    for (t, pos, a) in chaves {
        let t = RationalTime::from_seconds(t);
        assert!(doc.key_the_path(bits, t, pos));
        assert!(doc.path_key_corner(bits, t));
        for i in 0..2_u16 {
            let v = AnimValue::Float(a[usize::from(i)]);
            doc.upsert_key(bits, ancora(i), t, v, Interp::Linear);
        }
    }
    for k in 60..=180 {
        ph2d_timeline::apply_from_doc(&mut w, &mut doc, f64::from(k) / 60.0);
        let p = w.get::<Transform>(e).unwrap().translation;
        let canal = |w: &World, i: u16| {
            let PropKind::Field(c) = ancora(i) else {
                unreachable!()
            };
            w.resource::<ph2d_ecs::CanaisEscritos>()
                .valor(bits, c)
                .unwrap()
        };
        let desvio = (p.x + canal(&w, 0) - 8.5).hypot(p.y + canal(&w, 1) - 2.0);
        assert!(desvio < 0.01, "quadro {k}: {desvio} m");
    }
}

/// A bolinha leva a mesma reta, para o pai e o filho, no passo aberto.
/// *Mutação: tirar o ramo ⇒ este teste fica sem `PathKeyCorner`.*
#[test]
fn a_bolinha_grava_a_trajetoria_reta() {
    let t = RationalTime::from_seconds(1.0);
    let mut i = vec![
        TimelineIntent::BeginEdit,
        TimelineIntent::AddKey {
            entity: 7,
            prop: ancora(0),
            t,
            value: AnimValue::Float(1.0),
            interp: Interp::Hold,
        },
        TimelineIntent::AddPathKey {
            entity: 7,
            t,
            at: [1.0, 2.0],
        },
        TimelineIntent::AddPathKey {
            entity: FILHO,
            t,
            at: [3.0, 4.0],
        },
        TimelineIntent::EndEdit,
    ];
    iguala_o_ritmo(&mut i, &TimelineDoc::new());
    let cantos: Vec<u64> = i
        .iter()
        .filter_map(|x| match x {
            TimelineIntent::PathKeyCorner { entity, .. } => Some(*entity),
            _ => None,
        })
        .collect();
    assert_eq!(cantos, vec![7, FILHO], "{i:?}");
    assert_eq!(i.last(), Some(&TimelineIntent::EndEdit), "{i:?}");
    // Sem pivô no lote, a curva fica a do artista.
    let mut so_posicao = vec![TimelineIntent::AddPathKey {
        entity: 7,
        t,
        at: [1.0, 2.0],
    }];
    iguala_o_ritmo(&mut so_posicao, &TimelineDoc::new());
    assert_eq!(so_posicao.len(), 1);
}
