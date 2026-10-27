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
    /// O simulador de cada Atrair (e da mola de cada centro de órbita), por `(Entity::to_bits, Item::id)`.
    pub sims: BTreeMap<(u64, u32), Simulador<2>>,
    /// O deslocamento de cada Atrair neste quadro, ou o atraso do centro de uma órbita com mola.
    pub deslocamentos: BTreeMap<(u64, u32), [f64; 2]>,
    /// A pose de um alvo SEM curva (arrastado à mão), no quadro em que mudou — o passado que a mola
    /// re-simula.
    pub historia: BTreeMap<(u64, u32), BTreeMap<i64, [f64; 2]>>,
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

/// O centro da órbita do dono em `dono` sem alvo: a `raio` dele, na direcção oposta ao `angulo` (no
/// instante 0 o dono está onde foi posto).
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
pub fn pivo_sem_alvo(c: &Comportamento, dono: [f64; 2]) -> [f64; 2] {
    let (_, cos, sin) = ph2d_node_motion_orbit::turn(c.angulo as f32, 0.0, 0.0);
    [
        dono[0] - c.raio * f64::from(cos),
        dono[1] - c.raio * f64::from(sin),
    ]
}

/// ⭐ O Orbitar: o ponto a `raio` à direita do `pivo`, girado pela lei do `motion.orbit` (`angle` =
/// o ângulo de partida, `speed` = a velocidade) — a posição na órbita no instante `t` e os graus da
/// volta desde a partida. A órbita é RELATIVA ao centro: ele anda e o raio fica.
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
pub fn orbita(c: &Comportamento, pivo: [f64; 2], t: f64) -> ([f32; 2], f32) {
    let (graus, cos, sin) =
        ph2d_node_motion_orbit::turn(c.angulo as f32, c.velocidade as f32, t as f32);
    let centro = [pivo[0] as f32, pivo[1] as f32];
    let p = [(pivo[0] + c.raio) as f32, centro[1]];
    let q = ph2d_node_motion_orbit::orbit_point(p, centro, cos, sin, 1.0);
    (q, graus - c.angulo as f32)
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
            let centro = pivo.map_or_else(|| pivo_sem_alvo(c, [dono.x, dono.y]), |p| [p.x, p.y]);
            let (q, graus) = orbita(c, centro, t);
            out.x = dono.x + (f64::from(q[0]) - dono.x) * f;
            out.y = dono.y + (f64::from(q[1]) - dono.y) * f;
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

/// O passo do `sim.step` (o tecto `MAX_DT` dele, 0,03 s).
const PASSO_MAX: f32 = 0.03;
/// Tecto de sub-passos por quadro: o do travão de 400/s a 24 q/s (`k·dt` ≈ 17) com folga — cada
/// sub-passo são duas leis de nó, e o kill da W5 mede o quadro.
const MAX_SUB_PASSOS: u32 = 32;

/// Quantos sub-passos fazem o quadro `dt` estável: cada um ≤ [`PASSO_MAX`], com `k·h ≤ 1` (o travão
/// explícito diverge a `k·h > 2` — medido: travão 56 a 24 q/s atirou a limalha a 1e19 m) e
/// `ω·h ≤ 0,5` na mola (`ω² = forca / pico`).
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "um número pequeno e positivo"
)]
fn sub_passos(dt: f32, k: f32, omega2: f64) -> u32 {
    let w = omega2.max(0.0).sqrt() as f32;
    let n = (dt / PASSO_MAX).max(k * dt).max(2.0 * w * dt).ceil();
    (n as u32).clamp(1, MAX_SUB_PASSOS)
}

/// ⭐ **O Atrair como [`Dinamica`]**: a entrada é o alvo MENOS a pose animada do dono; o estado é o
/// deslocamento do dono e a sua velocidade. Um passo: a aceleração da lei do `force.attractor`, depois
/// o travão do `force.drag` somado, e o passo semi-implícito do `sim.step` (velocidade, posição com a
/// velocidade NOVA).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atracao {
    pub forca: f64,
    pub raio: f64,
    /// O travão do `force.drag` (≥ 0, por segundo).
    pub arrasto: f64,
    /// O `peak` do perfil do nó: até ele o puxão cresce com a distância (uma mola).
    pub pico: f64,
}

impl From<&Comportamento> for Atracao {
    fn from(c: &Comportamento) -> Self {
        Self {
            forca: c.forca,
            raio: c.raio,
            arrasto: c.arrasto,
            pico: c.pico,
        }
    }
}

impl Atracao {
    /// A curva LINEAR do nó com o `peak` do perfil: até ao pico o puxão é `forca · d / pico` — uma mola,
    /// que com o travão crítico segue atrás sem passar à frente (medido na sonda da cena 10).
    #[must_use]
    #[expect(clippy::cast_possible_truncation, reason = "a lei do nó é f32")]
    pub fn puxao(&self) -> Puxao {
        Puxao::of(
            self.forca as f32,
            self.raio as f32,
            0.0,
            false,
            [0.0, self.pico as f32, 0.0],
        )
    }
}

impl Dinamica<2> for Atracao {
    fn chave(&self) -> [u64; 3] {
        [
            self.forca.to_bits(),
            self.raio.to_bits(),
            self.arrasto.to_bits() ^ self.pico.to_bits().rotate_left(1),
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
        let (puxao, k, alvo) = (
            self.puxao(),
            self.arrasto.max(0.0) as f32,
            [e[0] as f32, e[1] as f32],
        );
        let n = sub_passos(dt, k, self.forca / self.pico.max(1e-3));
        #[expect(clippy::cast_precision_loss, reason = "n ≤ MAX_SUB_PASSOS")]
        let h = dt / n as f32;
        let mut v = s.vel;
        let mut q = s.valor;
        for _ in 0..n {
            let puxa = puxao.accel(q, alvo, 1.0);
            let trava = ph2d_node_force_drag::arrasto(v, k, 1.0, [1.0; 2]);
            for c in 0..2 {
                v[c] += (puxa[c] + trava[c]) * h;
                q[c] += v[c] * h;
            }
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
    let r = w.get_resource::<Comportamentos>();
    let atraso = r
        .and_then(|r| r.deslocamentos.get(&(e.to_bits(), id)).copied())
        .unwrap_or_default();
    // Com mola, o centro da órbita é o alvo mais o atraso dela (o dono busca a órbita aos poucos).
    let pivo = alvo
        .and_then(|t| ph2d_ecs::world_transform(w, t))
        .map(|t| crate::pose(&t))
        .map(|p| match (c.perfil, c.segue) {
            (Perfil::Orbitar, Some(_)) => Pose2 {
                x: p.x + atraso[0],
                y: p.y + atraso[1],
                ..p
            },
            _ => p,
        });
    if c.perfil.precisa_de_alvo() && pivo.is_none() {
        return Err(crate::Estado::SemAlvo);
    }
    let peso_em = c
        .peso
        .and_then(|p| idx.get(&ph2d_ecs::StableId(p.objecto)))
        .and_then(|&o| ph2d_ecs::world_transform(w, o))
        .map(|t| [f64::from(t.translation.x), f64::from(t.translation.y)]);
    let t = r.map_or(0.0, |r| r.t);
    Ok(aplica(c, dono, pivo, peso_em, t, atraso, influencia))
}
