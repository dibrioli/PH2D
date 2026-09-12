//! **`ph2d-app-host` — a porta por onde uma família de módulo fala com a shell** (W2, ADR-0075).
//!
//! # Por que ela existe
//!
//! `shells/desktop` é uma crate de 493 k linhas e é a última unidade de todo build grande
//! ([auditoria de velocidade][audit] §4-C2). Dentro dela vivem **famílias inteiras** — `motion_*`,
//! `physics_*`, `sculpt3d_*`, `vec_*`, `flip_*`, `field3d_*` — que só ali estão por inércia: elas
//! precisam de meia dúzia de coisas da `App` e, por causa dessa meia dúzia, pagam (e fazem pagar) a
//! recompilação da maior crate do repo.
//!
//! Esta crate é essa meia dúzia, escrita uma vez.
//!
//! # ⚠️ O trait é o FALLBACK, nunca a primeira ferramenta (ADR-0075)
//!
//! A ordem de preferência, quando uma família precisa de alguma coisa, é:
//!
//! 1. **É estado da família?** ⇒ vive **na família** — num recurso/componente do ECS, ou num
//!    `thread_local` da própria crate. ⛔ **Não** é um campo novo na `App`.
//! 2. **É comunicação entre sistemas?** ⇒ **evento/recurso**, não uma chamada (ADR-0075: *systems
//!    não se chamam*).
//! 3. **É genuinamente da SHELL** — a janela, o `gfx`, o índice de acerto do chrome, a captura de
//!    undo, o diálogo do sistema operativo? ⇒ **e só então** é um método deste trait.
//!
//! *A prova de que a ordem funciona é o piloto:* a `field3d` guarda o estado dela num
//! `thread_local` da própria família **desde que existe**, com o doc-comment a dizer porquê
//! (*«`app_state.rs` é compartilhado e a `line/sculpt3d` edita-o — um campo novo lá é uma colisão
//! por conveniência»*). É exactamente por isso que ela é a família mais desacoplada das seis — **1**
//! `impl App` contra os 22 da física — e é por isso que ela foi o piloto.
//!
//! ⇒ **Uma família que precisa de um método por campo da `App` que hoje toca não precisa de um
//! trait maior: precisa de tirar o campo da `App`.** O trait abaixo tem **cinco** métodos, e o
//! número é a medida de que a regra foi seguida.
//!
//! # O que este trait NÃO é
//!
//! ⛔ **Não é um `&mut App` disfarçado.** Um método que devolva a `App`, o `AppGfx` ou o
//! `HeroScreen` ao chamador desfaz a fronteira inteira — a família volta a poder tudo, e a crate
//! volta a ter de recompilar quando a shell muda. Cada método aqui responde a **uma pergunta**, e a
//! resposta é um valor, nunca um handle.
//!
//! ⛔ **Não é onde uma família põe o que lhe convém.** Ver a lista de dependências no `Cargo.toml`:
//! toda dependência desta crate é paga pelas seis.
//!
//! [audit]: ../../../docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md

#![forbid(unsafe_code)]

pub mod canvas_area;
pub mod family;
pub mod modal;

pub use family::{AppFamily, AppFamilyRegistry, SmokeRouter};

use ph2d_editor_core::zones::Rect;

/// **Os modificadores do teclado, no vocabulário desta fronteira.**
///
/// ⚠️ **Não é o `ModifiersState` do `winit`, de propósito.** O trait é a fronteira entre a shell e
/// uma família, e pô-la a falar o tipo do backend de janela obrigaria toda crate de família a
/// depender do `winit` para perguntar *«o shift está em baixo?»*. A família que trata de botões de
/// rato continua a receber o `MouseButton` do `winit` como **parâmetro** — isso é o evento dela;
/// isto aqui é **estado do host**, e o host traduz.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HostMods {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    /// A tecla «logo» (⌘ no Mac, Super/Meta no Linux).
    pub super_key: bool,
}

impl HostMods {
    /// **A pergunta que o app inteiro faz para «juntar à selecção»** — `Shift`, `Super` ou `Ctrl`.
    ///
    /// ⚠️ **É uma porta e não três `||` espalhados**: o `input_dispatch` do canvas 2D já fazia
    /// exactamente esta conta, e a `field3d_input` copiou-a com o comentário *«um terceiro
    /// vocabulário de modificador no mesmo app é onde a mão aprende errado»*. Duas cópias da mesma
    /// lei é a forma de a terceira divergir.
    #[must_use]
    pub fn additive(self) -> bool {
        self.shift || self.super_key || self.control
    }
}

/// **O que uma família de módulo pode perguntar à shell — e nada mais.**
///
/// A shell implementa isto uma vez (`shells/desktop/src/app_host.rs`); cada família recebe-o como
/// `&mut dyn AppHost` e deixa de conhecer a `App`.
///
/// # ⚠️ Antes de acrescentar um método aqui, leia o §«O trait é o FALLBACK» no topo do ficheiro
///
/// Os cinco que existem passaram o teste *«isto é genuinamente da shell?»*:
///
/// | método | porque não pode ser da família |
/// |---|---|
/// | [`pointer`] | posição do rato na **janela** — o backend de janela é da shell |
/// | [`mods`] | idem, estado de teclado |
/// | [`pointer_over_chrome`] | pergunta ao **índice de acerto** do quadro (o que o chrome pintou) |
/// | [`modal_takes_the_pointer`] | um modal de tela cheia é da shell, e não publica `panel_rect` |
/// | [`note_authored_change`] | a **captura de undo** é da shell, e mede um quadro inteiro |
///
/// [`pointer`]: AppHost::pointer
/// [`mods`]: AppHost::mods
/// [`pointer_over_chrome`]: AppHost::pointer_over_chrome
/// [`modal_takes_the_pointer`]: AppHost::modal_takes_the_pointer
/// [`note_authored_change`]: AppHost::note_authored_change
pub trait AppHost {
    /// Onde o ponteiro está, em pixels de janela.
    fn pointer(&self) -> (f32, f32);

    /// Que modificadores estão em baixo **agora**.
    ///
    /// ⚠️ **Lido a cada pergunta, nunca congelado na pegada de um gesto**: o `winit` não manda
    /// modificadores no evento de movimento, então quem quer saber se o `Ctrl` está em baixo a meio
    /// de um arrasto tem de perguntar ao host, todo quadro.
    fn mods(&self) -> HostMods;

    /// **A moldura do app reclama este ponto?** — i.e. o chrome pintou alguma coisa aqui neste
    /// quadro.
    ///
    /// ⚠️ É a pergunta ao **índice de acerto**, e não a uma lista de ids escrita à mão: uma lista
    /// apodrece no dia em que uma faixa nova é pintada, e o sintoma é o artista a não conseguir
    /// clicar nos menus (Enio, 2026-08-30). O gizmo **não** conta como moldura — ele é desenhado
    /// *sobre* a obra.
    fn pointer_over_chrome(&self, x: f32, y: f32) -> bool;

    /// **Um modal de tela cheia está aberto?** Se sim, a família cala-se: teclado, ponteiro e roda.
    ///
    /// ⚠️ **A guarda do chrome não responde a isto**, e é instrutivo porquê: um modal de tela cheia
    /// **não publica `panel_rect` nem regista fundo**, logo [`pointer_over_chrome`] diz «não» com o
    /// modal a tapar o ecrã inteiro. Foi um defeito real com dois sintomas de uma vez — *«o modal
    /// não funciona, não fecha»* + *«os modelos do modal não são criados»*.
    ///
    /// ⚠️ **O SOLTAR fica de fora, de propósito**: não se pode *começar* um gesto através do modal,
    /// mas um gesto **já em curso** tem de poder acabar — senão o arrasto fica pousado para sempre.
    ///
    /// [`pointer_over_chrome`]: AppHost::pointer_over_chrome
    fn modal_takes_the_pointer(&self) -> bool;

    /// **Declara que este quadro AUTOROU uma mudança** — o diff de undo tem de o ver.
    ///
    /// ⚠️ Um gesto que escreve no documento tem de marcar o quadro em que a mudança entrou. Sem
    /// isto o passo só se regista colado à próxima acção do artista, seja ela qual for.
    fn note_authored_change(&mut self);

    /// A área do canvas que está de facto visível, dado o `viewport` da janela.
    ///
    /// Tem implementação por omissão (devolve o `viewport`) para que uma shell de teste não tenha
    /// de encenar um `HeroScreen`; a shell real responde com [`canvas_area::visible`].
    fn canvas_visible(&self, viewport: Rect) -> Rect {
        viewport
    }
}
