//! **Uma corrente resolvida de uma vez** — os Pins marcados [`JointDesc::one_chain`] ganham um gémeo
//! de coordenadas reduzidas (`MultibodyJointSet`), crescido a partir de CADA corpo fixo que alcançam.
//!
//! Porquê (medido, `bridge_chain_probe.rs`, doc `docs/JointEditor/02_gabaritos_medidos.md`): o solver
//! iterativo não segura uma corrente leve sob uma carga pesada — a ponte de 8 tábuas com uma caixa
//! 200× mais pesada afunda 36 cm em Ropes e escorrega da caixa em Pins; resolvida a partir dos dois
//! postes (duas árvores que se encontram no meio por UM Pin de impulso) afunda 2 cm, e a 2000:1 segura
//! onde o impulso esmaga até ao chão. Uma árvore só, de um poste ao outro, explode (o fecho de impulso
//! contra a árvore inteira) — por isso a raiz é TODO corpo fixo, em largura, ao mesmo tempo.
//!
//! A lei das duas representações (`ik.rs`): o `ImpulseJoint` continua a ser o REGISTO — handle, âncoras,
//! limiares, o que o canvas desenha e o que o Inspector lê; o gémeo é DERIVADO dele (o mesmo
//! `GenericJoint`, copiado) e o impulso fica desligado enquanto o gémeo existe. O conjunto refaz-se
//! quando a estrutura muda (junta nasce ou morre, corpo sai); um retune copia o `data` para o gémeo.
//! A aresta que fecharia um laço fica de impulso — o rapier recusa ciclos na árvore.
//!
//! ⚠️ Refazer a árvore a meio do Play semeia as coordenadas pelas poses reais, mas zera a velocidade
//! generalizada dos elos (a cena perde o balanço que tinha). Na autoria, que é onde a estrutura muda,
//! as peças estão em repouso.

use super::PhysicsWorld;
use super::joint_desc::{JointDesc, JointKind};
use rapier2d::dynamics::{GenericJoint, ImpulseJointHandle, MultibodyJointHandle, RigidBodyHandle};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

type Key = (u32, u32);

fn joint_key(h: ImpulseJointHandle) -> Key {
    h.0.into_raw_parts()
}

fn body_key(h: RigidBodyHandle) -> Key {
    h.0.into_raw_parts()
}

/// O estado das correntes: quem pediu, quem tem gémeo, e se o conjunto precisa de ser refeito.
#[derive(Clone, Default)]
pub struct Chains {
    members: BTreeMap<Key, ImpulseJointHandle>,
    twins: BTreeMap<Key, MultibodyJointHandle>,
    dirty: bool,
}

/// **Este joint entra numa corrente?** Um Pin ligado, inquebrável e sem motor.
///
/// A quebra lê a reação do impulso, que o gémeo não tem; o motor do multibody não foi medido.
#[must_use]
pub fn takes_one_chain(desc: &JointDesc) -> bool {
    desc.one_chain
        && desc.kind == JointKind::Pin
        && desc.enabled
        && desc.motor.is_none()
        && desc.break_force.is_infinite()
        && desc.break_torque.is_infinite()
}

/// A coordenada (ângulo) de um Pin entre `parent` e `child`, dados os frames do joint.
fn pin_angle(data: &GenericJoint, rel: &crate::rmath::Pose) -> f32 {
    (data.local_frame1.inverse() * rel * data.local_frame2)
        .rotation
        .angle()
}

impl PhysicsWorld {
    /// O joint acabou de nascer ou de ser reafinado: entra ou sai da corrente.
    pub(super) fn chain_note(&mut self, handle: ImpulseJointHandle, desc: &JointDesc) {
        let k = joint_key(handle);
        let wants = takes_one_chain(desc);
        let was = self.chains.members.contains_key(&k);
        if wants != was {
            if wants {
                self.chains.members.insert(k, handle);
            } else {
                self.chains.members.remove(&k);
            }
            self.chains.dirty = true;
            return;
        }
        // Mesma pertença: o número novo vai ao gémeo, e o impulso continua desligado.
        if let Some(&mh) = self.chains.twins.get(&k)
            && let Some(live) = self.impulse_joints.get_mut(handle, true)
        {
            let mut data = live.data;
            live.data.set_enabled(false);
            data.set_enabled(true);
            if let Some((mb, link)) = self.multibody_joints.get_mut(mh)
                && let Some(l) = mb.link_mut(link)
            {
                let flipped = l.rigid_body_handle() != live.body2();
                if flipped {
                    data.flip();
                }
                l.joint.data = data;
            }
        }
    }

    /// O joint saiu (ou um corpo saiu e levou joints consigo): o conjunto refaz-se no próximo passo.
    pub(super) fn chain_forget(&mut self, handle: Option<ImpulseJointHandle>) {
        if let Some(h) = handle {
            let k = joint_key(h);
            if let Some(mh) = self.chains.twins.remove(&k) {
                self.multibody_joints.remove(mh, true);
            }
            self.chains.members.remove(&k);
        }
        if !self.chains.members.is_empty() || !self.chains.twins.is_empty() {
            self.chains.dirty = true;
        }
    }

    /// `true` quando o joint é resolvido pelo gémeo — está em vigor, embora o impulso esteja desligado.
    #[must_use]
    pub fn joint_is_in_one_chain(&self, handle: ImpulseJointHandle) -> bool {
        self.chains.twins.contains_key(&joint_key(handle))
    }

    /// Refaz as árvores, se a estrutura mudou. Chamado no início de cada passo.
    pub(super) fn refresh_chains(&mut self) {
        if !self.chains.dirty {
            return;
        }
        self.chains.dirty = false;
        for (k, mh) in std::mem::take(&mut self.chains.twins) {
            self.multibody_joints.remove(mh, true);
            if let Some(&ih) = self.chains.members.get(&k)
                && let Some(j) = self.impulse_joints.get_mut(ih, true)
            {
                j.data.set_enabled(true);
            }
        }
        let ij = &self.impulse_joints;
        self.chains.members.retain(|_, h| ij.get(*h).is_some());

        let edges: Vec<(Key, ImpulseJointHandle, RigidBodyHandle, RigidBodyHandle)> = self
            .chains
            .members
            .iter()
            .filter_map(|(k, &h)| ij.get(h).map(|j| (*k, h, j.body1(), j.body2())))
            .collect();
        let dynamic = |b: RigidBodyHandle| self.bodies.get(b).is_some_and(|rb| rb.is_dynamic());
        let mut roots: BTreeSet<Key> = BTreeSet::new();
        let mut handles: BTreeMap<Key, RigidBodyHandle> = BTreeMap::new();
        for &(_, _, a, b) in &edges {
            for x in [a, b] {
                handles.insert(body_key(x), x);
                if self.bodies.get(x).is_some() && !dynamic(x) {
                    roots.insert(body_key(x));
                }
            }
        }
        // Em largura, de todas as raízes ao mesmo tempo: cada tábua pendura-se no fixo mais perto.
        let mut seen: BTreeSet<Key> = roots.clone();
        let mut queue: VecDeque<RigidBodyHandle> = roots.iter().map(|k| handles[k]).collect();
        let mut used: BTreeSet<Key> = BTreeSet::new();
        let mut grown: Vec<(
            ImpulseJointHandle,
            MultibodyJointHandle,
            RigidBodyHandle,
            RigidBodyHandle,
        )> = Vec::new();
        while let Some(p) = queue.pop_front() {
            for &(k, ih, a, b) in &edges {
                if used.contains(&k) || (a != p && b != p) {
                    continue;
                }
                let o = if a == p { b } else { a };
                if seen.contains(&body_key(o)) || !dynamic(o) {
                    continue;
                }
                let Some(j) = self.impulse_joints.get(ih) else {
                    continue;
                };
                let mut data = j.data;
                data.set_enabled(true);
                if a != p {
                    data.flip();
                }
                if let Some(mh) = self.multibody_joints.insert(p, o, data, true) {
                    used.insert(k);
                    seen.insert(body_key(o));
                    queue.push_back(o);
                    grown.push((ih, mh, p, o));
                    self.chains.twins.insert(k, mh);
                }
            }
        }
        for &(ih, ..) in &grown {
            if let Some(j) = self.impulse_joints.get_mut(ih, true) {
                j.data.set_enabled(false);
            }
        }
        self.seed_chains(&grown);
    }

    /// As coordenadas novas dizem «ângulo zero»; põe-nas de acordo com as poses reais dos corpos.
    fn seed_chains(
        &mut self,
        grown: &[(
            ImpulseJointHandle,
            MultibodyJointHandle,
            RigidBodyHandle,
            RigidBodyHandle,
        )],
    ) {
        let mut per_body: BTreeMap<Key, (MultibodyJointHandle, f32)> = BTreeMap::new();
        for &(_, mh, p, o) in grown {
            let (Some(pp), Some(po)) = (self.bodies.get(p), self.bodies.get(o)) else {
                continue;
            };
            let rel = pp.position().inverse() * po.position();
            if let Some((mb, link)) = self.multibody_joints.get(mh)
                && let Some(l) = mb.link(link)
            {
                let want = pin_angle(&l.joint.data, &rel);
                let tree = pin_angle(&l.joint.data, l.local_to_parent());
                per_body.insert(body_key(o), (mh, want - tree));
            }
        }
        let mut done: BTreeSet<Key> = BTreeSet::new();
        for &(_, mh, ..) in grown {
            let Some((mb, _)) = self.multibody_joints.get_mut(mh) else {
                continue;
            };
            let root = body_key(mb.root().rigid_body_handle());
            if !done.insert(root) {
                continue;
            }
            let mut disp = vec![0.0; mb.ndofs()];
            let mut dof = 0usize;
            for l in mb.links() {
                let n = l.joint().ndofs();
                if n == 1
                    && let Some(&(_, mut d)) = per_body.get(&body_key(l.rigid_body_handle()))
                {
                    let tau = std::f32::consts::TAU;
                    while d > std::f32::consts::PI {
                        d -= tau;
                    }
                    while d <= -std::f32::consts::PI {
                        d += tau;
                    }
                    disp[dof] = d;
                }
                dof += n;
            }
            if disp.iter().any(|d| *d != 0.0) {
                mb.apply_displacements(&disp);
                mb.forward_kinematics(&self.bodies, false);
            }
        }
    }
}
