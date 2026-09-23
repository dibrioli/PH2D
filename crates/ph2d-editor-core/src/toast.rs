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

use crate::paint::{
    Paint, PaintCtx, fill_rounded_rect, paint_icon, paint_text_centered, rect_to_vello, resolve,
};
use crate::zones::Rect;
use ph2d_a11y::{Live, Node, NodeBuilder, Role};
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;
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

// ⚠️ **O pintor da fila mora com a fila** (auditoria A10, 2026-09-12). Vivia no `paint.rs`, e
// era por ele que a pintura da fundação dependia do `toast` e do `progress` (a régua da coluna,
// `progress::column_row`, é partilhada com as barras de trabalho) — duas das arestas que fechavam o
// ciclo entre os módulos da fundação. O corpo não mudou uma linha.

/// O lado do ícone de severidade de um balão de aviso.
///
/// ⚠️ **Não há token para este tamanho, e a ausência é a nota:** o `chrome.inline-icon` (14) é o
/// glifo de uma linha e o `chrome.icon-btn-size` (36) é um botão — este está no meio, num balão
/// que não é nem linha nem botão. Fica NOMEADO em vez de escrito no meio da pintura.
const TOAST_ICON_PX: f32 = 24.0;
/// ⭐⭐⭐ **O ORÇAMENTO DE TEXTO de um balão de aviso** — a largura, em píxeis, em que a mensagem
/// tem de caber para ser LIDA.
///
/// ⛔⛔ **Ela existe por um report do dono** (22/09, sobre o aviso da exportação da escultura):
/// *«as mensagens estão cortadas com … não consigo ler tudo»*. A fila vive numa coluna de
/// [`crate::progress`] com largura FIXA, e o que sobra para o texto depois da faixa, do ícone e
/// dos dois recuos é bem menos do que ela — *uma mensagem que não cabe é elidida, e um balão vive
/// três segundos: chegar lá com o rato para ver o balão da elisão não é uma cura*.
///
/// ⚠️ **UMA régua, DOIS consumidores:** o pintor abaixo e quem quiser PERGUNTAR se a frase dele
/// cabe. Escrita duas vezes, a resposta do gate e a do produto divergem no dia em que um dos
/// recuos mudar de token — e a que o artista vê é a errada.
#[must_use]
pub fn text_budget_px() -> f32 {
    use ph2d_tokens::Spacing;
    (crate::progress::toast_column_w()
        - Spacing::Xl.px() * 2.0
        - TOAST_ICON_PX
        - ph2d_tokens::icon_label_gap_px())
    .max(0.0)
}
// LITERAL-PX-OK: lado do ícone de severidade do balão de aviso

impl Paint for ToastQueue {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        use crate::toast::ToastSeverity;
        use ph2d_tokens::{Radius, Spacing, StrokeToken, TypeToken};
        // The toast stream owns the TOP of the top-center column and the job bars
        // (`progress::JobQueue`) stack under it — a toast lives three seconds and gets one
        // chance to be read, so its slot must not move because some background job happens to
        // be running. `column_row` is the shared ruler for both tenants; it lives over there
        // because this file is at its frozen LOC ceiling (see the workspace LOC-cap gate).
        let radius = crate::paint::frame_radius(ctx.theme, Radius::Md.px());
        for (i, toast) in self.iter().enumerate() {
            let r = crate::progress::column_row(ctx.viewport, i);
            // Body uses BgElev so the toast lifts off the canvas
            // independently of its severity tint; the severity color
            // is reserved for the icon + accent stripe on the left.
            fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, ctx.theme));
            // ⭐ **Pela porta do tema** (wave 26): até aqui isto era um `stroke_rounded_rect` cru
            //    a 1 px, logo o balão desenhava num tema moderno o contorno que a pele plana
            //    apagou em toda a casa. ⛔ E o censo da moldura não o via: o `paint.rs` está
            //    ISENTO com o motivo *«é a PORTA»* — verdade para o corpo do `stroke_frame`, e o
            //    balão vive 290 linhas abaixo, no mesmo ficheiro.
            crate::paint::stroke_frame(
                scene,
                r,
                radius,
                ctx.theme,
                ph2d_tokens::visuals::Feel::Rest,
                StrokeToken::Thin.px(),
                resolve(ColorToken::Border, ctx.theme),
            );

            let (severity_token, icon) = match toast.severity {
                ToastSeverity::Info => (ColorToken::Info, crate::icons::IconId::Info),
                ToastSeverity::Success => (ColorToken::Success, crate::icons::IconId::Check),
                ToastSeverity::Warning => (ColorToken::Warn, crate::icons::IconId::Warning),
                ToastSeverity::ErrorState => (ColorToken::Danger, crate::icons::IconId::Error),
            };
            let severity_color = resolve(severity_token, ctx.theme);

            // Faixa de acento à esquerda, na cor da severidade. ⚠️ O recuo dela é a LARGURA DA
            // MOLDURA do tema — num tema moderno não há moldura, e a faixa passa a encostar à
            // borda em vez de deixar um fio do fundo a aparecer.
            let inset = ph2d_tokens::visuals::Chrome::of(ctx.theme)
                .panel_border
                .width;
            let stripe = Rect::new(
                r.x + inset,
                r.y + inset,
                Spacing::Xs.px(),
                r.h - inset * 2.0,
            );
            scene.fill_rect(rect_to_vello(stripe), severity_color);

            // Ícone da severidade, centrado depois da faixa.
            let icon_rect = Rect::new(
                r.x + Spacing::Xl.px(),
                r.y + (r.h - TOAST_ICON_PX) * 0.5,
                TOAST_ICON_PX,
                TOAST_ICON_PX,
            );
            paint_icon(
                scene,
                icon,
                icon_rect,
                severity_color,
                StrokeToken::Default.px(),
            );

            // Message text fills the rest, left-aligned with padding.
            let text_x = icon_rect.x + TOAST_ICON_PX + ph2d_tokens::icon_label_gap_px();
            // ⚠️ A largura sai da PORTA (`text_budget_px`), que é a mesma que um gate pode
            //   perguntar. O `debug_assert` é o controlo de que as duas contas não derivaram.
            let text_rect = Rect {
                x: text_x,
                y: r.y,
                w: (r.x + r.w - text_x - Spacing::Xl.px()).max(0.0),
                h: r.h,
            };
            debug_assert!(
                (text_rect.w - text_budget_px()).abs() < 0.5,
                "a porta do orçamento e a conta do pintor divergiram"
            );
            paint_text_centered(
                ctx.text,
                scene,
                &toast.message,
                text_rect,
                TypeToken::Base.px(),
                resolve(ColorToken::Text1, ctx.theme),
            );
        }
    }
}

#[cfg(test)]
#[path = "toast_orcamento_tests.rs"]
mod orcamento_tests;

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

#[cfg(test)]
mod paint_tests {
    use super::*;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;

    #[test]
    fn toast_queue_paint_with_three_severities() {
        use crate::toast::Toast;
        let mut q = ToastQueue::new();
        q.push(Toast::info("info"));
        q.push(Toast::success("success"));
        q.push(Toast::warning("warn"));
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut ctx = PaintCtx {
            theme: Theme::Sunstone,
            viewport: Rect::new(0.0, 0.0, 800.0, 600.0),
            text: &mut text,
        };
        q.paint(&mut scene, &mut ctx);
    }
}
