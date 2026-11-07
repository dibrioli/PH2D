//! **O ▶ é só prévia** ([ficha](../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md), decisão
//! delegada A2): com o Auto Key armado, o ▶ não grava chave de «qual animação toca», de «a tocar», nem da célula
//! que o rebobinar escreve. A LINHA da lista continua a gravar: é o gesto que escolhe, e o Auto Key grava tudo (dono, 19/10).
//!
//! O gesto só se conhece no bus, e o mundo escrito por ele é igual ao da linha: quem viu o ▶ deixa aqui o objecto
//! marcado, e o passe do Auto Key tira esses campos do diff enquanto a marca durar. Não é documento.

use ph2d_ecs::canal_componente as cc;
use ph2d_ecs::{FieldKey, Resource, World};
use ph2d_editor_core::AnimFieldEdit;
use std::collections::BTreeSet;

/// Os objectos cujo tocador está como o ▶ o deixou.
#[derive(Resource, Default, Debug)]
pub struct PreviaDoPlay(BTreeSet<u64>);

/// Lê o gesto, antes de ele se aplicar: `Playing` é o ▶ e o ⏸; `SetCurrent` sozinho é a linha, e tira a marca.
/// O ▶ noutra linha manda os dois por esta ordem, e fica marcado.
pub fn nota_o_gesto(world: &mut World, entity_bits: u64, edit: &AnimFieldEdit) {
    match edit {
        AnimFieldEdit::Playing(_) => {
            world
                .get_resource_or_init::<PreviaDoPlay>()
                .0
                .insert(entity_bits);
        }
        AnimFieldEdit::SetCurrent(_) => {
            if let Some(mut p) = world.get_resource_mut::<PreviaDoPlay>() {
                p.0.remove(&entity_bits);
            }
        }
        _ => {}
    }
}

/// Os marcados agora: uma cópia para o estado do passe do Auto Key.
#[must_use]
pub fn marcados(world: &World) -> BTreeSet<u64> {
    world
        .get_resource::<PreviaDoPlay>()
        .map_or_else(BTreeSet::new, |p| p.0.clone())
}

/// `k` é um dos campos que o ▶ escreve: qual toca, se toca, e a célula do rebobinar.
#[must_use]
pub fn e_campo_da_previa(k: FieldKey) -> bool {
    cc::resolver(k).is_some_and(|(d, f, _)| match d.canonical_name {
        "ph2d::ecs::SpriteAnimator" => matches!(f.stored_key(), "current" | "playing"),
        "ph2d::ecs::SpriteGrid" => f.stored_key() == "frame",
        _ => false,
    })
}
