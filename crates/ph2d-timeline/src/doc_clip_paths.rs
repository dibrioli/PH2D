//! **Onde a trajetória mora, e quem a resolve** — as portas por-clip do motion path
//! ([ADR-0141]).
//!
//! A trajetória mudou-se do BINDING para o CLIP em 2026-07-30 (Enio: *"cada clip novo deve
//! ser um branco e criar do zero seu próprio PATH"*), e este módulo é a consequência: duas
//! portas de LEITURA que respondem a perguntas diferentes e não podem ser colapsadas —
//!
//! - [`TimelineDoc::clip_path`] é a **crua**: o desenho e o hit-test do canvas perguntam
//!   por aqui, e um clip que não autorou trajetória não tem trajetória. Sem recuo, porque
//!   um recuo aqui É a alça fantasma que o report do Enio mostrou;
//! - [`TimelineDoc::path_for`] é a de **RESOLUÇÃO**: o avaliador (distância → ponto) e as
//!   conversões perguntam por aqui, e ela recua para o primeiro clip que tenha uma, porque
//!   sob o Arrange o clip ativo pode ser um que nunca autorou nenhuma.
//!
//! Mora ao lado de `doc.rs` (módulo filho por `#[path]`) porque alcança o campo privado
//! `clips`, e porque `doc.rs` está sob o teto de 700 LOC.
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use ph2d_anim::AnimTarget;

use crate::doc::TimelineDoc;

impl TimelineDoc {
    /// **A trajetória que o clip `clip` autora para `target`** — a leitura crua, sem
    /// fallback nenhum. É esta que o desenho e o hit-test do canvas fazem: um clip que
    /// não autorou trajetória **não tem** trajetória, e por isso não pode mostrar alça
    /// nem oferecer curva a agarrar (`NamedClip::paths`).
    #[must_use]
    pub fn clip_path(&self, clip: usize, target: AnimTarget) -> Option<&crate::MotionPath> {
        self.clips.get(clip)?.paths.get(&target)
    }

    /// **A trajetória com que o AVALIADOR põe o objeto no lugar** — o clip ativo, e como
    /// recuo o primeiro clip que tenha uma.
    ///
    /// ⚠️ **O recuo é o que mantém a composição viva.** Sob o Arrange o clip ativo é o
    /// que o dropdown selecionou, e pode ser um que nunca autorou trajetória — sem o
    /// recuo, um documento inteiro autorado no clip 1 e composto no Arrange deixaria de
    /// mapear distância→ponto e o objeto pararia na origem.
    ///
    /// ⚠️ **E o limite fica NOMEADO, porque é decisão de produto e não descuido:** o
    /// blend compõe DISTÂNCIAS (`prop.rs`), que só têm significado sobre UMA curva, então
    /// um crossfade entre dois clips de trajetórias DIFERENTES percorre a que este door
    /// escolher. Compor PONTOS — a resposta certa para esse caso — é wave própria, e não
    /// é regressão: antes desta wave existia uma trajetória só, para o documento inteiro.
    #[must_use]
    pub fn path_for(&self, target: AnimTarget) -> Option<&crate::MotionPath> {
        self.clip_path(self.active_clip, target)
            .or_else(|| self.clips.iter().find_map(|c| c.paths.get(&target)))
    }

    /// **Instala a trajetória do clip `clip`** para `target` — a porta pública de quem
    /// monta um documento (as cenas de smoke, os fixtures). A autoria pelo gesto passa
    /// pelo [`crate::TimelineDoc::add_path_key`], que escreve no clip ATIVO.
    ///
    /// ⚠️ Nomeia o CLIP de propósito: desde 2026-07-30 a trajetória é do clip, e uma porta
    /// que a instalasse "no documento" já não teria a quem perguntar qual.
    pub fn set_clip_path(&mut self, clip: usize, target: AnimTarget, path: crate::MotionPath) {
        if let Some(c) = self.clips.get_mut(clip) {
            c.paths.insert(target, path);
        }
    }

    /// Remove a trajetória do clip `clip` para `target`.
    pub fn clear_clip_path(&mut self, clip: usize, target: AnimTarget) {
        if let Some(c) = self.clips.get_mut(clip) {
            c.paths.remove(&target);
        }
    }

    /// A trajetória do clip ATIVO, mutável — **a porta única de EDIÇÃO** (todo gesto de
    /// autoria de caminho escreve por aqui, e portanto no clip que está aberto).
    pub(crate) fn active_path_mut(&mut self, target: AnimTarget) -> Option<&mut crate::MotionPath> {
        self.clips[self.active_clip].paths.get_mut(&target)
    }

    /// Instala (ou substitui) a trajetória do clip ATIVO para `target`.
    pub(crate) fn set_active_path(&mut self, target: AnimTarget, path: crate::MotionPath) {
        self.clips[self.active_clip].paths.insert(target, path);
    }

    /// Remove a trajetória do clip ATIVO para `target`, devolvendo-a.
    pub(crate) fn take_active_path(&mut self, target: AnimTarget) -> Option<crate::MotionPath> {
        self.clips[self.active_clip].paths.remove(&target)
    }
}
