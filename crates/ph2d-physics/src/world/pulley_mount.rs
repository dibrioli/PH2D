//! **A ROLDANA MONTADA NUM CORPO** — a cadernal móvel de uma talha (W-Pulley W3).
//!
//! Módulo FILHO de [`super`] pelo cap de LOC, e o corte é por responsabilidade:
//! lá mora a DINÂMICA da corda (o impulso, a massa efetiva, o guincho, a
//! ruptura); aqui, *o que muda quando o eixo de uma roldana deixa de ser um ponto
//! do cenário e passa a ser um ponto de um CORPO*.
//!
//! É esta wave que traz a vantagem mecânica de volta — e não por um número: uma
//! roldana montada é sustentada por DOIS ramos da mesma corda, então a magnitude
//! do Jacobiano daquele eixo é **2** num enlace de 180°, e o "2" nunca é escrito.
//! Medido em `tests/measure_pulley_tackle.rs`: um bloco de 2 kg equilibra com
//! **1,00 kg** de contrapeso na talha e **2,00 kg** amarrado direto na corda.

use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use rapier2d::na::{Point2, Vector2};

use super::rope_route::{self, RopeWheel, Tangent};
use super::{End, end};

/// **Onde as roldanas MONTADAS estão AGORA** — uma vez por sub-passo, na ARENA
/// (W3).
///
/// ⚠️ **Na arena, e não numa cópia**, e é isso que impede a segunda opinião: a
/// arena é a lista que o **DESENHO** lê (`PhysicsWorld::pulley_wheels`), então
/// refrescar um clone deixaria o solver usando a pose viva enquanto o overlay
/// desenha a corda passando por onde a roldana foi autorada. É a mesma lei que o
/// filtro de eixo rompido já segue em duas camadas.
///
/// ⚠️ **Por SUB-PASSO, não por dispatch.** A ponte reinstala a arena uma vez por
/// frame; dentro de um tique o corpo se move `substeps` vezes, e uma cadernal
/// móvel cuja geometria ficasse um tique atrasada injetaria energia — a corda
/// puxaria numa direção que já não é a dela.
///
/// Um corpo que sumiu **desmonta** a roldana (`body = None`) em vez de a deixar
/// meio-montada: assim `k`, `Ċ` e o impulso concordam por construção, em vez de
/// cada um decidir sozinho o que fazer com um handle morto.
pub fn refresh_mounts(bodies: &RigidBodySet, arena: &mut [RopeWheel]) {
    for w in arena.iter_mut() {
        let Some(handle) = w.body else { continue };
        match mount_point(bodies, handle, w.local) {
            Some(p) => w.centre = [p.x, p.y],
            None => w.body = None,
        }
    }
}

/// Onde um ponto body-local está, em mundo.
///
/// Porta única para as duas perguntas que precisam dela — *onde a roldana está*
/// (acima) e *que ponto do corpo o impulso alcança* ([`end`]) —, porque as duas
/// TÊM de dar o mesmo ponto: a massa efetiva é calculada no braço de alavanca até
/// ali, e um impulso aplicado a um centímetro de onde a geometria diz gira o corpo
/// por um torque que ninguém escreveu.
pub(super) fn mount_point(
    bodies: &RigidBodySet,
    handle: RigidBodyHandle,
    local: [f32; 2],
) -> Option<Point2<f32>> {
    let body = bodies.get(handle)?;
    Some(body.position() * Point2::new(local[0], local[1]))
}

/// **O eixo MONTADO da roldana `i`** (W3): o corpo que ela carrega, a ponta
/// resolvida naquele ponto, e o Jacobiano daquele centro. `None` para uma roldana
/// pregada no cenário — que é o que toda roldana era até esta wave.
///
/// ⚠️ **Porta ÚNICA, e a razão é dura:** a massa efetiva e o impulso percorrem
/// esta mesma lista em DOIS momentos (`k` tem de estar completo antes de `λ`
/// existir), e se cada laço decidisse por conta própria quem está montado o
/// sistema ficaria **estável E errado** — a massa dizendo uma coisa e o impulso
/// outra, com a corda parecendo apenas "meio frouxa".
///
/// ⚠️ **`j` NÃO é unitário** — a magnitude dele é a vantagem mecânica daquele
/// enlace (2 num enlace de 180°). O [`End::k`] é a forma quadrática `vᵀM⁻¹v`, que
/// vale para qualquer vetor, e o [`End::rate`] é uma projeção: nenhum dos dois
/// pede versor, e é por isso que a talha sai sem um caminho de código próprio.
pub(super) fn mounted_axle(
    bodies: &RigidBodySet,
    live: &[RopeWheel],
    legs: &[Tangent],
    i: usize,
) -> Option<(RigidBodyHandle, End, Vector2<f32>)> {
    let w = live.get(i)?;
    let handle = w.body?;
    let j = rope_route::wheel_jacobian(legs, i)?;
    let e = end(bodies, handle, w.local)?;
    Some((handle, e, Vector2::new(j[0], j[1])))
}
