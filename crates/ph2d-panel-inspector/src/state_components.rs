//! **Os instantâneos das secções da FILA DO TOP-20** — irmão do [`super::state`] por `#[path]`.
//!
//! ⚠️ **O corte foi imposto pelo tecto de LOC dos painéis (600) e está certo por RESPONSABILIDADE:**
//! o ficheiro-mãe guarda o estado VIVO do painel (a rolagem, o que está aberto, o que o rato toca);
//! este guarda as **fotografias que a shell publica** das secções que a fila do TOP-20 trouxe —
//! timer, tabela de acções, áudio, câmera, fábrica, mover de vista de cima, projéctil.
//!
//! ⚠️⚠️ **E esta linha dizia o CONTRÁRIO até 2026-09-19:** *«os `thread_local!` continuam no
//! ficheiro-mãe, e de propósito … movê-los partiria a ordem de declaração de um bloco que já tem dez
//! entradas»*. Aquele bloco chegou a **trinta** entradas e o ficheiro-mãe voltou a passar o tecto ao
//! ganhar o RAIO (`607` contra `600`) ⇒ as **treze** que tinham acessor aqui vieram para cá, com as
//! duas que já cá estavam. *Uma nota que descreve a casa de outra época lê-se exactamente como uma
//! que descreve a de agora* — e a linha que ela defendia era a que fazia uma secção nova escrever em
//! DOIS ficheiros, com o segundo a ser só uma célula numa lista.
//!
//! ⛔ **Nunca subir o número do cap: ele só desce.**

use ph2d_editor_core::parallax_edits::InspectorParallaxInfo;
use ph2d_editor_core::mesh3d_edits::InspectorMesh3dInfo;
use ph2d_editor_core::particles_edits::InspectorParticlesInfo;
use ph2d_editor_core::path_follow_edits::InspectorPathFollowInfo;
use ph2d_editor_core::projectile_edits::InspectorProjectileInfo;
use ph2d_editor_core::ray_edits::InspectorRayInfo;
use ph2d_editor_core::screens::hero::{
    InspectorActionInfo, InspectorAudioInfo, InspectorCameraInfo, InspectorFactoryInfo,
    InspectorTimerInfo,
};
use ph2d_editor_core::script_edits::InspectorScriptInfo;
use ph2d_editor_core::statemachine_edits::InspectorStateMachineInfo;
use ph2d_editor_core::topdown_edits::InspectorTopDownInfo;
use ph2d_editor_core::tween_edits::InspectorTweenInfo;
use ph2d_editor_core::vida_edits::InspectorVidaInfo;
use ph2d_editor_core::weapon_edits::InspectorWeaponInfo;

pub fn set_current_inspector_timer(info: Option<InspectorTimerInfo>) {
    CURRENT_INSPECTOR_TIMER.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_timer() -> Option<InspectorTimerInfo> {
    CURRENT_INSPECTOR_TIMER.with(|c| c.borrow().clone())
}

/// ⭐ **O instantâneo do TWEEN** (suplente #22) — irmão do do timer, e pela mesma razão.
pub fn set_current_inspector_tween(info: Option<InspectorTweenInfo>) {
    CURRENT_INSPECTOR_TWEEN.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_tween() -> Option<InspectorTweenInfo> {
    CURRENT_INSPECTOR_TWEEN.with(|c| c.borrow().clone())
}

/// ⭐ **O instantâneo do SEGUIDOR DE CAMINHO** (suplente #23) — irmão do do tween.
pub fn set_current_inspector_path_follow(info: Option<InspectorPathFollowInfo>) {
    CURRENT_INSPECTOR_PATH_FOLLOW.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_path_follow() -> Option<InspectorPathFollowInfo> {
    CURRENT_INSPECTOR_PATH_FOLLOW.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_action(info: Option<InspectorActionInfo>) {
    CURRENT_INSPECTOR_ACTION.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_action() -> Option<InspectorActionInfo> {
    CURRENT_INSPECTOR_ACTION.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_audio(info: Option<InspectorAudioInfo>) {
    CURRENT_INSPECTOR_AUDIO.with(|c| *c.borrow_mut() = info);
}

pub fn set_current_inspector_factory(info: Option<InspectorFactoryInfo>) {
    CURRENT_INSPECTOR_FACTORY.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_factory() -> Option<InspectorFactoryInfo> {
    CURRENT_INSPECTOR_FACTORY.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do MOVER DE VISTA DE CIMA (TOP-20 #13) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_topdown(info: Option<InspectorTopDownInfo>) {
    CURRENT_INSPECTOR_TOPDOWN.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_topdown() -> Option<InspectorTopDownInfo> {
    CURRENT_INSPECTOR_TOPDOWN.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_projectile(info: Option<InspectorProjectileInfo>) {
    CURRENT_INSPECTOR_PROJECTILE.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_projectile() -> Option<InspectorProjectileInfo> {
    CURRENT_INSPECTOR_PROJECTILE.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do RAIO (suplente #21) — a shell escreve-o todo o quadro, porque ele carrega a
/// leitura VIVA e não só os campos.
pub fn set_current_inspector_ray(info: Option<InspectorRayInfo>) {
    CURRENT_INSPECTOR_RAY.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_ray() -> Option<InspectorRayInfo> {
    CURRENT_INSPECTOR_RAY.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot da PARALAXE (plano 24) — a shell escreve-o todo o quadro, porque ele carrega uma
/// leitura do MUNDO (*há câmera do jogo?*) e não só os campos do objecto.
pub fn set_current_inspector_parallax(info: Option<InspectorParallaxInfo>) {
    CURRENT_INSPECTOR_PARALLAX.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_parallax() -> Option<InspectorParallaxInfo> {
    CURRENT_INSPECTOR_PARALLAX.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot da ARMA — a shell escreve-o todo o quadro, porque ele carrega a munição VIVA e
/// não só os campos.
pub fn set_current_inspector_weapon(info: Option<InspectorWeaponInfo>) {
    CURRENT_INSPECTOR_WEAPON.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_weapon() -> Option<InspectorWeaponInfo> {
    CURRENT_INSPECTOR_WEAPON.with(|c| c.borrow().clone())
}

/// Publica o instantâneo das secções HEALTH e DAMAGE (plano 28, W3).
pub fn set_current_inspector_vida(info: Option<InspectorVidaInfo>) {
    CURRENT_INSPECTOR_VIDA.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_vida() -> Option<InspectorVidaInfo> {
    CURRENT_INSPECTOR_VIDA.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do CATAVENTO — a shell escreve-o todo o quadro, porque ele carrega o *«já foi
/// assado?»* e não só os campos.
pub fn set_current_inspector_mesh3d(info: Option<InspectorMesh3dInfo>) {
    CURRENT_INSPECTOR_MESH3D.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_mesh3d() -> Option<InspectorMesh3dInfo> {
    CURRENT_INSPECTOR_MESH3D.with(|c| *c.borrow())
}

/// ⭐ O snapshot do CÉREBRO (TOP-20 #15) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_statemachine(info: Option<InspectorStateMachineInfo>) {
    CURRENT_INSPECTOR_STATEMACHINE.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_statemachine() -> Option<InspectorStateMachineInfo> {
    CURRENT_INSPECTOR_STATEMACHINE.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do HUD (TOP-20 #20) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_hud(info: Option<ph2d_editor_core::hud_edits::InspectorHudInfo>) {
    CURRENT_INSPECTOR_HUD.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_hud() -> Option<ph2d_editor_core::hud_edits::InspectorHudInfo> {
    CURRENT_INSPECTOR_HUD.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot da CUTSCENE (TOP-20 #19) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_sequence(
    info: Option<ph2d_editor_core::sequence_edits::InspectorSequenceInfo>,
) {
    CURRENT_INSPECTOR_SEQUENCE.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_sequence()
-> Option<ph2d_editor_core::sequence_edits::InspectorSequenceInfo> {
    CURRENT_INSPECTOR_SEQUENCE.with(|c| c.borrow().clone())
}

thread_local! {
    /// ⭐⭐⭐ **O snapshot da secção COUNTER WATCH** — a vigia do contador.
    ///
    /// ⚠️ **Ela mora AQUI e não no [`crate::state`], ao contrário das irmãs mais velhas**, e o
    /// corte foi imposto pelo tecto de 600 LOC daquele ficheiro (ele chegou a `601`). ⭐ É o
    /// certo por responsabilidade: aquele módulo é sobre o que o PAINEL lembra entre quadros, e
    /// isto é um instantâneo que a shell escreve — que é exactamente o assunto deste ficheiro,
    /// onde o `set`/`current` dela já viviam.
    static CURRENT_INSPECTOR_COUNTER_WATCH:
        std::cell::RefCell<
            Option<ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo>,
        > = const { std::cell::RefCell::new(None) };

    /// ⭐ O snapshot do GATILHO (suplente #24) — a shell escreve-o todo o quadro.
    static CURRENT_INSPECTOR_ACTION_TRIGGER:
        std::cell::RefCell<
            Option<ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo>,
        > = const { std::cell::RefCell::new(None) };

    /// ⭐ O snapshot do ABANÃO DA CÂMERA (suplente #25) — a shell escreve-o todo o quadro.
    static CURRENT_INSPECTOR_SHAKE:
        std::cell::RefCell<Option<ph2d_editor_core::shake_edits::InspectorShakeInfo>> =
        const { std::cell::RefCell::new(None) };

    /// ⭐ O snapshot do EMISSOR DE ABANÃO (suplente #25) — a shell escreve-o todo o quadro.
    static CURRENT_INSPECTOR_EMITTER:
        std::cell::RefCell<Option<ph2d_editor_core::shake_edits::InspectorEmitterInfo>> =
        const { std::cell::RefCell::new(None) };

    // ⭐⭐⭐ **E AS TREZE IRMÃS MAIS VELHAS, que estavam no [`crate::state`]** — o corte é o MESMO
    // que a vigia do contador fez acima, e pela mesma razão: o `state.rs` voltou a passar o tecto
    // de 600 LOC (chegou a `607` ao ganhar o RAIO), e a linha já estava desenhada pelo nome deste
    // ficheiro — *o snapshot e o acessor dele moram juntos*.
    //
    // ⛔ **Curado por CORTE, nunca por uma entrada no `FILE_OVERAGE_OK`** — aquela lista está
    // VAZIA, e é isso que a torna a catraca mais apertada que existe.
    //
    // ⚠️ Antes disto, uma secção nova escrevia em DOIS ficheiros e o segundo era só uma célula
    // numa lista — que é exactamente como um deles envelhece sem ninguém ver.

    /// TIMERS — o snapshot da entidade selecionada. `RefCell` pela mesma razão da §11.
    static CURRENT_INSPECTOR_TIMER:
        std::cell::RefCell<Option<InspectorTimerInfo>> = const { std::cell::RefCell::new(None) };

    /// TWEEN — o snapshot da entidade selecionada (suplente #22).
    static CURRENT_INSPECTOR_TWEEN:
        std::cell::RefCell<Option<InspectorTweenInfo>> = const { std::cell::RefCell::new(None) };

    /// PATH FOLLOW — o snapshot da entidade selecionada (suplente #23).
    static CURRENT_INSPECTOR_PATH_FOLLOW:
        std::cell::RefCell<Option<InspectorPathFollowInfo>> =
        const { std::cell::RefCell::new(None) };

    /// SIGNAL ACTIONS — o snapshot da entidade selecionada.
    static CURRENT_INSPECTOR_ACTION:
        std::cell::RefCell<Option<InspectorActionInfo>> = const { std::cell::RefCell::new(None) };

    /// AUDIO — o snapshot da entidade selecionada (TOP-20 #4).
    static CURRENT_INSPECTOR_AUDIO:
        std::cell::RefCell<Option<InspectorAudioInfo>> = const { std::cell::RefCell::new(None) };

    /// CAMERA — o snapshot da entidade selecionada (TOP-20 #7).
    static CURRENT_INSPECTOR_CAMERA:
        std::cell::RefCell<Option<InspectorCameraInfo>> = const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot das secções FACTORY e LIFECYCLE** (TOP-20 #11 e #12).
    static CURRENT_INSPECTOR_FACTORY:
        std::cell::RefCell<Option<InspectorFactoryInfo>> = const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção TOP-DOWN PLAYER** (TOP-20 #13).
    static CURRENT_INSPECTOR_TOPDOWN:
        std::cell::RefCell<Option<InspectorTopDownInfo>> = const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção PROJECTILE MOTION** (TOP-20 #14).
    static CURRENT_INSPECTOR_PROJECTILE:
        std::cell::RefCell<Option<InspectorProjectileInfo>> =
        const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção RAY SENSOR** (suplente #21).
    ///
    /// ⚠️ Ele carrega a LEITURA VIVA (o que o raio vê agora), logo a shell reescreve-o **todo o
    /// quadro** — ao contrário dos campos, que só mudam quando alguém os edita.
    static CURRENT_INSPECTOR_RAY:
        std::cell::RefCell<Option<InspectorRayInfo>> = const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção PARALLAX** (plano 24).
    ///
    /// ⚠️ Ele carrega uma leitura do MUNDO — *há uma câmera do jogo na cena?* —, que é a primeira
    /// das duas razões para nada se mexer; logo a shell reescreve-o **todo o quadro**.
    static CURRENT_INSPECTOR_PARALLAX:
        std::cell::RefCell<Option<InspectorParallaxInfo>> = const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção WEAPON.**
    ///
    /// ⚠️ Ele carrega a MUNIÇÃO VIVA (quantas balas ela tem agora, e se está a recarregar), logo a
    /// shell reescreve-o **todo o quadro** — ao contrário dos campos, que só mudam quando alguém os
    /// edita.
    static CURRENT_INSPECTOR_WEAPON:
        std::cell::RefCell<Option<InspectorWeaponInfo>> = const { std::cell::RefCell::new(None) };
    /// ⭐⭐⭐ **O snapshot das secções HEALTH e DAMAGE** (plano 28, W3).
    static CURRENT_INSPECTOR_VIDA:
        std::cell::RefCell<Option<InspectorVidaInfo>> = const { std::cell::RefCell::new(None) };
    /// ⭐⭐⭐ **O snapshot da secção LIVE MESH** (o CATAVENTO, `docs/3D/02.2` rota B).
    ///
    /// ⚠️ Ele carrega **se o sprite está ASSADO**, que não vem de campo nenhum — vem do mapa de
    /// formas assadas, onde a shell vive. É dele que sai a queixa que torna a secção uma ferramenta
    /// em vez de três números.
    static CURRENT_INSPECTOR_MESH3D:
        std::cell::RefCell<Option<InspectorMesh3dInfo>> = const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção STATE MACHINE** (TOP-20 #15).
static CURRENT_INSPECTOR_STATEMACHINE:
    std::cell::RefCell<Option<InspectorStateMachineInfo>> =
    const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção SCRIPT** (TOP-20 #16).
    static CURRENT_INSPECTOR_SCRIPT: std::cell::RefCell<Option<InspectorScriptInfo>> =
        const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção HUD** (TOP-20 #20).
    static CURRENT_INSPECTOR_HUD:
        std::cell::RefCell<Option<ph2d_editor_core::hud_edits::InspectorHudInfo>> =
        const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção SEQUENCE** (TOP-20 #19).
    static CURRENT_INSPECTOR_SEQUENCE:
        std::cell::RefCell<Option<ph2d_editor_core::sequence_edits::InspectorSequenceInfo>> =
        const { std::cell::RefCell::new(None) };

    /// ⭐⭐⭐ **O snapshot da secção PARTICLES** (TOP-20 #18).
    static CURRENT_INSPECTOR_PARTICLES:
        std::cell::RefCell<Option<InspectorParticlesInfo>> =
        const { std::cell::RefCell::new(None) };
}

/// ⭐ O snapshot da VIGIA DO CONTADOR — a shell escreve-o todo o quadro.
pub fn set_current_inspector_counter_watch(
    info: Option<ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo>,
) {
    CURRENT_INSPECTOR_COUNTER_WATCH.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_counter_watch()
-> Option<ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo> {
    CURRENT_INSPECTOR_COUNTER_WATCH.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do GATILHO — a shell escreve-o todo o quadro.
pub fn set_current_inspector_action_trigger(
    info: Option<ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo>,
) {
    CURRENT_INSPECTOR_ACTION_TRIGGER.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_action_trigger()
-> Option<ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo> {
    CURRENT_INSPECTOR_ACTION_TRIGGER.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do ABANÃO DA CÂMERA (suplente #25) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_shake(
    info: Option<ph2d_editor_core::shake_edits::InspectorShakeInfo>,
) {
    CURRENT_INSPECTOR_SHAKE.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_shake() -> Option<ph2d_editor_core::shake_edits::InspectorShakeInfo>
{
    CURRENT_INSPECTOR_SHAKE.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do EMISSOR DE ABANÃO (suplente #25) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_emitter(
    info: Option<ph2d_editor_core::shake_edits::InspectorEmitterInfo>,
) {
    CURRENT_INSPECTOR_EMITTER.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_emitter()
-> Option<ph2d_editor_core::shake_edits::InspectorEmitterInfo> {
    CURRENT_INSPECTOR_EMITTER.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do EMISSOR DE PARTÍCULAS (TOP-20 #18) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_particles(info: Option<InspectorParticlesInfo>) {
    CURRENT_INSPECTOR_PARTICLES.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_particles() -> Option<InspectorParticlesInfo> {
    CURRENT_INSPECTOR_PARTICLES.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do SCRIPT (TOP-20 #16) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_script(info: Option<InspectorScriptInfo>) {
    CURRENT_INSPECTOR_SCRIPT.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_script() -> Option<InspectorScriptInfo> {
    CURRENT_INSPECTOR_SCRIPT.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_camera(info: Option<InspectorCameraInfo>) {
    CURRENT_INSPECTOR_CAMERA.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_camera() -> Option<InspectorCameraInfo> {
    CURRENT_INSPECTOR_CAMERA.with(|c| c.borrow().clone())
}

pub(crate) fn current_inspector_audio() -> Option<InspectorAudioInfo> {
    CURRENT_INSPECTOR_AUDIO.with(|c| c.borrow().clone())
}
