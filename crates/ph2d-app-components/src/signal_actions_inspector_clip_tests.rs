//! ⭐ **O ▾ do *Play Clip*** (onda 5, item c) — o retrato oferece os clipes da Animação do ALVO,
//! lidos do `ClipNames` que a timeline publica, e só quando o alvo é UM objecto conhecido na edição.

use super::*;
use ph2d_ecs::{ClipNames, Name, Transform};

fn com_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    sim.world_mut()
        .spawn((Transform::default(), Name::new(nome)))
        .id()
}

/// O objecto da tabela (o «eu»), uma `Porta` com clipes e os `ClipNames` que a timeline publicaria.
/// ⚠️ Os ids: no produto o `upkeep` da timeline atribui-os a cada quadro; o retrato só LÊ.
fn cena(linha: SignalAction) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::default();
    let eu = com_nome(&mut sim, "Herói");
    sim.world_mut()
        .entity_mut(eu)
        .insert(SignalActions(vec![linha]));
    let porta = com_nome(&mut sim, "Porta");
    sim.world_mut().insert_resource(ClipNames(vec![
        (ClipNames::SCENE, "intro".into()),
        (eu.to_bits(), "andar".into()),
        (porta.to_bits(), "abrir".into()),
        (eu.to_bits(), "correr".into()),
        (porta.to_bits(), "fechar".into()),
    ]));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, eu, porta)
}

fn tocar(alvo: &str, target_by: SignalTarget) -> SignalAction {
    SignalAction {
        on: "abre".into(),
        target: alvo.into(),
        verb: SignalVerb::PlayClip,
        target_by,
        ..SignalAction::default()
    }
}

fn opcoes(sim: &SimWorld, eu: Entity) -> Option<Vec<String>> {
    build_action_info(sim.world(), &ph2d_tags::TagTree::new(), eu.to_bits(), 1)
        .expect("o objecto tem a tabela")
        .rows[0]
        .arg_options
        .clone()
}

/// ⭐ Alvo vazio = ESTE objecto ⇒ os clipes DELE, na ordem do documento, e nenhum de outro dono.
///
/// **Mutação que deve sangrar:** a escolha do alvo (`Named ⇒ Some`) trocada por `None`.
#[test]
fn play_clip_neste_objecto_oferece_os_clipes_dele() {
    let (sim, eu, _) = cena(tocar("", SignalTarget::Named));
    assert_eq!(
        opcoes(&sim, eu),
        Some(vec!["andar".to_string(), "correr".to_string()])
    );
}

/// ⭐ Alvo pelo NOME ⇒ os clipes do objecto com esse nome (a porta da ponte resolve-o).
#[test]
fn play_clip_num_objecto_nomeado_oferece_os_clipes_dele() {
    let (sim, eu, _) = cena(tocar("Porta", SignalTarget::Named));
    assert_eq!(
        opcoes(&sim, eu),
        Some(vec!["abrir".to_string(), "fechar".to_string()])
    );
}

/// Um nome que não é de ninguém, ou um mundo sem `ClipNames`, oferece uma lista VAZIA (o ▾ diz que o
/// alvo não tem Animação) — nunca o texto livre.
#[test]
fn sem_alvo_ou_sem_recurso_a_lista_e_vazia() {
    let (sim, eu, _) = cena(tocar("Ninguém", SignalTarget::Named));
    assert_eq!(opcoes(&sim, eu), Some(vec![]));
    let (mut sim, eu, _) = cena(tocar("", SignalTarget::Named));
    sim.world_mut().remove_resource::<ClipNames>();
    assert_eq!(opcoes(&sim, eu), Some(vec![]));
}

/// ⚠️ A fronteira: por TAG ou QUEM BATEU os alvos são muitos ou só existem no Play ⇒ texto livre.
/// E os outros verbos (o nome de um timer) nunca têm ▾.
#[test]
fn tag_quem_bateu_e_outros_verbos_ficam_no_texto_livre() {
    let (sim, eu, _) = cena(tocar("", SignalTarget::Tagged(3)));
    assert_eq!(opcoes(&sim, eu), None);
    let (sim, eu, _) = cena(tocar("", SignalTarget::Other));
    assert_eq!(opcoes(&sim, eu), None);
    let mut timer = tocar("", SignalTarget::Named);
    timer.verb = SignalVerb::StartTimer;
    let (sim, eu, _) = cena(timer);
    assert_eq!(opcoes(&sim, eu), None);
}

/// ⭐ O ▾ tem um id por clipe que UMA Animação pode ter — o painel não vê a timeline, então este
/// gate prende os dois (o molde do `TIMELINE_CLIP_OPT`).
#[test]
fn os_ids_do_seletor_de_clipe_cobrem_o_tecto_da_animacao() {
    assert_eq!(
        ph2d_panel_inspector::ids::INSP_ACTION_CLIP.len(),
        ph2d_timeline::MAX_CLIPS,
        "suba MAX_CLIPS e INSP_ACTION_CLIP juntos — um clipe a mais seria uma opção sem id"
    );
}
