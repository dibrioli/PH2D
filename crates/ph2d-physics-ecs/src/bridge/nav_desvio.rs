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
use crate::bridge::PhysicsBridge;
use crate::components::BodyKind;

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
        // FORMA, em discos (ver [`discos`]). ⚠️ O ALVO de alguém fica um disco só: quem o persegue
        // ignora-o por um índice, e um disco é o que esse índice nomeia.
        let alvos: BTreeSet<Entity> = pedidas.iter().filter_map(|p| p.alvo).collect();
        // (o aberto da W5) As PEÇAS de cada corpo (um filho só com `Collider`): fazem parte da forma.
        let mut pecas: BTreeMap<Entity, Vec<&super::super::parts::PartRef>> = BTreeMap::new();
        for p in self.parts.values().filter(|p| !p.rest.is_sensor) {
            pecas.entry(p.owner).or_default().push(p);
        }
        for (&e, b) in &self.bodies {
            if b.kind == BodyKind::Static || b.rest.is_sensor || indice.contains_key(&e) {
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
                let mut f = discos(&b.rest);
                for p in minhas {
                    // Os discos da peça, do referencial dela para o do corpo (`local` = onde ela está).
                    let [lx, ly, lr] = p.local.map(f64::from);
                    let (s, c) = libm::sincos(lr);
                    f.extend(
                        discos(&p.rest)
                            .into_iter()
                            .map(|([x, y], r)| ([lx + c * x - s * y, ly + s * x + c * y], r)),
                    );
                }
                f
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
        for (k, p) in pedidas.iter().enumerate() {
            corpos[k].ignores = p.alvo.and_then(|a| indice.get(&a).copied());
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
        let mut multidao = ph2d_orca::Crowd::new(corpos, ph2d_orca::Params::PRODUCT);
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

/// ⭐ (o aberto da W6) **Um corpo que ANDA como o desvio o vê: discos ao longo da forma**, no
/// referencial dele — `(centro, raio)`. Um disco só, o que envolve a forma, fazia de uma porta de
/// `4 m` um círculo de `2 m` de raio, e ela desviava os agentes de longe (gate
/// `uma_porta_comprida_a_andar_desvia_se_pela_forma`: `1,55 m` fora do caminho).
///
/// A forma alongada (meio-comprimento `a`, meia-espessura `b`) parte-se em `n = ⌈a/b⌉` células de
/// `s = 2a/n` ≤ `2b`, e cada disco é o CIRCUNSCRITO da sua célula (`√(b² + (s/2)²)` ≤ `b·√2`) —
/// a união cobre a forma sem a encolher. A cápsula e o estádio deitam-se em `y` (o de fábrica).
fn discos(d: &BodyDesc) -> Vec<([f64; 2], f64)> {
    let (a, b, em_x) = match d.shape {
        ShapeDesc::Cuboid { half_x, half_y } => {
            (half_x.max(half_y), half_x.min(half_y), half_x >= half_y)
        }
        ShapeDesc::Ellipse { rx, ry } => (rx.max(ry), rx.min(ry), rx >= ry),
        ShapeDesc::Capsule {
            half_height,
            radius,
        } => (half_height + radius, radius, false),
        ShapeDesc::Stadium {
            half_height,
            rx,
            ry,
        } => (half_height + ry, rx, false),
        ShapeDesc::Ball { radius } => (radius, radius, true),
    };
    let (a, b) = (f64::from(a.abs()), f64::from(b.abs()));
    let o = [f64::from(d.offset[0]), f64::from(d.offset[1])];
    if b <= 0.0 || a <= b {
        return vec![(
            o,
            f64::from(raio_que_envolve(d)) - (o[0] * o[0] + o[1] * o[1]).sqrt(),
        )];
    }
    let n = (a / b).ceil();
    let s = 2.0 * a / n;
    let raio = (b * b + s * s / 4.0).sqrt();
    (0..n as u32)
        .map(|k| {
            let t = -a + s * (f64::from(k) + 0.5);
            let c = if em_x {
                [o[0] + t, o[1]]
            } else {
                [o[0], o[1] + t]
            };
            (c, raio)
        })
        .collect()
}
