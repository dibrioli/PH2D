//! **De que é feito o chão que o raio achou?** (`W-Surface`) — a tabela que
//! responde, e a razão de ela ser tão pequena.
//!
//! # ⚠️ A superfície NÃO viaja no `BodyDesc`, e a decisão tem duas metades
//!
//! **É VIVA.** O artista arrasta o slider de Grip e o personagem tem de
//! responder no tique seguinte. Se ela ridesse a receita de spawn, mudá-la
//! exigiria re-nascer o collider — ou um passe de re-carimbo por dispatch, que é
//! literalmente a máquina que o `bridge::damping` teve de construir por ter
//! escolhido o outro caminho. Lida do ECS a cada dispatch ela é viva de graça, e
//! um rewind não tem nada que re-armar: a fonte da verdade nunca saiu do lugar.
//!
//! **E o SOLVER não precisa dela.** O `one_way` mora no `user_data` do collider
//! porque o hook de contato corre DENTRO do rapier; `grip` e `belt` só são lidos
//! aqui, no laço do player. Enfiá-los no `BodyDesc` custaria uma linha em cada um
//! dos ~147 sítios que o constroem — para dar ao wrapper de física uma opinião
//! sobre caminhar, que ele não tem.
//!
//! # ⚠️ A tabela é do tamanho do que foi AUTORADO, não da cena
//!
//! A chave é escolhida por quem a vai procurar: uma **peça** entra pelo handle do
//! COLLIDER dela (a ponte sabe onde pendurou cada uma), e um corpo de forma
//! única entra pelo handle do CORPO — porque a ponte não guarda o handle de
//! collider de um corpo, e adivinhá-lo pela ordem de inserção é a suposição que a
//! primeira peça pendurada quebraria (o mesmo raciocínio do `bridge::triggers`).
//!
//! Numa cena que nunca autorou uma superfície os dois mapas ficam **vazios**, a
//! consulta sai no primeiro `if`, e a wave inteira é gratuita.

use std::collections::BTreeMap;

use ph2d_ecs::SimWorld;
use ph2d_physics::CastHit;

use super::PhysicsBridge;
use crate::WalkSurface;

/// Chave de handle — o mesmo `(u32, u32)` que o `bridge::triggers` usa, e pela
/// mesma razão: `BTreeMap` para a iteração ser determinística cross-OS.
type Key = (u32, u32);

/// As superfícies vivas, indexadas por onde a consulta as procura.
#[derive(Default)]
pub(super) struct Surfaces {
    /// Peças (um filho com `Collider` e sem `RigidBody`): pelo COLLIDER.
    by_collider: BTreeMap<Key, WalkSurface>,
    /// Corpos de forma única: pelo CORPO.
    by_body: BTreeMap<Key, WalkSurface>,
}

impl Surfaces {
    /// Nada autorado — o caso comum, e o que torna a consulta gratuita.
    fn is_empty(&self) -> bool {
        self.by_collider.is_empty() && self.by_body.is_empty()
    }

    /// **A superfície do chão que ESTE raio achou.**
    ///
    /// ⚠️ **O collider primeiro, e a ordem é a lei:** o `hit.body` de uma peça é
    /// o corpo DONO dela, então perguntar ao corpo antes daria a superfície do
    /// tronco a quem pisou na peça — e uma plataforma com uma face de gelo e
    /// outra de borracha passaria a ter uma só.
    #[must_use]
    pub(super) fn at(&self, hit: &CastHit) -> WalkSurface {
        if self.is_empty() {
            return WalkSurface::NEUTRAL;
        }
        if let Some(s) = self.by_collider.get(&hit.collider.into_raw_parts()) {
            return *s;
        }
        hit.body
            .and_then(|b| self.by_body.get(&b.into_raw_parts()))
            .copied()
            .unwrap_or(WalkSurface::NEUTRAL)
    }
}

impl PhysicsBridge {
    /// Reconstrói a tabela a partir do ECS. Chamado no prólogo, **depois** das
    /// peças: uma superfície numa peça precisa do collider dela já pendurado.
    ///
    /// ⚠️ **Do zero a cada dispatch, e é isso que a mantém honesta:** ela
    /// descreve o que está autorado AGORA, então um slider arrastado morde no
    /// tique seguinte e um componente removido some sem deixar entrada morta.
    /// O custo é proporcional ao que o artista autorou, não à cena.
    pub(super) fn reconcile_surfaces(&mut self, sim: &SimWorld) {
        self.surfaces.by_collider.clear();
        self.surfaces.by_body.clear();
        let world = sim.world();
        let mut q = self.surface_query.take().expect("query built in prepare");
        for (e, surf) in q.iter(world) {
            // ⚠️ **O neutro não entra na tabela.** Ele é indistinguível da
            // ausência do componente, e uma entrada que não muda nada só faria a
            // consulta deixar de ser gratuita numa cena que não autorou nada.
            if surf.is_neutral() {
                continue;
            }
            if let Some(p) = self.parts.get(&e) {
                self.surfaces
                    .by_collider
                    .insert(p.handle.into_raw_parts(), *surf);
            } else if let Some(b) = self.bodies.get(&e) {
                self.surfaces
                    .by_body
                    .insert(b.handle.into_raw_parts(), *surf);
            }
        }
        self.surface_query = Some(q);
    }
}

/// **A amostra que a lei recebe**, montada a partir do que o raio achou e do que
/// a superfície diz.
///
/// ⚠️ **A correia entra na `ground_velocity`, ao longo da TANGENTE** — e não num
/// campo próprio: a lei já mede tudo relativo ao chão, e uma esteira é
/// literalmente *um chão que anda sem o corpo andar*. Somada aqui, ela chega de
/// graça a todo consumidor daquele campo (a caminhada, a subida relativa, o que
/// o chão ainda deve ao integrador cinemático) em vez de precisar de um segundo
/// termo em cada um deles.
///
/// ⚠️ **E a tangente é `perp_cw(normal)`, a MESMA da caminhada** — se as duas
/// derivassem o eixo por conta própria, uma correia numa rampa empurraria numa
/// direção e o motor a perseguiria noutra.
#[must_use]
pub(super) fn ground_velocity_with_belt(
    contact_velocity: [f32; 2],
    normal: [f32; 2],
    surface: WalkSurface,
) -> [f32; 2] {
    if surface.belt == 0.0 || !surface.belt.is_finite() {
        return contact_velocity;
    }
    let axis = ph2d_platformer::perp_cw(normal);
    [
        contact_velocity[0] + axis[0] * surface.belt,
        contact_velocity[1] + axis[1] * surface.belt,
    ]
}
