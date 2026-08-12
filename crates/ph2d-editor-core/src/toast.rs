//! [`ToastQueue`] — non-modal notification stream (ADR-0023 §2).
//!
//! "Notificações flutuantes não-modais no topo do canvas
//! (ex.: 'Undo: Brush Stroke', 'Saved 2 s ago') — informam sem
//! interromper."
//!
//! - **Stack** at the top-center of the canvas (above all panels).
//! - **Auto-dismiss** after `Toast::ttl_s` (default 3 s, de relógio de PAREDE).
//! - **Live region** in the a11y tree (per ADR-0023 §10) so screen
//!   readers announce the latest toast without stealing focus.
//! - **Bounded queue** (32 entries) — runaway notifications get
//!   dropped silently. Per HR-9 backpressure principle (same as
//!   `ph2d_script::WriteQueue`).

use ph2d_a11y::{Live, Node, NodeBuilder, Role};
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ToastSeverity {
    Info,
    Success,
    Warning,
    ErrorState,
}

impl ToastSeverity {
    /// Mapping to the `accesskit::Live` priority. Errors interrupt
    /// (`Assertive`); everything else is polite.
    pub fn live_priority(self) -> Live {
        match self {
            Self::ErrorState => Live::Assertive,
            _ => Live::Polite,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub severity: ToastSeverity,
    /// Quanto tempo, em **SEGUNDOS**, este toast vive.
    pub ttl_s: f32,
    /// **Segundos** desde que o toast foi empurrado. O chamador chama `tick(dt)` uma vez por
    /// quadro com o `dt` de PAREDE; o toast remove-se quando `age_s >= ttl_s`.
    pub age_s: f32,
}

impl Toast {
    /// ⚠️ **Três segundos, e agora eles são mesmo três segundos.**
    ///
    /// Isto era `DEFAULT_TTL_FRAMES: u32 = 180 // 3 s @ 60 Hz` — uma contagem de QUADROS, que a
    /// 30 fps dava **6 s** e a 120 dava **1,5 s**. O mesmo repositório já tinha aprendido a lição
    /// um arquivo adiante, com o motivo escrito no comentário do `wall_dt`
    /// (`render_loop/mod.rs`: *"…which made the sprites race + jitter"*) — *o conhecimento existia
    /// no prédio e não tinha atravessado a porta*.
    pub const DEFAULT_TTL_S: f32 = 3.0;

    pub fn new(message: impl Into<String>, severity: ToastSeverity) -> Self {
        Self {
            message: message.into(),
            severity,
            ttl_s: Self::DEFAULT_TTL_S,
            age_s: 0.0,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Info)
    }
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Success)
    }
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Warning)
    }
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::ErrorState)
    }

    pub fn ttl_seconds(mut self, s: f32) -> Self {
        self.ttl_s = s;
        self
    }

    /// Build the AccessKit live-region node for this toast. Screen
    /// readers will announce the message according to severity.
    pub fn build_a11y(&self) -> Node {
        NodeBuilder::new(Role::GenericContainer)
            .label(&self.message)
            .live(self.severity.live_priority())
            .build()
    }
}

pub struct ToastQueue {
    inner: VecDeque<Toast>,
    cap: usize,
}

/// ⚠️ **`Default` delega ao `new()`, e o `derive` estava ERRADO.**
///
/// `usize::default()` é **0**, então uma fila derivada tinha capacidade zero — e o `push` dela
/// devolvia `false` e **descartava todo toast em silêncio**. O produto escapou por acidente
/// (`init.rs` chama `new()`), mas quarenta sítios constroem por `default()`, e o primeiro deles que
/// passasse a mostrar uma mensagem ao artista teria um sistema de avisos que nunca avisa, **sem um
/// erro, sem um warning e com todos os gates verdes**.
///
/// Uma fila de capacidade zero não tem uso legítimo: ela é um descartador com nome de fila.
impl Default for ToastQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastQueue {
    pub const DEFAULT_CAP: usize = 32;

    pub fn new() -> Self {
        Self::with_cap(Self::DEFAULT_CAP)
    }

    pub fn with_cap(cap: usize) -> Self {
        Self {
            inner: VecDeque::new(),
            cap,
        }
    }

    /// Push a toast. Returns false if the queue is full (silent drop).
    pub fn push(&mut self, toast: Toast) -> bool {
        if self.inner.len() >= self.cap {
            return false;
        }
        self.inner.push_back(toast);
        true
    }

    /// Anda `dt` **segundos de parede**; larga os expirados.
    ///
    /// ⚠️ **Segundos, nunca quadros** — ver [`Toast::DEFAULT_TTL_S`].
    pub fn tick(&mut self, dt: f32) {
        for t in &mut self.inner {
            t.age_s += dt;
        }
        while self.inner.front().is_some_and(|t| t.age_s >= t.ttl_s) {
            self.inner.pop_front();
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Toast> {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_iterate() {
        let mut q = ToastQueue::new();
        q.push(Toast::info("Saved"));
        q.push(Toast::warning("Slow load"));
        assert_eq!(q.len(), 2);
        let labels: Vec<&str> = q.iter().map(|t| t.message.as_str()).collect();
        assert_eq!(labels, vec!["Saved", "Slow load"]);
    }

    #[test]
    fn full_queue_drops_silently() {
        let mut q = ToastQueue::with_cap(2);
        assert!(q.push(Toast::info("a")));
        assert!(q.push(Toast::info("b")));
        assert!(!q.push(Toast::info("c"))); // dropped
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn tick_expires_old_toasts() {
        let mut q = ToastQueue::new();
        q.push(Toast::info("ephemeral").ttl_seconds(2.0 / 60.0));
        for _ in 0..3 {
            q.tick(1.0 / 60.0);
        }
        assert!(q.is_empty());
    }

    #[test]
    fn tick_keeps_fresh_toasts() {
        let mut q = ToastQueue::new();
        q.push(Toast::info("fresh").ttl_seconds(1.0));
        for _ in 0..30 {
            q.tick(1.0 / 60.0);
        }
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn error_severity_is_assertive() {
        assert_eq!(ToastSeverity::ErrorState.live_priority(), Live::Assertive);
        assert_eq!(ToastSeverity::Info.live_priority(), Live::Polite);
        assert_eq!(ToastSeverity::Warning.live_priority(), Live::Polite);
        assert_eq!(ToastSeverity::Success.live_priority(), Live::Polite);
    }

    #[test]
    fn convenience_constructors() {
        assert_eq!(Toast::info("x").severity, ToastSeverity::Info);
        assert_eq!(Toast::success("x").severity, ToastSeverity::Success);
        assert_eq!(Toast::warning("x").severity, ToastSeverity::Warning);
        assert_eq!(Toast::error("x").severity, ToastSeverity::ErrorState);
    }

    #[test]
    fn a11y_node_uses_live_region() {
        let t = Toast::error("Save failed");
        let n = t.build_a11y();
        assert_eq!(n.role(), Role::GenericContainer);
        assert_eq!(n.label(), Some("Save failed"));
        assert_eq!(n.live(), Some(Live::Assertive));
    }

    /// **Uma fila construída por `default()` ACEITA um toast.**
    ///
    /// ⚠️ Red-first: com o `#[derive(Default)]` que estava aqui, `cap` nascia **0** e o `push`
    /// devolvia `false` — uma fila que descarta tudo em silêncio. O oráculo é o `push`, não o
    /// `cap`: é o `push` que o chamador vê, e um teste sobre o campo interno passaria a mentir no
    /// dia em que a política de cheio mudar.
    #[test]
    fn a_default_queue_accepts_a_toast() {
        let mut q = ToastQueue::default();
        assert!(q.push(Toast::info("hello")), "a fila default DESCARTOU");
        assert_eq!(q.len(), 1);

        // E ela tem a mesma capacidade que a construída à mão — duas portas para "uma fila nova"
        // que discordassem dariam avisos que aparecem num caminho e somem no outro.
        let mut d = ToastQueue::default();
        let mut n = ToastQueue::new();
        for _ in 0..ToastQueue::DEFAULT_CAP {
            assert!(n.push(Toast::info("x")));
            assert!(d.push(Toast::info("x")));
        }
        assert_eq!(d.len(), n.len());
        assert!(!d.push(Toast::info("x")), "a default enche depois da new");
        assert!(!n.push(Toast::info("x")));
    }
}

#[cfg(test)]
mod wall_clock_tests {
    use super::*;

    /// ⭐ **TRÊS SEGUNDOS SÃO TRÊS SEGUNDOS A QUALQUER TAXA DE QUADROS.**
    ///
    /// Este era o único relógio do chrome, e ele contava QUADROS: a 30 fps um toast de "3 s" durava
    /// **6**, a 120 durava **1,5**. *Mutação: voltar a `age += 1` e comparar com 180 ⇒ as duas
    /// taxas divergem e o gate diz quanto.*
    #[test]
    fn a_toast_lives_three_seconds_at_any_frame_rate() {
        for fps in [30.0_f32, 60.0, 120.0] {
            let mut q = ToastQueue::default();
            q.push(Toast::info("oi"));
            let dt = 1.0 / fps;
            let mut t = 0.0_f32;
            // Um pouco antes dos 3 s ele ainda está lá.
            while t < 2.9 {
                q.tick(dt);
                t += dt;
            }
            assert_eq!(q.len(), 1, "morreu cedo a {fps} fps (t = {t})");
            // E um pouco depois, não está.
            while t < 3.1 {
                q.tick(dt);
                t += dt;
            }
            assert_eq!(q.len(), 0, "sobreviveu aos 3 s a {fps} fps (t = {t})");
        }
    }
}
