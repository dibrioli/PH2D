//! ***Behaviours*, lado da pilha** (W5.4, `docs/BlenderConstraints/06_W5_resto.md` §3.2): cada um chama a
//! LEI do nó do Motion de igual nome (`Tremor`, `Onda`, `turn`/`orbit_point`, `Puxao`, `field`) — uma
//! lei, dois chamadores; os gates de paridade comparam com o `eval` do nó. O Atrair é o único com
//! estado: a força integrada quadro a quadro no [`Simulador`] (scrub = o mesmo bit).

use std::collections::BTreeMap;

use bevy_ecs::prelude::Resource;
use ph2d_constraints::Pose2;
use ph2d_constraints::comportamento::{Comportamento, Onde, Perfil, Peso};
use ph2d_node_force_attractor::Puxao;
use ph2d_node_motion_oscillator::Onda;
use ph2d_node_motion_wiggle::{AXIS_ROW_OFFSET, Tremor};

use crate::relogio::{Dinamica, Passo, Simulador};

/// O tempo dos *behaviours* deste quadro, preparado pela família antes da pilha.
#[derive(Resource, Default, Debug)]
pub struct Comportamentos {
    /// O instante da vista, em segundos.
    pub t: f64,
    /// O simulador de cada Atrair, por `(Entity::to_bits, Item::id)`.
    pub sims: BTreeMap<(u64, u32), Simulador<2>>,
    /// O deslocamento de cada Atrair neste quadro.
    pub deslocamentos: BTreeMap<(u64, u32), [f64; 2]>,
}

/// O Tremer, na lei do `motion.wiggle` (o resto dos knobs nos defaults do nó).
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
pub fn tremor(c: &Comportamento) -> Tremor {
    Tremor {
        amplitude: c.amplitude as f32,
        frequency: c.frequencia as f32,
        seed: c.semente as f32,
        ..Tremor::default()
    }
}

/// O Flutuar, na lei do `motion.oscillator` (senóide; a semente é a fase em ciclos).
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
pub fn onda(c: &Comportamento) -> Onda {
    Onda {
        amplitude: c.amplitude as f32,
        frequency: c.frequencia as f32,
        phase: c.semente as f32,
        ..Onda::default()
    }
}

/// O que o Tremer e o Flutuar somam no instante `t`: `[x, y, graus]` — a saída do nó para o
/// elemento `0`, antes da máscara.
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "o relógio do nó é f32")]
pub fn delta(c: &Comportamento, t: f64) -> [f32; 3] {
    let t = t as f32;
    let um = |v: f32| match c.onde {
        Onde::X => [v, 0.0, 0.0],
        Onde::Y | Onde::XY => [0.0, v, 0.0],
        Onde::Rot => [0.0, 0.0, v],
    };
    match c.perfil {
        Perfil::Tremer => {
            let tr = tremor(c);
            if c.onde == Onde::XY {
                [tr.sample(t, 0.0, 0.0), tr.sample(t, 0.0, AXIS_ROW_OFFSET), 0.0]
            } else {
                um(tr.sample(t, 0.0, 0.0))
            }
        }
        Perfil::Flutuar => um(onda(c).sample(t, 0.0, None)),
        Perfil::Orbitar | Perfil::Atrair => [0.0; 3],
    }
}

/// O peso por distância (a lei do `motion.falloff`, disco, sem rotação) do dono em `p` ao objecto
/// em `o`.
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
pub fn peso(w: &Peso, p: [f64; 2], o: [f64; 2]) -> f32 {
    let (dx, dy) = (p[0] as f32 - o[0] as f32, p[1] as f32 - o[1] as f32);
    ph2d_node_motion_falloff::field(
        0,
        dx,
        dy,
        w.raio as f32,
        i32::from(w.curva),
        w.inverte,
    )
}

/// O Orbitar: a posição `p` girada em volta de `pivo` no instante `t`, misturada por `f`, e os graus
/// da volta (para quem leva a rotação).
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
pub fn orbita(c: &Comportamento, p: [f64; 2], pivo: [f64; 2], t: f64, f: f32) -> ([f32; 2], f32) {
    let (graus, cos, sin) = ph2d_node_motion_orbit::turn(0.0, c.velocidade as f32, t as f32);
    let q = ph2d_node_motion_orbit::orbit_point(
        [p[0] as f32, p[1] as f32],
        [pivo[0] as f32, pivo[1] as f32],
        cos,
        sin,
        f,
    );
    (q, graus)
}

/// ⭐ **A pose do dono com o *behaviour*** à força `k` (a influência). `pivo` é o alvo (Orbitar,
/// Atrair), `peso_em` o objecto do peso, `atrair` o deslocamento do simulador deste quadro.
#[must_use]
pub fn aplica(
    c: &Comportamento,
    dono: Pose2,
    pivo: Option<Pose2>,
    peso_em: Option<[f64; 2]>,
    t: f64,
    atrair: [f64; 2],
    k: f64,
) -> Pose2 {
    let w = match (c.peso, peso_em) {
        (Some(p), Some(o)) => f64::from(peso(&p, [dono.x, dono.y], o)),
        _ => 1.0,
    };
    let f = k.clamp(0.0, 1.0) * w;
    let mut out = dono;
    match c.perfil {
        Perfil::Tremer | Perfil::Flutuar => {
            let [dx, dy, graus] = delta(c, t);
            out.x += f64::from(dx) * f;
            out.y += f64::from(dy) * f;
            out.rot += f64::from(graus).to_radians() * f;
        }
        Perfil::Orbitar => {
            let centro = pivo.map_or([dono.x - c.raio, dono.y], |p| [p.x, p.y]);
            #[expect(clippy::cast_possible_truncation, reason = "a mistura do nó é f32")]
            let (q, graus) = orbita(c, [dono.x, dono.y], centro, t, f as f32);
            out.x = f64::from(q[0]);
            out.y = f64::from(q[1]);
            if c.leva_rotacao {
                out.rot += f64::from(graus).to_radians() * f;
            }
        }
        Perfil::Atrair => {
            out.x += atrair[0] * f;
            out.y += atrair[1] * f;
        }
    }
    out
}

/// ⭐ **O Atrair como [`Dinamica`]**: a entrada é o alvo MENOS a pose animada do dono; o estado é o
/// deslocamento do dono e a sua velocidade. Um passo: a aceleração da lei do `force.attractor`, depois
/// o passo semi-implícito do `sim.step` (velocidade, atrito linear, posição com a velocidade NOVA).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atracao {
    pub forca: f64,
    pub raio: f64,
    /// A velocidade que fica ao fim de um segundo, `0..=1` (acima de 1 o passo injecta energia).
    pub amortece: f64,
}

impl From<&Comportamento> for Atracao {
    fn from(c: &Comportamento) -> Self {
        Self {
            forca: c.forca,
            raio: c.raio,
            amortece: c.amortece,
        }
    }
}

impl Atracao {
    /// A curva Smooth, sem repulsão nem perfil — os defaults do nó.
    #[must_use]
    #[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
    pub fn puxao(&self) -> Puxao {
        Puxao::of(self.forca as f32, self.raio as f32, 2.0, false, [0.0; 3])
    }
}

impl Dinamica<2> for Atracao {
    fn chave(&self) -> [u64; 3] {
        [
            self.forca.to_bits(),
            self.raio.to_bits(),
            self.amortece.to_bits(),
        ]
    }

    fn angular(&self) -> [bool; 2] {
        [false; 2]
    }

    fn repouso(&self, e: [f64; 2]) -> Passo<2> {
        Passo {
            valor: [0.0; 2],
            vel: [0.0; 2],
            entrada: e,
        }
    }

    #[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
    fn passo(&self, s: Passo<2>, e: [f64; 2], dt: f32) -> Passo<2> {
        let a = self.puxao().accel(s.valor, [e[0] as f32, e[1] as f32], 1.0);
        let keep = 1.0 - (1.0 - self.amortece.clamp(0.0, 1.0) as f32) * dt;
        let mut v = s.vel;
        let mut q = s.valor;
        for c in 0..2 {
            v[c] = (v[c] + a[c] * dt) * keep;
            q[c] += v[c] * dt;
        }
        // Um estado que divergiu fica onde estava (o `sim.step` faz o mesmo).
        if v.iter().chain(&q).all(|x| x.is_finite()) {
            Passo {
                valor: q,
                vel: v,
                entrada: e,
            }
        } else {
            Passo { entrada: e, ..s }
        }
    }

    fn saida(&self, s: Passo<2>) -> [f64; 2] {
        s.valor.map(f64::from)
    }
}

/// O item `(dono, id)` na pilha: lê o relógio, o alvo e o objecto do peso do mundo.
pub(crate) fn na_pilha(
    sim: &ph2d_ecs::SimWorld,
    idx: &BTreeMap<ph2d_ecs::StableId, bevy_ecs::entity::Entity>,
    (e, id): (bevy_ecs::entity::Entity, u32),
    (c, alvo): (&Comportamento, Option<bevy_ecs::entity::Entity>),
    dono: Pose2,
    influencia: f64,
) -> Result<Pose2, crate::Estado> {
    let w = sim.world();
    let pivo = alvo
        .and_then(|t| ph2d_ecs::world_transform(w, t))
        .map(|t| crate::pose(&t));
    if c.perfil.precisa_de_alvo() && pivo.is_none() {
        return Err(crate::Estado::SemAlvo);
    }
    let peso_em = c
        .peso
        .and_then(|p| idx.get(&ph2d_ecs::StableId(p.objecto)))
        .and_then(|&o| ph2d_ecs::world_transform(w, o))
        .map(|t| [f64::from(t.translation.x), f64::from(t.translation.y)]);
    let r = w.get_resource::<Comportamentos>();
    let t = r.map_or(0.0, |r| r.t);
    let atrair = r
        .and_then(|r| r.deslocamentos.get(&(e.to_bits(), id)).copied())
        .unwrap_or_default();
    Ok(aplica(c, dono, pivo, peso_em, t, atrair, influencia))
}
