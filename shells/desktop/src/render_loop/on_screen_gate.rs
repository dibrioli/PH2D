//! ⭐⭐ **O `OnScreenEnabler` a fazer o que o rótulo promete** — *«só corre/aparece quando está
//! no ecrã»*.
//!
//! ⛔⛔ **Até 2026-08-30 o [`ph2d_ecs::OnScreenEnabler::contains`] tinha ZERO chamadores de
//! produção.** Os cinco campos da §8 do Inspector (a caixa + os quatro números do rect) gravavam no
//! `.ph2dproj`, viajavam no undo, apareciam no censo de componentes — e **nada** no app lhes
//! perguntava alguma coisa. É a segunda espécie de controlo morto da caça de 2026-08-30: *o fio
//! está completo até ao modelo, e o modelo não tem consumidor.*
//!
//! ⚠️ **A régua que faltava é a TERCEIRA pergunta da costura** — não *«o clique chega?»* (chegava:
//! `INSP_VIS_ON_SCREEN` tem braço no `inspector_visibility.rs`) nem *«o campo é escrito?»*
//! (era: `queue_insert` põe o componente), e sim ***«alguém DECIDE alguma coisa com o valor?»***.
//!
//! # As DUAS respostas, porque o `EnableMode` tem duas famílias
//!
//! | modo | o que este módulo faz | quem pergunta |
//! |---|---|---|
//! | `HideVisible` | a entidade (e a subárvore) **não emite instância** | [`hides`] ← `sim_extract` |
//! | `InheritPause` | o **processamento** dela e o dos descendentes pára | [`processing_paused`] ← `sprite_anim_tick` |
//! | `PauseProcessing` | o processamento **só dela** pára | idem |
//!
//! ⚠️ **A diferença entre os dois modos de pausa TEM de ser real.** O doc do
//! [`ph2d_ecs::EnableMode`] contrasta *«pause only this node's processing»* com o default herdado;
//! se os dois fizessem a mesma coisa, um deles seria exactamente o controlo morto que este módulo
//! existe para curar — um enum de três opções em que duas são a mesma. Por isso `InheritPause`
//! desce a subárvore e `PauseProcessing` pára na entidade que o carrega.
//!
//! # ⚠️ O ponto é o da PRÓPRIA entidade, sempre
//!
//! O rect é mundo (metros), e a pergunta é *«esta entidade saiu do rect DELA?»*. Numa caminhada de
//! ancestrais é o mundo do **ancestral** contra o rect do **ancestral** — nunca o ponto do filho
//! contra o rect do pai, que responderia a outra pergunta (*«o filho saiu do rect do pai?»*) e
//! faria um pai enorme pausar filhos que estão dentro dele.
//!
//! # Custo
//!
//! Sem nenhum `OnScreenEnabler` na cena, [`hides`] é **uma falha de componente por entidade** mais
//! a subida de `ChildOf` que a maioria das entidades (raízes) nem começa — e o quadro é
//! byte-idêntico ao de antes desta cura. A conversão de pose (`world_transform`) só corre quando
//! um ancestral de facto carrega o componente.

use ph2d_ecs::{ChildOf, EnableMode, Entity, OnScreenEnabler, World};

/// Teto de profundidade da caminhada de ancestrais — defesa contra um save corrompido, não limite
/// de produto. O mesmo número do [`crate::vec_entities::MAX_DEPTH`], pela mesma razão.
const MAX_DEPTH: usize = 64;

/// **A pergunta atómica:** o enabler DESTA entidade recusa-a agora?
///
/// `None` quando ela não tem enabler (nunca recusa) ou quando o ponto está dentro do rect.
/// `Some(mode)` diz **como** recusar — quem decide o que fazer com isso é o chamador.
fn refuses(world: &World, entity: Entity, pos: [f32; 2]) -> Option<EnableMode> {
    let e = world.get::<OnScreenEnabler>(entity)?;
    (!e.contains(pos[0], pos[1])).then_some(e.mode)
}

/// [`refuses`] para quem **não** tem a pose em mãos — resolve-a a partir do mundo.
///
/// ⛔⛔ **A ORDEM das duas perguntas é o custo.** O `world_transform` sobe a hierarquia inteira até
/// à raiz: perguntá-lo ANTES de saber se a entidade sequer tem o componente faria a caminhada de
/// ancestrais custar `O(profundidade²)` **por sprite e por quadro**, numa cena onde nenhum
/// `OnScreenEnabler` existe. O `get` do componente é uma falha de tabela; ele vem primeiro, e o
/// caminho comum paga exactamente o que as outras caminhadas deste ficheiro já pagam
/// (`cascade_tint_with_ancestors`, `resolve_clip_grouping`): uma falha de componente por nível.
fn refuses_here(world: &World, entity: Entity) -> Option<EnableMode> {
    let e = *world.get::<OnScreenEnabler>(entity)?;
    let t = ph2d_ecs::world_transform(world, entity)?;
    (!e.contains(t.translation.x, t.translation.y)).then_some(e.mode)
}

/// ⭐ **«Esta entidade está escondida por um enabler?»** — a metade de DESENHO.
///
/// `own_pos` é a pose de mundo que o extract já calculou (o `GlobalTransform` do quadro): pedi-la
/// em vez de a recalcular mantém a resposta exactamente igual à que o desenho usa, e é de graça.
///
/// ⚠️ **A subárvore vai junto**, e está no doc do [`ph2d_ecs::EnableMode::HideVisible`]:
/// *«Make the node (and subtree) invisible off-screen»*. Só esse modo esconde — um modo de pausa
/// que também escondesse tornaria os três indistinguíveis para quem olha a tela.
#[must_use]
pub(crate) fn hides(world: &World, entity: Entity, own_pos: [f32; 2]) -> bool {
    if refuses(world, entity, own_pos) == Some(EnableMode::HideVisible) {
        return true;
    }
    let mut cur = world.get::<ChildOf>(entity).map(|c| c.parent());
    for _ in 0..MAX_DEPTH {
        let Some(a) = cur else { return false };
        if refuses_here(world, a) == Some(EnableMode::HideVisible) {
            return true;
        }
        cur = world.get::<ChildOf>(a).map(|c| c.parent());
    }
    false
}

/// ⭐ **«O processamento desta entidade está pausado por um enabler?»** — a metade de CORRER.
///
/// Hoje o consumidor é o tique da animação de sprite ([`super::sprite_anim_tick`]), que é o único
/// comportamento por-entidade e por-quadro que o artista autora nesta shell. ⚠️ **A física NÃO
/// entra**: o solver é global e vive noutra crate (`ph2d-physics-ecs`); pausá-la por-corpo é outra
/// decisão, com outro dono.
///
/// - o enabler DELA pausa, em qualquer dos dois modos;
/// - o de um ANCESTRAL pausa só em `InheritPause` — é isso que *«inherit»* quer dizer, e é a única
///   coisa que separa os dois modos de pausa um do outro.
#[must_use]
pub(crate) fn processing_paused(world: &World, entity: Entity) -> bool {
    if matches!(
        refuses_here(world, entity),
        Some(EnableMode::InheritPause | EnableMode::PauseProcessing)
    ) {
        return true;
    }
    let mut cur = world.get::<ChildOf>(entity).map(|c| c.parent());
    for _ in 0..MAX_DEPTH {
        let Some(a) = cur else { return false };
        if refuses_here(world, a) == Some(EnableMode::InheritPause) {
            return true;
        }
        cur = world.get::<ChildOf>(a).map(|c| c.parent());
    }
    false
}

#[cfg(test)]
#[path = "on_screen_gate_tests.rs"]
mod tests;
