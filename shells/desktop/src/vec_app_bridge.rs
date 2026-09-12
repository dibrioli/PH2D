//! ⭐ **As PONTES da família `vec` que ainda tocam a `App`** — e só elas.
//!
//! # Por que estas duas ficaram
//!
//! O corpo dos módulos [`ph2d_app_vec::marquee`] e [`ph2d_app_vec::pencil_input`] é lei pura e
//! mudou-se para a crate na Fase B (2.ª volta). O que sobrou são dois métodos de `App` que o
//! `input_dispatch` chama em **quatro** sítios, e que perguntam a campos da `App` que as **cinco
//! portas** do `ph2d_app_host::AppHost` não cobrem:
//!
//! | método | o que ele precisa da `App` |
//! |---|---|
//! | `marquee_shape_for_press` | `vec_draw_config.marquee` — o chip pegajoso que a tool publica |
//! | `pointer_dynamics` | `Self::timestamp_ns()` — o relógio de parede da shell |
//!
//! ⛔ **Isto NÃO é um pedido de sexto método de host.** O HOWTO §1.5 é explícito: *«se a sua
//! família precisa de um método por campo da `App` que hoje toca, ela não precisa de um trait
//! maior: precisa de tirar o campo da `App`»*. O `vec_draw_config` é campo **desta** família e o
//! sítio dele é o [`ph2d_app_vec::state::VecState`] — movê-lo é trabalho de Fase C, com o preço
//! medido no handoff. Até lá as duas pontes vivem aqui, **juntas e nomeadas**, em vez de
//! manterem dois ficheiros inteiros na shell.
//!
//! ⚠️ **Os quatro sítios de chamada ficam byte a byte iguais** — é um `impl crate::App`, logo
//! `self.marquee_shape_for_press()` continua a escrever-se assim (HOWTO §1.5).

// ⚠️ **Do sítio onde ela de facto vive**, não do módulo que a re-exporta: dentro da crate o
// `marquee` faz `use ph2d_tool_vector::params::MarqueeShape;` — um `use` PRIVADO, logo
// `ph2d_app_vec::marquee::MarqueeShape` não existe do lado de cá da fronteira. *Uma fronteira
// nova torna visível a diferença entre um tipo e o atalho para ele.*
use ph2d_tool_vector::params::MarqueeShape;

impl crate::App {
    /// **A porta única: que forma tem o gesto que começa AGORA?**
    ///
    /// O chip pegajoso do painel (`vec_draw_config.marquee`, o espelho que a tool publica) e o
    /// **Ctrl** segurado neste press, compostos por [`MarqueeShape::for_gesture`]. Os dois braços
    /// de press do canvas (o com Shift e o sem) perguntam a ela — uma cópia num deles é como o
    /// laço deixa de existir no gesto ADITIVO, que é justamente onde o artista mais o quer.
    ///
    /// ⚠️ **Chamada UMA vez, no press.** O resultado viaja no gesto ([`VecMarquee::shape`]) e não
    /// é relido; ver o porquê no cabeçalho deste módulo.
    pub(crate) fn marquee_shape_for_press(&self) -> MarqueeShape {
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        MarqueeShape::for_gesture(self.vec_draw_config.marquee, ctrl)
    }
}

impl crate::App {
    /// **A DINÂMICA que o ponteiro carrega agora** — a porta única do W1d.
    ///
    /// A `pencil_width` deriva a largura de duas grandezas: a **pressão** do dispositivo e o
    /// **relógio de parede** (de onde sai a velocidade). Esta função é o único lugar da shell que
    /// as responde para o lápis.
    ///
    /// ⚠️ **A pressão é `1.0`, e é um fato MEDIDO da shell, não um placeholder solto.** Os dois
    /// únicos sítios que constroem um `PointerEvent` (`input_dispatch.rs`) cravam `pressure: 1.0`
    /// com `source: PointerSource::Mouse`, e o laço de eventos do winit **não casa
    /// `WindowEvent::Touch`** — o único evento que carrega `force`. O `CursorMoved`, que é o que
    /// a shell escuta, não tem pressão no protocolo. Logo, hoje, nenhum dispositivo entrega
    /// pressão a este app.
    ///
    /// ⚠️ **E ligar o `Touch` NÃO seria a cura — medido em 2026-08-12, e é a metade que faltava
    /// a esta nota.** Em `winit 0.30.13` o `force` é uma constante nos **três** backends de
    /// desktop: `x11/event_processor.rs` escreve `force: None, // TODO`, o
    /// `wayland/seat/touch/mod.rs` escreve `force: None`, e o `windows/event_loop.rs` escreve
    /// `force: None, // WM_TOUCH doesn't support pressure information`. Só `android`, `ios` e
    /// `web` o preenchem. **Não há função a escrever: o que falta é a dependência** (a API
    /// unificada de ponteiro de um winit mais recente, ou um caminho por plataforma) — e isso é
    /// decisão do Enio, não uma wave a começar. O estudo da UI viva tinha isto marcado como ⭐ de
    /// tamanho **P** (*«custa uma função»*) e a §8 dele carrega hoje a refutação.
    ///
    /// Ela mora aqui numa função só **exatamente por isso**: quando o caminho do tablet existir,
    /// é ESTA linha que muda, e a fonte `Pressure` do lápis passa a funcionar sem que nada mais
    /// se mexa. Repetir o literal no press e no move seria a terceira cópia de um número que já
    /// mente em duas.
    pub(crate) fn pointer_dynamics(&self) -> ph2d_vec_edit::pencil_width::PenDynamics {
        ph2d_vec_edit::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: Self::timestamp_ns(),
        }
    }
}
