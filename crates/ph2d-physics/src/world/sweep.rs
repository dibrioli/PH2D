//! **VARRER O CORPO** (`W-ShapeCast`) — a pergunta que um raio não sabe fazer:
//! *"o corpo cabe se eu andar para ali?"*.
//!
//! # ⚠️ Por que isto não é um parâmetro do [`super::cast`]
//!
//! Um raio pergunta *"esta LINHA toca alguma coisa?"* e uma varredura pergunta
//! *"este CORPO toca alguma coisa ao andar?"*. As duas devolvem o mesmo
//! [`CastHit`] — *o que um cast encontrou* é a mesma resposta —, mas a pergunta
//! difere no que importa: um raio tem largura zero, e é essa largura zero que
//! deixa passar tudo o que cabe entre duas amostras.
//!
//! O preço disso estava **medido** antes desta porta existir
//! (`measure_the_gap_between_rays`): o sensor do agachar lê o teto com **três
//! raios** sobre um corpo de 0,40 m de largura, o que deixa vãos de 0,20 m — e um
//! pilar de 8 cm posto num desses vãos é invisível. O personagem levanta-se, a
//! cabeça chega a **1,267** contra uma face de pedra em **1,25**, e o solver
//! segura-o ali dentro.
//!
//! ⚠️ **E isso contradiz uma afirmação escrita:** o doc do `probe_headroom` diz
//! que *"o erro possível é ficar agachado onde caberia, nunca levantar-se para
//! dentro da pedra"*. A frase é verdadeira sobre a **caixa envolvente contra a
//! cápsula** (a razão para que foi escrita) e **falsa** sobre o que cabe entre
//! dois raios. Quem move o número que tornava algo impossível tem de reconferir
//! a nota (CLAUDE.md §0).
//!
//! # As três metades do filtro, e nenhuma é opcional
//!
//! São as mesmas de [`super::cast::PhysicsWorld::cast_ray`], pelas mesmas razões,
//! e a terceira é ainda mais obrigatória aqui:
//!
//! - **as CAMADAS valem** (`groups_for`, a porta do solver) — um sensor e o
//!   solver não podem discordar sobre o que é sólido;
//! - **um SENSOR não é matéria** (`EXCLUDE_SENSORS`) — a mesma frase que o
//!   `buoyancy` escreve do outro lado;
//! - **o próprio corpo sai** (`exclude_rigid_body`). Num raio isto é higiene;
//!   numa varredura é aritmética: a forma NASCE exactamente em cima do próprio
//!   collider, então sem a exclusão toda varredura devolveria impacto em zero.
//!
//! # ⚠️ Um corpo pode ter VÁRIAS formas
//!
//! A `W-Compound` deu a um corpo mais de um collider, e a `W-PartFace` mostrou o
//! preço de supor o contrário em quatro sítios. Esta porta varre **todos** os
//! colliders do corpo e fica com o impacto mais próximo — a mesma forma do
//! [`super::queries::PhysicsWorld::body_aabb`], que funde as caixas de todos.
//!
//! A ordem de `rb.colliders()` é a de inserção (determinística) e o empate fica
//! com o **primeiro**, então a resposta não depende de em que ordem o BVH os
//! visitou.

use crate::rmath::Vector;
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::parry::query::{DefaultQueryDispatcher, ShapeCastOptions};
use rapier2d::pipeline::{QueryFilter, QueryFilterFlags, QueryPipeline};

use super::cast::CastHit;
use super::{PhysicsWorld, groups_for};

impl PhysicsWorld {
    /// **Varrer o corpo** de onde ele está, ao longo de `dir`, por até
    /// `max_dist` metros. Devolve o primeiro impacto, ou `None` se o caminho
    /// está livre.
    ///
    /// `layer` é a camada de quem pergunta, e a matriz do mundo decide o que ela
    /// enxerga — igual ao raio.
    ///
    /// Devolve `None` também para entrada degenerada (direção nula ou `NaN`,
    /// `max_dist` negativo ou `NaN`, corpo inexistente ou sem forma nenhuma), e
    /// pela mesma razão de camadas que o [`super::cast::PhysicsWorld::cast_ray`]
    /// documenta: recusar cedo é mais barato que descer no BVH para descobrir.
    ///
    /// ⚠️ **`dir` não precisa vir normalizado** — esta porta o normaliza. O
    /// `max_time_of_impact` do parry é medido em múltiplos da norma da
    /// velocidade, então um chamador que passasse `dir` não-unitário obteria um
    /// alcance diferente do que pediu, **em silêncio**. É a mesma armadilha, e a
    /// mesma cura, do raio.
    ///
    /// ⚠️ **`distance == 0` significa PENETRAÇÃO** (a forma já começa dentro de
    /// alguma coisa), exactamente como no raio — e aqui, ao contrário do raio, a
    /// `normal` continua **confiável** nesse caso: o parry calcula a geometria do
    /// impacto mesmo em penetração por default
    /// (`compute_impact_geometry_on_penetration`), e esta porta não o desliga.
    /// O contrato publicado do [`CastHit`] continua o mesmo — *pode* ser
    /// degenerada —, porque um consumidor que já trata o caso não deve ter de
    /// aprender de qual das duas portas o `CastHit` veio.
    #[must_use]
    pub fn sweep_body(
        &self,
        handle: RigidBodyHandle,
        dir: [f32; 2],
        max_dist: f32,
        layer: u8,
    ) -> Option<CastHit> {
        let d = Vector::new(dir[0], dir[1]);
        let n = d.length();
        if !n.is_finite() || n <= f32::EPSILON || !max_dist.is_finite() || max_dist < 0.0 {
            return None;
        }
        let unit = d / n;
        let rb = self.bodies.get(handle)?;

        let dispatcher = DefaultQueryDispatcher;
        let filter = QueryFilter {
            groups: Some(groups_for(layer as usize, self.layer_matrix)),
            exclude_rigid_body: Some(handle),
            flags: QueryFilterFlags::EXCLUDE_SENSORS,
            ..QueryFilter::default()
        };
        let pipeline: QueryPipeline<'_> =
            self.broad_phase
                .as_query_pipeline(&dispatcher, &self.bodies, &self.colliders, filter);
        let options = ShapeCastOptions::with_max_time_of_impact(max_dist);

        let mut best: Option<CastHit> = None;
        for &ch in rb.colliders() {
            let Some(c) = self.colliders.get(ch) else {
                continue;
            };
            let Some((other, hit)) = pipeline.cast_shape(c.position(), unit, c.shape(), options)
            else {
                continue;
            };
            // Empate fica com o primeiro: `rb.colliders()` é ordem de inserção.
            if best.is_some_and(|b| b.distance <= hit.time_of_impact) {
                continue;
            }
            best = Some(CastHit {
                collider: other,
                body: self.colliders.get(other).and_then(|c| c.parent()),
                distance: hit.time_of_impact,
                point: [hit.witness1.x, hit.witness1.y],
                normal: [hit.normal1.x, hit.normal1.y],
            });
        }
        best
    }
}

#[cfg(test)]
#[path = "sweep_tests.rs"]
mod tests;
