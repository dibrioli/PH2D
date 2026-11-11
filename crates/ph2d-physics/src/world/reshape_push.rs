//! **A forma que cresce encostada a um rígido não o atravessa** (GEF02 onda 8 item 2).
//!
//! Reafinar a forma de um corpo vivo (`copia_colisor`) com contactos num chão, parede ou corpo
//! cinemático afundava-a: a altura que cresce em volta do centro mete a metade de baixo no chão, e
//! a mola do solver só a tira ao longo de vários tiques. Medido (caixote 1 → 2 m, os testes de
//! `ph2d-physics-ecs/tests/it/forma_animada.rs`): 0,30 cm numa rampa
//! de 120 tiques, 1,54 cm em 20, 5,0 cm em 5. A cura translada o corpo, no tique que reafina, pelo
//! que a forma nova afunda A MAIS do que a velha em cada rígido tocado: exacto no rígido, sem
//! velocidade injectada (como o pivô na base, que já dava 0), e a mola (`solver_params.rs`) fica.
//!
//! O vizinho dinâmico empurrado para cima ou para o lado vai junto (um nível; a pilha medida: o
//! topo afundava 45 cm e saltava 12 cm numa rampa de 5 tiques). Dois rígidos opostos a pedir
//! (espremido) não se resolvem por translação, e o corpo fica onde está.

use rapier2d::dynamics::{LockedAxes, RigidBodyHandle};
use rapier2d::geometry::{Collider, ColliderHandle, SharedShape};
use rapier2d::parry::query;

use super::PhysicsWorld;
use crate::rmath::{Pose, Real, Vector};

/// O alcance da consulta da forma VELHA: um contacto previsto (folga) conta como afundar 0.
const ALCANCE_M: Real = 0.1;
/// Abaixo disto o afundar a mais é o ruído do solver, não a forma (o repouso medido é 0,002 cm).
const AFUNDA_MIN_M: Real = 1e-6;
/// Duas normais mais opostas que isto pedem o impossível (espremido entre dois rígidos).
const OPOSTAS: Real = -0.5;
/// O vizinho solto só é empurrado se a normal não aponta para baixo mais do que isto (contra a
/// gravidade); abaixo disto ele segura a forma como um chão (medido: empurrá-lo metia-o 8,4 cm no
/// chão; o solver sozinho, 2,3 cm).
const ABAIXO: Real = -0.1;
/// Passes de Gauss-Seidel sobre os pedidos (um basta com uma normal; dois ortogonais, um também).
const PASSES: usize = 4;

impl PhysicsWorld {
    /// Depois de `ch` vestir a forma nova (`velha`/`pos_velha` = a forma e a pose de mundo de
    /// antes; `rel` = a pose nova relativa ao corpo), afasta o corpo dinâmico dos rígidos que a
    /// forma nova afunda a mais, e depois os vizinhos dinâmicos que ela empurra para cima ou para o
    /// lado (um nível: a caixa em cima sobe com a de baixo). O vizinho dinâmico POR BAIXO segura
    /// como um rígido: quem sobe é a forma, e ele não vai ao chão.
    pub(super) fn afasta_dos_rigidos(
        &mut self,
        ch: ColliderHandle,
        velha: &SharedShape,
        pos_velha: Pose,
        rel: Pose,
    ) {
        let Some(c) = self.colliders.get(ch) else {
            return;
        };
        if c.is_sensor() || !c.is_enabled() {
            return;
        }
        let Some(bh) = c.parent() else {
            return;
        };
        let Some(b) = self.bodies.get(bh) else {
            return;
        };
        if !b.is_dynamic() {
            return;
        }
        let pos_nova = *b.position() * rel;
        let nova = c.shared_shape().clone();
        let a_mais = |o: &Collider, pos: &Pose| -> Option<(Vector, Real)> {
            let k = query::contact(pos, &*nova, o.position(), o.shape(), 0.0).ok()??;
            let antes = query::contact(&pos_velha, &**velha, o.position(), o.shape(), ALCANCE_M)
                .ok()
                .flatten()
                .map_or(0.0, |v| v.dist.min(0.0));
            let d = antes - k.dist;
            (d > AFUNDA_MIN_M).then_some((k.normal1, d))
        };
        let cima = -self.gravity.normalize_or_zero();
        let mut rigidos: Vec<(Vector, Real)> = Vec::new();
        let mut soltos: Vec<(RigidBodyHandle, ColliderHandle)> = Vec::new();
        for par in self.narrow_phase.contact_pairs_with(ch) {
            if !par.has_any_active_contact() {
                continue;
            }
            let oh = if par.collider1 == ch {
                par.collider2
            } else {
                par.collider1
            };
            let Some(o) = self.colliders.get(oh) else {
                continue;
            };
            if o.is_sensor() || !o.is_enabled() {
                continue;
            }
            let Some((n, d)) = a_mais(o, &pos_nova) else {
                continue;
            };
            match o
                .parent()
                .filter(|p| self.bodies.get(*p).is_some_and(|ob| ob.is_dynamic()))
            {
                Some(p) if n.dot(cima) >= ABAIXO => soltos.push((p, oh)),
                _ => rigidos.push((-n, d)),
            }
        }
        let s = translacao(&rigidos).unwrap_or(Vector::ZERO);
        let s = self.translada(bh, s);
        let pos_final = Pose::from_translation(s) * pos_nova;
        let mut pedidos: Vec<(RigidBodyHandle, Vector, Real)> = Vec::new();
        for (p, oh) in soltos {
            let Some(o) = self.colliders.get(oh) else {
                continue;
            };
            if let Some((n, d)) = a_mais(o, &pos_final)
                && n.dot(cima) >= ABAIXO
            {
                pedidos.push((p, n, d));
            }
        }
        let mut corpos: Vec<RigidBodyHandle> = Vec::new();
        for q in &pedidos {
            if !corpos.contains(&q.0) {
                corpos.push(q.0);
            }
        }
        for p in corpos {
            let deste: Vec<(Vector, Real)> = pedidos
                .iter()
                .filter(|q| q.0 == p)
                .map(|q| (q.1, q.2))
                .collect();
            if let Some(t) = translacao(&deste) {
                self.translada(p, t);
            }
        }
    }

    /// Translada o corpo `h` por `s` nos eixos que ele tem livres; devolve o que andou.
    fn translada(&mut self, h: RigidBodyHandle, s: Vector) -> Vector {
        let Some(b) = self.bodies.get_mut(h) else {
            return Vector::ZERO;
        };
        let lock = b.locked_axes();
        let s = Vector::new(
            if lock.contains(LockedAxes::TRANSLATION_LOCKED_X) {
                0.0
            } else {
                s.x
            },
            if lock.contains(LockedAxes::TRANSLATION_LOCKED_Y) {
                0.0
            } else {
                s.y
            },
        );
        if s != Vector::ZERO {
            let t = b.translation() + s;
            b.set_translation(t, true);
        }
        s
    }
}

/// A translação mínima que satisfaz `s·n ≥ d` em cada pedido (Gauss-Seidel); `None` sem pedidos ou
/// com dois opostos.
fn translacao(pedidos: &[(Vector, Real)]) -> Option<Vector> {
    if pedidos.is_empty() {
        return None;
    }
    for (i, (a, _)) in pedidos.iter().enumerate() {
        if pedidos[i + 1..].iter().any(|(b, _)| a.dot(*b) < OPOSTAS) {
            return None;
        }
    }
    let mut s = Vector::ZERO;
    for _ in 0..PASSES {
        for &(n, d) in pedidos {
            let falta = d - s.dot(n);
            if falta > 0.0 {
                s += n * falta;
            }
        }
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_translacao_satisfaz_cada_pedido_e_recusa_o_espremido() {
        let chao = (Vector::new(0.0, 1.0), 0.02);
        let parede = (Vector::new(-1.0, 0.0), 0.01);
        let s = translacao(&[chao, parede]).expect("dois ortogonais");
        assert!((s - Vector::new(-0.01, 0.02)).length() < 1e-7, "{s:?}");
        assert_eq!(translacao(&[chao, (Vector::new(0.0, -1.0), 0.02)]), None);
        assert_eq!(translacao(&[]), None);
    }
}
