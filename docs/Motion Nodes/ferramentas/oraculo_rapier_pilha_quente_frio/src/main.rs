// Oráculo 2 (doc 121 §9.20): o mundo do rapier PERSISTENTE ("quente") contra RECONSTRUÍDO a cada tique ("frio").
// args: k passos iter modo(0 quente, 1 frio) segundos
use rapier2d::prelude::*;
use std::time::Instant;

struct Corpo {
    p: Vector,
    a: f32,
    v: Vector,
    w: f32,
}

struct Mundo {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad: DefaultBroadPhase,
    narrow: NarrowPhase,
    joints: ImpulseJointSet,
    multi: MultibodyJointSet,
    ccd: CCDSolver,
    handles: Vec<RigidBodyHandle>,
}

fn monta(corpos: &[Corpo], lado: f32, taca: &[Vector]) -> Mundo {
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    if !taca.is_empty() { colliders.insert(
        ColliderBuilder::polyline(taca.to_vec(), None)
            .friction(0.6)
            .friction_combine_rule(CoefficientCombineRule::GeometricMean)
            .build(),
    ); }
    let mut handles = Vec::with_capacity(corpos.len());
    for c in corpos {
        let rb = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(c.p)
                .rotation(c.a)
                .linvel(c.v)
                .angvel(c.w)
                .can_sleep(false)
                .build(),
        );
        colliders.insert_with_parent(
            ColliderBuilder::cuboid(lado, lado)
                .friction(0.5)
                .friction_combine_rule(CoefficientCombineRule::GeometricMean)
                .build(),
            rb,
            &mut bodies,
        );
        handles.push(rb);
    }
    Mundo {
        bodies,
        colliders,
        pipeline: PhysicsPipeline::new(),
        islands: IslandManager::new(),
        broad: DefaultBroadPhase::new(),
        narrow: NarrowPhase::new(),
        joints: ImpulseJointSet::new(),
        multi: MultibodyJointSet::new(),
        ccd: CCDSolver::new(),
        handles,
    }
}

fn le(m: &Mundo) -> Vec<Corpo> {
    m.handles
        .iter()
        .map(|h| {
            let b = &m.bodies[*h];
            Corpo { p: b.translation(), a: b.rotation().angle(), v: b.linvel(), w: b.angvel() }
        })
        .collect()
}

fn main() {
    let arg = |i: usize, d: f32| std::env::args().nth(i).and_then(|v| v.parse().ok()).unwrap_or(d);
    let k = arg(1, 32.0) as usize;
    let passos = arg(2, 1.0) as usize;
    let iter = arg(3, 4.0) as usize;
    let modo = arg(4, 0.0) as usize;
    let frio = modo == 1;
    let copia = modo == 2;
    let mut guardado = Vec::new();
    let mut tcopia: Vec<f64> = Vec::new();
    let segundos = arg(5, 3.0);
    let fora = arg(6, 0.0) > 0.5;
    let (gap, lado, altura, taca_y) = (0.32_f32, 0.11_f32, -0.35_f32, -1.2_f32);
    let raio = k as f32 * gap * 0.75 + (altura - taca_y) + 0.2;
    let n = 512;
    let taca: Vec<Vector> = (0..=n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            Vector::new(raio * a.cos(), taca_y + raio * a.sin())
        })
        .collect();
    let meio = (k as f32 - 1.0) * 0.5;
    let corpos: Vec<Corpo> = (0..k * k)
        .map(|i| {
            let (c, r) = ((i % k) as f32, (i / k) as f32);
            Corpo {
                p: Vector::new((c - meio) * gap, altura + (r - meio) * gap),
                a: 0.0,
                v: Vector::ZERO,
                w: 0.0,
            }
        })
        .collect();
    let taca_mundo: Vec<Vector> = if fora { Vec::new() } else { taca.clone() };
    let mut m = monta(&corpos, lado, &taca_mundo);
    let mut params = IntegrationParameters::default();
    params.dt = 1.0 / 60.0 / passos as f32;
    params.num_solver_iterations = iter;
    params.num_internal_stabilization_iterations = arg(7, 1.0) as usize;
    let g = Vector::new(0.0, -4.0);
    let mut t = Vec::new();
    let tiques = (segundos * 60.0) as usize;
    for _ in 0..tiques {
        let t0 = Instant::now();
        if frio {
            let c = le(&m);
            m = monta(&c, lado, &taca_mundo);
        }
        if copia {
            let tc = Instant::now();
            guardado.push((m.bodies.clone(), m.colliders.clone(), m.islands.clone(), m.broad.clone(), m.narrow.clone()));
            if guardado.len() > 4 { guardado.remove(0); }
            tcopia.push(tc.elapsed().as_secs_f64() * 1e3);
        }
        for _ in 0..passos {
            m.pipeline.step(
                g, &params, &mut m.islands, &mut m.broad, &mut m.narrow, &mut m.bodies, &mut m.colliders,
                &mut m.joints, &mut m.multi, &mut m.ccd, &(), &(),
            );
            if fora {
                projecta(&mut m, lado, raio, taca_y);
            }
        }
        t.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let c = le(&m);
    // As réguas da `prova_dos_impulsos_da_placa`: vizinho mediano, velocidade média, sobreposição mais funda (centros
    // mais próximos que o lado, em % do lado — a mesma aproximação para todas as variantes).
    let ps: Vec<[f32; 2]> = c.iter().map(|b| [b.p.x, b.p.y]).collect();
    let mut viz: Vec<f32> = ps
        .iter()
        .enumerate()
        .map(|(i, a)| {
            ps.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                .fold(f32::INFINITY, f32::min)
        })
        .collect();
    viz.sort_by(f32::total_cmp);
    let vel = c.iter().map(|b| b.v.length()).sum::<f32>() / c.len() as f32;
    let fundo = viz.first().copied().unwrap_or(0.0);
    let mut todos = t.clone();
    todos.sort_by(f64::total_cmp);
    let ymin = ps.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    let bits: u64 = ps.iter().fold(0xcbf29ce484222325_u64, |h, p| {
        (h ^ u64::from(p[0].to_bits()) ^ (u64::from(p[1].to_bits()) << 32)).wrapping_mul(0x100000001b3)
    });
    tcopia.sort_by(f64::total_cmp);
    if let Some(c) = tcopia.get(tcopia.len() / 2) { println!("  copia do mundo: med {c:.3} ms · max {:.3}", tcopia[tcopia.len()-1]); }
    println!(
        "{} caixas {} passo(s) {} it {} | tique med {:.2} p95 {:.2} max {:.2} ms | vizinho med {:.4} vel {:.3} par mais junto {:.4} ({:.0}% do lado) | y min {:.3} fundo {:.3} | bits {bits:016x}",
        k * k,
        passos,
        iter,
        if frio { "FRIO " } else { "QUENTE" },
        todos[todos.len() / 2],
        todos[todos.len() * 95 / 100],
        todos[todos.len() - 1],
        viz[viz.len() / 2],
        vel,
        fundo,
        fundo / (2.0 * lado) * 100.0,
        ymin,
        taca_y - raio
    );
}

/// A taça FORA do mundo: a projecção do `sim.collide` Bowl (suporte da caixa ao longo da normal, salto 0,05, atrito 0,6).
fn projecta(m: &mut Mundo, lado: f32, raio: f32, taca_y: f32) {
    for h in &m.handles {
        let b = &mut m.bodies[*h];
        let p = b.translation();
        let d = Vector::new(p.x, p.y - taca_y);
        let dist = d.length();
        if dist <= 1e-9 { continue; }
        let n = d / dist; // para fora
        let a = b.rotation().angle();
        let (e1, e2) = (Vector::new(a.cos(), a.sin()), Vector::new(-a.sin(), a.cos()));
        let sup = lado * (n.dot(e1).abs() + n.dot(e2).abs());
        let fundo = dist + sup - raio;
        if fundo > 0.0 {
            b.set_translation(p - n * fundo, false);
            let v = b.linvel();
            let vn = v.dot(n);
            if vn > 0.0 {
                let vt = v - n * vn;
                let corte = (0.6 * vn * 1.05).min(vt.length());
                let vt = if vt.length() > 1e-9 { vt - vt / vt.length() * corte } else { vt };
                b.set_linvel(vt - n * (0.05 * vn), false);
            }
        }
    }
}
