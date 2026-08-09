//! **O CONTROLADOR CINEMÁTICO** (W-KinMove) — *"quanto deste deslocamento o
//! mundo deixa acontecer?"*, e nada mais.
//!
//! # ⚠️ Esta porta não decide movimento nenhum
//!
//! Quem decide **quanto** o personagem quer andar é a lei pura
//! (`ph2d_platformer::kinematic_advance`); quem decide **onde está o chão** é o
//! [`crate::PhysicsWorld::cast_ray`], pela `footing()` (K4 do plano 07). O que
//! sobra para cá é a única pergunta que precisa do mundo indexado: **o
//! shapecast-and-slide**.
//!
//! É deliberado que a resposta seja um deslocamento e não uma escrita: quem
//! escreve a pose é a ponte, no mesmo lugar em que ela escreve a de um corpo
//! dinâmico, e é isso que mantém *"quem é o dono da pose deste corpo?"* com uma
//! resposta só.
//!
//! # ⚠️ O `grounded` que o rapier devolve é DIAGNÓSTICO
//!
//! O [`EffectiveCharacterMovement`] traz um `grounded: bool`, e consumi-lo seria
//! a **segunda resposta** para o fato de que a lei inteira depende — a doença
//! que este módulo curou quatro vezes. Ele é carregado no [`CharacterMove`]
//! porque uma sonda quer poder compará-lo com a `footing()`, e há arch-gate a
//! afirmar que nada da lei o lê.
//!
//! # ⚠️ Um SENSOR não bloqueia um personagem, pela mesma razão do raio
//!
//! O `cast_ray` desta crate exclui sensores porque *"um sensor é um marcador,
//! não matéria"* — e um controlador que não fizesse o mesmo seria **parado por
//! um volume de gatilho**, que é o mesmo defeito um grau pior: o personagem
//! dinâmico ficava de pé sobre a água, o cinemático bateria numa parede
//! invisível.

use rapier2d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::{ColliderHandle, SharedShape};
use rapier2d::na::{Isometry2, Unit, Vector2};
use rapier2d::parry::query::DefaultQueryDispatcher;
use rapier2d::parry::shape::Compound;
use rapier2d::pipeline::{QueryFilter, QueryFilterFlags, QueryPipeline};

use super::{PhysicsWorld, groups_for};

/// **O que o mundo deixou acontecer.**
///
/// Uma struct e não um `[f32; 2]` porque o segundo campo tem de vir com o aviso
/// colado: ele **não** é a resposta sobre estar no chão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CharacterMove {
    /// O deslocamento EFETIVO, em mundo. Somar isto à pose do corpo.
    pub translation: [f32; 2],
    /// ⚠️ **DIAGNÓSTICO, nunca decisão** (K4) — a resposta da lei sobre chão é
    /// a `footing()`, e há gate a impedir que esta vire uma segunda.
    pub grounded: bool,
}

/// **Em que o personagem bateu** (W-KinPush) — o fato cru do shapecast, sem lei
/// nenhuma em cima.
///
/// ⚠️ **Fatos, não decisões.** Quanto de um bloqueio volta para o corpo atingido
/// é a [`ph2d_platformer::push_transfer`], que é pura e não conhece rapier; o que
/// só o mundo indexado sabe responder é *em quê, onde, e com que normal* — e é
/// exatamente isso, e nada mais, que atravessa esta fronteira.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CharacterHit {
    /// O corpo atingido. `None` para um collider sem corpo — nada a empurrar.
    pub body: Option<RigidBodyHandle>,
    /// O ponto de contato, em MUNDO.
    ///
    /// ⚠️ É ele que produz o TORQUE (`r × F`) quando o empurrão é aplicado, e é
    /// a razão de um caixote atingido em cima tombar e um atingido no meio
    /// deslizar — a mesma lição do `apply_player_reaction`.
    pub point: [f32; 2],
    /// A normal da superfície no contato.
    ///
    /// ⚠️ **O SENTIDO dela não é contrato**, e nenhuma lei deste repo pode
    /// depender dele: qual das duas testemunhas de um shapecast a produz é
    /// convenção da biblioteca. A [`ph2d_platformer::push_transfer`] projeta na
    /// LINHA dela, que é invariante à troca de sinal.
    pub normal: [f32; 2],
}

/// Os parâmetros do controlador que **saem da configuração do player**, para
/// não existir um segundo número dizendo a mesma coisa.
///
/// ⚠️ Cada campo aqui é lido de um valor que o artista já autora:
/// `snap_distance` é o `cling_distance` (a faixa que a perna já chama de *"ainda
/// é chão"*), `max_slope_deg` é o *Max Slope* que a `footing()` honra. Um
/// default próprio do rapier em qualquer um deles seria o produto a discordar de
/// si mesmo numa inclinação: segurado por uma metade, recusado pela outra.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CharacterParams {
    /// Para onde é "cima".
    pub up: [f32; 2],
    /// Até que distância abaixo dos pés o controlador ainda cola no chão.
    pub snap_distance: f32,
    /// O limite de rampa, em graus — o MESMO que a `footing()` usa.
    pub max_slope_deg: f32,
    /// A altura do degrau que ele sobe sozinho, em metros.
    ///
    /// ⚠️ Zero **desliga** o autostep, e é o que um personagem sem degraus quer:
    /// o autostep é a feature mais cara do controlador (o doc do rapier o diz), e
    /// ela custa por tique mesmo onde não há nada a subir.
    pub step_height: f32,
}

impl PhysicsWorld {
    /// **A que distância do ORIGEM ficam os pés deste corpo** — o quanto ele
    /// mede para baixo, ao longo de `-up`.
    ///
    /// # ⚠️ Por que a lei precisa disto, e só sob Snap
    ///
    /// O sensor de chão mede da ORIGEM do corpo para baixo, e a lei compara essa
    /// distância com o `float_height` — *a que altura o personagem paira*. Sob
    /// Spring esse número é AUTORADO (é o comprimento da perna, e o corpo de
    /// facto flutua ali). **Sob Snap não há perna:** a cápsula é a silhueta e ela
    /// POUSA, então *"em repouso"* é um FATO da geometria, não uma escolha.
    ///
    /// Sem isto, a lei diz *"estou no chão"* com o personagem a 0,4 m no ar (o
    /// alcance da perna sobre um corpo que não a tem), e a consequência foi
    /// medida: ele congela no ar à altura em que nasceu, com todos os outros
    /// gates verdes.
    ///
    /// Devolve `None` para handle morto ou corpo sem forma sólida.
    #[must_use]
    pub fn body_foot_distance(&self, handle: RigidBodyHandle, up: [f32; 2]) -> Option<f32> {
        let body = self.bodies.get(handle)?;
        let o = body.position().translation.vector;
        let mut deepest: Option<f32> = None;
        for &ch in body.colliders() {
            let Some(c) = self.colliders.get(ch).filter(|c| !c.is_sensor()) else {
                continue;
            };
            let aabb = c.compute_aabb();
            // ⚠️ Os QUATRO cantos, e não `mins.y`: a caixa é alinhada aos eixos do
            // MUNDO e o `up` é um parâmetro — projetar um canto escolhido a dedo
            // acertaria só no caso vertical, que é o único que hoje se usa.
            for (x, y) in [
                (aabb.mins.x, aabb.mins.y),
                (aabb.maxs.x, aabb.mins.y),
                (aabb.mins.x, aabb.maxs.y),
                (aabb.maxs.x, aabb.maxs.y),
            ] {
                let d = -((x - o.x) * up[0] + (y - o.y) * up[1]);
                deepest = Some(deepest.map_or(d, |m: f32| m.max(d)));
            }
        }
        deepest.filter(|d| d.is_finite() && *d > 0.0)
    }

    /// **Mover um corpo cinemático por `wanted`, deslizando no que encontrar.**
    ///
    /// Devolve o deslocamento que de facto coube — e **não escreve nada**: a
    /// pose é escrita por quem é dono dela.
    ///
    /// `exclude_collider` é a plataforma jump-through que este personagem está a
    /// atravessar (W12): sem ela o controlador o pararia em cima de exactamente
    /// aquilo que ele pediu para atravessar, que é o mesmo defeito que o
    /// `cast_ray_skipping` existe para não ter no sensor.
    ///
    /// Um handle morto, um corpo sem forma, ou um `wanted` degenerado devolvem
    /// deslocamento zero — o chamador é um laço por-entidade e não deve ter de
    /// perguntar antes.
    ///
    /// # ⚠️ `hits` é um buffer do CHAMADOR, e é sempre LIMPO
    ///
    /// Ele sai por parâmetro em vez de dentro do [`CharacterMove`] por duas
    /// razões que apontam para o mesmo lado: o `CharacterMove` é `Copy` (um
    /// `Vec` lá dentro o tiraria de todo chamador que hoje o guarda de graça) e
    /// o laço de players da ponte quer **um** buffer reusado por tique, não uma
    /// alocação por personagem.
    ///
    /// ⚠️ Ele é limpo AQUI, e não pelo chamador: uma lista que só é limpa por
    /// quem lembrar acumula os contatos do tique anterior, e o sintoma seria um
    /// empurrão fantasma contra um corpo que já não se toca.
    #[must_use]
    pub fn move_character(
        &self,
        handle: RigidBodyHandle,
        wanted: [f32; 2],
        params: CharacterParams,
        exclude_collider: Option<ColliderHandle>,
        layer: u8,
        hits: &mut Vec<CharacterHit>,
    ) -> CharacterMove {
        hits.clear();
        let none = CharacterMove {
            translation: [0.0, 0.0],
            grounded: false,
        };
        if !wanted[0].is_finite() || !wanted[1].is_finite() {
            return none;
        }
        let Some(body) = self.bodies.get(handle) else {
            return none;
        };
        // ⚠️ **A forma é a do COLLIDER, e um corpo composto é um `Compound`**
        // (o W-Compound deu a um corpo várias formas). O caso de uma forma só
        // — o comum — passa direto, sem construir árvore nenhuma: um
        // `Compound::new` de um elemento pagaria uma BVH por tique para
        // responder o que a forma já responde.
        let shapes: Vec<ColliderHandle> = body
            .colliders()
            .iter()
            .copied()
            .filter(|h| {
                self.colliders
                    .get(*h)
                    .is_some_and(|c| !c.is_sensor() && Some(*h) != exclude_collider)
            })
            .collect();
        let (shape, pos): (SharedShape, Isometry2<f32>) = match shapes.as_slice() {
            [] => return none,
            [one] => {
                let c = &self.colliders[*one];
                (c.shared_shape().clone(), *c.position())
            }
            many => {
                // Em coordenadas do CORPO, e a pose do cast é a do corpo: as
                // formas de um composto já estão dispostas umas em relação às
                // outras, e é essa disposição que tem de viajar.
                let parts: Vec<(Isometry2<f32>, SharedShape)> = many
                    .iter()
                    .map(|h| {
                        let c = &self.colliders[*h];
                        (
                            *c.position_wrt_parent().unwrap_or(&Isometry2::identity()),
                            c.shared_shape().clone(),
                        )
                    })
                    .collect();
                (SharedShape::new(Compound::new(parts)), *body.position())
            }
        };

        let up = Vector2::new(params.up[0], params.up[1]);
        let up = Unit::try_new(up, 1.0e-6).unwrap_or_else(Vector2::y_axis);
        let controller = KinematicCharacterController {
            up,
            slide: true,
            // ⚠️ Zero DESLIGA — ver [`CharacterParams::step_height`].
            autostep: (params.step_height > 0.0).then_some(CharacterAutostep {
                max_height: CharacterLength::Absolute(params.step_height),
                min_width: CharacterLength::Absolute(params.step_height * 0.5),
                // Subir num caixote solto é o que um personagem faz; recusá-lo
                // seria uma regra que só o artista descobre a tropeçar.
                include_dynamic_bodies: true,
            }),
            max_slope_climb_angle: params.max_slope_deg.clamp(0.0, 90.0).to_radians(),
            // ⚠️ **Igual ao limite de subida, e é o mesmo número de propósito:**
            // uma faixa entre os dois seria uma inclinação em que ele nem sobe
            // nem escorrega — fica parado numa rampa que a lei já recusou.
            min_slope_slide_angle: params.max_slope_deg.clamp(0.0, 90.0).to_radians(),
            snap_to_ground: (params.snap_distance > 0.0)
                .then_some(CharacterLength::Absolute(params.snap_distance)),
            ..KinematicCharacterController::default()
        };

        let dispatcher = DefaultQueryDispatcher;
        let filter = QueryFilter {
            groups: Some(groups_for(layer as usize, self.layer_matrix)),
            exclude_rigid_body: Some(handle),
            exclude_collider,
            // Ver o aviso do módulo: um gatilho não é uma parede.
            flags: QueryFilterFlags::EXCLUDE_SENSORS,
            ..QueryFilter::default()
        };
        let queries: QueryPipeline<'_> =
            self.broad_phase
                .as_query_pipeline(&dispatcher, &self.bodies, &self.colliders, filter);

        let moved = controller.move_shape(
            self.dt(),
            &queries,
            shape.as_ref(),
            &pos,
            Vector2::new(wanted[0], wanted[1]),
            |c| {
                // ⚠️ **A testemunha 1 é a do CORPO QUE SE MOVE**, e a pose dela é
                // a `character_pos` — é assim que o próprio rapier a leva para o
                // mundo no `solve_character_collision_impulses`. Levá-la pela pose
                // de ENTRADA daria o ponto onde o personagem estava antes de
                // deslizar, que num tique de contato não é onde ele encostou.
                let p = c.character_pos * c.hit.witness1;
                hits.push(CharacterHit {
                    body: self.colliders.get(c.handle).and_then(|col| col.parent()),
                    point: [p.x, p.y],
                    normal: [c.hit.normal1.x, c.hit.normal1.y],
                });
            },
        );
        CharacterMove {
            translation: [moved.translation.x, moved.translation.y],
            grounded: moved.grounded,
        }
    }
}

#[cfg(test)]
#[path = "character_tests.rs"]
mod tests;
