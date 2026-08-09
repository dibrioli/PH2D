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
