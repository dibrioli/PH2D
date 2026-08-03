//! **O push POR-TICK dos parâmetros de um joint** — o irmão exato do
//! [`super::kinematic`], na outra metade da mesma frase.
//!
//! O `drive_kinematic` existe porque a pose de um corpo cinemático é uma
//! **entrada por tick**, e o replay tinha de aprender a pedi-la ([`SceneAtTick`]).
//! Um parâmetro de joint KEYFRAMADO é a mesma coisa dita de outro número: a
//! velocidade de um guincho ou o alvo de um servo passam a ser função do tick, e
//! não do estado autorado em repouso.
//!
//! ⚠️ **Sem este passe a wave seria verde e ERRADA, e o mecanismo é o mesmo que
//! a auditoria do W4b nomeou nos corpos cinemáticos:** o
//! [`super::joints::reconcile_joints`] roda uma vez por DISPATCH, e os laços de
//! play e de replay dão N passos dentro dele. Um `motor_target` keyframado
//! chegaria ao solver **uma vez por quadro** (e não por tick), com o valor do
//! quadro aplicado a todos os ticks devidos — e, num replay, **nunca**, porque
//! ali o reconcile não roda de todo. Play e scrub responderiam números
//! diferentes para o mesmo tick, que é precisamente o invariante em que a ponte
//! inteira se apoia.
//!
//! ⚠️ **E ele NÃO reconstrói o joint** — chama
//! [`ph2d_physics::PhysicsWorld::retune_joint`], que sobrescreve o `data` do
//! `ImpulseJoint` sem tocar a arena. Remover-e-inserir escreveria o mesmo
//! número e **limparia o ring de checkpoints em todo tique de play**, matando o
//! scrub bit-exato do W1.5 pelo resto da cena.

use ph2d_ecs::SimWorld;

use super::PhysicsBridge;
use crate::joint::PhysicsJoint;

impl PhysicsBridge {
    /// Empurrar ao solver os parâmetros que o documento diz que este tick tem.
    ///
    /// `force` ignora o memo e reescreve todos — o que um **seed do ring**
    /// exige: depois dele o solver segura os parâmetros do CHECKPOINT, enquanto
    /// o memo (`JointRef::rest`) segue descrevendo o que a cena autorou. Sem o
    /// force, a comparação diria *"nada mudou"* e o replay correria com os
    /// números de um tick que não é este.
    ///
    /// A geometria vem do `rest` VIVO (âncoras e eixo) e não do componente: um
    /// param não move âncora, e re-derivá-la aqui seria uma segunda resposta a
    /// *"onde este joint prende?"* — a pergunta que o `anchored` do
    /// W-AnchorFollow já respondeu uma vez.
    ///
    /// ⚠️ **Uma troca de TIPO não passa por aqui.** Ela muda a política de
    /// âncora (`shares_a_point`), e reafinar com as âncoras antigas prenderia o
    /// tipo novo no lugar do velho; o `reconcile_joints` do próximo quadro faz
    /// isso direito, com a conversão inteira.
    pub(super) fn drive_joint_params(&mut self, sim: &SimWorld, force: bool) {
        if self.joints.is_empty() {
            return;
        }
        let world = sim.world();
        // ⚠️ A CHAVE do mapa é a entidade-JOINT; `JointRef::entities` são os dois
        // CORPOS. Ler o componente do corpo A daria `None` em toda cena sã e o
        // passe inteiro seria um no-op silencioso.
        for (&e, j) in self.joints.iter_mut() {
            let Some(pj) = world.get::<PhysicsJoint>(e) else {
                continue;
            };
            let Some(want) = super::joint_desc::joint_desc(
                &pj.clamped(),
                j.rest.anchor_a,
                j.rest.anchor_b,
                (j.rest.axis_a, j.rest.axis_b),
            ) else {
                continue;
            };
            if want.kind != j.rest.kind {
                continue;
            }
            if !force && want == j.rest {
                continue;
            }
            if self.world.retune_joint(j.handle, &want) {
                j.rest = want;
            }
        }
    }
}
