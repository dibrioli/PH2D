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
//! Quatro colunas não vêm de campo nenhum: **a câmera do jogo está ACTIVA?** · **o dolly dela
//! ATRAVESSA esta camada?** · **o passe SALTA esta camada?** · e, por argumento da shell, **outro
//! motor conduz este objecto?** e **a pré-visualização está ligada?**. A primeira é a razão nº 1 para
//! nada se mexer (sem ela a fase não recebe rectângulo e **nenhuma** camada anda); o neutro é o valor
//! de FÁBRICA do componente — *o artista anexa a paralaxe, nada muda, e sem esta linha ele lê «a
//! paralaxe não funciona» sobre um motor a obedecer.* As outras três nasceram na auditoria 26.
//!
//! ⚠️⚠️ **O neutro é lido pela PORTA do motor** ([`ScrollFactor::e_neutro`]) e nunca por um
//! `k == [1,1]` escrito aqui: é a mesma função que o passe consulta para saltar a camada, logo as
//! duas respostas não podem divergir. A crate do painel não a pode chamar (ela vive abaixo do
//! `ph2d-ecs` no DAG), e é por isso que a resposta viaja no instantâneo.

use ph2d_ecs::{
    Entity, GameCamera, ScrollFactor, ScrollLimits, ScrollMotion, ScrollRepeat, SimWorld, World,
};
use ph2d_editor_core::parallax_edits::{CameraDoJogo, InspectorParallaxInfo, ParallaxFieldEdit};

/// **O instantâneo.** `None` para quem não tem `ScrollFactor` (ADR-0166).
///
/// `pre_visualizacao` é a vista do editor a mostrar a câmera do jogo; `outro_motor` é a resposta do
/// LEDGER (`PreviewDrive::drives_other_than`) — as duas vivem na shell e chegam por argumento.
///
/// ⭐ **A câmera é a ACTIVA, pela porta da lei** (`active_camera_of`) — a mesma que a fase e a ponte
/// usam. ⛔ A 1.ª redacção perguntava «existe UMA?» varrendo o mundo inteiro a cada quadro
/// (`529 µs` a 100 k entidades, auditoria 26 §2.8) e respondia `true` para uma câmera desligada.
#[must_use]
pub fn build_parallax_info(
    sim: &mut SimWorld,
    entity_bits: u64,
    selected_count: usize,
    pre_visualizacao: bool,
    outro_motor: bool,
) -> Option<InspectorParallaxInfo> {
    let e = Entity::from_bits(entity_bits);
    let world = sim.world_mut();
    let factor = world.get::<ScrollFactor>(e).copied()?;
    let repeat = world.get::<ScrollRepeat>(e).map(|r| r.tile);
    let motion_c = world.get::<ScrollMotion>(e).copied();
    let limits = world.get::<ScrollLimits>(e).map(|l| (l.min, l.max));
    let activa = ph2d_ecs::active_camera_of(world);
    let camera = match activa {
        Some(_) => CameraDoJogo::Activa,
        None if ph2d_ecs::camera_count(world) > 0 => CameraDoJogo::Desligada,
        None => CameraDoJogo::Nenhuma,
    };
    let dolly = activa
        .and_then(|c| world.get::<GameCamera>(c).map(|g| g.dolly))
        .unwrap_or(0.0);
    // ⚠️ **As duas respostas são as PORTAS do passe** — `escala` (a mesma que a ponte usa para
    // largar a camada) e o salto do neutro com a deriva inerte. Escritas aqui outra vez divergiriam.
    let e_neutra = factor.e_neutro() && motion_c.is_none_or(|m| m.e_inerte());
    let atravessa = factor.escala(dolly).is_none();
    Some(InspectorParallaxInfo {
        entity_bits,
        factor: factor.k,
        repeat,
        motion: motion_c.map(|m| m.velocity),
        limits,
        camera,
        atravessa,
        pre_visualizacao,
        outro_motor,
        e_neutra,
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
        // ⛔⛔ **O applier trava ao DOMÍNIO DA LEI, e é o único escritor** (auditoria 26, §2.6): a
        // caixa numérica do painel NÃO aplica a faixa ao digitar (só o arrasto e o stepper a
        // aplicam), e um Repeat `−5` ficava gravado sem efeito nenhum. ⚠️ O domínio e não a pista:
        // a pista é ergonomia do arrasto, e um `k = 5` digitado é uma camada legítima (cinco vezes
        // mais perto) — se o dolly a atravessar, o painel di-lo. Não-finito é recusado inteiro.
        ParallaxFieldEdit::Factor(k) => {
            if !finitos(*k) {
                return false;
            }
            let Some(mut c) = world.get_mut::<ScrollFactor>(e) else {
                return false;
            };
            c.k = *k;
        }
        ParallaxFieldEdit::Repeat(t) => {
            if !finitos(*t) {
                return false;
            }
            let Some(mut c) = world.get_mut::<ScrollRepeat>(e) else {
                return false;
            };
            // Um ladrilho negativo não existe — `0` é «não repete», a omissão da lei.
            c.tile = [t[0].max(0.0), t[1].max(0.0)];
        }
        ParallaxFieldEdit::Motion(v) => {
            if !finitos(*v) {
                return false;
            }
            let Some(mut c) = world.get_mut::<ScrollMotion>(e) else {
                return false;
            };
            c.velocity = *v;
        }
        ParallaxFieldEdit::LimitsMin(v) => {
            if !finitos(*v) {
                return false;
            }
            let Some(mut c) = world.get_mut::<ScrollLimits>(e) else {
                return false;
            };
            c.min = *v;
        }
        ParallaxFieldEdit::LimitsMax(v) => {
            if !finitos(*v) {
                return false;
            }
            let Some(mut c) = world.get_mut::<ScrollLimits>(e) else {
                return false;
            };
            c.max = *v;
        }
    }
    true
}

/// Os dois eixos são finitos — a entrada mínima de qualquer campo desta secção.
fn finitos(v: [f32; 2]) -> bool {
    v[0].is_finite() && v[1].is_finite()
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
