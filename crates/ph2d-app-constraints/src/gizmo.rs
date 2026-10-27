//! ⭐ **O GIZMO das constraints no palco** (plano §4): da entidade escolhida, a linha até cada alvo,
//! o ARCO de cada Limit Rotation com as duas pontas arrastáveis e o CÍRCULO de cada Limit Distance
//! com a pega do raio (do lado oposto ao dono). Arrastar escreve o campo pela porta do cartão ([`crate::campos`]) — o campo
//! move o arco e o arco move o campo, sem segunda lei.
//!
//! A geometria é uma conta só, em px de TELA ([`pecas`]): o desenho ([`desenhar`]) e o clique
//! ([`pega_em`]) leem-na, para o que se vê ser o que se pega.

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_constraints::{Kind, Space};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::canal_constraint::{DISTANCIA, LIMITE_MAX, LIMITE_MIN};
use ph2d_ecs::{SimWorld, entity_with_stable_id, parent_world_transform, world_transform};
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Circle, Color, Fill, Line, Stroke, VectorScene};

/// O raio do arco do Limit Rotation na tela.
pub const ARCO_PX: f32 = 72.0; // LITERAL-PX-OK: chrome de overlay
/// O raio de uma pega e a folga do clique à volta dela.
pub const PEGA_PX: f32 = 5.0; // LITERAL-PX-OK: chrome de overlay
const FOLGA_PX: f32 = 4.0; // LITERAL-PX-OK: chrome de overlay
const TRACEJADO_PX: [f64; 2] = [6.0, 4.0]; // LITERAL-PX-OK: chrome de overlay
/// Segmentos de uma volta inteira do arco.
const LEQUE_POR_VOLTA: f32 = 64.0;

/// Que pega se segura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parte {
    /// A ponta de baixo do arco (o Min).
    Min,
    /// A ponta de cima (o Max).
    Max,
    /// A pega do raio do Limit Distance.
    Raio,
    /// O ponto de um Pino da cola (`item` = o id da cola, `dono` = o objecto que a guarda).
    Pino,
}

/// Uma pega: `(dono, Item::id, parte)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pega {
    pub dono: u64,
    pub item: u32,
    pub parte: Parte,
}

/// Uma peça do desenho, em px de tela.
#[derive(Clone, Debug, PartialEq)]
pub enum Peca {
    /// Do dono ao alvo.
    Linha([f32; 2], [f32; 2]),
    /// O arco do limite: o leque de pontos de Min a Max e as duas pontas.
    Arco {
        centro: [f32; 2],
        leque: Vec<[f32; 2]>,
        pontas: [(Pega, [f32; 2]); 2],
    },
    /// O círculo da distância, à volta do alvo, com a pega do raio.
    Circulo {
        centro: [f32; 2],
        raio: f32,
        pega: (Pega, [f32; 2]),
    },
    /// O ponto de um Pino da cola: arrastá-lo move o pino nos DOIS objectos de uma vez.
    Pino((Pega, [f32; 2])),
}

/// A pega que a mão segura agora (estado de sessão: premir arma, largar desarma).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Segurando(pub Option<Pega>);

fn tela(camera: &Camera2d, window: ph2d_host::WindowSize, p: [f64; 2]) -> [f32; 2] {
    #[expect(clippy::cast_possible_truncation, reason = "px de tela")]
    let (x, y) = camera.world_to_screen([p[0] as f32, p[1] as f32], window);
    [x, y]
}

/// O ângulo do referencial do limite: o do pai no espaço Local, zero no mundo.
fn base_do_espaco(w: &World, e: Entity, space: Space) -> f64 {
    match space {
        Space::World => 0.0,
        Space::Local => f64::from(parent_world_transform(w, e).rotation),
    }
}

/// ⭐ **As peças do gizmo da entidade `sel`** — a única conta do desenho e do clique.
#[must_use]
pub fn pecas(
    w: &World,
    sel: Option<u64>,
    camera: &Camera2d,
    window: ph2d_host::WindowSize,
) -> Vec<Peca> {
    let Some(e) = sel.and_then(Entity::try_from_bits) else {
        return Vec::new();
    };
    let mut out = pinos(w, e, camera, window);
    let (Some(c), Some(t)) = (w.get::<Constraints>(e), world_transform(w, e)) else {
        return out;
    };
    let o = [f64::from(t.translation.x), f64::from(t.translation.y)];
    let so = tela(camera, window, o);
    // ⚠️ A escala é MEDIDA na tela pela câmera (a lição do anel da luz).
    let s1 = tela(camera, window, [o[0] + 1.0, o[1]]);
    let ppm = f64::from((s1[0] - so[0]).hypot(s1[1] - so[1]));
    if !(ppm.is_finite() && ppm > 0.0) {
        return out;
    }
    for it in c.items.iter().filter(|i| i.enabled) {
        let alvo = it
            .target
            .and_then(|s| entity_with_stable_id(w, s))
            .and_then(|a| world_transform(w, a))
            .map(|a| [f64::from(a.translation.x), f64::from(a.translation.y)]);
        if let Some(a) = alvo {
            out.push(Peca::Linha(so, tela(camera, window, a)));
        }
        let pega = |parte| Pega {
            dono: e.to_bits(),
            item: it.id,
            parte,
        };
        match it.kind {
            Kind::Limit(l) => {
                let Some([lo, hi]) = l.rot else { continue };
                let b = base_do_espaco(w, e, l.space);
                let r = f64::from(ARCO_PX) / ppm;
                let ponto =
                    |ang: f64| tela(camera, window, [o[0] + r * ang.cos(), o[1] + r * ang.sin()]);
                let faixa = (hi - lo).clamp(0.0, std::f64::consts::TAU);
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "um punhado de segmentos"
                )]
                let n = ((faixa / std::f64::consts::TAU) as f32 * LEQUE_POR_VOLTA)
                    .ceil()
                    .max(1.0) as usize;
                #[expect(clippy::cast_precision_loss, reason = "um punhado de segmentos")]
                let leque = (0..=n)
                    .map(|i| ponto(b + lo + faixa * i as f64 / n as f64))
                    .collect();
                out.push(Peca::Arco {
                    centro: so,
                    leque,
                    pontas: [
                        (pega(Parte::Min), ponto(b + lo)),
                        (pega(Parte::Max), ponto(b + hi)),
                    ],
                });
            }
            Kind::Distance(d) => {
                let Some(a) = alvo else { continue };
                let (dx, dy) = (o[0] - a[0], o[1] - a[1]);
                // Do lado OPOSTO ao dono: no limite, o dono senta no círculo e a pega roubaria o
                // clique de quem o quer pegar.
                let ang = std::f64::consts::PI
                    + if dx == 0.0 && dy == 0.0 {
                        0.0
                    } else {
                        dy.atan2(dx)
                    };
                let p = [a[0] + d.distance * ang.cos(), a[1] + d.distance * ang.sin()];
                #[expect(clippy::cast_possible_truncation, reason = "px de tela")]
                out.push(Peca::Circulo {
                    centro: tela(camera, window, a),
                    raio: (d.distance * ppm) as f32,
                    pega: (pega(Parte::Raio), tela(camera, window, p)),
                });
            }
            _ => {}
        }
    }
    out
}

/// Os pinos das colas que tocam `e`, no ponto deles (o do lado `a`).
fn pinos(w: &World, e: Entity, camera: &Camera2d, window: ph2d_host::WindowSize) -> Vec<Peca> {
    let Some(s) = ph2d_ecs::stable_id_of(w, e) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (dono, cola) in crate::cola::todas(w) {
        let ph2d_constraints_cola::Tipo::Pino { em_a, .. } = cola.tipo else {
            continue;
        };
        if !cola.toca(s.0) {
            continue;
        }
        let Some(pa) = crate::cola::pose_mundial(w, cola.a) else {
            continue;
        };
        let pega = Pega {
            dono: dono.to_bits(),
            item: cola.id,
            parte: Parte::Pino,
        };
        out.push(Peca::Pino((pega, tela(camera, window, pa.ponto(em_a)))));
    }
    out
}

/// ⭐ **A pega sob o ponteiro** `ponto` (px de tela), a mais próxima.
#[must_use]
pub fn pega_em(
    w: &World,
    sel: Option<u64>,
    ponto: (f32, f32),
    camera: &Camera2d,
    window: ph2d_host::WindowSize,
) -> Option<Pega> {
    pecas(w, sel, camera, window)
        .into_iter()
        .flat_map(|p| match p {
            Peca::Arco { pontas, .. } => pontas.to_vec(),
            Peca::Circulo { pega, .. } | Peca::Pino(pega) => vec![pega],
            Peca::Linha(..) => Vec::new(),
        })
        .map(|(g, c)| ((ponto.0 - c[0]).hypot(ponto.1 - c[1]), g))
        .filter(|(d, _)| *d <= PEGA_PX + FOLGA_PX)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, g)| g)
}

/// ⭐ **Arrastar a pega** até `mundo`: a ponta do arco aponta para lá (pelo arco curto a partir da
/// outra ponta, como o `bone_limit::drag_edge`); a pega do raio põe a distância ao alvo. Escreve
/// pela porta do cartão. `true` se o documento mudou.
pub fn arrastar(sim: &mut SimWorld, pega: Pega, mundo: [f64; 2]) -> bool {
    let Some(e) = Entity::try_from_bits(pega.dono) else {
        return false;
    };
    if pega.parte == Parte::Pino {
        return crate::cola::mover_pino(sim, e, pega.item, mundo);
    }
    let w = sim.world();
    let (Some(c), Some(t)) = (w.get::<Constraints>(e), world_transform(w, e)) else {
        return false;
    };
    let Some(it) = c.item(pega.item) else {
        return false;
    };
    let (campo, valor) = match (it.kind, pega.parte) {
        (Kind::Limit(l), Parte::Min | Parte::Max) => {
            let Some([lo, hi]) = l.rot else { return false };
            let (dx, dy) = (
                mundo[0] - f64::from(t.translation.x),
                mundo[1] - f64::from(t.translation.y),
            );
            if dx == 0.0 && dy == 0.0 {
                return false;
            }
            let a = dy.atan2(dx) - base_do_espaco(w, e, l.space);
            let (campo, outro) = if pega.parte == Parte::Max {
                (LIMITE_MAX, lo)
            } else {
                (LIMITE_MIN, hi)
            };
            (campo, outro + ph2d_constraints::wrap_pi(a - outro))
        }
        (Kind::Distance(_), Parte::Raio) => {
            let Some(a) = it
                .target
                .and_then(|s| entity_with_stable_id(w, s))
                .and_then(|a| world_transform(w, a))
            else {
                return false;
            };
            (
                DISTANCIA,
                (mundo[0] - f64::from(a.translation.x))
                    .hypot(mundo[1] - f64::from(a.translation.y)),
            )
        }
        _ => return false,
    };
    #[expect(clippy::cast_possible_truncation, reason = "o campo do cartão é f32")]
    let v = if campo == DISTANCIA {
        valor as f32
    } else {
        valor.to_degrees() as f32
    };
    sim.world_mut()
        .get_mut::<Constraints>(e)
        .and_then(|mut c| {
            c.item_mut(pega.item)
                .map(|it| crate::campos::escrever(&mut it.kind, campo, v))
        })
        .unwrap_or(false)
}

/// O gesto do ponteiro primário sobre o palco.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gesto {
    Premir,
    Mover,
    Largar,
}

/// ⭐ **O ponteiro no gizmo** — a única porta da shell: premir sobre uma pega segura-a, mover
/// arrasta-a até `mundo`, largar solta-a. `true` = o gesto é do gizmo (a cadeia de baixo não o vê).
#[allow(clippy::too_many_arguments)] // as peças que a shell tem à mão, como o `light2d::gizmo`
pub fn gesto(
    sim: &mut SimWorld,
    gesto: Gesto,
    sel: Option<u64>,
    tela: (f32, f32),
    mundo: Option<[f64; 2]>,
    camera: &Camera2d,
    window: ph2d_host::WindowSize,
) -> bool {
    let segura = sim.world().get_resource::<Segurando>().and_then(|s| s.0);
    match gesto {
        Gesto::Premir => {
            let Some(p) = pega_em(sim.world(), sel, tela, camera, window) else {
                return false;
            };
            sim.world_mut().insert_resource(Segurando(Some(p)));
            true
        }
        Gesto::Mover => {
            let Some(p) = segura else {
                return false;
            };
            if let Some(m) = mundo {
                arrastar(sim, p, m);
            }
            true
        }
        Gesto::Largar => {
            sim.world_mut().insert_resource(Segurando(None));
            segura.is_some()
        }
    }
}

/// **Desenha o gizmo da entidade escolhida.**
pub fn desenhar(
    w: &World,
    sel: Option<u64>,
    theme: Theme,
    camera: &Camera2d,
    window: ph2d_host::WindowSize,
    cena: &mut VectorScene,
) {
    let segurada = w.get_resource::<Segurando>().and_then(|s| s.0);
    let cor = |t| ph2d_editor_core::paint::resolve(t, theme);
    let (linha, arco, fundo, viva) = (
        cor(ColorToken::Text2),
        cor(ColorToken::Accent),
        cor(ColorToken::Bg0),
        cor(ColorToken::Text1),
    );
    let s = cena.inner_mut();
    let p = |v: [f32; 2]| (f64::from(v[0]), f64::from(v[1]));
    let pega = |s: &mut ph2d_vector::Scene, g: Pega, c: [f32; 2]| {
        let disco = Circle::new(p(c), f64::from(PEGA_PX));
        let tinta: Color = if segurada == Some(g) { viva } else { arco };
        s.fill(Fill::NonZero, Affine::IDENTITY, tinta, None, &disco);
        s.stroke(&Stroke::new(1.0), Affine::IDENTITY, fundo, None, &disco);
    };
    for peca in pecas(w, sel, camera, window) {
        match peca {
            Peca::Linha(a, b) => s.stroke(
                &Stroke::new(1.0).with_dashes(0.0, TRACEJADO_PX),
                Affine::IDENTITY,
                linha,
                None,
                &Line::new(p(a), p(b)),
            ),
            Peca::Arco {
                centro,
                leque,
                pontas,
            } => {
                let mut leque_path = BezPath::new();
                leque_path.move_to(p(centro));
                for q in &leque {
                    leque_path.line_to(p(*q));
                }
                leque_path.close_path();
                s.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    cor(ColorToken::AccentSoft),
                    None,
                    &leque_path,
                );
                s.stroke(&Stroke::new(1.5), Affine::IDENTITY, arco, None, &leque_path);
                for (g, c) in pontas {
                    pega(s, g, c);
                }
            }
            Peca::Circulo {
                centro,
                raio,
                pega: (g, c),
            } => {
                s.stroke(
                    &Stroke::new(1.5).with_dashes(0.0, TRACEJADO_PX),
                    Affine::IDENTITY,
                    arco,
                    None,
                    &Circle::new(p(centro), f64::from(raio)),
                );
                pega(s, g, c);
            }
            Peca::Pino((g, c)) => {
                s.stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    arco,
                    None,
                    &Circle::new(p(c), f64::from(PEGA_PX + FOLGA_PX)),
                );
                pega(s, g, c);
            }
        }
    }
}
