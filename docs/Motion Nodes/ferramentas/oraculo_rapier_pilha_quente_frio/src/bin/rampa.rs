// Uma bola numa rampa de 12°: o contacto ACHATADO (dois pontos a ±d) segura-a?
use rapier2d::prelude::*;
struct Achata(f32, bool);
impl PhysicsHooks for Achata {
    fn modify_solver_contacts(&self, ctx: &mut ContactModificationContext) {
        if ctx.solver_contacts.len() != 1 { return; }
        let n = *ctx.normal;
        let t = Vector::new(-n.y, n.x) * self.0;
        let mut a = ctx.solver_contacts[0];
        let mut b = a;
        a.anchor1 += t; a.anchor2 += t; b.anchor1 -= t; b.anchor2 -= t;
        ctx.solver_contacts[0] = a;
        if self.1 { ctx.solver_contacts.push(b); } else {
            // um ponto só, à frente do rolar (o lado para onde a bola desce)
            ctx.solver_contacts[0] = b;
        }
    }
}
fn corre(mu_r: f32, reciclar: bool, dois: bool, seg: f32) -> (f32, f32) {
    let r = 0.2_f32;
    let ang = 12.0_f32.to_radians();
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let n = Vector::new(-ang.sin(), ang.cos());
    colliders.insert(ColliderBuilder::halfspace(rapier2d::na::Unit::new_unchecked(n)).friction(1.0).build());
    let h = bodies.insert(RigidBodyBuilder::dynamic().translation(n * r).can_sleep(false).build());
    let ganchos = if mu_r > 0.0 { ActiveHooks::MODIFY_SOLVER_CONTACTS } else { ActiveHooks::empty() };
    colliders.insert_with_parent(ColliderBuilder::ball(r).friction(1.0).active_hooks(ganchos).build(), h, &mut bodies);
    let mut p = IntegrationParameters::default();
    p.contact_recycling = reciclar;
    p.num_internal_stabilization_iterations = 4;
    let (mut pipe, mut isl, mut br, mut na, mut j, mut mj, mut ccd) = (PhysicsPipeline::new(), IslandManager::new(), DefaultBroadPhase::new(), NarrowPhase::new(), ImpulseJointSet::new(), MultibodyJointSet::new(), CCDSolver::new());
    let hooks = Achata(mu_r * r, dois);
    let p0 = bodies[h].translation();
    for _ in 0..(seg * 60.0) as usize {
        pipe.step(Vector::new(0.0, -4.0), &p, &mut isl, &mut br, &mut na, &mut bodies, &mut colliders, &mut j, &mut mj, &mut ccd, &hooks, &());
    }
    ((bodies[h].translation() - p0).length(), bodies[h].angvel())
}
fn main() {
    for mu in [0.0_f32, 0.1, 0.25, 0.5, 0.75] {
        for rec in [true, false] {
            for dois in [true, false] {
                let (d, w) = corre(mu, rec, dois, 2.0);
                println!("mu_r {mu:.2} reciclar {rec:5} {} : andou {d:.4} em 2 s, w {w:+.3}", if dois { "dois pontos" } else { "um ponto  " });
            }
        }
    }
}
