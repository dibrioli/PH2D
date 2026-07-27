//! **POSAR PELA JUNTA** (W-FK) — o gesto de cinemática DIRETA.
//!
//! Irmão do [`crate::body_pose`] em tudo que é fiação (o mesmo relógio parado, o
//! mesmo `Transform` autorado, o mesmo passo de undo, a mesma porta de escrita)
//! e o oposto dele em INTENÇÃO:
//!
//! | | a POSE (W-IK) | a JUNTA (W-FK) |
//! |---|---|---|
//! | o artista arrasta | a **ponta** da cadeia | **um elo** qualquer |
//! | o que ele pede | *ponha a mão ali* | *dobre o cotovelo assim* |
//! | quem se move | tudo entre a raiz e a ponta | o elo e os **descendentes** |
//! | como | um solver amortecido | geometria exata |
//!
//! Nenhuma substitui a outra, e é por isso que todo pacote de animação carrega
//! as duas: a IK acerta um ALVO e a FK autora um ÂNGULO. O artista alterna entre
//! elas dentro do mesmo plano.
//!
//! # As condições, e a que NÃO está aqui
//!
//! 1. **O modo de joint em mãos é o FK** — `JointTool::gesture()`, a porta única
//!    (e é ela que faz o **Alt** suprimir o gesto: Alt significa *leve o rig
//!    inteiro*, que é um arrasto e não uma pose).
//! 2. **O relógio está PARADO.** O resultado é `Transform` autorado, e com o
//!    relógio andando o `readback` o sobrescreveria no mesmo frame.
//! 3. **Há uma junta com grau de liberdade acima do corpo pego** — respondido
//!    pelo `fk_begin` da ponte, não por uma cópia da regra aqui.
//! 4. **A trava é honrada** (`Locked`), como em todo gesto de autoria.
//!
//! ⚠️ O toggle `Physics` do transporte **não** é condição, exatamente como na
//! IK: este gesto não dá passo nenhum no solver, e exigi-lo obrigaria o artista
//! a armar a simulação para *não* simular.
//!
//! # A escrita é a MESMA porta da IK
//!
//! [`crate::body_pose::write_world_pose`] converte mundo → local contra o pai,
//! preserva escala e skew e recusa o não-finito. Uma segunda conversão aqui
//! divergiria no primeiro corpo parenteado — o defeito que o W5 levou quatro
//! waves para encontrar porque toda fixture usava corpo-raiz.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::PhysicsBridge;

use crate::App;

/// **A porta única do press.** `true` = a FK pegou, e o chamador NÃO deve abrir
/// arrasto de gizmo nem mexer na seleção.
///
/// Mesma divisão do `body_pose::take_pose` e do `body_grab::take_hold`: as
/// condições que dependem do relógio e da ferramenta chegam como argumento (é o
/// chamador que as vê), a estrutural é da ponte. É assim que a decisão inteira
/// fica testável sem janela.
pub(crate) fn take_fk(
    physics: &mut PhysicsBridge,
    sim: &SimWorld,
    is_fk: bool,
    entity: Entity,
    cursor: [f32; 2],
    playing: bool,
) -> bool {
    if playing || !is_fk {
        return false;
    }
    physics.fk_begin(sim, entity, cursor)
}

impl App {
    /// **O elo segue o cursor em torno da própria junta.** No-op sem sessão,
    /// então roda em todo Move ao lado dos outros `advance_*`.
    ///
    /// Marca `any_input_this_frame` pelo mesmo motivo do irmão da IK: este gesto
    /// AUTORA a cena, então o diff de undo tem de ver o frame em que a pose
    /// mudou.
    pub(crate) fn advance_body_fk(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        if !gfx.physics.is_posing_fk() {
            return;
        }
        let window = gfx.surface.size();
        let world = gfx.camera.screen_to_world(self.last_pointer, window);
        let poses = gfx.physics.fk_move(world);
        for (e, translation, rotation) in poses {
            crate::body_pose::write_world_pose(&mut gfx.sim, e, translation, rotation);
        }
        self.any_input_this_frame = true;
    }

    /// Encerra o gesto. Idempotente — soltar sem ter posado é o caso de quase
    /// todo release do app, e é por isso que ele mora no topo do handler junto
    /// dos irmãos.
    pub(crate) fn release_body_fk(&mut self) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.physics.fk_end();
        }
    }
}

#[cfg(test)]
#[path = "body_fk_tests.rs"]
mod tests;
