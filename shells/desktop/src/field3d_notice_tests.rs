//! ⭐ **Os gates da VOZ do módulo** (W25): uma peça que não cozinha diz porquê, e o clique que a
//! partia deixa de existir.
//!
//! ⚠️ **O defeito que estes gates fixam é de UM CLIQUE.** Selecionar uma escultura e carregar em
//! `Shell` escrevia um modificador que o documento **recusa** — e a recusa chegava a
//! `cook(...).and_then(Result::ok)`, que a deitava fora. Resultado: a peça **inteira** desaparecia da
//! tela, a Hierarquia continuava a mostrar tudo, e nada dizia uma palavra. *Um erro engolido é pior
//! do que um erro: ele parece um bug de câmera.*

use super::*;
use ph2d_ecs::SimWorld;

/// Uma peça com uma escultura e um cilindro, subtraídos. Devolve `(sim, entidade da escultura)`.
fn a_part_with_a_sculpture() -> (SimWorld, bevy_ecs::entity::Entity) {
    let key = "/tmp/ph2d-w25-escultura.obj";
    let mesh = ph2d_mesh::shapes::uv_sphere(12, 24, 1.0);
    let field = ph2d_field_mesh::SampledField::from_mesh(&mesh, 24).expect("esfera");
    crate::field3d_smoke::register_sampled(key, std::sync::Arc::new(field));

    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(&crate::field3d_smoke::scene(2)), 0.0);
    crate::field3d_smoke::ask_spawn_sculpt(key.to_string());
    crate::field3d_scene::sync_scene(&mut sim, None, 0.0);

    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    let sculpture = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|e| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e).map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Sampled { .. })
            )
        })
        .expect("a escultura está na peça");
    (sim, sculpture)
}

/// ⭐ **Um clique num modificador NÃO pode apagar a peça** — o defeito, pela porta que o artista usa.
///
/// ⚠️ O caminho inteiro: o botão do painel → o intent → o mundo → o cozimento. O gate dirige-o de
/// ponta a ponta de propósito: a recusa vive no documento (`ModsOnSampled`), e o que faltava era
/// **ninguém a consultar antes de escrever**.
#[test]
fn a_modifier_click_on_a_sculpture_never_blanks_the_piece() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, sculpture) = a_part_with_a_sculpture();
    let before = crate::field3d_scene::sync_scene(&mut sim, None, 0.0);
    assert!(
        before.is_some(),
        "a fixture tem de cozinhar ANTES do clique"
    );

    for slot in 0..ph2d_field::UnaryKind::ALL.len() {
        ph2d_panel_model3d::state::push_intent_for_test(
            ph2d_panel_model3d::ModelIntent::ToggleMod { slot },
        );
        let after = crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[sculpture],
            0.0,
            &crate::field3d_scene::no_drawing(),
        )
        .0;
        assert!(
            after.is_some(),
            "o modificador nº {slot} sobre uma escultura APAGOU a peça inteira — e nada na tela o \
             diz. A recusa é do documento; quem tem de a consultar é a porta"
        );
    }
}

/// ⭐ **O painel não OFERECE o que a escultura não aceita.**
///
/// ⚠️ Recusar sem tirar o botão seria a metade errada: o artista carrega e não acontece nada, que é
/// a forma mais cara de um controle mentir. A lei já está escrita neste módulo para a fileira de
/// operações — *um controle que aparece e não faz nada é pior do que um que não aparece*.
///
/// ⚠️ **O gate lê o que o PAINEL recebe** (`state::current()`), e não o ajudante que o constrói: o
/// que interessa é a fileira que aparece na tela, e uma medição do helper deixaria passar o dia em
/// que alguém o parasse de chamar.
#[test]
fn the_panel_offers_no_modifiers_for_a_sculpture() {
    let (mut sim, sculpture) = a_part_with_a_sculpture();
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    let leaf = {
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldNode)>();
        q.iter(world)
            .find(|(_, n)| matches!(n.shape, ph2d_field::NodeShape::Leaf(_)))
            .map(|(e, _)| e)
            .expect("a peça tem uma primitiva")
    };

    crate::field3d_scene::publish_snapshot(world, root, &[sculpture], 2.4, 0.0);
    assert!(
        ph2d_panel_model3d::state::current().mods.is_empty(),
        "uma escultura não aceita modificadores — a fileira não pode ser pintada"
    );

    // ⚠️ O CONTROLE: numa forma normal a fileira continua lá. Sem isto o gate passaria com a
    // fileira apagada para toda a gente.
    crate::field3d_scene::publish_snapshot(world, root, &[leaf], 2.4, 0.0);
    assert!(
        !ph2d_panel_model3d::state::current().mods.is_empty(),
        "numa forma normal os modificadores TÊM de continuar a aparecer"
    );
}

/// ⭐ **Uma peça que não cozinha DIZ porquê** — a voz, para todo o resto.
///
/// ⚠️ **Este gate escreve o estado inválido à mão, e é de propósito.** Depois da cura acima, o
/// clique já não o produz — mas um projeto de uma versão anterior, uma linha nova com outro
/// escritor, ou um erro que ainda não existe produzem. *A voz não é sobre este bug; é sobre a classe
/// dele.*
#[test]
fn a_piece_that_cannot_cook_says_why() {
    let (mut sim, sculpture) = a_part_with_a_sculpture();
    let _ = drain();
    forget_last();

    // Pela porta de trás: o componente entra no mundo sem passar pela recusa.
    sim.world_mut()
        .entity_mut(sculpture)
        .insert(ph2d_field_ecs::FieldMods {
            stack: vec![ph2d_field::Unary::Shell { thickness: 0.05 }],
        });
    let cooked = crate::field3d_scene::sync_scene(&mut sim, None, 0.0);
    assert!(
        cooked.is_none(),
        "a fixture tem de produzir mesmo um documento inválido — senão o gate não mede a voz"
    );

    let said = drain();
    assert_eq!(
        said.len(),
        1,
        "um documento inválido tem de virar exactamente um aviso; disse {said:?}"
    );
    assert!(
        said[0].to_lowercase().contains("sculpture"),
        "e o aviso tem de dizer O QUE está errado, não «erro»: disse {:?}",
        said[0]
    );
}

/// ⭐ **A voz não se repete por quadro.**
///
/// ⚠️ O cozimento corre a cada quadro, e uma peça inválida **continua inválida** — sem esta lei
/// seriam 60 avisos por segundo sobre a mesma coisa, e a tela ficaria ilegível exactamente quando o
/// artista precisa de a ler. É a irmã da lei da W23 (uma tentativa por nome), um nível acima.
#[test]
fn the_same_notice_is_not_said_twice_in_a_row() {
    let _ = drain();
    forget_last();
    say("A sculpture cannot take modifiers".into());
    say("A sculpture cannot take modifiers".into());
    assert_eq!(
        drain().len(),
        1,
        "a mesma frase duas vezes seguidas é ruído — o canal não a repete"
    );

    // …mas uma frase DIFERENTE passa, e a primeira volta a passar depois dela: o que se recusa é a
    // repetição, não a frase.
    forget_last();
    say("A".into());
    say("B".into());
    say("A".into());
    assert_eq!(
        drain().len(),
        3,
        "três frases distintas em sequência passam"
    );
}

/// **Cada erro do documento tem uma frase própria** — nenhuma cai num «erro desconhecido».
///
/// ⚠️ Um `match` com braço `_` compilaria e deixaria o erro NOVO sem frase, que é a forma mais
/// silenciosa de uma mensagem apodrecer. Aqui a lista é enumerada à mão de propósito: quem
/// acrescentar uma variante ao documento vê este gate a falhar.
#[test]
fn every_document_error_has_words_of_its_own() {
    use ph2d_field::FieldError::*;
    let all = [
        BadRoot,
        ForwardReference {
            parent: 1,
            child: 2,
        },
        EmptyCombine { node: 0 },
        NonPositive {
            node: 0,
            what: "radius",
        },
        RoundTooLarge {
            node: 0,
            round: 1.0,
            limit: 0.5,
        },
        BadScale { node: 0 },
        ProfileCrossesAxis {
            node: 0,
            min_x: -0.2,
        },
        EmptySampledKey { node: 0 },
        ModsOnSampled { node: 0 },
    ];
    let mut said: Vec<String> = Vec::new();
    for e in &all {
        let m = explain(e);
        assert!(
            m.len() > 8 && !m.to_lowercase().contains("error"),
            "«{m}» não é uma frase que o artista entenda"
        );
        said.push(m);
    }
    let mut uniq = said.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(
        uniq.len(),
        said.len(),
        "dois erros diferentes com a mesma frase mandam o artista procurar a coisa errada: {said:?}"
    );
}
