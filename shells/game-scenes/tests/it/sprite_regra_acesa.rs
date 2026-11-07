//! ⭐ **A regra acesa no Play** ([ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md), E5):
//! o evento real (`anima_personagens`, com tecla premida ou sinal no barramento) muda o índice aceso; cada objecto lê o
//! seu; fora do Play nada acende; e nada disto entra no documento.

use crate::comp::registo;
use ph2d_app_game::personagem::{anima_personagens, regra_acesa};
use ph2d_app_sprite_anim::inspector::build_anim_info;
use ph2d_ecs::scene::{WorldSnapshot, world_to_snapshot};
use ph2d_ecs::{AnimationTag, Entity, SimWorld, SpriteAnimations, SpriteAnimator};
use ph2d_physics_ecs::{AnimRule, AnimRules, AnimWhen, PhysicsBridge};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_runtime::{Signal, SignalOutbox};
use std::collections::BTreeSet;

fn objeto(sim: &mut SimWorld, regras: Vec<AnimRule>) -> Entity {
    let mut lib = SpriteAnimations::new();
    for (i, n) in ["idle", "run", "hit"].into_iter().enumerate() {
        let c = u32::try_from(i).expect("três");
        lib.insert(AnimationTag::new(n, c, c)).expect("tag");
    }
    let mut tocador = SpriteAnimator::new("idle");
    tocador.playing = true;
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::default(),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
            lib,
            tocador,
        ))
        .id();
    if !regras.is_empty() {
        sim.world_mut().entity_mut(e).insert(AnimRules(regras));
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    e
}

fn regra(when: AnimWhen, play: &str) -> AnimRule {
    AnimRule {
        when,
        play: play.into(),
    }
}

fn tecla(nome: &str) -> AnimWhen {
    AnimWhen::Action(nome.into())
}

fn premidas(nomes: &[&str]) -> BTreeSet<String> {
    nomes.iter().map(|n| (*n).to_owned()).collect()
}

fn quadro(sim: &mut SimWorld, teclas: &[&str], outbox: &SignalOutbox) {
    anima_personagens(
        sim.world_mut(),
        &PhysicsBridge::new(),
        &premidas(teclas),
        outbox,
    );
}

fn acesa(sim: &SimWorld, e: Entity) -> Option<u8> {
    regra_acesa(sim.world(), e.to_bits(), true)
}

/// A condição muda, o índice aceso muda: a de cima ganha, e sem condição nada acende. Fora do Play, nada.
/// *Mutação: acender sempre a 1.ª regra, ou não limpar a acesa ⇒ vermelho.*
#[test]
fn a_regra_acesa_segue_a_condicao_que_muda() {
    let mut sim = SimWorld::new();
    let e = objeto(
        &mut sim,
        vec![regra(tecla("hit"), "hit"), regra(tecla("go"), "run")],
    );
    let out = SignalOutbox::new();
    quadro(&mut sim, &[], &out);
    assert_eq!(acesa(&sim, e), None, "nenhuma condição");
    quadro(&mut sim, &["go"], &out);
    assert_eq!(acesa(&sim, e), Some(1));
    assert_eq!(regra_acesa(sim.world(), e.to_bits(), false), None, "parado");
    quadro(&mut sim, &["go", "hit"], &out);
    assert_eq!(acesa(&sim, e), Some(0), "a de cima ganha");
    quadro(&mut sim, &["go"], &out);
    assert_eq!(acesa(&sim, e), Some(1));
    quadro(&mut sim, &[], &out);
    assert_eq!(acesa(&sim, e), None, "soltou: apaga");
}

/// Uma regra de sinal segura a animação até ela fechar uma volta: enquanto segura, é ela que manda, e fica acesa.
#[test]
fn o_sinal_fica_aceso_enquanto_segura_a_animacao() {
    let mut sim = SimWorld::new();
    let e = objeto(&mut sim, vec![regra(AnimWhen::Signal("s".into()), "hit")]);
    let mut out = SignalOutbox::new();
    quadro(&mut sim, &[], &out);
    assert_eq!(acesa(&sim, e), None);
    out.publish(Signal::from_animation("s", e.to_bits(), 0));
    quadro(&mut sim, &[], &out);
    assert_eq!(acesa(&sim, e), Some(0), "o sinal ganhou");
    quadro(&mut sim, &[], &out);
    assert_eq!(acesa(&sim, e), Some(0), "continua a segurar");
    sim.world_mut()
        .get_mut::<SpriteAnimator>(e)
        .expect("tocador")
        .repeat_count = 1;
    quadro(&mut sim, &[], &out);
    assert_eq!(acesa(&sim, e), None, "fechou a volta: solta");
}

/// O cartão que está à vista é o do objecto dele: a seleção doutro, ou o cartão preso num, lê o índice de quem mostra.
/// *Mutação: guardar uma só regra acesa por mundo ⇒ `b` acende a de `a`.*
#[test]
fn outro_objeto_nao_acende_o_errado() {
    let mut sim = SimWorld::new();
    let a = objeto(
        &mut sim,
        vec![regra(tecla("hit"), "hit"), regra(tecla("go"), "run")],
    );
    let b = objeto(&mut sim, vec![regra(tecla("go"), "run")]);
    let sem_regras = objeto(&mut sim, Vec::new());
    quadro(&mut sim, &["go"], &SignalOutbox::new());
    assert_eq!(acesa(&sim, a), Some(1));
    assert_eq!(acesa(&sim, b), Some(0));
    assert_eq!(acesa(&sim, sem_regras), None);
    // O retrato de cada cartão nasce sem aceso: só a composição o preenche, pelo id do próprio retrato.
    for e in [a, b, sem_regras] {
        let info = build_anim_info(sim.world(), e.to_bits(), 1).expect("tem tocador");
        assert_eq!(info.entity_bits, e.to_bits());
        assert_eq!(info.card.live_rule, None);
    }
}

fn foto(sim: &mut SimWorld) -> WorldSnapshot {
    let r = registo();
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &r, &mut snap).expect("captura");
    snap
}

/// A acesa não é documento: acender e apagar deixa a foto do save e do undo igual ao bit, e o animador em paz.
/// *Controlo: a tecla que troca a animação muda a foto; acender uma regra que toca a que já toca, não.*
#[test]
fn a_regra_acesa_nao_entra_no_documento() {
    let mut sim = SimWorld::new();
    let e = objeto(&mut sim, vec![regra(tecla("idle"), "idle")]);
    let out = SignalOutbox::new();
    let antes = foto(&mut sim);
    quadro(&mut sim, &["idle"], &out);
    assert_eq!(acesa(&sim, e), Some(0));
    assert_eq!(foto(&mut sim), antes, "acesa e a foto não mudou");
    quadro(&mut sim, &[], &out);
    assert_eq!(foto(&mut sim), antes, "apagou e a foto não mudou");
    sim.world_mut()
        .entity_mut(e)
        .insert(AnimRules(vec![regra(tecla("go"), "run")]));
    let controlo = foto(&mut sim);
    quadro(&mut sim, &["go"], &out);
    assert_ne!(foto(&mut sim), controlo, "a régua vê a troca de animação");
}

/// O componente a mais não vira ruído: `SpriteAnimStart` e `SpriteCutNotice` não dão ícone novo na linha da Hierarchy,
/// nem entram na paleta nem têm secção. *Mutação: declará-los `authored` no catálogo ⇒ vermelho.*
#[test]
fn inicio_e_aviso_nao_viram_ruido_na_hierarquia() {
    use ph2d_app_components::row_components::RowComponents;
    use ph2d_ecs::{SpriteAnimStart, SpriteCutNotice};
    let mut sim = SimWorld::new();
    let e = objeto(&mut sim, Vec::new());
    let r = registo();
    let icones =
        |sim: &SimWorld| RowComponents::of(sim.world(), &r).of_entity(sim.world(), e.to_bits());
    let antes = icones(&sim);
    assert!(!antes.is_empty(), "controlo: o tocador tem ícone");
    sim.world_mut().entity_mut(e).insert((
        SpriteAnimStart("run".into()),
        SpriteCutNotice { grid: [8, 4] },
    ));
    assert_eq!(icones(&sim), antes, "os dois não acrescentam família");
    for nome in ["ph2d::ecs::SpriteAnimStart", "ph2d::ecs::SpriteCutNotice"] {
        let d = r
            .iter()
            .find(|x| x.canonical_name == nome)
            .and_then(|x| x.desc)
            .expect("no catálogo");
        assert!(
            !d.is_offered() && !d.may_have_section(),
            "{nome} é maquinaria"
        );
    }
}

/// ⭐ O aviso é estado que o artista ainda não resolveu: viaja no save enquanto a imagem está errada e reaparece ao
/// reabrir; respondido (Keep the cut ou Cut again), não volta. *Mutação: tirar-lhe o registo ⇒ o reabrir perde-o.*
#[test]
fn o_aviso_reabre_so_enquanto_ninguem_respondeu() {
    use ph2d_ecs::SpriteCutNotice;
    use ph2d_ecs::scene::snapshot_to_world;
    let abre = |snap: &WorldSnapshot| {
        let mut s = SimWorld::new();
        snapshot_to_world(s.world_mut(), snap, &registo()).expect("repõe");
        let w = s.world_mut();
        let e = w
            .query::<(Entity, &SpriteAnimator)>()
            .iter(w)
            .next()
            .expect("o tocador")
            .0;
        w.get::<SpriteCutNotice>(e).copied()
    };
    let mut sim = SimWorld::new();
    let e = objeto(&mut sim, Vec::new());
    sim.world_mut()
        .entity_mut(e)
        .insert(SpriteCutNotice { grid: [8, 4] });
    assert_eq!(
        abre(&foto(&mut sim)),
        Some(SpriteCutNotice { grid: [8, 4] }),
        "a pergunta continua aberta"
    );
    assert!(ph2d_app_sprite_anim::sprite_card::keep_cut(
        sim.world_mut(),
        e
    ));
    assert_eq!(abre(&foto(&mut sim)), None, "respondida, não volta");
}
