//! Keyboard event handling — winit `KeyboardInput` → host key event +
//! hero-pipeline key/text forwarding + M12 demo controls. Extracted
//! from `input_dispatch.rs` to keep that file under the HR-18 LOC cap.

use winit::event::{ElementState, KeyEvent as WinitKeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use ph2d_host::{HostHandler, KeyEvent, KeyKind};

use crate::App;
use crate::forwarding::{forward_key_to_hero, forward_text_to_hero};
use crate::keymap::winit_to_editor_keycode;

impl App {
    /// Whether the bottom-docked general timeline panel is currently visible
    /// (the context in which Ctrl+Z routes to timeline undo/redo).
    pub(crate) fn timeline_panel_open(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.is_panel_visible("timeline"))
    }

    /// O evento de teclado do winit. **Só desembrulha** — a política toda mora no
    /// [`Self::key_input`], que é dirigível sem winit.
    ///
    /// A separação não é arrumação: o `winit::KeyEvent` tem campo privado e **não pode ser
    /// construído** fora do winit, então enquanto o corpo morava aqui **nenhum teste
    /// conseguia apertar uma tecla**. O roteamento do Ctrl+Z (quem consome antes de quem) era
    /// exatamente o que o Enio disse estar quebrado, e era a única parte do input que nenhum
    /// gate alcançava.
    pub(crate) fn on_keyboard_input(&mut self, event: WinitKeyEvent) {
        let WinitKeyEvent {
            physical_key,
            state,
            repeat,
            text,
            ..
        } = event;
        self.key_input(physical_key, state, repeat, text);
    }

    /// O caminho de teclado de verdade: roteamento, atalhos e forward pros widgets.
    pub(crate) fn key_input(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
        text: Option<winit::keyboard::SmolStr>,
    ) {
        self.any_input_this_frame = true;
        // ⭐ **Estação ZERO do Ctrl+Z.** Ver [`crate::App::diag_undo_chord`] — ela corre antes de
        // toda guarda de propósito, porque o que este report precisa de saber primeiro é se a
        // tecla sequer entrou na janela.
        if state == ElementState::Pressed
            && !repeat
            && physical_key == PhysicalKey::Code(KeyCode::KeyZ)
            && (self.modifiers.control_key() || self.modifiers.super_key())
        {
            self.diag_undo_chord("RECEBIDA pela janela");
        }
        let keycode = match physical_key {
            PhysicalKey::Code(code) => code as u32,
            PhysicalKey::Unidentified(_) => 0,
        };
        let kind = match (state, repeat) {
            (ElementState::Pressed, false) => KeyKind::Down,
            (ElementState::Pressed, true) => KeyKind::Repeat,
            (ElementState::Released, _) => KeyKind::Up,
        };
        // ⭐⭐⭐ **A ESCUTA DO INPUT MAP É O PRIMEIRO RAMO DESTA FUNÇÃO, E TEM DE SER** — o
        // mecanismo inteiro está em [`super::keyboard_bind_capture`], cortado para lá pelo teto de
        // LOC. Abaixo desta linha há ~20 `return`, e nenhum deles pode ver a tecla antes dela.
        if self.capture_binding_if_listening(physical_key, kind) {
            return;
        }
        // ADR-0150 W2: a cena 3D toma as teclas dela ANTES do store.
        //
        // ⚠️ **A justificativa que morava aqui ENVELHECEU, e a nota virou o bug.** Ela
        // dizia *"inerte (e portanto invisível) sem cena armada — num run normal
        // `sculpt3d` é `None`"*, o que era verdade enquanto o módulo vivia atrás de uma
        // variável de ambiente, e ficou **falso no dia do pill** (W-Pill, 2026-08-10):
        // num run normal a cena passa a existir ao primeiro clique, e **sair do modo
        // nunca a destrói**. Como este `return` corre ANTES do `handler.on_key` logo
        // abaixo, uma porta que só perguntava *"a cena existe?"* passou a comer os dez
        // dígitos e ~26 letras de todo painel do app, para sempre.
        //
        // Quem responde agora é [`Self::sculpt3d_keys_live`] (dentro da porta), pela
        // MESMA pergunta que o ponteiro daquela cena já fazia. *Quem move o número que
        // tornava uma nota verdadeira tem de reconferir a nota.*
        // ⭐⭐⭐ **`Ctrl+Alt+Q` — a divisão do canvas da ESCULTURA**, a mesma tecla (e a mesma
        // lei dos três modificadores por nome) do módulo de modelagem.
        //
        // ⚠️ **Ela corre ANTES do `sculpt3d_key`, e a ordem é a cura**: aquele tem um catch-all
        // (`if ctrl { … return false }`) que engole todo `Ctrl+` que não seja o desfazer, e foi
        // ele que matou a primeira redacção desta tecla (report do Enio, 2026-09-08).
        #[cfg(feature = "sculpt3d")]
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code) = physical_key
            && self.sculpt3d_quad_key(code)
        {
            return;
        }
        #[cfg(feature = "sculpt3d")]
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code) = physical_key
            && self.sculpt3d_key(
                code,
                self.modifiers.control_key(),
                self.modifiers.shift_key(),
            )
        {
            return;
        }
        // ⭐ **AS TECLAS DO MODELADOR 3D, numa porta só** — ver
        // [`keyboard_field3d`](super::keyboard_field3d).
        if self.field3d_keys(physical_key, state) {
            return;
        }

        self.handler.on_key(KeyEvent {
            keycode,
            modifiers: Self::convert_modifiers(self.modifiers),
            kind,
            timestamp_ns: Self::timestamp_ns(),
        });

        // ⭐ **F9 — o chrome legado volta.** Porquê um interruptor: `crate::legacy_chrome`.
        if state == ElementState::Pressed
            && physical_key == PhysicalKey::Code(KeyCode::F9)
            && self.toggle_legacy_chrome()
        {
            return;
        }
        // ⭐ **As teclas do PALETTE vivem no irmão** — ver [`super::keyboard_palette`]. Ele é
        // MODAL: se devolve `true`, engoliu a tecla inteira (press e release).
        if self.command_palette_keys(physical_key, state, text.as_deref()) {
            return;
        }

        // O PEEK do Flip (Shift & Trace fatia 2): F1/F2/F3 são o flip de papel —
        // SEGURAR mostra só o desenho vizinho (anterior/atual/seguinte) sem mover o
        // playhead; soltar volta. A política é pura (`flip_peek::key_transition`):
        // press só arma com a tool Flip ativa; release SEMPRE desarma (trocar de tool
        // com a tecla presa não pode deixar o peek preso).
        // ⚠️ **O dedo do jogador OBSERVA, nunca consome** (W3). A seta já tem
        // dono (o nudge de nó do Vector), e roubá-la aqui faria esta wave
        // regredir uma ferramenta que ninguém pediu para mexer — o evento segue
        // o caminho de sempre, e o que muda é um par de bools que ninguém lê
        // numa cena sem player. A política é pura (`ph2d_app_physics::player_input`),
        // porque um `winit::KeyEvent` não pode ser construído num teste.
        // ⚠️ **E um ACORDE nunca é entrada de jogo** (report do Enio, cena 112:
        // *"os players pulam e se movem sozinhos"*). O dedo do jogador observa a
        // tecla FÍSICA, e as seis que ele reclama moram todas debaixo de atalhos
        // que o artista aperta o tempo todo: **Ctrl+Z** punha `jump`, **Ctrl+A**
        // punha `left`, **Ctrl+D** `right`, **Ctrl+S** `down`.
        //
        // ⚠️ **A varredura de conflito do `player_input` foi feita e ainda assim
        // errou**, porque mediu a tecla NUA: o doc do `KeyS` afirma que *"o único
        // `KeyS` do repo é o Ctrl+S de salvar projeto, que corre dentro do braço
        // guardado por modificador — um botão de player nunca vê aquele
        // caminho"*. Ele vê: esta observação corre ANTES de toda guarda de
        // modificador do arquivo.
        //
        // ⚠️ **O RELEASE passa sempre, e a assimetria é a mesma do peek do Flip
        // logo acima:** soltar uma tecla que foi apertada sem modificador,
        // enquanto o Ctrl está preso, tem de chegar — senão o guard troca um
        // pulo espúrio por um personagem que anda sozinho para sempre.
        if let PhysicalKey::Code(code) = physical_key {
            let pressed = state == ElementState::Pressed;
            let chord = self.modifiers.control_key()
                || self.modifiers.alt_key()
                || self.modifiers.super_key();
            if !pressed || !chord {
                // ⭐ **O retrato dos dispositivos que o Input Map resolve** (plano 30 W5).
                //
                // ⚠️ **A guarda de acorde é a de sempre:** uma tecla premida com `Ctrl` segurado é
                // um atalho de ficheiro, não um passo do jogador — e o RELEASE passa sempre, senão
                // soltar com o `Ctrl` preso deixaria o personagem a andar para sempre.
                //
                // ⚠️ **Normalizador TOTAL** (`winit_to_input_keycode`), não o do editor: este tem
                // de alcançar o `W`/`S`/`Z`/`Q` que o do editor deixa cair de propósito.
                if let Some(k) = crate::keymap::winit_to_input_keycode(code) {
                    self.input.apply_event(if pressed {
                        ph2d_input::Event::KeyDown(ph2d_input::Key(k))
                    } else {
                        ph2d_input::Event::KeyUp(ph2d_input::Key(k))
                    });
                }
            }
        }

        // ⭐⭐⭐ **UM MODO EM CURSO É DONO DA ENTRADA DELE** — a porta está em
        // [`super::keyboard_modal`], cortada para lá pelo teto de LOC. Ela corre **depois** do
        // retrato dos dispositivos (senão o modo ficaria inerte com o teclado tomado) e **antes**
        // de todo atalho (senão a tecla faria duas coisas).
        if self.modal_owns_the_keyboard(physical_key, state, repeat) {
            return;
        }

        if let PhysicalKey::Code(code) = physical_key {
            let (next, consumed) = ph2d_app_flip::peek::key_transition(
                self.flip_state.peek,
                code,
                state == ElementState::Pressed,
                self.flip_state.active,
            );
            self.flip_state.peek = next;
            if consumed {
                return;
            }
        }

        if self.ramo_teclas_texto_flip_e_vetor(physical_key, state, repeat, &text) {
            return;
        }

        if self.ramo_teclas_nos_ficheiros_e_acordes(physical_key, state, repeat) {
            return;
        }

        if self.ramo_teclas_timeline_painter_hierarquia(physical_key, state, repeat) {
            return;
        }

        // Hero pipeline (ADR-0024): translate winit's physical KeyCode
        // into the editor's KEY_* constants and route to the focused
        // widget.
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code) = physical_key
            && let Some(editor_keycode) = winit_to_editor_keycode(code)
        {
            forward_key_to_hero(
                self.gfx.as_mut(),
                KeyEvent {
                    keycode: editor_keycode,
                    modifiers: Self::convert_modifiers(self.modifiers),
                    kind,
                    timestamp_ns: Self::timestamp_ns(),
                },
            );
        }
        // Printable text from this key event (winit already resolved
        // layout + dead-keys + shift). Send each char through the
        // text-input dispatcher so focused TextInput/NumberInput/
        // Combobox buffers update. EXCEPT for ' ' coming from the
        // physical Space key and 'a'/'A' coming from KeyA with Cmd/
        // Ctrl held — those are inserted by the dispatch's key handler
        // directly.
        if state == ElementState::Pressed
            && let Some(s) = text.as_ref()
        {
            let is_space_key = matches!(physical_key, PhysicalKey::Code(KeyCode::Space));
            let cmd_held = self.modifiers.super_key() || self.modifiers.control_key();
            for ch in s.chars() {
                if ch.is_control() {
                    continue;
                }
                if is_space_key && ch == ' ' {
                    continue;
                }
                // Cmd/Ctrl chord with a letter: skip the text-event so
                // Cmd+A's select-all isn't overwritten by 'a' insertion.
                if cmd_held && ch.is_ascii_alphabetic() {
                    continue;
                }
                forward_text_to_hero(self.gfx.as_mut(), ch);
            }
        }

        // ⭐ O que sobra depois de toda a cadeia — ver [`tail`].
        self.key_tail(state, repeat, physical_key);
    }
}

/// ⭐ Os dois ramos que correm depois de toda a cadeia — ver [`tail`].
#[path = "keyboard_tail.rs"]
mod tail;

/// A cadeia do `key_input` (3D, texto/Flip/vetor, nós/ficheiros/acordes, timeline/Painter/Hierarquia) — os ramos.
#[path = "keyboard_cadeia.rs"]
mod cadeia;
