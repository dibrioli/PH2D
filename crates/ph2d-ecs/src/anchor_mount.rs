//! **O CONSUMIDOR de uma âncora** — o que faz de um socket algo mais que autoria.
//!
//! O [ADR-0072] unificou socket · slice · image point num tipo só e, até 2026-08-22, **nada no
//! app lia uma âncora**: o artista marcava a boca da arma, via a cruz no canvas, e não havia
//! forma de prender coisa nenhuma ali. *Autoria sem consumidor é a metade barata do problema* —
//! e é a metade que parece pronta.
//!
//! # A tese: uma âncora é um QUADRO na hierarquia
//!
//! Uma entidade que monta numa âncora **já é filha** da entidade que a possui. O que o
//! [`AnchorMount`] diz é apenas *qual* quadro do pai serve de origem: em vez da pose do pai, a
//! pose do pai **composta com a da âncora**. O `Transform` local do filho continua a ser o dele,
//! relativo a esse quadro — exatamente o que o socket do Paper 2D faz.
//!
//! Isto compra três coisas de graça, e é por isso que é este o desenho:
//!
//! 1. **Ordem.** A propagação já visita o pai antes do filho, então a âncora está resolvida
//!    quando o filho a pede. Um vínculo *entre árvores* (por nome, ao estilo do
//!    `RemoteTransform2D` do Godot) exigiria ordenação topológica, um segundo passe e deteção de
//!    ciclos — e um ciclo aqui é impossível por construção, porque a hierarquia já o proíbe.
//! 2. **Netos.** Quem monta pode ter filhos próprios, e eles herdam sem uma linha de código.
//! 3. **Undo e save.** O quadro é **derivado**; nada aqui escreve `Transform`. A lei do
//!    [ADR-0153] — *o passe publica onde as coisas ficam, não escreve onde elas estão* — é o que
//!    impede cada quadro de um gesto de virar um passo de undo.
//!
//! ⚠️ **A alternativa que empurra a pose foi recusada por esta razão**, não por gosto: um sistema
//! que escrevesse o `Transform` do filho a cada quadro registaria um passo de undo por quadro, e
//! a segunda porta para «onde está o filho» divergiria da primeira no dia em que alguém compusesse
//! numa e atribuísse cru na outra (`docs/Physics/BUGS_physics.md` #2, medido a um offset de pai
//! inteiro).
//!
//! # ⚠️ AS DUAS travessias têm de perguntar a MESMA coisa
//!
//! Este repositório responde «onde está esta entidade?» por dois caminhos, de propósito:
//! [`crate::propagate_transforms`] (DFS de cima para baixo, por quadro, para o renderer) e
//! [`crate::world_transform`] (subida pela cadeia, sob demanda, para gizmos, pick e física). O
//! doc de [`crate::transform_inverse`] já diz porquê e o que custa: *duas respostas para «onde
//! está esta entidade» é precisamente o bug que esta família não para de produzir.*
//!
//! Um quadro de âncora injetado **só numa** delas seria essa família outra vez, e do pior tipo:
//! a espada **desenharia** na mão e todo gesto — clicar, arrastar, colidir — leria-a na origem do
//! pai. Por isso a lei mora aqui, numa função só ([`mount_state`]), e as duas travessias chamam-na.
//! O gate `the_two_walks_agree_about_a_mounted_child` prende-as.
//!
//! # ⚠️ «A âncora não existe» é um ESTADO, não uma ausência
//!
//! Renomear uma âncora, ou apagá-la, deixa quem a montava a apontar para um nome que já não está
//! lá. Cair em silêncio na origem do pai é a resposta certa **no desenho** (o filho fica visível,
//! perto de onde estava) e a errada **na comunicação**: o artista vê a espada saltar e não tem o
//! que ler. Por isso [`MountState`] tem três valores e não dois — [`MountState::Dangling`] é um
//! facto que a UI mostra, e não um `None` que se confunde com «esta entidade não monta em nada».
//!
//! [ADR-0072]: ../../../docs/architecture/decisions/0072-named-anchor-unification.md
//! [ADR-0153]: ../../../docs/architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};

use crate::named_anchor::{NamedAnchor, NamedAnchorList};
use crate::transform::Transform;
use crate::{ChildOf, SimComponent};

/// **Esta entidade monta numa âncora do seu PAI.**
///
/// O campo é o NOME da âncora, e é o nome de propósito: é o mesmo identificador que o artista
/// escreve, que o Aseprite importa e que o script pede. ⚠️ Guardar um índice tornaria o vínculo
/// dependente da ORDEM da lista — apagar a âncora `0` faria toda a gente descer uma casa em
/// silêncio —, e guardar os bits da entidade seria pior ainda: *o undo respawna tudo com bits
/// novos*, e bits dentro dos bytes de um componente envenenam o próprio undo.
///
/// Um nome vazio é [`MountState::Free`], não um erro: é o estado em que o componente existe e
/// não monta em nada, que é o que a UI escreve quando o artista escolhe «—».
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnchorMount {
    /// O nome da âncora do pai. Vazio = não monta.
    pub anchor: String,
}

impl AnchorMount {
    /// Monta na âncora com este nome.
    pub fn new(anchor: impl Into<String>) -> Self {
        Self {
            anchor: anchor.into(),
        }
    }

    /// O estado «existe o componente, mas não monta em nada».
    pub fn free() -> Self {
        Self::default()
    }

    /// `true` quando este vínculo aponta para algum nome.
    pub fn is_bound(&self) -> bool {
        !self.anchor.is_empty()
    }
}

impl SimComponent for AnchorMount {}

/// O que a montagem desta entidade É — os três estados, nomeados.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MountState {
    /// Não monta em âncora nenhuma: o quadro do pai é a pose do pai, como sempre foi.
    Free,
    /// Monta, e a âncora existe. O `Transform` é o quadro EXTRA, **local ao pai**.
    Mounted(Transform),
    /// Monta num nome que o pai não tem — renomeado, apagado, ou o pai mudou.
    ///
    /// ⚠️ Geometricamente comporta-se como [`Self::Free`] (o filho fica no pai, não salta para a
    /// origem do mundo). O que o distingue é que ele **se pode mostrar**, e é por isso que existe.
    Dangling,
}

impl MountState {
    /// O quadro extra a compor, ou nada. `Dangling` e `Free` respondem igual **de propósito**.
    #[inline]
    pub fn frame(self) -> Option<Transform> {
        match self {
            Self::Mounted(t) => Some(t),
            Self::Free | Self::Dangling => None,
        }
    }
}

/// **A LEI.** O quadro que o `parent` entrega a este `child`.
///
/// É a única função que sabe ler um vínculo, e as duas travessias de mundo chamam-na.
///
/// ⚠️ `parent` é passado, e não derivado de `child`, porque quem chama já o tem: a propagação
/// está a visitá-lo, e a subida acabou de o ler. Voltar a perguntar ao mundo custaria uma leitura
/// por nó no caminho quente e — pior — permitiria que os dois lados discordassem sobre quem é o
/// pai a meio de uma reparentação.
#[must_use]
pub fn mount_state(world: &World, parent: Entity, child: Entity) -> MountState {
    let Some(mount) = world.get::<AnchorMount>(child) else {
        return MountState::Free;
    };
    if !mount.is_bound() {
        return MountState::Free;
    }
    let Some(list) = world.get::<NamedAnchorList>(parent) else {
        return MountState::Dangling;
    };
    match list.get(&mount.anchor) {
        Some(a) => MountState::Mounted(a.transform),
        None => MountState::Dangling,
    }
}

/// [`mount_state`] reduzido ao quadro — a forma que as travessias usam.
#[inline]
#[must_use]
pub fn mount_frame(world: &World, parent: Entity, child: Entity) -> Option<Transform> {
    mount_state(world, parent, child).frame()
}

/// O estado de montagem desta entidade em relação ao pai que ela de facto tem.
///
/// Conveniência para a UI, que tem a entidade e não o pai. `Free` também quando não há pai —
/// uma raiz não monta em coisa nenhuma.
#[must_use]
pub fn mount_state_of(world: &World, child: Entity) -> MountState {
    match world.get::<ChildOf>(child).map(|c| c.parent()) {
        Some(p) => mount_state(world, p, child),
        None => {
            // Sem pai, um vínculo escrito não pode resolver-se — e dizer `Free` esconderia que
            // ele lá está. Um sprite arrastado para fora do pai mantém o componente.
            match world.get::<AnchorMount>(child) {
                Some(m) if m.is_bound() => MountState::Dangling,
                _ => MountState::Free,
            }
        }
    }
}

/// **A pose de uma âncora, sob a pose do dono.** A lei PURA — sem mundo, sem hierarquia.
///
/// ⚠️ Tudo o que precise de saber «onde está esta âncora» passa por aqui: a montagem
/// ([`mount_state`] compõe o mesmo quadro), o desenho da cruz no canvas, as alças do gizmo e a
/// API de runtime. É uma linha de álgebra, e é exatamente por ser uma linha que ela se
/// reimplementa sem ninguém reparar — e aí a alça agarra num sítio e a espada aparece noutro.
#[inline]
#[must_use]
pub fn anchor_pose_under(owner_world: Transform, anchor: &NamedAnchor) -> Transform {
    Transform::compose(owner_world, anchor.transform)
}

/// **A API de runtime do ADR-0072 §2.6, em Rust:** a pose de MUNDO de uma âncora nomeada.
///
/// ```ignore
/// let muzzle = anchor_world_pose(world, entity, "muzzle")?;
/// spawn_bullet(muzzle.translation, muzzle.rotation);
/// ```
///
/// `None` quando a entidade não é posicionável, não tem âncoras, ou não tem esta.
///
/// ⚠️ **A rotação e a escala vêm juntas, e é isso que faz a bala sair na direção certa** quando o
/// personagem está virado. Uma API que devolvesse só a posição obrigaria cada consumidor a
/// recompor a orientação a partir do pai — e cada um recomporia à sua maneira.
#[must_use]
pub fn anchor_world_pose(world: &World, entity: Entity, name: &str) -> Option<Transform> {
    let owner_world = crate::world_transform(world, entity)?;
    let anchor = world.get::<NamedAnchorList>(entity)?.get(name)?;
    Some(anchor_pose_under(owner_world, anchor))
}

/// Os nomes das âncoras desta entidade — o `sprite_anchor_list` do ADR §2.6.
///
/// Devolve um vetor porque quem chama são as bordas do sistema (script, MCP, painel), nunca o
/// caminho quente; a leitura por-quadro é [`mount_state`], que não aloca.
#[must_use]
pub fn anchor_names(world: &World, entity: Entity) -> Vec<String> {
    world
        .get::<NamedAnchorList>(entity)
        .map(|l| l.iter().map(|a| a.name.clone()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;

    fn anchored(world: &mut World, at: Vec2, name: &str) -> Entity {
        let mut list = NamedAnchorList::new();
        let mut a = NamedAnchor::socket(name);
        a.transform.translation = at;
        list.insert(a).unwrap();
        world.spawn((Transform::IDENTITY, list)).id()
    }

    /// Os três estados são distinguíveis, e o que distingue `Dangling` de `Free` **não** é a
    /// geometria: é haver o que dizer ao artista.
    #[test]
    fn the_three_states_are_told_apart() {
        let mut w = World::new();
        let parent = anchored(&mut w, Vec2::new(1.0, 0.0), "muzzle");

        let free = w.spawn((Transform::IDENTITY, ChildOf(parent))).id();
        assert_eq!(mount_state(&w, parent, free), MountState::Free);

        let bound = w
            .spawn((
                Transform::IDENTITY,
                ChildOf(parent),
                AnchorMount::new("muzzle"),
            ))
            .id();
        assert!(matches!(
            mount_state(&w, parent, bound),
            MountState::Mounted(_)
        ));

        let lost = w
            .spawn((
                Transform::IDENTITY,
                ChildOf(parent),
                AnchorMount::new("hand_r"),
            ))
            .id();
        assert_eq!(mount_state(&w, parent, lost), MountState::Dangling);
        assert_eq!(
            mount_state(&w, parent, lost).frame(),
            None,
            "geometricamente um vinculo perdido tem de se comportar como livre"
        );
    }

    /// Um nome vazio é «não monta», e não um nome que nenhuma âncora tem.
    ///
    /// ⚠️ A diferença é visível: se o vazio fosse `Dangling`, escolher «—» no painel acenderia
    /// um aviso de erro sobre o estado que o artista acabou de pedir.
    #[test]
    fn the_empty_name_is_free_not_dangling() {
        let mut w = World::new();
        let parent = anchored(&mut w, Vec2::ZERO, "muzzle");
        let c = w
            .spawn((Transform::IDENTITY, ChildOf(parent), AnchorMount::free()))
            .id();
        assert_eq!(mount_state(&w, parent, c), MountState::Free);
    }

    /// Um pai SEM lista nenhuma responde `Dangling` a quem diz montar — não `Free`.
    #[test]
    fn a_parent_with_no_anchors_leaves_the_rider_dangling() {
        let mut w = World::new();
        let parent = w.spawn(Transform::IDENTITY).id();
        let c = w
            .spawn((
                Transform::IDENTITY,
                ChildOf(parent),
                AnchorMount::new("muzzle"),
            ))
            .id();
        assert_eq!(mount_state(&w, parent, c), MountState::Dangling);
    }

    /// `mount_state_of` acha o pai sozinho — e uma RAIZ com vínculo escrito fica `Dangling`,
    /// porque o componente lá está e não tem onde resolver.
    #[test]
    fn a_root_that_claims_a_mount_is_dangling_not_free() {
        let mut w = World::new();
        let orphan = w
            .spawn((Transform::IDENTITY, AnchorMount::new("muzzle")))
            .id();
        assert_eq!(mount_state_of(&w, orphan), MountState::Dangling);
        let plain = w.spawn(Transform::IDENTITY).id();
        assert_eq!(mount_state_of(&w, plain), MountState::Free);
    }

    /// **A API de runtime do §2.6** devolve pose de MUNDO — com a cadeia do dono dentro dela.
    #[test]
    fn the_runtime_api_answers_in_world_space_through_the_whole_chain() {
        let mut w = World::new();
        let root = w
            .spawn(Transform {
                translation: Vec2::new(10.0, 0.0),
                ..Transform::default()
            })
            .id();
        let mut list = NamedAnchorList::new();
        let mut a = NamedAnchor::socket("muzzle");
        a.transform.translation = Vec2::new(0.0, 3.0);
        list.insert(a).unwrap();
        let sprite = w
            .spawn((
                Transform {
                    translation: Vec2::new(0.0, 5.0),
                    ..Transform::default()
                },
                list,
                ChildOf(root),
            ))
            .id();

        let pose = anchor_world_pose(&w, sprite, "muzzle").expect("a ancora existe");
        assert_eq!(pose.translation, Vec2::new(10.0, 8.0));
        assert_eq!(anchor_world_pose(&w, sprite, "nope"), None);
        assert_eq!(anchor_world_pose(&w, root, "muzzle"), None, "o pai nao a tem");
        assert_eq!(anchor_names(&w, sprite), vec!["muzzle".to_string()]);
        assert!(anchor_names(&w, root).is_empty());
    }

    /// A pose da âncora **roda e escala com o dono** — é isso que faz a bala sair virada.
    #[test]
    fn the_anchor_pose_carries_the_owners_rotation_and_scale() {
        let mut w = World::new();
        let mut list = NamedAnchorList::new();
        let mut a = NamedAnchor::socket("muzzle");
        a.transform.translation = Vec2::new(2.0, 0.0);
        a.transform.rotation = 0.0;
        list.insert(a).unwrap();
        let e = w
            .spawn((
                Transform {
                    rotation: std::f32::consts::FRAC_PI_2,
                    scale: Vec2::new(3.0, 3.0),
                    ..Transform::default()
                },
                list,
            ))
            .id();
        let pose = anchor_world_pose(&w, e, "muzzle").unwrap();
        // 2 m para +X, rodados 90° e escalados 3× ⇒ 6 m para +Y.
        assert!((pose.translation.x - 0.0).abs() < 1e-5, "{pose:?}");
        assert!((pose.translation.y - 6.0).abs() < 1e-5, "{pose:?}");
        assert!((pose.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert_eq!(pose.scale, Vec2::new(3.0, 3.0));
    }
}
