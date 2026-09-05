//! ⭐ **O que a SHELL publica por quadro e os PINTORES lêem** — os quatro `thread_local` de
//! aparência: a escala de raio, o estilo das linhas de propriedade, a aparência (clássico /
//! redesenho) e a estratégia de texto.
//!
//! ⚠️ **Irmão do `paint.rs` pelo tecto de 700 LOC** (a porta da moldura, `stroke_frame`, empurrou-o
//! a 731 em 2026-09-05), e o corte é por **responsabilidade**: o `paint.rs` responde *como se
//! desenha uma primitiva*; isto responde *em que estado de aparência o quadro corre*. ⛔ Os
//! caminhos públicos não mudaram — o `paint.rs` re-exporta tudo (`crate::paint::set_ui_look` e
//! irmãos continuam a existir para os 43 sítios que os chamam).
//!
//! # Por que `thread_local`, e não um argumento
//!
//! Cada um destes valores é lido em **centenas** de sítios de pintura e escrito **uma** vez por
//! quadro pela shell. Enfiá-los na assinatura de cada pintor seria centenas de edições para
//! responder, centenas de vezes, a uma pergunta que o app responde uma vez por quadro. É o mesmo
//! padrão do [`ph2d_tokens::num_runtime`]: *a shell publica, a folha lê*.

// Thread-local global radius multiplier. Lets the user pick Sharp /
// Default / Round in the topbar context menu and have every rounded
// surface (200+ sites) scale uniformly without threading the value
// through every painter signature. Set via `set_radius_scale` before
// paint, read via `radius_scale`. Defaults to 1.0.
thread_local! {
    static RADIUS_SCALE: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
}

/// Set the global radius scale for the current thread (paint runs).
/// Clamped to non-negative; pass `1.0` to reset.
pub fn set_radius_scale(scale: f32) {
    RADIUS_SCALE.with(|s| s.set(scale.max(0.0)));
}

/// Read the current global radius scale.
pub fn radius_scale() -> f32 {
    RADIUS_SCALE.with(|s| s.get())
}

thread_local! {
    static SLIDER_STYLE: std::cell::Cell<ph2d_tokens::SliderStyle> =
        const { std::cell::Cell::new(ph2d_tokens::SliderStyle {
            design: ph2d_tokens::SliderDesign::Underline,
            radius: ph2d_tokens::Radius::Xs,
            density: ph2d_tokens::Density::Compact,
        }) };
}

/// **A aparência das linhas de propriedade**, publicada uma vez por quadro pelo shell.
///
/// ⭐ Decisão do Enio (2026-09-02): o padrão do app é `Underline` · raio `4` · linha `22`.
///
/// ⚠️ **O `const` acima repete o [`SliderStyle::default()`](ph2d_tokens::SliderStyle) porque um
/// `thread_local!` `const`-inicializado não pode chamar `Default::default()`** — e é a inicialização
/// `const` que o torna barato. ⛔ Os dois têm de concordar, e há gate a exigi-lo
/// (`the_paint_default_matches_the_token_default`): *duas escritas do mesmo default é exactamente a
/// forma que diverge em silêncio.*
pub fn set_slider_style(style: ph2d_tokens::SliderStyle) {
    SLIDER_STYLE.with(|s| s.set(style));
}

/// Lê a aparência activa das linhas de propriedade.
#[must_use]
pub fn slider_style() -> ph2d_tokens::SliderStyle {
    SLIDER_STYLE.with(std::cell::Cell::get)
}

// ⭐⭐⭐ **A APARÊNCIA do app**, publicada uma vez por quadro como o `SliderStyle` e o
// `TextRendering`.
//
// ⚠️ **O neutro é `UiLook::Redesign`** desde 2026-09-03 (ordem do Enio). O clássico volta com
// `PH2D_UI_NEW=0` — a convenção da casa para bissecar. Ver o doc do enum.
// ⛔ Ele TEM de repetir o `UiLook::default()`, porque um `thread_local!` `const` não pode chamar
// `Default::default()` — e há gate a exigir que os dois concordem.
thread_local! {
    static UI_LOOK: std::cell::Cell<ph2d_tokens::UiLook> =
        const { std::cell::Cell::new(ph2d_tokens::UiLook::Redesign) };
}

/// Publica a aparência para o quadro. Chamado pelo shell.
pub fn set_ui_look(look: ph2d_tokens::UiLook) {
    UI_LOOK.with(|c| c.set(look));
}

/// A aparência em vigor. ⚠️ Lida pelos **pintores**, nunca pelos painéis: quem decide o desenho de
/// um widget é o widget.
#[must_use]
pub fn ui_look() -> ph2d_tokens::UiLook {
    UI_LOOK.with(std::cell::Cell::get)
}

/// ⭐ Atalho: *estamos no redesenho?* — a pergunta que os três pintores fazem.
#[must_use]
pub fn ui_is_redesign() -> bool {
    ui_look() == ph2d_tokens::UiLook::Redesign
}

/// Set the active text-rendering strategy for the current thread.
/// Called by the shell once per frame, mirroring `set_radius_scale`.
/// Delegates to `ph2d_text::set_active_text_rendering` — the canonical
/// thread-local lives there so `TextSystem::prefix_width` can read it
/// internally (fixing the caret-position bug under CrispHeavy where
/// measurements used Medium 500 while glyphs render ExtraBold 800).
pub fn set_text_rendering(mode: ph2d_tokens::TextRendering) {
    ph2d_text::set_active_text_rendering(mode);
}

/// Read the active text-rendering strategy. Delegates to
/// `ph2d_text::active_text_rendering` — see `set_text_rendering` for
/// why the state lives in ph2d-text.
pub fn text_rendering() -> ph2d_tokens::TextRendering {
    ph2d_text::active_text_rendering()
}
