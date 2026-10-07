//! ⭐ **O DESVIO na ponte da navegação** (plano 30, W5) — a metade do tique que vem DEPOIS da
//! condução de todos os agentes: [`ph2d_orca`] corrige cada direcção contra os outros corpos que andam
//! e contra as paredes da malha do raio de cada um, e só então se escreve a intenção do mover. Ver o
//! cabeçalho de [`super`]. ⚠️ Módulo FILHO de `nav.rs` (não irmão): lê os campos privados da
//! `NavWorld`, e saiu dele pelo tecto de LOC, nunca por fronteira de responsabilidade nova.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::Entity;
use ph2d_nav::V2;
use ph2d_physics::{BodyDesc, ShapeDesc};

use super::raio_que_envolve;
use crate::PlayerInput;
use crate::bridge::{BodyRef, PhysicsBridge};
use crate::components::BodyKind;
use ph2d_physics::PhysicsWorld;

/// ⭐ (W11) **As paredes de uma malha** como o desvio as lê: por mosaicos, e as montadas com a versão da
/// malha de onde vieram. Uma mudança refaz só os mosaicos cujas paredes mudaram — as mesmas
/// [`ph2d_orca::Walls`], ao bit, que `from_walkable_walls(m.verts(), m.walls())` (plano 30 §19).
#[derive(Default)]
pub(super) struct ParedesDaMalha {
    blocos: ph2d_orca::ParedesPorBlocos,
    montadas: ph2d_orca::Walls,
    versao: Option<u64>,
}

impl ParedesDaMalha {
    /// As paredes da malha de AGORA (refeitas se ela mudou desde a última vez).
    pub(super) fn paredes(&mut self, tm: &ph2d_navmesh::TiledMesh) -> &ph2d_orca::Walls {
        if self.versao != Some(tm.versao()) {
            self.montadas = self.monta(tm);
            self.versao = Some(tm.versao());
        }
        &self.montadas
    }

    /// As da última [`Self::paredes`].
    pub(super) fn montadas(&self) -> &ph2d_orca::Walls {
        &self.montadas
    }

    fn monta(&mut self, tm: &ph2d_navmesh::TiledMesh) -> ph2d_orca::Walls {
        let (m, faixas) = (tm.mesh(), tm.paredes_por_mosaico());
        let (verts, walls) = (m.verts(), m.walls());
        self.blocos
            .retem(|k| faixas.binary_search_by(|f| f.chave.cmp(&k)).is_ok());
        for f in faixas {
            self.blocos.poe(
                f.chave,
                f.lo,
                f.hi,
                walls[f.paredes.clone()]
                    .iter()
                    .map(|&(de, para)| (verts[de as usize], verts[para as usize])),
            );
        }
        self.blocos.monta()
    }
}

/// O que a condução pediu a um agente neste tique — a entrada do desvio.
pub(super) struct Pedida {
    pub(super) entity: Entity,
    pub(super) pos: V2,
    pub(super) dir: V2,
    pub(super) speed: f64,
    pub(super) raio: f64,
    pub(super) malha: Option<super::ChaveMalha>,
    pub(super) avoidance: bool,
    pub(super) alvo: Option<Entity>,
}

impl PhysicsBridge {
    /// ⭐ **O desvio** (ver o cabeçalho): da direcção que a condução pediu à intenção do mover.
    pub(super) fn desvia(&mut self, mut pedidas: Vec<Pedida>, dt: f64) {
        // A ordem da sequência é a das ENTIDADES (a da consulta do ECS é a das tabelas).
        pedidas.sort_by_key(|p| p.entity);
        let mut corpos: Vec<ph2d_orca::Agent> = Vec::with_capacity(pedidas.len());
        let mut indice: BTreeMap<Entity, u32> = BTreeMap::new();
        for p in &pedidas {
            indice.insert(p.entity, corpos.len() as u32);
            corpos.push(ph2d_orca::Agent {
                pos: p.pos,
                vel: self.velocidade_de(p.entity),
                pref: [p.dir[0] * p.speed, p.dir[1] * p.speed],
                radius: p.raio,
                max_speed: p.speed,
                avoids: p.avoidance,
                ignores: None,
            });
        }
        // Todo corpo SÓLIDO que anda e não é agente: um obstáculo que se move, que não desvia — pela
        // FORMA, em polígonos com velocidade (ver [`forma`]; a bola é um disco). ⚠️ O ALVO de alguém
        // fica um disco só: quem o persegue ignora-o por um índice, e um disco é o que esse índice nomeia.
        let mut moveis: Vec<ph2d_orca::Movel> = Vec::new();
        // Os mesmos polígonos, no mundo, com a folga de cada um — o que o CAMINHO contorna ([`contorna`]).
        let mut contornar: Vec<(Vec<V2>, f64)> = Vec::new();
        let alvos: BTreeSet<Entity> = pedidas.iter().filter_map(|p| p.alvo).collect();
        // (o aberto da W5) As PEÇAS de cada corpo (um filho só com `Collider`): fazem parte da forma.
        let mut pecas: BTreeMap<Entity, Vec<&super::super::parts::PartRef>> = BTreeMap::new();
        for p in self.parts.values().filter(|p| !p.rest.is_sensor) {
            pecas.entry(p.owner).or_default().push(p);
        }
        for (&e, b) in &self.bodies {
            // ⚠️ (report do dono, 05/10) Um PROJÉCTIL não é um obstáculo a contornar, é um golpe: com
            // ele no desvio os morcegos da arena ESQUIVAVAM-SE dos tiros do herói (o gate
            // `um_tiro_mata_um_morcego` sangrou quando o corpo deles cresceu para o quadrado desenhado).
            if b.kind == BodyKind::Static
                || b.rest.is_sensor
                || indice.contains_key(&e)
                || self.projectile_state.contains_key(&e)
            {
                continue;
            }
            let Some(pose) = self.world.body_pose(b.handle) else {
                continue;
            };
            let c = [f64::from(pose.translation.x), f64::from(pose.translation.y)];
            let vel = self.velocidade_de(e);
            let w = f64::from(self.world.body_angvel(b.handle).unwrap_or(0.0));
            // ⚠️ `libm`, nunca o `sin_cos` do `std`: a libc de cada SO muda o último ulp (o hash c9).
            let (sin, cos) = libm::sincos(f64::from(pose.rotation.angle()));
            indice.insert(e, corpos.len() as u32);
            let minhas = pecas.get(&e).map_or(&[][..], Vec::as_slice);
            let forma = if alvos.contains(&e) {
                // O disco que envolve o corpo E as peças, contado do centro do corpo.
                let r = minhas
                    .iter()
                    .fold(f64::from(raio_que_envolve(&b.rest)), |r, p| {
                        let [lx, ly, _] = p.local;
                        let (lx, ly) = (f64::from(lx), f64::from(ly));
                        let d = (lx * lx + ly * ly).sqrt();
                        r.max(d + f64::from(raio_que_envolve(&p.rest)))
                    });
                vec![([0.0, 0.0], r)]
            } else {
                // As formas do corpo e das peças, no referencial do corpo.
                let mut f = vec![forma(&b.rest)];
                for p in minhas {
                    // Da peça para o corpo (`local` = onde ela está).
                    let [lx, ly, lr] = p.local.map(f64::from);
                    let (s, c) = libm::sincos(lr);
                    let poe = |[x, y]: V2| [lx + c * x - s * y, ly + s * x + c * y];
                    f.push(match forma(&p.rest) {
                        Forma::Disco(o, r) => Forma::Disco(poe(o), r),
                        Forma::Poligono(pts) => Forma::Poligono(pts.into_iter().map(poe).collect()),
                    });
                }
                let mundo = |[lx, ly]: V2| [c[0] + cos * lx - sin * ly, c[1] + sin * lx + cos * ly];
                let mut poligonos: Vec<Vec<V2>> = Vec::new();
                let mut discos = Vec::new();
                for x in f {
                    match x {
                        Forma::Disco(o, r) => discos.push((o, r)),
                        Forma::Poligono(pts) => {
                            poligonos.push(pts.into_iter().map(mundo).collect())
                        }
                    }
                }
                for pts in &poligonos {
                    // A folga do desvio contra este corpo: o que o ponto mais rápido dele anda num
                    // horizonte (a mesma conta das linhas dos móveis em `ph2d_orca`).
                    let u = pts.iter().fold(0.0_f64, |m, q: &V2| {
                        let (rx, ry) = (q[0] - c[0], q[1] - c[1]);
                        let (vx, vy) = (vel[0] - w * ry, vel[1] + w * rx);
                        m.max((vx * vx + vy * vy).sqrt())
                    });
                    contornar.push((
                        pts.clone(),
                        u * ph2d_orca::Params::PRODUCT.time_horizon_walls,
                    ));
                }
                if !poligonos.is_empty() {
                    moveis.push(ph2d_orca::Movel {
                        walls: ph2d_orca::Walls::from_polygons(&poligonos),
                        vel,
                        centro: c,
                        omega: w,
                    });
                }
                discos
            };
            for ([lx, ly], raio) in forma {
                let r = [cos * lx - sin * ly, sin * lx + cos * ly];
                let v = [vel[0] - w * r[1], vel[1] + w * r[0]];
                corpos.push(ph2d_orca::Agent {
                    pos: [c[0] + r[0], c[1] + r[1]],
                    vel: v,
                    pref: v,
                    radius: raio,
                    max_speed: (v[0] * v[0] + v[1] * v[1]).sqrt(),
                    avoids: false,
                    ignores: None,
                });
            }
        }
        for (k, p) in pedidas.iter_mut().enumerate() {
            corpos[k].ignores = p.alvo.and_then(|a| indice.get(&a).copied());
            // ⭐ (plano 30 §25, B) O caminho até ao próximo canto atravessa um corpo que anda (a malha
            // não o tem): aponta à tangente dele, do lado escolhido.
            let canto = self
                .nav
                .agents
                .get(&p.entity)
                .and_then(|rt| rt.path.get(rt.next));
            if self.nav.sonda.contorno
                && p.avoidance
                && let Some(&g) = canto
            {
                let ate = ((g[0] - p.pos[0]).powi(2) + (g[1] - p.pos[1]).powi(2)).sqrt();
                if let Some(d) = contornar
                    .iter()
                    .find_map(|(pts, folga)| contorna(p.pos, p.dir, ate, p.raio + folga, pts))
                {
                    p.dir = d;
                    corpos[k].pref = [d[0] * p.speed, d[1] * p.speed];
                }
            }
        }
        for p in &pedidas {
            if let Some(chave) = p.malha
                && let Some(tm) = self.nav.meshes.get(&chave)
            {
                self.nav.walls.entry(chave).or_default().paredes(tm);
            }
        }
        let paredes: Vec<Option<&ph2d_orca::Walls>> = pedidas
            .iter()
            .map(|p| {
                p.malha
                    .and_then(|k| self.nav.walls.get(&k))
                    .map(ParedesDaMalha::montadas)
            })
            .collect();
        let mut multidao =
            ph2d_orca::Crowd::new(corpos, ph2d_orca::Params::PRODUCT).with_moving(moveis);
        let seguras =
            multidao.solve_all_why(|i| paredes.get(i).copied().flatten().map(|w| (w, 0.0)), dt);
        self.nav.avanco.clear();
        for (p, (v, outros)) in pedidas.iter().zip(&seguras) {
            // ⚠️ Só quando um OUTRO corpo cortou o pedido: sozinho, a quina também trava (medido).
            if *outros && p.avoidance && p.speed > 0.0 {
                let a = (v[0] * p.dir[0] + v[1] * p.dir[1]) / p.speed;
                self.nav.avanco.insert(p.entity, a as f32);
            }
            // ⚠️ A intenção é a velocidade em FRACÇÃO da máxima: o mover em modo livre passa-a
            // intacta (o comprimento incluído), logo um agente que trava para dar passagem anda
            // mesmo mais devagar. Um agente sem desvio leva a direcção da condução, ao bit.
            let dir = if p.avoidance && p.speed > 0.0 {
                [v[0] / p.speed, v[1] / p.speed]
            } else {
                p.dir
            };
            self.player_input.insert(
                p.entity,
                PlayerInput {
                    drive: dir[0] as f32,
                    drive_y: dir[1] as f32,
                    ..PlayerInput::default()
                },
            );
        }
    }

    /// A velocidade de AGORA de um corpo: a do mover de vista de cima se ele tiver um (é a que ele
    /// vai seguir: o comando MAIS o empurrão de um golpe, a mesma soma do mover), senão a do solver.
    fn velocidade_de(&self, e: Entity) -> V2 {
        if let Some(st) = self.topdown_state.get(&e) {
            let v = [
                st.velocity[0] + st.knockback[0],
                st.velocity[1] + st.knockback[1],
            ];
            return [f64::from(v[0]), f64::from(v[1])];
        }
        self.bodies
            .get(&e)
            .and_then(|b| self.world.body_velocity(b.handle))
            .map_or([0.0, 0.0], |v| [f64::from(v[0]), f64::from(v[1])])
    }
}

/// Uma forma como o desvio a vê, no referencial do corpo.
enum Forma {
    Disco(V2, f64),
    Poligono(Vec<V2>),
}

/// ⭐ (W14) **Um corpo que ANDA como o desvio o vê** (plano 30 §22.5): a bola é um disco (exacto); o
/// resto, um POLÍGONO anti-horário que o contém — a caixa exacta; a elipse, a cápsula e o estádio pelo
/// octógono circunscrito de cada ponta (só `sqrt`). ⛔ Medido e recusado: a fileira de discos da W6 ao
/// longo de uma forma comprida — os discos fazem uma serra, e um corpo largo que vem de frente PRENDIA
/// o agente (o Godot também: preso com discos, contorna com vértices).
fn forma(d: &BodyDesc) -> Forma {
    let o = [f64::from(d.offset[0]), f64::from(d.offset[1])];
    let (meia, ax, ay) = match d.shape {
        ShapeDesc::Ball { radius } => return Forma::Disco(o, f64::from(radius.abs())),
        ShapeDesc::Cuboid { half_x, half_y } => {
            let (x, y) = (f64::from(half_x.abs()), f64::from(half_y.abs()));
            return Forma::Poligono(
                [[-x, -y], [x, -y], [x, y], [-x, y]]
                    .map(|[px, py]| [o[0] + px, o[1] + py])
                    .to_vec(),
            );
        }
        ShapeDesc::Ellipse { rx, ry } => (0.0, rx, ry),
        ShapeDesc::Capsule {
            half_height,
            radius,
        } => (half_height, radius, radius),
        ShapeDesc::Stadium {
            half_height,
            rx,
            ry,
        } => (half_height, rx, ry),
    };
    let (h, ax, ay) = (
        f64::from(meia.abs()),
        f64::from(ax.abs()),
        f64::from(ay.abs()),
    );
    // O octógono circunscrito ao círculo unitário (`t = tan 22,5° = √2 − 1`), escalado por ponta e
    // esticado em `y` pela meia-altura: o casco das duas pontas.
    let t = 2.0_f64.sqrt() - 1.0;
    Forma::Poligono(
        [
            [1.0, -t],
            [1.0, t],
            [t, 1.0],
            [-t, 1.0],
            [-1.0, t],
            [-1.0, -t],
            [-t, -1.0],
            [t, -1.0],
        ]
        .map(|[x, y]| [o[0] + ax * x, o[1] + ay * y + h * y.signum()])
        .to_vec(),
    )
}

/// ⭐ (report do dono, 05/10) **A saída de um teletransporte está LIVRE para um corpo de raio `r`**:
/// nenhum corpo sólido que não é parede (um estático é parede da malha, e a saída está nela) a menos de
/// `r` dela, pela forma de cada um (o disco, ou o polígono do desvio). Medido antes: quem saltava aterrava
/// a `2 cm` do centro do herói parado na saída, e prendia-o.
pub(super) fn saida_livre(
    corpos: &BTreeMap<Entity, BodyRef>,
    mundo: &PhysicsWorld,
    quem: Entity,
    p: V2,
    r: f64,
) -> bool {
    corpos.iter().all(|(&e, b)| {
        if e == quem || b.kind == BodyKind::Static || b.rest.is_sensor {
            return true;
        }
        let Some(pose) = mundo.body_pose(b.handle) else {
            return true;
        };
        let c = [f64::from(pose.translation.x), f64::from(pose.translation.y)];
        // ⚠️ `libm`, nunca o `sin_cos` do `std` (o hash c9).
        let (sin, cos) = libm::sincos(f64::from(pose.rotation.angle()));
        let no_mundo = |[x, y]: V2| [c[0] + cos * x - sin * y, c[1] + sin * x + cos * y];
        match forma(&b.rest) {
            Forma::Disco(o, raio) => {
                let o = no_mundo(o);
                let (dx, dy) = (o[0] - p[0], o[1] - p[1]);
                (dx * dx + dy * dy).sqrt() >= raio + r
            }
            Forma::Poligono(pts) => {
                let w: Vec<V2> = pts.into_iter().map(no_mundo).collect();
                distancia_ao_poligono(&w, p) >= r
            }
        }
    })
}

/// A distância de `p` a um polígono CONVEXO anti-horário (`0` dentro dele).
pub(super) fn distancia_ao_poligono(w: &[V2], p: V2) -> f64 {
    let n = w.len();
    let mut dentro = true;
    let mut menor = f64::INFINITY;
    for i in 0..n {
        let (a, b) = (w[i], w[(i + 1) % n]);
        let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
        let (px, py) = (p[0] - a[0], p[1] - a[1]);
        dentro &= ex * py - ey * px >= 0.0;
        let l2 = ex * ex + ey * ey;
        let s = if l2 > 0.0 {
            ((px * ex + py * ey) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (qx, qy) = (px - s * ex, py - s * ey);
        menor = menor.min((qx * qx + qy * qy).sqrt());
    }
    if dentro { 0.0 } else { menor }
}

/// ⭐ (plano 30 §25, B) **O caminho contorna um corpo que ANDA** (a malha não o tem — só o vê parado):
/// se o troço até ao próximo canto (`ate` metros na direcção `dir`) passa a menos de `r` do polígono
/// CONVEXO `pts`, a direcção nova é a TANGENTE ao polígono engordado por `r`, do lado mais curto — e,
/// num empate, pela DIREITA, o lado do peso do desvio ([`ph2d_orca::SIDE_BIAS`]). Medido antes: o
/// caminho apontava ao alvo através da barreira e puxava o agente contra o peso de lado (`2,9×` o
/// CONTROLO a `3 m` e `0,3 m/s`, §22.6). Só `+ − × ÷ sqrt` (a cerca do hash).
pub(super) fn contorna(pos: V2, dir: V2, ate: f64, r: f64, pts: &[V2]) -> Option<V2> {
    let fim = [pos[0] + dir[0] * ate, pos[1] + dir[1] * ate];
    if pts.len() < 3 || distancia_do_troco(pos, fim, pts) >= r {
        return None;
    }
    let cruz = |a: V2, b: V2| a[0] * b[1] - a[1] * b[0];
    // A tangente de `pos` ao disco de raio `r` em cada vértice, dos dois lados (`sinal` = +1 à
    // esquerda): o mais extremo de cada lado é a tangente ao polígono engordado.
    let tangente = |v: V2, sinal: f64| {
        let (wx, wy) = (v[0] - pos[0], v[1] - pos[1]);
        let l = (wx * wx + wy * wy).sqrt();
        let (ux, uy) = (wx / l, wy / l);
        let s = (r / l).min(1.0) * sinal;
        let c = (1.0 - s * s).max(0.0).sqrt();
        [ux * c - uy * s, ux * s + uy * c]
    };
    let extremo = |sinal: f64| {
        pts.iter()
            .map(|&v| tangente(v, sinal))
            .reduce(|a, b| if cruz(a, b) * sinal > 0.0 { b } else { a })
    };
    let (esq, dir_) = (extremo(1.0)?, extremo(-1.0)?);
    let perto = |t: V2| t[0] * dir[0] + t[1] * dir[1];
    let d = if perto(dir_) >= perto(esq) { dir_ } else { esq };
    d.iter().all(|x| x.is_finite()).then_some(d)
}

/// A distância do troço `a`–`b` a um polígono CONVEXO anti-horário (`0` se o toca).
fn distancia_do_troco(a: V2, b: V2, w: &[V2]) -> f64 {
    let n = w.len();
    let mut menor = distancia_ao_poligono(w, a).min(distancia_ao_poligono(w, b));
    for i in 0..n {
        let (p, q) = (w[i], w[(i + 1) % n]);
        let lado =
            |o: V2, x: V2, y: V2| (x[0] - o[0]) * (y[1] - o[1]) - (x[1] - o[1]) * (y[0] - o[0]);
        if lado(a, b, p) * lado(a, b, q) < 0.0 && lado(p, q, a) * lado(p, q, b) < 0.0 {
            return 0.0;
        }
        menor = menor.min(ao_troco(p, a, b)).min(ao_troco(q, a, b));
    }
    menor
}

/// A distância de `p` ao troço `a`–`b`.
fn ao_troco(p: V2, a: V2, b: V2) -> f64 {
    let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
    let l2 = ex * ex + ey * ey;
    let t = if l2 > 0.0 {
        (((p[0] - a[0]) * ex + (p[1] - a[1]) * ey) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (p[0] - a[0] - t * ex, p[1] - a[1] - t * ey);
    (dx * dx + dy * dy).sqrt()
}
