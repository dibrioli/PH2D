//! [`ToastQueue`] — non-modal notification stream (ADR-0023 §2).
//!
//! "Notificações flutuantes não-modais no topo do canvas
//! (ex.: 'Undo: Brush Stroke', 'Saved 2 s ago') — informam sem
//! interromper."
//!
//! - **Stack** at the top-center of the canvas (above all panels).
//! - **Auto-dismiss** after `Toast::ttl_s` — o tempo de LER a frase, nunca abaixo de 3 s, de
//!   relógio de PAREDE ([`crate::toast_reading::reading_ttl_s`]).
//! - **Live region** in the a11y tree (per ADR-0023 §10) so screen
//!   readers announce the latest toast without stealing focus.
//! - **Bounded queue** (32 entries) — runaway notifications get
//!   dropped silently. Per HR-9 backpressure principle (same as
//!   `ph2d_script::WriteQueue`).

use crate::paint::{
    Paint, PaintCtx, fill_rounded_rect, paint_icon, paint_text_block, rect_to_vello, resolve,
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
    pub const DEFAULT_TTL_S: f32 = crate::toast_reading::MIN_TTL_S;

    /// O tempo de vida é o de LER a frase ([`crate::toast_reading::reading_ttl_s`]); o
    /// [`Self::ttl_seconds`] do chamador manda sobre ele.
    pub fn new(message: impl Into<String>, severity: ToastSeverity) -> Self {
        let message = message.into();
        Self {
            ttl_s: crate::toast_reading::reading_ttl_s(&message),
            message,
            severity,
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
    /// ⚠️ **Segundos, nunca quadros** — ver [`Toast::DEFAULT_TTL_S`]. `retain` e não «tira da
    /// frente»: com o tempo de vida a crescer com a frase, um aviso curto atrás de um longo expira
    /// primeiro.
    pub fn tick(&mut self, dt: f32) {
        for t in &mut self.inner {
            t.age_s += dt;
        }
        self.inner.retain(|t| t.age_s < t.ttl_s);
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
// ciclo entre os módulos da fundação.

/// A porta do orçamento de texto mora na lei de leitura; este é o endereço de sempre.
pub use crate::toast_reading::text_budget_px;
use crate::toast_reading::{ICON_PX as TOAST_ICON_PX, Shown};

/// Onde cada balão e cada barra da COLUNA DO TOPO ficam — a conta única que a pintura e os gates
/// leem.
#[derive(Clone, Debug)]
pub struct TopColumn {
    /// O cartão de cada aviso, pela ordem da fila, e o texto que ele pinta.
    pub toasts: Vec<(Rect, Shown)>,
    /// O cartão de cada barra de trabalho, por baixo dos avisos.
    pub bars: Vec<Rect>,
    /// As linhas da coluna que os avisos ocupam.
    pub rows: usize,
}

/// ⭐ **A coluna do topo**: os avisos de cima para baixo, cada um com as linhas que a altura do
/// texto dele pede ([`crate::toast_reading::rows_for_height`]), e `bars` barras de trabalho por
/// baixo — pela CONTAGEM DE LINHAS ocupadas, nunca pelo número de avisos.
///
/// Os avisos são donos do TOPO: um aviso tem uma oportunidade de ser lido, e o lugar dele não se
/// move porque um trabalho de fundo arrancou; a barra é persistente e mexe-se, pode descer.
pub fn top_column(
    toasts: &ToastQueue,
    bars: usize,
    ts: &mut ph2d_text::TextSystem,
    viewport: Rect,
) -> TopColumn {
    let mut row = 0;
    let mut out = Vec::with_capacity(toasts.len());
    for toast in toasts.iter() {
        let primeira = crate::progress::column_row(viewport, row);
        let shown = crate::text_elide::balao::na_area(primeira, || {
            crate::toast_reading::shown(ts, &toast.message)
        });
        let rows = crate::toast_reading::rows_for_height(shown.height);
        out.push((crate::progress::column_span(viewport, row, rows), shown));
        row += rows;
    }
    TopColumn {
        toasts: out,
        bars: (0..bars)
            .map(|i| crate::progress::column_row(viewport, row + i))
            .collect(),
        rows: row,
    }
}

/// ⭐ **Pinta a coluna do topo inteira** — os avisos e, por baixo, as barras de trabalho, nos
/// lugares do [`top_column`]. A shell chama ISTO, nunca os dois pintores à parte.
pub fn paint_top_column(
    toasts: &ToastQueue,
    jobs: &crate::progress::JobQueue,
    scene: &mut VectorScene,
    ctx: &mut PaintCtx,
) {
    let col = top_column(toasts, jobs.len(), ctx.text, ctx.viewport);
    for (toast, (r, shown)) in toasts.iter().zip(&col.toasts) {
        paint_toast(toast, *r, shown, scene, ctx);
    }
    for (p, r) in jobs.iter().zip(&col.bars) {
        crate::progress::paint_bar(p, *r, scene, ctx);
    }
}

impl Paint for ToastQueue {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        let col = top_column(self, 0, ctx.text, ctx.viewport);
        for (toast, (r, shown)) in self.iter().zip(&col.toasts) {
            paint_toast(toast, *r, shown, scene, ctx);
        }
    }
}

/// Um balão no cartão `r`, com o texto que a lei de leitura deu (`shown`).
fn paint_toast(toast: &Toast, r: Rect, shown: &Shown, scene: &mut VectorScene, ctx: &mut PaintCtx) {
    use ph2d_tokens::{Radius, Spacing, StrokeToken};
    let radius = crate::paint::frame_radius(ctx.theme, Radius::Md.px());
    // Body uses BgElev so the toast lifts off the canvas
    // independently of its severity tint; the severity color
    // is reserved for the icon + accent stripe on the left.
    fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, ctx.theme));
    // ⭐ **Pela porta do tema** (wave 26): a moldura do tema moderno é nenhuma em repouso.
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
    // MOLDURA do tema — num tema moderno não há moldura, e a faixa encosta à borda.
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

    // Ícone da severidade, depois da faixa, centrado na 1.ª linha do texto.
    let (text_y, meio_da_1a) = crate::toast_reading::text_y(r, shown);
    let icon_rect = Rect::new(
        r.x + Spacing::Xl.px(),
        meio_da_1a - TOAST_ICON_PX * 0.5,
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

    // A mensagem enche o resto, alinhada à esquerda, QUEBRADA à largura com que a lei de leitura
    // a mediu — um orçamento só: o `debug_assert` confere essa largura contra o cartão.
    let text_x = icon_rect.x + TOAST_ICON_PX + ph2d_tokens::icon_label_gap_px();
    debug_assert!(
        ((r.x + r.w - text_x - Spacing::Xl.px()) - shown.width).abs() < 0.5,
        "a porta do orçamento e a conta do pintor divergiram"
    );
    paint_text_block(
        ctx.text,
        scene,
        &shown.text,
        text_x,
        text_y,
        crate::toast_reading::font_px(),
        shown.width,
        resolve(ColorToken::Text1, ctx.theme),
    );
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

#[cfg(test)]
#[path = "toast_column_tests.rs"]
mod column_tests;
