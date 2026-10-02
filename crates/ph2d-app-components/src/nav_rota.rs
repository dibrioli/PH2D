//! ⭐⭐ **A ROTA DA PATRULHA** (plano 30, W6) — da forma desenhada com a caneta aos pontos que a ponte
//! da navegação visita ([`ph2d_physics_ecs::NavRoute`], derivado e não registado).
//!
//! # Porque mora na família
//!
//! A geometria vectorial não entra no ECS (a doutrina do `VecPathRef`): quem vê a curva e o mundo
//! ao mesmo tempo é a família, como no seguidor de caminho ([`crate::path_follow_bridge`], que a
//! chama a cada quadro). A ponte só lê os pontos.
//!
//! # Que pontos
//!
//! Os da curva COZIDA (o que o artista vê na tela), em MUNDO: as âncoras do artista e, onde a curva
//! se afasta da corda mais do que a `arrive_distance` do agente, pontos a meio — partidos ao meio até
//! a flecha caber nela ou o pedaço ser mais curto que ela (um pedaço assim não se afasta mais que o
//! próprio comprimento). ⚠️ A régua é a do EXECUTOR: a ronda avança quando o agente chega a um ponto
//! a essa distância, logo detalhe mais fino que ela o agente não distingue. Uma `arrive_distance`
//! nula dá só as âncoras.

use ph2d_ecs::{Entity, Name, SimWorld, stable_name_id};
use ph2d_physics_ecs::{NavAgent, NavRoute, NavTarget};
use ph2d_vec_scene::VecScene;
use ph2d_vec_scene::arc_path::ArcPath;

/// ⭐ **Põe em dia o [`NavRoute`] de todo agente em patrulha** (e tira-o de quem saiu dela, ou cuja
/// forma não existe — o agente fica parado e o Inspector diz porquê). Só escreve quando muda.
pub fn escreve(sim: &mut SimWorld, cena: &VecScene) {
    let world = sim.world_mut();
    let Some(mut q) = world.try_query::<(Entity, &NavAgent)>() else {
        return;
    };
    let agentes: Vec<(Entity, Option<u64>, f32)> = q
        .iter(world)
        .map(|(e, a)| match a.target {
            NavTarget::Patrol(id) => (e, Some(id), a.arrive_distance),
            _ => (e, None, 0.0),
        })
        .collect();
    for (e, id, chegada) in agentes {
        let rota = id.and_then(|id| rota_de(sim, cena, id, f64::from(chegada.max(0.0))));
        let w = sim.world_mut();
        match rota {
            Some(r) => {
                if w.get::<NavRoute>(e) != Some(&r) {
                    w.entity_mut(e).insert(r);
                }
            }
            None => {
                if w.get::<NavRoute>(e).is_some() {
                    w.entity_mut(e).remove::<NavRoute>();
                }
            }
        }
    }
}

/// **A forma que se chama assim** — a entidade com aquele nome que carrega uma forma vectorial.
pub(crate) fn forma_chamada(
    sim: &SimWorld,
    id: u64,
) -> Option<(Entity, ph2d_vec_scene::VecPathId)> {
    if id == 0 {
        return None;
    }
    let world = sim.world();
    let mut q = world.try_query::<(Entity, &Name)>()?;
    let e = q
        .iter(world)
        .find(|(_, n)| stable_name_id(n.as_str()) == id)
        .map(|(e, _)| e)?;
    Some((e, world.get::<ph2d_ecs::VecPathRef>(e)?.0))
}

fn rota_de(sim: &SimWorld, cena: &VecScene, id: u64, tol: f64) -> Option<NavRoute> {
    let (dono, path) = forma_chamada(sim, id)?;
    let cozido = cena.path(path)?.cooked();
    let arco = ArcPath::from_contour(&cozido.verts, cozido.closed)?;
    let afim = ph2d_vec_entities::transform::xform_of_transform(
        ph2d_vec_entities::transform::world_transform(sim, dono),
    );
    let ponto = |s: f64| afim.apply(arco.frame_at(s).0);
    let mut ancoras: Vec<f64> = arco.anchor_arcs().to_vec();
    if !arco.closed() {
        ancoras.push(arco.total());
    }
    let mut out: Vec<[f64; 2]> = Vec::new();
    for w in ancoras.windows(2) {
        out.push(ponto(w[0]));
        parte(&ponto, w[0], w[1], tol, &mut out);
    }
    if arco.closed() {
        if let Some(&ultima) = ancoras.last() {
            out.push(ponto(ultima));
            parte(&ponto, ultima, arco.total(), tol, &mut out);
        }
    } else if let Some(&fim) = ancoras.last() {
        out.push(ponto(fim));
    }
    out.dedup();
    Some(NavRoute {
        points: out.iter().map(|p| [p[0] as f32, p[1] as f32]).collect(),
        closed: arco.closed(),
    })
}

/// Os pontos ENTRE `a` e `b` (sem os extremos), partindo ao meio enquanto a flecha passa de `tol` e
/// o pedaço é mais comprido que ela. Ver o cabeçalho.
fn parte(ponto: &impl Fn(f64) -> [f64; 2], a: f64, b: f64, tol: f64, out: &mut Vec<[f64; 2]>) {
    if tol <= 0.0 || b - a <= tol {
        return;
    }
    let (p, q, m) = (ponto(a), ponto(b), ponto(0.5 * (a + b)));
    if flecha(p, q, m) <= tol {
        return;
    }
    let meio = 0.5 * (a + b);
    parte(ponto, a, meio, tol, out);
    out.push(m);
    parte(ponto, meio, b, tol, out);
}

/// A distância de `m` à corda `p → q` (à recta, ou a `p` se a corda é um ponto).
fn flecha(p: [f64; 2], q: [f64; 2], m: [f64; 2]) -> f64 {
    let d = [q[0] - p[0], q[1] - p[1]];
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let v = [m[0] - p[0], m[1] - p[1]];
    if l <= 0.0 {
        return (v[0] * v[0] + v[1] * v[1]).sqrt();
    }
    (d[0] * v[1] - d[1] * v[0]).abs() / l
}

#[cfg(test)]
#[path = "nav_rota_tests.rs"]
mod tests;
