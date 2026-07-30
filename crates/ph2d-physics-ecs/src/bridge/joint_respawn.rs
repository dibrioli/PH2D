//! **Como um joint VOLTA depois que o mundo foi trocado** (a metade do rewind).
//!
//! Separado de [`super::joints`] pela mesma linha de corte que
//! [`super::rewind`] usa contra o passo normal: aquele arquivo responde *o que o
//! artista AUTOROU vira que restrição?* (o reconcile, todo dispatch), e este
//! responde *como isso é remontado quando o `PhysicsWorld` inteiro é
//! substituído?* — que acontece uma vez por rebuild e tem invariantes próprios.
//!
//! O corte é por RESPONSABILIDADE e não por tamanho: foi o cap de LOC que pediu
//! a hora, mas é aqui que a lição do Weston mora, e ela merecia arquivo.

use ph2d_ecs::Entity;

use super::PhysicsBridge;
use super::joints::JointRef;

impl PhysicsBridge {
    /// Re-attach every joint after the bodies have been rebuilt from their rest
    /// descriptions (`rebuild_from_rest`, which has just handed out fresh body
    /// handles). The joints have to come back **in the same call**, not on the
    /// next frame: the rewind replays the owed steps immediately, and a replay
    /// without the joints is a different simulation.
    pub(super) fn respawn_joints_from_rest(&mut self) {
        let existing: Vec<(Entity, JointRef)> = self.joints.iter().map(|(&e, &j)| (e, j)).collect();
        self.joints.clear();
        for (e, mut j) in existing {
            // Copy the handles out before touching `self.world` — and read them
            // from `self.bodies`, which the rebuild has already refreshed.
            let a = self.bodies.get(&j.entities.0).map(|r| r.handle);
            // ⚠️ **O lado B pode ser o MUNDO, e é aqui que a lição do Weston
            // morde:** `rebuild_from_rest` já TROCOU o `PhysicsWorld`, então a
            // âncora do mundo velho morreu com ele — recriá-la é obrigatório, e
            // no MESMO chamado, porque o replay roda logo abaixo. Foi assim que
            // a tabela de polias sumiu e *um rewind replayava sem as cordas*.
            //
            // ⚠️ Calado num **Reset** (`target == 0` replaya zero passos, então
            // o dispatch seguinte reconstrói tudo de qualquer jeito) — o gate
            // tem de scrubbar para um tique do MEIO, que é onde o replay corre.
            let b = match (j.entities.1, j.world_anchor) {
                (Some(eb), _) => self.bodies.get(&eb).map(|r| r.handle),
                (None, Some(p)) => Some(self.world.spawn_world_anchor(p)),
                (None, None) => None,
            };
            let (Some(a), Some(b)) = (a, b) else {
                continue;
            };
            j.bodies = (a, b);
            if let Some(handle) = self.world.spawn_joint(a, b, j.rest) {
                j.handle = handle;
                self.joints.insert(e, j);
            }
        }
    }
}
