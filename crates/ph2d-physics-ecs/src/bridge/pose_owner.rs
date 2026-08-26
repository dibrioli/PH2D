//! **QUEM ESCREVE A POSE DESTE CORPO** — a pergunta feita UMA vez (W-KinMove).
//!
//! # ⚠️ Por que ela virou uma porta, e não mais um `if`
//!
//! Antes desta wave havia **dois** reclamantes e a resposta era um booleano no
//! próprio tipo: [`BodyKind::solver_owns_pose`], cujo doc já enuncia o
//! invariante — *"um corpo que os dois lados reclamassem teria a pose escrita
//! duas vezes por tique, e o que pousasse por último venceria em silêncio"*.
//!
//! O player cinemático é o **terceiro**: o corpo é `Kinematic` (a cena o
//! dirigiria) e a pose é escrita pela lei do player. Com três respostas, um
//! booleano não chega — e a alternativa de espalhar `&& !é_player` pelos
//! estágios seria a **enumeração** que apodrece no dia em que um quarto
//! reclamante existir.
//!
//! # ⚠️ E a MESMA resposta escolhe o [`Support`]
//!
//! É deliberado, e é o que impede o defeito que mataria a wave: a lei do player
//! tem de saber se há uma perna elástica a segurá-lo, e a pergunta *"há?"* é
//! exatamente *"quem escreve a pose?"*. Perguntadas em dois lugares, elas
//! discordariam num corpo em que o `PlayerMode` diz uma coisa e o `RigidBody`
//! diz outra — e um personagem com a lei de Snap num corpo dinâmico **cai
//! através do mundo**, sem que nada na tela diga por quê.
//!
//! ⚠️ **A discordância é possível e o modo de falha é o SEGURO:** com o corpo
//! `Dynamic` a resposta é [`PoseOwner::Solver`] ⇒ [`Support::Spring`] ⇒ o player
//! de sempre, funcionando. O `PlayerMode` só passa a valer quando o corpo é
//! cinemático — e o gesto da §14 escreve os dois por uma porta, então pela UI
//! isso nunca acontece.

use ph2d_ecs::{Entity, World};
use ph2d_platformer::Support;

use crate::components::{BodyKind, PlatformPlayer, PlayerMode};

/// As três respostas, e elas se excluem.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum PoseOwner {
    /// **O solver.** Um corpo dinâmico — a pose sai por `readback`.
    Solver,
    /// **A cena.** Uma curva da timeline, um arrasto de gizmo, um bake — a pose
    /// entra por `drive_kinematic`.
    Scene,
    /// **O PLAYER.** A lei cinemática escreve a pose, e ela sai por `readback`
    /// como a de um corpo dinâmico: o dono mudou, a direção do fluxo não.
    ///
    /// ⚠️ **O modo viaja DENTRO da resposta** (W-KinPure), e não ao lado dela:
    /// perguntar `world.get::<PlayerMode>` uma segunda vez seria uma leitura
    /// que **não passa pela reconciliação com o `BodyKind`** — um
    /// [`PlayerMode::Pure`] num corpo `Dynamic` silenciaria a 3ª lei de um
    /// player que a ponte está a tratar como dinâmico, que é precisamente a
    /// discordância que este módulo existe para tornar impossível.
    Player(PlayerMode),
}

impl PoseOwner {
    /// O corpo tem a pose lida do solver para o `Transform`?
    ///
    /// ⚠️ Verdadeiro para o player cinemático **de propósito**: a pose que sai é
    /// a que a lei acabou de escrever, e sem ela o sprite ficaria parado
    /// enquanto o collider anda — a forma exata de *"o personagem não se mexe"*
    /// com todos os números certos.
    pub(super) fn flows_out(self) -> bool {
        matches!(self, PoseOwner::Solver | PoseOwner::Player(_))
    }

    /// A cena aponta este corpo por tique?
    pub(super) fn driven_by_scene(self) -> bool {
        matches!(self, PoseOwner::Scene)
    }

    /// **A LEI escreve a própria pose?** — o caminho cinemático do player.
    pub(super) fn writes_own_pose(self) -> bool {
        matches!(self, PoseOwner::Player(_))
    }

    /// **O que segura o personagem** — ver o aviso do módulo.
    pub(super) fn support(self) -> Support {
        match self {
            PoseOwner::Player(_) => Support::Snap,
            _ => Support::Spring,
        }
    }

    /// **O mundo ouve este personagem?** (W-KinPure)
    ///
    /// ⚠️ Um corpo cuja pose sai do SOLVER responde SIM incondicionalmente — o
    /// player dinâmico transmite pela mola desde a W6, e um `PlayerMode` que o
    /// contradissesse estaria a descrever um corpo que a ponte não está a
    /// simular.
    pub(super) fn transmits(self) -> bool {
        match self {
            PoseOwner::Player(mode) => mode.transmits(),
            _ => true,
        }
    }
}

/// **O QUE A LEI DE FACTO LÊ DESTE PERSONAGEM** — o veredito do
/// [`pose_owner`], publicado para quem PINTA a §14.
///
/// # ⚠️ Por que isto nasce aqui e não na shell
///
/// Cada campo é a condição literal de um `if` do `drive_players`, e a shell
/// respondia às mesmas quatro perguntas por conta própria, a partir do
/// `PlayerMode` lido do mundo. Isso é a **segunda cópia** de uma resposta que
/// este módulo existe para dar uma vez — e ela já mentia num caso real: um
/// player **ASSADO** (`Kinematic` + [`PlatformPlayer`], sem `PlayerMode` ⇒
/// `PlayerMode::default()` é `Dynamic`) resolve para [`PoseOwner::Scene`], a
/// lei **não corre**, e o painel pintava os doze cards como se corresse.
///
/// ⚠️ **É um struct de bools, não um enum:** as quatro perguntas não são
/// mutuamente exclusivas (o modo `Pure` corre a lei, tem perna rígida e **não**
/// transmite), e um enum obrigaria o painel a re-derivar a combinação — que é
/// precisamente a segunda cópia que se está a remover.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PlayerLiveness {
    /// **A lei corre neste corpo?** — `drive_players` nem entra no laço quando
    /// a cena dirige a pose, e sai no `else` seguinte sem o componente.
    pub law_runs: bool,
    /// **A perna é uma MOLA?** — [`Support::Spring`]. Sob [`Support::Snap`] o
    /// `float_height` é sobrescrito pela geometria do corpo e os dois números da
    /// mola não são lidos por ninguém.
    pub spring: bool,
    /// **O mundo ouve este personagem?** — a 3ª lei inteira (o card `REACT`).
    pub transmits: bool,
    /// **O empurrão LATERAL é lido?** — só o laço dos `KinMove` o corre, e sob
    /// `Pure` a `ReactionConfig` já chega zerada.
    pub pushes: bool,
}

impl PlayerLiveness {
    /// **A lei NÃO corre** — sem componente, ou a pose é da cena.
    ///
    /// ⚠️ É o neutro que mantém a §14 OFERECIDA onde ela tem de estar: a face
    /// vazia (o botão *Make Platform Player*) é o gesto que faz o comportamento
    /// existir, e escondê-la aqui tornaria a física alcançável só onde já há
    /// física — a lição da §11 do W2a.
    pub const INERT: Self = Self {
        law_runs: false,
        spring: false,
        transmits: false,
        pushes: false,
    };
    /// O player DINÂMICO: perna elástica, transmite pelo solver, sem empurrão
    /// lateral próprio (ele já empurra pelo contato).
    pub const SPRING: Self = Self {
        law_runs: true,
        spring: true,
        transmits: true,
        pushes: false,
    };
    /// O player CINEMÁTICO: a perna pousa, e o empurrão lateral é a única via
    /// pela qual ele move um caixote.
    pub const SNAP: Self = Self {
        law_runs: true,
        spring: false,
        transmits: true,
        pushes: true,
    };
}

/// **A tradução, feita UMA vez** — cada linha nomeia o `if` que ela espelha.
pub(super) fn liveness(world: &World, entity: Entity, kind: BodyKind) -> PlayerLiveness {
    // `drive_players`: `let Some(cfg) = world.get::<PlatformPlayer>(…) else { continue }`.
    if world.get::<PlatformPlayer>(entity).is_none() {
        return PlayerLiveness::INERT;
    }
    let owner = pose_owner(world, entity, kind);
    // `drive_players`: `if owner.driven_by_scene() { continue }`.
    if owner.driven_by_scene() {
        return PlayerLiveness::INERT;
    }
    PlayerLiveness {
        law_runs: true,
        spring: owner.support() == Support::Spring,
        transmits: owner.transmits(),
        // ⚠️ A conjunção é a mesma do `PlayerMode::pushes_bodies`, e agora sai
        // do OWNER: o `writes_own_pose` é o que põe o corpo no laço dos
        // `KinMove` (o único que chama `push_from_hits`), e o `transmits` é o
        // que impede o `Pure` de o herdar.
        pushes: owner.writes_own_pose() && owner.transmits(),
    }
}

impl crate::PhysicsBridge {
    /// **O que a §14 pode oferecer sem mentir.**
    ///
    /// ⚠️ **O `kind` vem do corpo CONSTRUÍDO, com o autorado como recuo** — a
    /// mesma preferência do [`pose_owner`], e o recuo existe porque o painel
    /// pergunta no MESMO quadro em que o artista clica *Add*: sem ele a seção
    /// piscaria inerte por um quadro, no gesto que a acabou de criar.
    /// ⭐ **O DOCUMENTO pode escrever a pose desta entidade?** — a porta que a condição (b) da
    /// [refutação 1] exige, e a razão de ela ser UMA.
    ///
    /// O sync mestre→instância (ADR-0164 / F4.3) propaga o `Transform` de uma peça **exceto**
    /// quando o dono dela é o solver ou o player: escrever na célula que o runtime possui punha
    /// dois autores no mesmo campo, e por tique o readback marcaria a diferença como se o artista
    /// a tivesse feito — um override espúrio por quadro, ou um corpo teleportado.
    ///
    /// ⚠️ **Re-derivar isto do lado de fora seria a segunda resposta.** A pergunta *«quem escreve
    /// a pose?»* tem um dono ([`pose_owner`]) e ele lê o `kind` do corpo **CONSTRUÍDO** — um
    /// `RigidBody` acabado de anexar e ainda não reconciliado responde diferente do componente.
    ///
    /// Uma entidade sem corpo nenhum é do documento: não há reclamante.
    ///
    /// [refutação 1]: https://github.com/dibrioli/PH2D/blob/main/docs/Components/pesquisa/instancias_2026-08-21/refutacao_1_sync_determinismo.md
    #[must_use]
    pub fn document_owns_pose(&self, world: &World, entity: Entity) -> bool {
        let Some(kind) = self.bodies.get(&entity).map(|b| b.kind).or_else(|| {
            world
                .get::<crate::components::RigidBody>(entity)
                .map(|b| b.kind)
        }) else {
            return true;
        };
        pose_owner(world, entity, kind).driven_by_scene()
    }

    #[must_use]
    pub fn player_liveness(&self, world: &World, entity: Entity) -> PlayerLiveness {
        let Some(kind) = self.bodies.get(&entity).map(|b| b.kind).or_else(|| {
            world
                .get::<crate::components::RigidBody>(entity)
                .map(|b| b.kind)
        }) else {
            return PlayerLiveness::INERT;
        };
        liveness(world, entity, kind)
    }
}

/// A porta. `kind` vem do registro de corpos da ponte (o que de facto foi
/// construído no rapier), e não do componente — é o corpo que existe que
/// importa, não o que foi pedido.
pub(super) fn pose_owner(world: &World, entity: Entity, kind: BodyKind) -> PoseOwner {
    if kind.solver_owns_pose() {
        return PoseOwner::Solver;
    }
    if kind == BodyKind::Kinematic && world.get::<PlatformPlayer>(entity).is_some() {
        let mode = world.get::<PlayerMode>(entity).copied().unwrap_or_default();
        if mode.drives_itself() {
            return PoseOwner::Player(mode);
        }
    }
    PoseOwner::Scene
}

#[cfg(test)]
#[path = "pose_owner_tests.rs"]
mod tests;
