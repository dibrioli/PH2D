//! ⭐⭐⭐ **A secção PARALLAX: o instantâneo e o dreno** (plano 24, W7).
//!
//! ⛔⛔ **Ela mora na CRATE DA FAMÍLIA e não na shell**, pela mesma razão que trouxe as irmãs do
//! raio e do tween para cá: a catraca `the_shell_only_shrinks` está apertada, e a cura que ela
//! prescreve por escrito é **MOVER para `crates/ph2d-app-<família>`, nunca subir o número**.
//!
//! ⚠️ **As duas metades vivem juntas de propósito:** quem lê o mundo para o painel e quem escreve a
//! edição de volta fazem a MESMA tradução (quatro componentes ⇄ uma secção), e separá-los seria a
//! porta pela qual as duas divergem.
//!
//! # ⭐⭐ O instantâneo lê a CENA, e não só os componentes
//!
//! Duas colunas não vêm de campo nenhum: **há uma `GameCamera` na cena?** e **esta camada é
//! NEUTRA?**. A primeira é a razão nº 1 para nada se mexer (sem ela a fase não recebe rectângulo e
//! **nenhuma** camada anda) e a segunda é o valor de FÁBRICA do componente — *o artista anexa a
//! paralaxe, nada muda, e sem esta linha ele lê «a paralaxe não funciona» sobre um motor a
//! obedecer.*
//!
//! ⚠️⚠️ **O neutro é lido pela PORTA do motor** ([`ScrollFactor::e_neutro`]) e nunca por um
//! `k == [1,1]` escrito aqui: é a mesma função que o passe consulta para saltar a camada, logo as
//! duas respostas não podem divergir. A crate do painel não a pode chamar (ela vive abaixo do
//! `ph2d-ecs` no DAG), e é por isso que a resposta viaja no instantâneo.

use ph2d_ecs::{Entity, GameCamera, ScrollFactor, ScrollLimits, ScrollMotion, ScrollRepeat, World};
use ph2d_editor_core::parallax_edits::{InspectorParallaxInfo, ParallaxFieldEdit};

/// **O instantâneo.** `None` para quem não tem `ScrollFactor` (ADR-0166).
#[must_use]
pub fn build_parallax_info(
    world: &World,
    entity_bits: u64,
    selected_count: usize,
) -> Option<InspectorParallaxInfo> {
    let e = Entity::from_bits(entity_bits);
    let factor = world.get::<ScrollFactor>(e).copied()?;
    let repeat = world.get::<ScrollRepeat>(e).map(|r| r.tile);
    let motion = world.get::<ScrollMotion>(e).map(|m| m.velocity);
    let limits = world.get::<ScrollLimits>(e).map(|l| (l.min, l.max));
    // ⚠️ **A pergunta é «existe UMA», nunca «qual manda»** — a fase usa a activa, e uma câmera
    // inactiva ainda faz a cena ter uma para ligar. *Dizer «não há câmera» a quem tem uma desligada
    // manda-o criar a segunda.*
    let tem_camera_do_jogo = world.iter_entities().any(|x| x.contains::<GameCamera>());
    Some(InspectorParallaxInfo {
        entity_bits,
        factor: factor.k,
        repeat,
        motion,
        limits,
        tem_camera_do_jogo,
        e_neutra: factor.e_neutro(),
        selected_count,
    })
}

/// **O dreno.** `true` = alguma coisa mudou.
///
/// ⚠️ **Um bloco sem componente é um no-op silencioso**, e não um pânico nem um `insert`: a fileira
/// só é pintada quando o componente está lá, logo esta rota não é alcançável — e se o componente
/// for retirado entre o quadro que pintou e o quadro que despacha, a escrita certa é nenhuma.
/// ⛔ Anexá-lo aqui faria um arrasto criar um componente que o artista nunca pediu, que é
/// exactamente o que o doc do `apply_camera_edit` já proíbe para o `CameraLimits`.
pub fn apply_parallax_edit(world: &mut World, entity_bits: u64, edit: &ParallaxFieldEdit) -> bool {
    let e = Entity::from_bits(entity_bits);
    match edit {
        ParallaxFieldEdit::Factor(k) => {
            let Some(mut c) = world.get_mut::<ScrollFactor>(e) else {
                return false;
            };
            c.k = *k;
        }
        ParallaxFieldEdit::Repeat(t) => {
            let Some(mut c) = world.get_mut::<ScrollRepeat>(e) else {
                return false;
            };
            c.tile = *t;
        }
        ParallaxFieldEdit::Motion(v) => {
            let Some(mut c) = world.get_mut::<ScrollMotion>(e) else {
                return false;
            };
            c.velocity = *v;
        }
        ParallaxFieldEdit::LimitsMin(v) => {
            let Some(mut c) = world.get_mut::<ScrollLimits>(e) else {
                return false;
            };
            c.min = *v;
        }
        ParallaxFieldEdit::LimitsMax(v) => {
            let Some(mut c) = world.get_mut::<ScrollLimits>(e) else {
                return false;
            };
            c.max = *v;
        }
    }
    true
}

/// **O dreno de um quadro inteiro** — a forma das irmãs (`ray_inspector::apply_all`).
pub fn apply_all(sim: &mut ph2d_ecs::SimWorld, edits: &[(u64, ParallaxFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if apply_parallax_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}

#[cfg(test)]
#[path = "parallax_inspector_tests.rs"]
mod tests;
