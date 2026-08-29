//! **Os gates do HOSPEDEIRO e da PROJEÇÃO** — irmão dos gates de autoria pelo teto de 600 LOC do
//! HR-18, cortado pela mesma linha que separa os módulos: ali *o que os verbos fazem*, aqui *quem
//! é o assunto e o que o painel vê dele*.

use super::super::tests::scene_with_host_and_child;
use super::*;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_vec_scene::{VecScene, rectangle};

/// Um mundo VAZIO — para os gates que falam só da PROJEÇÃO que o painel recebe.
///
/// ⚠️ Eles não precisam de árvore: uma seleção de UMA forma é hospedeira de si própria (o caso
/// degenerado da lei), e é isso que estes gates exercitam.
fn bare() -> (SimWorld, VecScene, VecEntityMap) {
    (SimWorld::new(), VecScene::default(), VecEntityMap::new())
}

/// ⭐ **UMA SELEÇÃO MÚLTIPLA TEM HOSPEDEIRO: a forma que a governa** (auditoria de 2026-08-23).
///
/// ⚠️ A regra era *"exatamente UMA forma, senão nada"*, e ela tornava a booleana viva
/// **inanimável pelo produto**: tocar um operando seleciona o GRUPO inteiro, então a seção STATES
/// — e o interruptor Preview com ela — não era sequer pintada.
///
/// A lei nova não inventa hospedeiro nenhum: ela **deriva** o que já existia. *"Gravar o estado
/// destas três formas"* continua a não ser uma pergunta com resposta; *"gravar o estado do widget
/// que as contém"* é, e é a que o `Rec` sempre respondeu (o estado é da SUB-ÁRVORE).
#[test]
fn a_multi_selection_is_hosted_by_the_shape_that_governs_it() {
    let (mut sim, mut scene, map, host, child) = scene_with_host_and_child();
    let mut states = StateSets::default();
    assert_eq!(
        host_of_selection(&sim, &scene, &map, &[host, child]),
        Some(host),
        "a forma que governa os dois nao foi encontrada"
    );
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host, child],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );
    assert!(
        !states.is_empty(),
        "gravar com o widget inteiro em maos nao gravou nada"
    );
    let v = publish(
        &sim,
        &scene,
        &map,
        &[host, child],
        &states,
        None,
        false,
        false,
    )
    .expect("a seccao e' oferecida");
    assert_eq!(
        v.host.as_deref(),
        Some("Host"),
        "o painel tem de NOMEAR o hospedeiro que a porta derivou"
    );
}

/// ⛔ **SEM forma que a governe, a seção EXISTE e diz o que falta.**
///
/// ⚠️ É a face VAZIA, e ela é a metade que importa: a seção inteira **desaparecia** — nem
/// cabeçalho, nem preview, nem uma palavra —, e desaparecia exactamente onde a feature mais
/// precisava de existir. *Uma seção que só existe onde a feature já foi usada é uma seção que não
/// existe em lugar nenhum.*
#[test]
fn a_selection_with_no_governing_shape_still_gets_the_section() {
    let (mut sim, mut scene, mut map, host, _child) = scene_with_host_and_child();
    // Uma segunda RAIZ: nada governa as duas ao mesmo tempo.
    let lone = scene.push_path(rectangle([9.0, 9.0], [10.0, 10.0]));
    let e = sim
        .world_mut()
        .spawn((
            Name("Lone".into()),
            Transform::IDENTITY,
            ph2d_ecs::VecPathRef(lone),
        ))
        .id();
    map.insert(lone, e.to_bits());

    assert_eq!(
        host_of_selection(&sim, &scene, &map, &[host, lone]),
        None,
        "duas raizes nao podem ter hospedeiro comum"
    );
    let v = publish(
        &sim,
        &scene,
        &map,
        &[host, lone],
        &StateSets::default(),
        None,
        false,
        false,
    )
    .expect("a seccao tem de EXISTIR, com a face vazia");
    assert_eq!(v.host, None, "sem hospedeiro, o painel nao pode nomear um");
    assert_eq!(
        v.preview, None,
        "sem hospedeiro nao se oferece um botao que age sobre coisa nenhuma"
    );

    // ⚠️ E o CONTROLE: a seleção VAZIA continua a não oferecer seção nenhuma.
    assert!(
        publish(
            &sim,
            &scene,
            &map,
            &[],
            &StateSets::default(),
            None,
            false,
            false
        )
        .is_none(),
        "sem selecao nenhuma a seccao nao tem assunto"
    );
}

/// **A seção é oferecida a qualquer forma única, com estados ou sem.**
///
/// ⚠️ A face VAZIA é a importante: uma seção que só existisse onde já há estados tornaria a
/// feature alcançável apenas onde ela já foi usada, ou seja em lugar nenhum. É a mesma lei da
/// seção de física.
#[test]
fn the_section_is_offered_before_there_is_anything_to_show() {
    let (sim, scene, map) = bare();
    let states = StateSets::default();
    let v = publish(&sim, &scene, &map, &[1], &states, None, false, false)
        .expect("a secao e' oferecida numa forma sem estados");
    assert_eq!(v.recorded, [false; 4]);
    assert!(
        (v.duration_s - 0.15).abs() < 1e-6,
        "o default nao chegou ao painel"
    );
}

/// **Todo papel tem nome traduzido, e o painel recebe o do CATÁLOGO.**
///
/// ⚠️ A metade que importa é a segunda: os rótulos são publicados por esta porta, então um papel
/// novo aparece nomeado sem ninguém tocar na UI. A metade da tradução pega o oposto — uma chave
/// que ninguém pôs no catálogo volta como a própria chave, e a linha do painel diria
/// `panel.vector.states.role.…` ao artista.
#[test]
fn every_role_reaches_the_panel_with_a_translated_name() {
    let (sim, scene, map) = bare();
    let v = publish(
        &sim,
        &scene,
        &map,
        &[1],
        &StateSets::default(),
        None,
        false,
        false,
    )
    .expect("a secao e' oferecida");
    for (i, &role) in StateRole::ALL.iter().enumerate() {
        let key = role.i18n_key();
        assert_ne!(
            v.role_labels[i], key,
            "o papel {role:?} nao tem traducao — a linha mostraria a chave crua"
        );
        assert_eq!(
            v.role_labels[i],
            ph2d_i18n::tr(key),
            "o rotulo publicado nao veio do catalogo"
        );
    }
}

/// **O interruptor da PREVIEW só é oferecido onde há o que pré-visualizar** (W7r).
///
/// ⚠️ A pergunta é sobre a CENA (*algum hospedeiro tem pose?*) dentro de uma seção da SELEÇÃO, e
/// é deliberado: a preview entrega o rato a todos os hospedeiros. O `None` é o que impede um
/// botão que não faz nada — [`crate::render_loop::ui_preview::UiPreview::enter`] recusa
/// exactamente a mesma condição, e um botão pintado sobre ela seria um clique que o artista não
/// tem como diagnosticar.
#[test]
fn the_preview_switch_is_offered_only_where_there_is_something_to_preview() {
    let (sim, scene, map) = bare();
    let states = StateSets::default();
    let v = publish(&sim, &scene, &map, &[1], &states, None, false, false)
        .expect("a secao e' oferecida");
    assert_eq!(
        v.preview, None,
        "o interruptor foi oferecido numa cena SEM pose nenhuma — ligar nao faria nada"
    );

    let (mut sim, mut scene, map, host, _child) = scene_with_host_and_child();
    let mut states = StateSets::default();
    apply(
        &mut sim,
        &mut scene,
        &map,
        &[host],
        &mut states,
        UiStateEdit::Record(StateRole::Default),
    );
    assert_eq!(
        publish(&sim, &scene, &map, &[host], &states, None, false, false)
            .unwrap()
            .preview,
        Some(false),
        "com uma pose gravada o interruptor tem de existir, desligado"
    );
    assert_eq!(
        publish(&sim, &scene, &map, &[host], &states, None, true, false)
            .unwrap()
            .preview,
        Some(true),
        "o estado LIGADO tem de chegar ao painel — senao o botao nunca acende"
    );
    // ⚠️ E a oferta é da CENA: um hospedeiro OUTRO que não o selecionado já basta.
    assert_eq!(
        publish(&sim, &scene, &map, &[999], &states, None, false, false)
            .unwrap()
            .preview,
        Some(false),
        "a oferta seguiu a SELECAO em vez da cena — a preview dirige todos os hospedeiros"
    );
}

/// **A FORMA no estado** — o que a autoria grava de geometria, e como ela viaja. Módulo FILHO
/// (e não irmão) de propósito: a fixture `scene_with_host_and_child` continua a ser **uma
/// porta**, e um irmão obrigaria a duplicá-la ou a torná-la pública para fora do assunto.
#[path = "vec_ui_state_shape_tests.rs"]
mod shape;

/// **A curva que o painel mostra é a que o documento guarda.**
///
/// ⚠️ O `publish` já buscava a curva e **deitava-a fora** (`let (duration, _easing) = …`) — era
/// literalmente um sublinhado a separar um campo persistido desde o v56 do artista que o queria.
/// Este gate é o que impede que ele volte a ser descartado.
#[test]
fn the_panel_is_shown_the_curve_the_document_holds() {
    let (sim, scene, map) = bare();
    use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M};
    let host: VecPathId = 7;
    let mut states = StateSets::default();
    let mine = Easing::new(F::Bounce, M::In);
    states.set_easing(host, mine);
    let snap = publish(&sim, &scene, &map, &[host], &states, None, false, false)
        .expect("um hospedeiro, uma seccao");
    assert_eq!(
        snap.easing, mine,
        "o painel receberia {:?} enquanto o documento guarda {:?}",
        snap.easing, mine
    );
}
