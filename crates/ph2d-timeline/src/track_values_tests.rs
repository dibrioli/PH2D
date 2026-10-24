//! Gates do [`super::TrackValues`] — o número que a dope-sheet mostra ao lado do nome.

use super::{Palavra, TrackValues};
use crate::{Extrap, PropKind, TrackView};
use ph2d_anim::AnimTarget;

/// Uma row mínima: o que o publicador lê é `target`, `entity` e `prop`.
fn row(target: u64, entity: u64, prop: PropKind) -> TrackView {
    TrackView {
        target: AnimTarget::new(target),
        prop,
        entity,
        missing: false,
        keys: Vec::new(),
        buffer_ghost: None,
        pre: Extrap::default(),
        post: Extrap::default(),
        expr: None,
    }
}

/// **DUAS ROWS DO MESMO OBJECTO CARREGAM NÚMEROS DIFERENTES** — a razão de a chave ser o ALVO.
///
/// ⚠️ Mutação que tem de sangrar: chavear por `entity`. Um objecto com X e Y animados mostraria o
/// mesmo número nas duas rows (a última a ser publicada), e a leitura seria *"o Y está errado"*.
#[test]
fn two_rows_of_the_same_object_carry_different_numbers() {
    let tracks = [
        row(1, 7, PropKind::TranslationX),
        row(2, 7, PropKind::TranslationY),
    ];
    let mut v = TrackValues::default();
    v.publish(&tracks, |_, p| {
        Some(if p == PropKind::TranslationX {
            3.0
        } else {
            -9.0
        })
    });
    assert_eq!(v.get(1), Some(3.0));
    assert_eq!(v.get(2), Some(-9.0));
}

/// **UM CANAL SEM NÚMERO NÃO PUBLICA ZERO** — ele não publica nada.
///
/// ⚠️ *Um zero de «não medido» e um zero de «vale zero» são o mesmo byte* (lei do repo). O
/// `TimeRemap` e o `Position` recusam na porta de amostragem por desenho, e uma entidade morta
/// recusa por acidente: nos três casos a row tem de ficar **sem número**, e não a anunciar `0,00`.
#[test]
fn a_channel_with_no_number_publishes_nothing_not_zero() {
    let tracks = [row(1, 7, PropKind::TimeRemap)];
    let mut v = TrackValues::default();
    v.publish(&tracks, |_, _| None);
    assert_eq!(v.get(1), None, "a ausencia nao pode virar um zero pintado");
}

/// **UMA ROW QUE SAIU LEVA O NÚMERO DELA** — a razão de a publicação ser uma porta que LIMPA.
///
/// ⚠️ O `bevy` recicla bits de entidade e o alvo é derivado deles: uma entrada velha não fica
/// «desactualizada», ela passa a descrever **outro objecto**.
#[test]
fn a_row_that_left_takes_its_number_with_it() {
    let mut v = TrackValues::default();
    v.publish(&[row(1, 7, PropKind::Opacity)], |_, _| Some(0.5));
    assert_eq!(v.get(1), Some(0.5));
    v.publish(&[], |_, _| Some(0.5));
    assert_eq!(v.get(1), None, "a track saiu e o numero dela ficou");
}

/// **UMA ROW DISCRETA LEVA AS SUAS PALAVRAS** — o valor de agora e o de cada chave; uma row de
/// número não leva nenhuma, e o texto `0` («nenhum») não nomeia nada.
///
/// ⚠️ Mutação que tem de sangrar: não levar as palavras das CHAVES (só a do valor de agora).
#[test]
fn a_text_row_carries_its_texts_and_a_number_row_none() {
    use ph2d_ecs::canal::familia;
    use ph2d_ecs::{FieldKey, TextosDosCanais, canal_vetor};
    let conteudo = FieldKey::new(familia::VETOR, canal_vetor::TEXTO_CONTEUDO, 0, 0);
    let mut tabela = TextosDosCanais::default();
    let (a, b) = (tabela.indice(Some("Box A")), tabela.indice(Some("Box C")));
    let mut mundo = bevy_ecs::world::World::new();
    mundo.insert_resource(tabela);
    let mut texto = row(1, 7, PropKind::Field(conteudo));
    texto.keys.push(crate::KeyView {
        id: crate::KeyId::new(1),
        t_seconds: 0.0,
        value: b,
        interp: ph2d_anim::Interp::Hold,
        selected: false,
        roving: false,
    });
    let tracks = [texto, row(2, 7, PropKind::Opacity)];
    let mut v = TrackValues::default();
    v.publish_com_mundo(&tracks, Some(&mundo), |_, p| {
        Some(if p == PropKind::Opacity { 1.0 } else { a })
    });
    let artista = |s: &str| Some(Palavra::DoArtista(s.to_owned()));
    assert_eq!(
        v.get(1).and_then(|x| v.palavra(1, x)).cloned(),
        artista("Box A"),
        "o valor de agora"
    );
    assert_eq!(
        v.palavra(1, b).cloned(),
        artista("Box C"),
        "o valor da chave"
    );
    assert_eq!(v.palavra(1, 0.0), None, "o 0 é «nenhum»");
    assert_eq!(
        v.palavra(2, 1.0),
        None,
        "uma row de número não publica palavra"
    );
}

/// ⭐ **A escolha publica a chave do rótulo da OPÇÃO** (F27 F) — a mesma palavra que o cartão do
/// HUD pinta (`Tag`), nunca o índice `3`.
#[test]
fn uma_escolha_publica_o_rotulo_da_opcao() {
    let fonte = ph2d_ecs::canal_componente::chaves_do_rotulo("component.field.label_fields.1")[0];
    let tracks = [row(1, 7, PropKind::Field(fonte))];
    let mut v = TrackValues::default();
    v.publish_com_mundo(&tracks, Some(&bevy_ecs::world::World::new()), |_, _| {
        Some(3.0)
    });
    let Some(Palavra::Chave(c)) = v.palavra(1, 3.0) else {
        panic!("a escolha não publicou palavra");
    };
    assert_eq!(
        ph2d_i18n::tr(c),
        ph2d_i18n::tr("panel.inspector.hud.source_tag")
    );
    assert_eq!(ph2d_i18n::tr(c), "Tag");
}
