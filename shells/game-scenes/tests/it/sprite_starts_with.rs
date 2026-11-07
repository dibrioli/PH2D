//! ⭐ **O «Starts with» pela porta do desfazer e do abrir** ([ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md)):
//! o componente viaja no instantâneo, repor o de antes desfaz, e o projeto antigo abre igual.

use crate::comp::registo;
use ph2d_app_sprite_anim::inspector::apply_anim_edit;
use ph2d_app_sprite_anim::tick::start_autoplay_animations;
use ph2d_ecs::scene::{
    EditorCommandQueue, WorldSnapshot, apply_editor_commands, snapshot_to_world, world_to_snapshot,
};
use ph2d_ecs::{
    AnimationTag, Entity, SimWorld, SpriteAnimStart, SpriteAnimations, SpriteAnimator, SpriteGrid,
};
use ph2d_editor_core::AnimFieldEdit;

fn cena() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("idle", 0, 3)).expect("idle");
    lib.insert(AnimationTag::new("run", 4, 7)).expect("run");
    let mut a = SpriteAnimator::new("idle");
    a.autoplay = true;
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::default(),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpriteGrid {
                hframes: 8,
                vframes: 1,
                frame: 0,
            },
            lib,
            a,
        ))
        .id();
    (sim, e)
}

fn edita(sim: &mut SimWorld, e: Entity, edit: AnimFieldEdit) {
    let (q, r) = (EditorCommandQueue::new(), registo());
    let _ = apply_anim_edit(sim, e.to_bits(), &edit, &q, &r);
    apply_editor_commands(sim.world_mut(), &q, &r).expect("o commit aplica");
}

fn foto(sim: &mut SimWorld) -> WorldSnapshot {
    let r = registo();
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &r, &mut snap).expect("captura");
    snap
}

fn aberto(snap: &WorldSnapshot) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    snapshot_to_world(sim.world_mut(), snap, &registo()).expect("repõe");
    let w = sim.world_mut();
    let e = w
        .query::<(Entity, &SpriteAnimator)>()
        .iter(w)
        .next()
        .expect("o tocador")
        .0;
    (sim, e)
}

fn atual(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<SpriteAnimator>(e)
        .expect("tocador")
        .current
        .clone()
}

/// ⭐ O gesto inteiro: ▶ em `run`, gravar e reabrir (a porta do undo e do save) começa em `idle`; desfazer tira o início.
/// *Mutação: deixar de registar o `SpriteAnimStart` ⇒ a foto perde-o e o reabrir fica em `run`.*
#[test]
fn o_starts_with_viaja_na_foto_e_o_desfazer_repoe() {
    let (mut sim, e) = cena();
    let antes = foto(&mut sim);
    edita(&mut sim, e, AnimFieldEdit::SetCurrent("run".into()));
    edita(&mut sim, e, AnimFieldEdit::Playing(true));
    assert_eq!(atual(&sim, e), "run");

    let (mut reaberto, r) = aberto(&foto(&mut sim));
    assert_eq!(
        reaberto
            .world()
            .get::<SpriteAnimStart>(r)
            .map(|s| s.0.as_str()),
        Some("idle")
    );
    start_autoplay_animations(&mut reaberto);
    assert_eq!(atual(&reaberto, r), "idle");

    let (desfeito, d) = aberto(&antes);
    assert!(desfeito.world().get::<SpriteAnimStart>(d).is_none());
    assert_eq!(atual(&desfeito, d), "idle");
}

/// O Play joga sobre a cópia que a fotografia guardou: começar no início não escreve no documento fotografado.
#[test]
fn a_foto_tirada_antes_de_comecar_guarda_a_previa() {
    let (mut sim, e) = cena();
    edita(&mut sim, e, AnimFieldEdit::SetCurrent("run".into()));
    let documento = foto(&mut sim);
    start_autoplay_animations(&mut sim);
    assert_eq!(atual(&sim, e), "idle", "o jogo");
    let (volta, d) = aberto(&documento);
    assert_eq!(atual(&volta, d), "run", "o Stop devolve o documento");
}

/// Sem o blob (projeto de antes): abre, e começa no `current` gravado.
#[test]
fn o_projeto_antigo_abre_igual() {
    let (mut sim, e) = cena();
    edita(&mut sim, e, AnimFieldEdit::SetCurrent("run".into()));
    sim.world_mut().entity_mut(e).remove::<SpriteAnimStart>();
    let gravado = foto(&mut sim);
    let (mut velho, v) = aberto(&gravado);
    // Abrir não inventa os componentes novos, e gravar de novo dá os mesmos bytes.
    assert!(velho.world().get::<SpriteAnimStart>(v).is_none());
    assert!(velho.world().get::<ph2d_ecs::SpriteCutNotice>(v).is_none());
    assert_eq!(foto(&mut velho), gravado);
    start_autoplay_animations(&mut velho);
    assert_eq!(atual(&velho, v), "run");
}

/// O Play sempre começa no 1.º quadro do início, mesmo com o tocador já nele e a tocar: nada de meio de ciclo.
/// *Mutação: voltar o `a.current == nome` em `vai_para_o_inicio` ⇒ o relógio fica onde estava.*
#[test]
fn o_play_comeca_no_primeiro_quadro_mesmo_ja_a_tocar_o_inicio() {
    for com_inicio in [true, false] {
        let (mut sim, e) = cena();
        if com_inicio {
            edita(&mut sim, e, AnimFieldEdit::StartsWith("idle".into()));
        }
        let w = sim.world_mut();
        let mut a = w.get_mut::<SpriteAnimator>(e).expect("tocador");
        a.playing = true;
        a.elapsed_ticks = 12_345;
        a.repeat_count = 2;
        w.get_mut::<SpriteGrid>(e).expect("grelha").frame = 2;
        start_autoplay_animations(&mut sim);
        let w = sim.world();
        let a = w.get::<SpriteAnimator>(e).expect("tocador");
        assert_eq!((a.elapsed_ticks, a.repeat_count, a.playing), (0, 0, true));
        assert_eq!(w.get::<SpriteGrid>(e).expect("grelha").frame, 0);
    }
}

/// ⭐ `PH2D_CARDS=0` não tem menu: a linha muda com o que começa (a lei antiga), e nada fica preso.
#[test]
fn sem_cartoes_clicar_na_linha_muda_com_o_que_comeca() {
    let (mut sim, e) = cena();
    for nome in ["run", "idle", "run"] {
        edita(&mut sim, e, AnimFieldEdit::SetCurrent(nome.into()));
        edita(&mut sim, e, AnimFieldEdit::StartsWith(nome.into()));
        start_autoplay_animations(&mut sim);
        assert_eq!(atual(&sim, e), nome);
    }
    assert_eq!(
        sim.world().get::<SpriteAnimStart>(e).map(|s| s.0.as_str()),
        Some("run")
    );
}

/// Suspeita descartada: o início e o aviso são componentes registados: duplicar e instanciar levam os dois, e a
/// instância recebe a mudança do início no mestre.
#[test]
fn inicio_e_aviso_viajam_no_duplicar_e_na_instancia() {
    use ph2d_app_components::{instance_docs, instantiate, test_support as ts};
    let r = registo();
    let mut sim = SimWorld::new();
    let m = ts::plain_master(&mut sim);
    let w = sim.world_mut();
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("idle", 0, 1)).expect("idle");
    lib.insert(AnimationTag::new("run", 2, 3)).expect("run");
    let mut a = SpriteAnimator::new("idle");
    a.autoplay = true;
    let aviso = ph2d_ecs::SpriteCutNotice { grid: [4, 1] };
    w.entity_mut(m)
        .insert((lib, a, SpriteAnimStart("idle".into()), aviso));
    let inst = ts::instantiate(&mut sim, &r, m, None).expect("instancia");
    let (mut sc, mut mp) = instance_docs::empty_docs();
    let mut docs = instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    let dup =
        instantiate::duplicate_subtree(&mut sim, &r, m, &mut docs, [1.0, 0.0]).expect("copia");
    let inicio = |s: &SimWorld, e| s.world().get::<SpriteAnimStart>(e).map(|i| i.0.clone());
    for e in [inst, dup] {
        assert_eq!(inicio(&sim, e).as_deref(), Some("idle"));
        let n = sim.world().get::<ph2d_ecs::SpriteCutNotice>(e);
        assert_eq!(n.map(|n| n.grid), Some([4, 1]));
    }
    let ponte = ph2d_physics_ecs::PhysicsBridge::new();
    let mut eco = ph2d_app_components::instance_sync::MasterEcho::default();
    ts::pass(&mut sim, &r, &ponte, &mut eco);
    ts::pass(&mut sim, &r, &ponte, &mut eco);
    sim.world_mut()
        .entity_mut(m)
        .insert(SpriteAnimStart("run".into()));
    ts::pass(&mut sim, &r, &ponte, &mut eco);
    assert_eq!(inicio(&sim, inst).as_deref(), Some("run"), "o mestre manda");
}
