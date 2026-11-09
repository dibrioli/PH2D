//! **W0 do Outline (plano f2): o custo de UMA forma de N peças convexas no motor real.**
//!
//! A forma «Outline» vira, no rapier, UM collider compound de peças convexas (as que
//! `ph2d_outline_trace::convex_pieces` deriva do alfa da imagem). O tecto de peças tem de dizer de
//! que recurso é (CLAUDE.md §0.0), então mede-se aqui o que cada peça custa:
//!
//! - `build`: montar o compound (o que um retraço paga: Detail chaveado, Fit to picture);
//! - `trace`: alfa → anéis → peças (a outra metade do retraço);
//! - `pousado`: um tique (`PhysicsWorld::step`, 4 sub-passos, os parâmetros do produto) com o corpo
//!   pousado num chão, TODAS as peças a tocar (um pente de dentes para baixo — o pior caso de
//!   contactos por peça);
//! - `par`: dois pentes empilhados (o de cima pousa nos dentes do de baixo, que pousa no chão).
//!
//! Base: a mesma cena com uma caixa (`Cuboid`) do mesmo tamanho. Método (`MEDIR_VELOCIDADE.md`):
//! as variantes intercaladas no MESMO processo, em rodadas, resumo = o MÍNIMO.
//!
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-physics --profile smoke --test it measure_outline_pieces -- --ignored --nocapture`

use ph2d_outline_trace::{Mask, Params, Q4, convex_pieces, trace};
use ph2d_physics::rmath::{Pose, Vector};
use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};
use rapier2d::geometry::SharedShape;
use std::time::Instant;

/// O Detail de fábrica (2 px).
const DETAIL_Q4: u16 = 2 * Q4 as u16;
/// 64 px por metro: um pente de 512 dentes tem 48 m — mais peças = arte maior, nunca dentes de 1 mm.
const M_POR_PX: f32 = 1.0 / 64.0;
const DENTES: [u32; 8] = [1, 4, 16, 32, 64, 128, 256, 512];
const RODADAS: usize = 7;
const ASSENTA: u32 = 90;
const MEDE: u32 = 120;

/// Um pente de `dentes` dentes para baixo: barra de 12 px, dentes de 3 px × 12 px, vão de 3 px.
fn pente(dentes: u32) -> Mask {
    let (dente, vao, barra, perna) = (3u32, 3u32, 12u32, 12u32);
    let w = dentes * (dente + vao) + vao;
    let h = barra + perna;
    let bits = (0..h)
        .flat_map(|y| (0..w).map(move |x| u8::from(y < barra || x % (dente + vao) >= vao)))
        .collect();
    Mask { w, h, bits }
}

/// As peças em metros, y para cima, centradas na caixa da imagem.
fn pecas_em_metros(m: &Mask, pecas: &[Vec<[i32; 2]>]) -> Vec<Vec<Vector>> {
    let (cx, cy) = (m.w as f32 / 2.0, m.h as f32 / 2.0);
    pecas
        .iter()
        .map(|p| {
            p.iter()
                .map(|&[x, y]| {
                    let (px, py) = (x as f32 / Q4 as f32, y as f32 / Q4 as f32);
                    Vector::new((px - cx) * M_POR_PX, -(py - cy) * M_POR_PX)
                })
                .collect()
        })
        .collect()
}

fn compound(pecas: &[Vec<Vector>]) -> SharedShape {
    let partes = pecas
        .iter()
        .map(|p| {
            (
                Pose::IDENTITY,
                SharedShape::convex_hull(p).expect("peça convexa"),
            )
        })
        .collect();
    SharedShape::compound(partes)
}

fn desc(body_type: RigidBodyType, x: f32, y: f32, half: [f32; 2]) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: half[0],
            half_y: half[1],
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        shape_pose: [0.0; 3],
        collide: [0; 2],
        shape_off: false,
        offset: [0.0, 0.0],
    }
}

fn veste(w: &mut PhysicsWorld, h: RigidBodyHandle, forma: &SharedShape) {
    let c = w.bodies().get(h).expect("corpo").colliders()[0];
    w.colliders_mut()
        .get_mut(c)
        .expect("collider")
        .set_shape(forma.clone());
}

/// O mundo da cena: o chão e `andares` corpos empilhados (1 = pousado, 2 = par); `forma` `None` = a caixa.
fn cena(
    meia: [f32; 2],
    forma: Option<&SharedShape>,
    andares: u32,
) -> (PhysicsWorld, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    let chao = [meia[0] + 2.0, 0.5];
    w.spawn_body(desc(RigidBodyType::Fixed, 0.0, -0.5, chao));
    let mut topo = None;
    for k in 0..andares {
        let y = meia[1] * (2 * k + 1) as f32 + 0.001 * (k + 1) as f32;
        let h = w.spawn_body(desc(RigidBodyType::Dynamic, 0.0, y, meia));
        if let Some(f) = forma {
            veste(&mut w, h, f);
        }
        topo = Some(h);
    }
    (w, topo.expect("um corpo"))
}

/// µs por tique, depois de assentar; e o y do corpo de cima (para ver que pousou).
fn tique(meia: [f32; 2], forma: Option<&SharedShape>, andares: u32) -> (f64, f32) {
    let (mut w, topo) = cena(meia, forma, andares);
    for _ in 0..ASSENTA {
        w.step();
    }
    let t = Instant::now();
    for _ in 0..MEDE {
        w.step();
    }
    let us = t.elapsed().as_secs_f64() * 1e6 / f64::from(MEDE);
    let y = w.bodies().get(topo).expect("corpo").translation().y;
    (us, y)
}

struct Caso {
    dentes: u32,
    mask: Mask,
    pecas: Vec<Vec<Vector>>,
    vertices: usize,
    meia: [f32; 2],
}

#[test]
#[ignore = "medição (W0 do Outline) — corra à mão com --profile smoke --ignored --nocapture"]
fn measure_outline_pieces() {
    let params = Params {
        detail_q4: DETAIL_Q4,
        min_island_px: 0,
    };
    let casos: Vec<Caso> = DENTES
        .iter()
        .map(|&dentes| {
            let mask = pente(dentes);
            let o = trace(&mask, params);
            let p = convex_pieces(&o, DETAIL_Q4);
            // Os pontos do CONTORNO (o que o tecto confere antes de partir em peças), não a soma das peças.
            let vertices = o.vertex_count();
            let pecas = pecas_em_metros(&mask, &p);
            let meia = [
                mask.w as f32 * M_POR_PX / 2.0,
                mask.h as f32 * M_POR_PX / 2.0,
            ];
            Caso {
                dentes,
                mask,
                pecas,
                vertices,
                meia,
            }
        })
        .collect();
    let n = casos.len();
    // [trace, build, caixa1, pente1, caixa2, pente2] por caso, o mínimo das rodadas.
    let mut min = vec![[f64::MAX; 6]; n];
    let mut ys = vec![[0.0f32; 4]; n];
    let mut aneis = vec![f64::MAX; n];
    for rodada in 0..RODADAS {
        // Rodadas alternam o sentido da varredura: nenhum caso corre sempre depois do mesmo.
        let ordem: Vec<usize> = if rodada % 2 == 0 {
            (0..n).collect()
        } else {
            (0..n).rev().collect()
        };
        for &i in &ordem {
            let c = &casos[i];
            let t = Instant::now();
            let o = trace(&c.mask, params);
            let so_aneis = t.elapsed().as_secs_f64() * 1e6;
            std::hint::black_box(convex_pieces(&o, DETAIL_Q4));
            let tr = t.elapsed().as_secs_f64() * 1e6;
            aneis[i] = aneis[i].min(so_aneis);
            let t = Instant::now();
            let forma = compound(&c.pecas);
            let bu = t.elapsed().as_secs_f64() * 1e6;
            let (cx1, y0) = tique(c.meia, None, 1);
            let (pe1, y1) = tique(c.meia, Some(&forma), 1);
            let (cx2, y2) = tique(c.meia, None, 2);
            let (pe2, y3) = tique(c.meia, Some(&forma), 2);
            let m = &mut min[i];
            for (k, v) in [tr, bu, cx1, pe1, cx2, pe2].into_iter().enumerate() {
                m[k] = m[k].min(v);
            }
            ys[i] = [y0, y1, y2, y3];
        }
    }
    println!(
        "dentes | peças | pontos do contorno | anéis µs | trace µs | build µs | caixa pousada | pente pousado | caixa par | pente par | µs/peça (par) | y topo (caixa·pente·caixa2·pente2) / esperado"
    );
    for (i, c) in casos.iter().enumerate() {
        let m = min[i];
        let pecas = c.pecas.len();
        let por_peca = (m[5] - m[4]) / pecas as f64;
        println!(
            "{:>4} | {:>5} | {:>6} | {:>9.1} | {:>9.1} | {:>8.1} | {:>8.1} | {:>8.1} | {:>8.1} | {:>8.1} | {:>6.2} | {:.3} {:.3} {:.3} {:.3} / {:.3} {:.3}",
            c.dentes,
            pecas,
            c.vertices,
            aneis[i],
            m[0],
            m[1],
            m[2],
            m[3],
            m[4],
            m[5],
            por_peca,
            ys[i][0],
            ys[i][1],
            ys[i][2],
            ys[i][3],
            c.meia[1],
            3.0 * c.meia[1],
        );
    }
}
