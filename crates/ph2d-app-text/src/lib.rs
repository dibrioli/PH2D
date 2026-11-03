//! ⭐ **Add ▸ Text** (dono, 02/11): o texto nasce JÁ escrito, em Draw ▸ Text, como no Figma — a 1.ª
//! tecla troca a palavra inteira. Com um HUD (ou um filho dele) seleccionado, nasce no centro dele e
//! FILHO dele. O *Add ▸ Vector* não muda: arma o pincel e nasce no 1.º traço.
//!
//! O pedido atravessa três quadros, por isso é estado ([`TextAdd`]): o Add arma a ferramenta do
//! vetor ([`TextAdd::add`]); com ela na mão a shell abre a sessão do vetor já escrita
//! ([`TextAdd::ready`] → [`TextAdd::opened`]); a entidade que o `sync` cria pendura-se no HUD
//! ([`TextAdd::upkeep`]). O Draw pede-se sozinho: a forma que nasce com a ferramenta na mão é a
//! recém-nascida do `vector_mode` do vetor. A família não conhece o vetor (camadas): a shell passa o
//! «armar» e o «armada?».

use ph2d_ecs::{ChildOf, Entity, SimWorld, UiCanvas};
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecPathId;

/// O texto.
pub const TEXT: AddEntry = AddEntry::new("object_add.vector.text", AddGroup::TwoD);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[TEXT];

/// Um *Add ▸ Text* em curso.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Request {
    /// Onde a baseline nasce, em MUNDO: o centro do HUD, senão o centro da vista.
    pub origin: [f64; 2],
    /// O HUD que o recebe como filho.
    pub parent: Option<u64>,
    /// A sessão já abriu (falta só a entidade).
    started: bool,
}

/// O estado da família. Runtime-only: o que fica é o texto, e esse é do vetor.
#[derive(Default, Debug)]
pub struct TextAdd {
    /// O pedido em curso.
    pub request: Option<Request>,
    /// A ORIGEM da sessão que abriu com a palavra de partida: a 1.ª tecla DELA troca-a inteira. A
    /// origem identifica a sessão — outra (um duplo-clique noutro texto) não herda a troca.
    fresh: Option<[f64; 2]>,
}

impl TextAdd {
    /// ⭐ **O pedido** — `true` se a entrada é esta. `selected` é a selecção do editor; `view_center`
    /// o centro da vista em mundo; `arm` pede a ferramenta do vetor na mão.
    pub fn add(
        &mut self,
        entry: AddEntry,
        sim: &SimWorld,
        selected: Option<u64>,
        view_center: [f64; 2],
        arm: impl FnOnce(),
    ) -> bool {
        if entry != TEXT {
            return false;
        }
        let parent = selected.and_then(|b| canvas_of(sim, Entity::from_bits(b)));
        let origin = parent.map_or(view_center, |p| {
            let t = ph2d_vec_entities::transform::world_transform(sim, p).translation;
            [f64::from(t.x), f64::from(t.y)]
        });
        arm();
        self.request = Some(Request {
            origin,
            parent: parent.map(Entity::to_bits),
            started: false,
        });
        true
    }

    /// ⭐ **A ferramenta do vetor já está na mão e a sessão por abrir** — devolve onde a abrir. A
    /// shell abre a sessão e chama [`Self::opened`].
    /// `armed` = o pedido de ferramenta do vetor ainda não foi consumido.
    pub fn ready(&mut self, armed: bool, tools: &ToolRegistry) -> Option<[f64; 2]> {
        let req = self.request.as_mut().filter(|r| !r.started)?;
        let in_hand = tools
            .active()
            .is_some_and(|t| t.id() == ToolId::new("vector"));
        if armed {
            return None;
        }
        if !in_hand {
            // O pedido foi consumido e a ferramenta não veio (ou já saiu): o Add morreu.
            self.request = None;
            return None;
        }
        req.started = true;
        Some(req.origin)
    }

    /// ⭐ **Enquanto a sessão que o Add abriu viver, a entidade dela é filha do HUD, sem se mover.**
    /// `session` = a origem da sessão viva e o path dela (`None` sem geometria). ⚠️ O pedido vive
    /// até a sessão acabar, e não até a 1.ª entidade: um Backspace inicial esvazia o texto, a
    /// entidade morre, e a letra seguinte nasce outra — que também tem de ir para o HUD. Outra
    /// sessão (outra origem) mata o pedido.
    pub fn upkeep(
        &mut self,
        sim: &mut SimWorld,
        map: &VecEntityMap,
        session: Option<([f64; 2], Option<VecPathId>)>,
    ) {
        let Some(req) = self.request.filter(|r| r.started) else {
            return;
        };
        let Some((_, editing)) = session.filter(|(o, _)| *o == req.origin) else {
            self.request = None;
            return;
        };
        let (Some(p), Some(&bits)) = (req.parent, editing.and_then(|id| map.get(&id))) else {
            return;
        };
        let (kid, hud) = (Entity::from_bits(bits), Entity::from_bits(p));
        if sim.world().get::<ChildOf>(kid).map(ChildOf::parent) != Some(hud) {
            ph2d_vec_entities::transform::reparent_keeping_world(sim, kid, hud);
        }
    }

    /// A sessão abriu (DEPOIS de fechar a anterior, que esquece a troca): o texto dela passa a ser a
    /// palavra de partida.
    pub fn opened(&mut self, origin: [f64; 2], text: &mut String) {
        *text = ph2d_i18n::tr("object_add.vector.text.placeholder").to_string();
        self.fresh = Some(origin);
    }

    /// ⭐ **A 1.ª tecla troca a palavra inteira** — chamado ANTES de cada tecla (escrever ou apagar)
    /// com a origem da sessão que a recebe: a 1.ª da sessão aberta pelo Add esvazia o texto; a de
    /// outra sessão só esquece a troca.
    pub fn before_key(&mut self, origin: [f64; 2], text: &mut String) {
        if self.fresh.take() == Some(origin) {
            text.clear();
        }
    }

    /// A sessão fechou (ou o undo a levou): esquece a troca.
    pub fn session_closed(&mut self) {
        self.fresh = None;
    }
}

/// O HUD de `e`: ele próprio ou o antepassado mais próximo com [`UiCanvas`].
fn canvas_of(sim: &SimWorld, mut e: Entity) -> Option<Entity> {
    let w = sim.world();
    while w.get::<UiCanvas>(e).is_none() {
        e = w.get::<ChildOf>(e)?.parent();
    }
    Some(e)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
