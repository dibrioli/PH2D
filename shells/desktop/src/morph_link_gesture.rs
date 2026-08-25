//! **O gesto das SETAS do Morph** (`DrawMode::MorphLink`, plano 32 W3b): pressiona numa forma,
//! arrasta, solta noutra — nasce uma **aresta** no grafo da máquina.
//!
//! # ⚠️ O mesmo movimento da mão do conector, outro produto
//!
//! O [`crate::connector_gesture`] produz uma **linha no documento** (que exporta, imprime e se
//! selecciona); esta produz uma **aresta num grafo**, que é chrome — a explicação de uma regra.
//! Dois produtos atrás do mesmo movimento precisam de dois modos, senão o artista não tem como
//! dizer qual deles quer.
//!
//! ⚠️ **E o hit-test é o MESMO** ([`crate::app_state::App::shape_under_cursor`]) — não uma cópia:
//! a pergunta é literalmente a mesma, *que forma está debaixo do dedo?*, e duas respostas
//! divergiriam no dia em que uma anotação nova nascesse.
//!
//! # ⛔ Sem um Morph SELECIONADO, o gesto não faz nada — e é a resposta honesta
//!
//! Uma seta é uma aresta **no grafo de alguém**. Sem dono, ela teria de inventar um: criar um
//! `VecMorph` do nada a meio de um arrasto poria no documento um objecto que o artista não pediu.
//! ⇒ o gesto exige o objecto de Morph em mãos, que é o mesmo que o painel de estados exige.
//!
//! # ⭐ A PRIMEIRA seta faz nascer a máquina, com `start` na forma de onde ela parte
//!
//! É por isso que o [`ph2d_ecs::VecMorphMachine`] **não tem `Default`**: ele chega com o gesto, e
//! o `start` é um facto do gesto — nunca um zero que alguém teria de corrigir depois.

use ph2d_ecs::{Entity, VecMorph, VecMorphMachine};
use ph2d_morph_machine::MorphEdge;
use ph2d_vec_scene::VecPathId;

use crate::app_state::App;

/// **A LEI da seta: liga `from` a `to` no grafo de `morph`.** Devolve `true` se a aresta nasceu.
///
/// ⭐ **Fora do `impl App` de propósito, e é o que a torna GATEÁVEL.** O gesto precisa de um
/// `AppGfx` — uma janela real e uma superfície de GPU —, e um teste não alcança isso (a mesma
/// parede que a linha do sculpt3d registou no undo do filtro). O que se pode provar é a **lei**, e
/// ela não precisa de nada disso: um mundo e três ids.
///
/// ⛔ **Ligar uma forma a ela própria não cria nada.** Um morph de uma forma para ela mesma é a
/// identidade — uma transição que não transita —, e aceitá-la poria no grafo uma seta que o
/// artista vê a apontar para si mesma sem nunca mudar nada na tela. ⚠️ Isto **não** é a mesma
/// decisão do conector, que aceita o laço de propósito: lá o laço é um **desenho** legítimo; aqui
/// seria uma regra vazia.
pub(crate) fn link_shapes(
    sim: &mut ph2d_ecs::SimWorld,
    morph: Entity,
    from: VecPathId,
    to: VecPathId,
) -> bool {
    if from == to {
        return false;
    }
    let mut ent = sim.world_mut().entity_mut(morph);
    // ⭐ A PRIMEIRA seta faz nascer a máquina, com o `start` na forma de onde ela parte. É por isso
    // que o `VecMorphMachine` não tem `Default`: o `start` é um facto do GESTO.
    if ent.get::<VecMorphMachine>().is_none() {
        ent.insert(VecMorphMachine::new(from));
    }
    let Some(mut m) = ent.get_mut::<VecMorphMachine>() else {
        return false;
    };
    // ⚠️ **Uma seta repetida não se duplica.** Duas arestas iguais seriam duas linhas idênticas no
    // painel, uma impossível de distinguir da outra ao apagar — a mesma lei que a ligação de tecla
    // do Input Map já paga.
    if m.graph.edges.iter().any(|e| e.from == from && e.to == to) {
        return false;
    }
    m.graph.edges.push(MorphEdge::new(from, to));
    true
}

/// A seta em construção (Down..Up).
#[derive(Clone, Copy, Debug)]
pub(crate) struct MorphLinkDrag {
    /// A forma de onde a seta parte.
    pub(crate) from: VecPathId,
    /// A entidade do Morph cujo grafo esta seta vai integrar.
    pub(crate) morph: Entity,
    /// Onde o arrasto começou, em mundo — para o traço de pré-visualização.
    pub(crate) from_world: [f64; 2],
}

impl App {
    /// **O objecto de Morph em mãos**, se houver um selecionado.
    fn selected_morph(&self) -> Option<Entity> {
        let gfx = self.gfx.as_ref()?;
        let bits = gfx.hero_screen.as_ref()?.gizmo.iter_selected().find(|&b| {
            gfx.sim
                .world()
                .get::<VecMorph>(Entity::from_bits(b))
                .is_some()
        })?;
        Some(Entity::from_bits(bits))
    }

    /// **Down**: arma a seta a partir da forma sob o cursor.
    pub(crate) fn morph_link_down(&mut self, world: [f64; 2]) {
        let Some(morph) = self.selected_morph() else {
            return;
        };
        let Some(from) = self.shape_under_cursor(world) else {
            return;
        };
        self.morph_link_drag = Some(MorphLinkDrag {
            from,
            morph,
            from_world: world,
        });
    }

    /// **Up**: se soltou sobre OUTRA forma, a aresta nasce — pela porta única [`link_shapes`].
    ///
    /// ⛔ Soltar no VAZIO não cria nada: uma aresta precisa de dois lados, e uma ponta solta não é
    /// um estado. (O conector aceita a ponta solta; aqui ela não teria o que nomear.)
    pub(crate) fn morph_link_up(&mut self, world: [f64; 2]) {
        let Some(drag) = self.morph_link_drag.take() else {
            return;
        };
        let Some(to) = self.shape_under_cursor(world) else {
            return;
        };
        if let Some(gfx) = self.gfx.as_mut() {
            link_shapes(&mut gfx.sim, drag.morph, drag.from, to);
        }
    }

    /// **Cancela** a seta em voo — o Esc, e o que a troca de modo tem de chamar.
    pub(crate) fn morph_link_cancel(&mut self) {
        self.morph_link_drag = None;
    }
}

#[cfg(test)]
#[path = "morph_link_gesture_tests.rs"]
mod tests;
