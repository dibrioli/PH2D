//! A porta pública `ph2d_app_sprite_anim::inspector::commit_anim_edit` vista de fora: mora aqui porque a catraca
//! `the_shell_only_shrinks_and_every_app_crate_too` conta `src/` e `tests/` da `ph2d-app-sprite-anim` (no tecto).

use ph2d_app_sprite_anim::inspector::{Commit, commit_anim_edit};
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, register_ecs_components};
use ph2d_ecs::{SimWorld, SpriteAnimations, SpriteGrid, Transform};
use ph2d_editor_core::AnimFieldEdit;

/// ⭐ **Um Add que pegou é FEITO**, mesmo que escolher a nova falhe depois (handoff GEF 10/11, ABERTO): o cartão não pode
/// mostrar erro de uma animação que está lá. Um defeito de registo (o nome do tocador com outro tipo; um nome que o
/// registo não conhece a fila salta-o em silêncio) deixa o Add gravar e o `SetCurrent` falhar.
/// *Mutação: o fim de `commit_anim_edit` devolver o resultado do `SetCurrent` cru ⇒ `Falhou`.*
#[test]
fn o_add_que_pegou_e_feito_mesmo_sem_escolher_a_nova() {
    let mut sim = SimWorld::new();
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    let grade = SpriteGrid {
        hframes: 8,
        vframes: 1,
        frame: 0,
        sheet: 0,
    };
    let sprite = ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]);
    let e = sim
        .world_mut()
        .spawn((Transform::default(), sprite, grade))
        .id();
    let queue = EditorCommandQueue::new();
    assert!(matches!(
        commit_anim_edit(
            &mut sim,
            e.to_bits(),
            &AnimFieldEdit::AddPlayer,
            &queue,
            &reg
        ),
        Commit::Feito
    ));
    let mut so_a_lista = ComponentRegistry::new();
    so_a_lista.register_default::<SpriteAnimations>("ph2d::ecs::SpriteAnimations");
    // Um `bool` não se lê do 1.º byte do tocador (o comprimento do nome da nova).
    so_a_lista.register_default::<ph2d_ecs::ZAsRelative>("ph2d::ecs::SpriteAnimator");
    let queue = EditorCommandQueue::new();
    let c = commit_anim_edit(
        &mut sim,
        e.to_bits(),
        &AnimFieldEdit::Add,
        &queue,
        &so_a_lista,
    );
    let Commit::FeitoComAviso(t) = c else {
        panic!("o Add pegou e escolher a nova falhou: {c:?}");
    };
    let lista = sim
        .world()
        .get::<SpriteAnimations>(e)
        .map(|l| l.iter().count());
    assert_eq!(lista, Some(1), "a nova está lá");
    let diz = ph2d_i18n::tr("anim_editor.new_not_selected");
    let antes = diz.split('{').next().unwrap_or_default();
    assert!(
        t.message.starts_with(antes),
        "o aviso diz o que falhou: {}",
        t.message
    );
}
