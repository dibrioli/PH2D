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

/// **Tudo o que uma superfície diz**, numa entrada só.
///
/// ⚠️ **Duas COMPONENTES, uma ENTRADA**, e as duas metades da decisão são
/// separadas de propósito: do lado do ARQUIVO são componentes irmãos (um campo
/// apendado à [`WalkSurface`] seria postcard posicional ⇒ bump de
/// `PROJECT_SCHEMA` ⇒ recusa de todo projeto salvo — a saída que o doc daquele
/// módulo já prescreve); do lado do RUNTIME esta tabela é interna ao bridge e
/// pode crescer à vontade. Fundi-las aqui é o que mantém **uma** lei de
/// resolução part-vs-corpo em vez de duas que divergem.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(super) struct SurfaceFacts {
    pub(super) walk: WalkSurface,
    /// A mão não agarra aqui (`W-WallMaterial`).
    pub(super) no_cling: bool,
}

/// As superfícies vivas, indexadas por onde a consulta as procura.
#[derive(Default)]
pub(super) struct Surfaces {
    /// Peças (um filho com `Collider` e sem `RigidBody`): pelo COLLIDER.
    by_collider: BTreeMap<Key, SurfaceFacts>,
    /// Corpos de forma única: pelo CORPO.
    by_body: BTreeMap<Key, SurfaceFacts>,
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
        self.facts_at(hit).walk
    }

    /// **Esta superfície é parede?** — a pergunta que o sensor de parede faz
    /// (`W-WallMaterial`), pela MESMA porta e portanto sob a mesma lei
    /// part-vs-corpo.
    #[must_use]
    pub(super) fn clings_at(&self, hit: &CastHit) -> bool {
        !self.facts_at(hit).no_cling
    }

    fn facts_at(&self, hit: &CastHit) -> SurfaceFacts {
        if self.is_empty() {
            return SurfaceFacts::default();
        }
        if let Some(s) = self.by_collider.get(&hit.collider.into_raw_parts()) {
            return *s;
        }
        hit.body
            .and_then(|b| self.by_body.get(&b.into_raw_parts()))
            .copied()
            .unwrap_or_default()
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
        // ⚠️ **Passe 2 primeiro? Não — a ORDEM não importa aqui, e é por isso que
        // ela pode ser lida sem susto:** cada passe escreve um campo DIFERENTE da
        // mesma entrada, e a entrada nasce no neutro. O que importa é que os dois
        // usem a mesma chave (peça pelo collider, corpo pelo corpo), senão a
        // mesma superfície teria duas identidades.
        let mut nq = self.no_cling_query.take().expect("query built in prepare");
        for (e, _) in nq.iter(world) {
            self.surface_entry(e).no_cling = true;
        }
        self.no_cling_query = Some(nq);

        let mut q = self.surface_query.take().expect("query built in prepare");
        for (e, surf) in q.iter(world) {
            // ⚠️ **O neutro não entra na tabela.** Ele é indistinguível da
            // ausência do componente, e uma entrada que não muda nada só faria a
            // consulta deixar de ser gratuita numa cena que não autorou nada.
            if surf.is_neutral() {
                continue;
            }
            self.surface_entry(e).walk = *surf;
        }
        self.surface_query = Some(q);
    }

    /// A entrada desta entidade na tabela, criada no neutro se ainda não existe.
    ///
    /// ⚠️ **A PEÇA primeiro, e é a mesma ordem que a consulta usa** — o
    /// `hit.body` de uma peça é o corpo DONO dela, então indexar uma peça pelo
    /// corpo daria a superfície do tronco a quem pisou nela.
    fn surface_entry(&mut self, e: ph2d_ecs::Entity) -> &mut SurfaceFacts {
        if let Some(p) = self.parts.get(&e) {
            let k = p.handle.into_raw_parts();
            return self.surfaces.by_collider.entry(k).or_default();
        }
        let k = self
            .bodies
            .get(&e)
            .map(|b| b.handle.into_raw_parts())
            .unwrap_or_default();
        self.surfaces.by_body.entry(k).or_default()
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
