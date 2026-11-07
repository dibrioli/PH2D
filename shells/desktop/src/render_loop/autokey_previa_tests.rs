//! ⭐ **O ▶ é só prévia** ([ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md), A2): o quadro
//! real (dreno, curva, gesto pela porta do Inspector, publicar, Auto Key armado) sobre um tocador de verdade.

use super::*;
use ph2d_app_sprite_anim::{inspector::apply_anim_edit, previa, tick};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{AnimationTag, SimWorld, SpriteAnimations, SpriteAnimator, SpriteGrid};
use ph2d_editor_core::AnimFieldEdit as Ed;

struct Cena {
    sim: SimWorld,
    reg: ComponentRegistry,
    e: u64,
    st: TimelineState,
    ph: Playhead,
    ak: AutokeyState,
}

fn cena() -> Cena {
    let mut sim = SimWorld::default();
    let mut lib = SpriteAnimations::new();
    for (n, de) in [("idle", 0), ("run", 2), ("jump", 4)] {
        lib.insert(AnimationTag::new(n, de, de + 1)).expect("tag");
    }
    let mut a = SpriteAnimator::new("idle");
    a.autoplay = true;
    let grade = SpriteGrid {
        hframes: 6,
        vframes: 1,
        frame: 0,
    };
    let e = sim.world_mut().spawn((grade, lib, a)).id().to_bits();
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let mut st = TimelineState::new();
    st.keys_mode = true;
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    ph.seek(0.5);
    let ak = AutokeyState::default();
    #[rustfmt::skip]
    let mut c = Cena { sim, reg, e, st, ph, ak };
    gesto(&mut c, &[Ed::StartsWith("idle".into())]);
    gesto(&mut c, &[]);
    c
}

/// Um quadro inteiro do Auto Key armado, com o gesto (`edits`) pela porta do Inspector.
fn gesto(c: &mut Cena, edits: &[Ed]) {
    let solo = c.st.keys_mode;
    super::super::timeline_bridge::run(
        c.sim.world_mut(),
        &mut c.st,
        &mut c.ph,
        &mut Vec::new(),
        &[],
        &mut c.ak,
        solo,
        None,
        &mut super::super::timeline_bridge::SignalEmitter::default(),
        &mut ph2d_preview_drive::PreviewDrive::default(),
    );
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    let _ = ph2d_app_components::canais_componente::aplicar(&mut c.sim, &c.reg, &mut drive);
    for edit in edits {
        let q = EditorCommandQueue::new();
        previa::nota_o_gesto(c.sim.world_mut(), c.e, edit);
        let _ = apply_anim_edit(&c.sim, c.e, edit, &q, &c.reg);
        apply_editor_commands(c.sim.world_mut(), &q, &c.reg).expect("o commit aplica");
    }
    ph2d_app_components::canais_componente::publicar(&mut c.sim, &c.reg, &[c.e]);
    let samples = vec![(c.e, sample_pose(c.sim.world(), c.e))];
    c.ak.campos = samples
        .iter()
        .map(|&(e, _)| (e, campos_lidos(&c.sim, e)))
        .collect();
    c.ak.previa = previa::marcados(c.sim.world());
    apply_samples(
        &mut c.st,
        &c.ph,
        &samples,
        false,
        true,
        false,
        &mut c.ak,
        &mut ph2d_editor_core::ToastQueue::new(),
    );
}

fn chaves(c: &Cena) -> usize {
    let clip = c.st.doc.active_clip();
    c.st.doc
        .bindings()
        .iter()
        .filter_map(|b| clip.track(b.target))
        .map(|t| t.keys().len())
        .sum()
}

fn toca(c: &Cena) -> String {
    let e = ph2d_ecs::Entity::from_bits(c.e);
    let a = c.sim.world().get::<SpriteAnimator>(e).expect("tocador");
    a.current.clone()
}

/// ⭐ A prova: início `idle`, Auto Key armado, ▶ em `run`: zero chaves, e o Play entra em `idle`.
/// *Mutação: tirar o filtro de `e_campo_da_previa` em `diff_each_sample` ⇒ nascem as chaves de `current` e `playing`.*
#[test]
fn o_play_armado_nao_grava_chave_e_o_jogo_entra_no_inicio() {
    let mut c = cena();
    gesto(&mut c, &[Ed::SetCurrent("run".into()), Ed::Playing(true)]);
    gesto(&mut c, &[]);
    assert_eq!(toca(&c), "run", "a prévia toca");
    assert_eq!(chaves(&c), 0);
    gesto(&mut c, &[Ed::Playing(false)]);
    gesto(&mut c, &[]);
    assert_eq!(chaves(&c), 0, "nem a pausa");
    tick::start_autoplay_animations(&mut c.sim);
    assert_eq!(toca(&c), "idle");
}

/// A linha continua a gravar qual animação toca, mesmo depois de um ▶: é o gesto que escolhe.
/// *Mutação: não tirar a marca no `SetCurrent` ⇒ zero chaves.*
#[test]
fn a_linha_armada_grava_chave_de_qual_toca() {
    let mut c = cena();
    gesto(&mut c, &[Ed::SetCurrent("run".into()), Ed::Playing(false)]);
    gesto(&mut c, &[]);
    assert_eq!(chaves(&c), 0);
    gesto(&mut c, &[Ed::SetCurrent("jump".into())]);
    gesto(&mut c, &[]);
    assert!(chaves(&c) >= 1, "a chave de `current`");
    assert_eq!(toca(&c), "jump");
}

/// ⭐ A lei que a decisão fixa: a chave do artista vence o «Starts with» enquanto existir a curva. O Play entra no
/// início, e o quadro seguinte da curva escreve o que a chave diz. O ▶ por cima da curva não grava e segura-se.
#[test]
fn a_chave_do_artista_vence_o_starts_with() {
    let mut c = cena();
    gesto(&mut c, &[Ed::SetCurrent("run".into())]);
    gesto(&mut c, &[]);
    let n = chaves(&c);
    assert!(n >= 1);
    tick::start_autoplay_animations(&mut c.sim);
    assert_eq!(toca(&c), "idle", "o Play entra no início");
    c.ph.play();
    gesto(&mut c, &[]);
    assert_eq!(toca(&c), "run", "a curva manda");
    assert_eq!(chaves(&c), n);
    c.ph.pause();
    gesto(&mut c, &[]);

    gesto(&mut c, &[Ed::SetCurrent("jump".into()), Ed::Playing(true)]);
    gesto(&mut c, &[]);
    gesto(&mut c, &[]);
    assert_eq!(chaves(&c), n, "o ▶ sobre a curva não grava");
    assert_eq!(toca(&c), "jump", "a prévia segura-se, como uma pose posada");
}
