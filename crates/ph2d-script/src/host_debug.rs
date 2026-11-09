//! ⭐ **O depurador visto do `ScriptHost`** (plano 13 Onda D, decisão 11 do dono): os pontos de
//! parada por script, Pause on Error, os valores ao vivo SÓ do script da aba aberta (de UM objecto:
//! o seleccionado se corre esse script, senão o 1.º pela identidade) e o Continue / Next line,
//! que a aba PEDE e quem tem as portas da cena executa ([`ScriptHost::debug_frame_with`]).

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_script_debug::LineLive;
use ph2d_script_doc::ScriptId;

use crate::ScriptHost;
use crate::lane::SceneDoors;
use crate::scene::{DebugStop, SceneReport};

/// A configuração do depurador — sobrevive à reconstrução da VM.
pub(crate) struct DebugConfig {
    breakpoints: BTreeMap<ScriptId, Vec<u32>>,
    pause_on_error: bool,
    /// O script da aba aberta e o objecto seleccionado.
    watch: Option<(ScriptId, Option<Entity>)>,
    /// De quem são os valores ao vivo agora.
    target: Option<(ScriptId, Entity)>,
    /// Um Continue (`false`) ou Next line (`true`) por executar.
    resume: Option<bool>,
    /// Os sinais que chegaram com o jogo parado no depurador: ouvem-se no próximo quadro que corre.
    pub(crate) heard_later: Vec<(String, Option<u64>)>,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            breakpoints: BTreeMap::new(),
            pause_on_error: true,
            watch: None,
            target: None,
            resume: None,
            heard_later: Vec::new(),
        }
    }
}

impl ScriptHost {
    /// Os pontos de parada de `id` (linhas 1-based do texto). Vazio = nenhum.
    pub fn set_breakpoints(&mut self, id: ScriptId, mut lines: Vec<u32>) {
        lines.sort_unstable();
        lines.dedup();
        if lines.is_empty() {
            self.debug.breakpoints.remove(&id);
        } else {
            self.debug.breakpoints.insert(id, lines);
        }
        self.arm_now();
    }

    /// Os pontos de parada pedidos para `id`.
    #[must_use]
    pub fn breakpoints(&self, id: ScriptId) -> &[u32] {
        self.debug.breakpoints.get(&id).map_or(&[], Vec::as_slice)
    }

    /// As linhas onde ficaram armados (a próxima linha com código; vazio se o script não carregou).
    #[must_use]
    pub fn armed_lines(&self, id: ScriptId) -> &[u32] {
        self.scene.armed_lines(id)
    }

    #[must_use]
    pub fn pause_on_error(&self) -> bool {
        self.debug.pause_on_error
    }

    pub fn set_pause_on_error(&mut self, on: bool) {
        self.debug.pause_on_error = on;
        if let Some(d) = self.debugger() {
            d.set_pause_on_error(on);
        }
    }

    /// Os valores ao vivo: `script` = o da aba aberta (`None` = nenhum), `selected` = o objecto
    /// seleccionado na cena.
    pub fn set_live_watch(&mut self, script: Option<ScriptId>, selected: Option<Entity>) {
        self.debug.watch = script.map(|s| (s, selected));
    }

    /// De que objecto são os valores ao vivo (o seleccionado ou o 1.º que corre o script).
    #[must_use]
    pub fn live_object(&self) -> Option<Entity> {
        self.debug.target.map(|(_, e)| e)
    }

    /// O que cada linha de `id` fez no último quadro (vazio se `id` não é o vigiado).
    #[must_use]
    pub fn live_values(&self, id: ScriptId) -> BTreeMap<u32, LineLive> {
        match (self.debug.target, self.debugger()) {
            (Some((s, _)), Some(d)) if s == id => d.live(),
            _ => BTreeMap::new(),
        }
    }

    /// Há um objecto parado? O jogo inteiro congela até ao Continue.
    #[must_use]
    pub fn is_held(&self) -> bool {
        self.scene.is_held()
    }

    /// Onde o jogo está parado.
    #[must_use]
    pub fn debug_stop(&self) -> Option<DebugStop> {
        self.scene.debug_stop(self.runtime.lua())
    }

    /// A aba pede **Continue**; executa-o o próximo [`Self::debug_frame_with`].
    pub fn request_continue(&mut self) {
        if self.is_held() {
            self.debug.resume = Some(false);
        }
    }

    /// A aba pede **Next line**.
    pub fn request_next_line(&mut self) {
        if self.is_held() {
            self.debug.resume = Some(true);
        }
    }

    /// Quantas paragens bateram onde não se pode parar (dentro de uma função C) desde a última
    /// leitura.
    pub fn take_blocked_stops(&mut self) -> u32 {
        self.debugger().map_or(0, |d| d.take_blocked())
    }

    /// ⭐ **No início de cada quadro, com as portas da cena**: executa o Continue / Next line pedido.
    pub fn debug_frame_with(
        &mut self,
        sim: &mut SimWorld,
        doors: &mut dyn SceneDoors,
    ) -> Option<SceneReport> {
        let step = self.debug.resume.take()?;
        let mut r = self
            .scene
            .resume_held(self.runtime.lua(), &self.deadline, sim, doors, step);
        self.scene.absorb_error_stop(&mut r);
        self.forget_the_dead(&r);
        Some(r)
    }

    /// O jogo parou no depurador antes de ouvir `signals`: guardam-se para o próximo quadro que corre
    /// ([`Self::scene_hear_with`] entrega-os primeiro) — o cursor de quem os leu já passou por eles.
    pub fn defer_heard(&mut self, signals: &[(&str, Option<u64>)]) {
        self.debug
            .heard_later
            .extend(signals.iter().map(|(n, de)| ((*n).to_owned(), *de)));
    }

    /// Fecha o quadro desenhado: o que o vigiado fez nele passa a ser o mostrado.
    pub fn debug_end_frame(&mut self) {
        if let Some(d) = self.debugger() {
            d.end_frame();
        }
    }

    /// Depois do `sync` da cena: arma os pontos de parada e escolhe o vigiado.
    pub(crate) fn debug_after_sync(&mut self) {
        self.arm_now();
        let target = self
            .debug
            .watch
            .and_then(|(s, sel)| Some((s, self.scene.live_target(s, sel)?)));
        if target != self.debug.target {
            self.debug.target = target;
            if let Some(d) = self.debugger() {
                d.unwatch();
                d.set_target(target.map(|(s, e)| (s.0, e.to_bits())));
            }
        }
        if let Some(d) = self.debugger() {
            d.set_pause_on_error(self.debug.pause_on_error);
        }
    }

    /// As corrotinas da corrida saem (rebobinar, outro projecto): o depurador esquece-as, e os
    /// sinais guardados durante uma paragem também.
    pub(crate) fn forget_debug_threads(&mut self) {
        self.debug.heard_later.clear();
        if let Some(d) = self.debugger() {
            d.forget_all();
        }
    }

    fn arm_now(&mut self) {
        let lua = self.runtime.lua();
        self.scene.arm_breakpoints(lua, &self.debug.breakpoints);
    }

    fn debugger(&self) -> Option<ph2d_script_debug::Debugger> {
        crate::scene::debugger_of(self.runtime.lua())
    }
}
