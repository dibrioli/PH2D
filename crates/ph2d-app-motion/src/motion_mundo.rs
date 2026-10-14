//! **O mundo que a cena publica para o Motion** (ADR-0186 §Emenda W3, «A fronteira»): os caminhos do
//! Vetor por nome, os objectos e a aparência deles, o cursor e a vista ([`publica_o_mundo`]); as
//! entradas do jogo; e os objectos Motion vivos (origem e visibilidade, da manutenção). Um produtor
//! — a porta do editor, uma vez por quadro, antes do `dispatch` —, e cada palco de uma macro lê o
//! mesmo retrato ([`MundoDaCena`], [`MotionState::recebe_o_mundo`]): o que a cena resolveu uma vez
//! (o atlas, as texturas cozidas, as afins) não se resolve outra vez por palco.

use std::collections::BTreeMap;

use ph2d_ecs::{MotionInputs, SimWorld};
use ph2d_editor_core::screens::layout::CenterSplit;
use ph2d_nodegraph::external::All;

use crate::motion_bridge::Appearance;
use crate::motion_objeto::Dono;
use crate::motion_state::MotionState;

/// O que a cena entrega neste quadro — tudo o que a porta precisa ter em mão ao mesmo tempo.
pub struct Cena<'a> {
    pub sim: &'a mut SimWorld,
    pub vec_scene: &'a ph2d_vec_scene::VecScene,
    pub entidades: &'a ph2d_vec_entities::entities::VecEntityMap,
    pub xforms: &'a ph2d_vec_scene::VecXforms,
    /// Os bits do objecto seleccionado (a forma do «Use Selected Path»).
    pub selecao: Option<u64>,
    pub aparencia: Appearance<'a>,
    /// O tempo do playhead (o canal deslocado resolve um param conduzido num instante).
    pub segundos: f64,
    pub camera: &'a ph2d_render::Camera2d,
    pub cursor: (f32, f32),
    pub split: CenterSplit,
    pub janela: ph2d_host::WindowSize,
}

/// **O retrato do mundo publicado**, que qualquer `MotionState` pode receber.
#[derive(Clone, Debug, Default)]
pub struct MundoDaCena {
    /// A tabela de externos que a cena escreveu (sem o que o cozimento de um documento lhe junta).
    pub externos: All,
    pub jogo: MotionInputs,
    pub donos: BTreeMap<u64, Dono>,
}

/// **A porta do editor**: publica o mundo em `motion` — as formas (limpam a tabela), os objectos (e
/// as entradas do jogo), os valores do editor, por esta ordem — e guarda o retrato em
/// `motion.mundo`. Corre depois da manutenção dos objectos Motion (`motion_objeto::upkeep`, que
/// decide os donos) e antes do `dispatch`, que coze por cima.
pub fn publica_o_mundo(motion: &mut MotionState, cena: Cena<'_>) {
    crate::motion_bridge::publish_shapes(
        motion,
        cena.sim,
        cena.vec_scene,
        cena.entidades,
        cena.xforms,
        cena.selecao,
    );
    crate::motion_bridge::publish_objects(motion, cena.sim, cena.aparencia, cena.segundos);
    crate::motion_bridge::publish_editor_inputs(
        motion,
        cena.camera,
        cena.cursor,
        cena.split,
        cena.janela,
    );
    motion.mundo = MundoDaCena::de(motion);
}

impl MundoDaCena {
    /// O que está publicado em `motion` agora — o retrato só vale entre a publicação e o cozimento,
    /// que junta à tabela o que o documento gera.
    #[must_use]
    pub fn de(motion: &MotionState) -> Self {
        Self {
            externos: motion.pump.cook.externals().clone(),
            jogo: motion.entradas.jogo.clone(),
            donos: motion.objeto.donos.clone(),
        }
    }
}

impl MotionState {
    /// **Recebe o mundo publicado** — o que a porta do editor deu ao seu `MotionState`, no estado em
    /// que ela o deixou antes de cozer.
    pub fn recebe_o_mundo(&mut self, mundo: &MundoDaCena) {
        self.pump.cook.clear_externals();
        for (nome, e) in &mundo.externos {
            self.pump.cook.set_external(nome.clone(), e.value.clone());
        }
        self.entradas.jogo.clone_from(&mundo.jogo);
        self.objeto.donos.clone_from(&mundo.donos);
    }
}

#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_mundo_tests.rs"]
mod tests;
