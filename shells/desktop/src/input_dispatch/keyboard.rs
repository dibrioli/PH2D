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
        let keycode = match physical_key {
            PhysicalKey::Code(code) => code as u32,
            PhysicalKey::Unidentified(_) => 0,
        };
        let kind = match (state, repeat) {
            (ElementState::Pressed, false) => KeyKind::Down,
            (ElementState::Pressed, true) => KeyKind::Repeat,
            (ElementState::Released, _) => KeyKind::Up,
        };
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
        // ADR-0161 W4: `Home` repõe a vista da janela 3D de modelagem — a volta que a
        // rotação LIVRE torna necessária (ela inclina o horizonte, de propósito).
        // Inerte sem o smoke armado; ver a nota de `field3d_home_key` sobre o dia em
        // que isso deixar de ser a única porta.
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code) = physical_key
            && self.field3d_home_key(code)
        {
            return;
        }
        // ADR-0161 W6: `G`/`R`/`S` trocam o verbo do gizmo 3D (mover/rodar/escalar), as letras
        // do Blender. ⚠️ Só com o ponteiro SOBRE a janela 3D — ver a nota de `field3d_mode_key`:
        // sem essa guarda, três letras comuns deixariam de chegar a qualquer campo de texto.
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code) = physical_key
            && self.field3d_mode_key(code)
        {
            return;
        }

        // ADR-0161 W15: `Numpad5` alterna a LENTE da janela 3D (convergente ↔ paralela), a tecla
        // do Blender para a mesma coisa. Mesma guarda de ponteiro das outras — ver `over_window`.
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code) = physical_key
            && self.field3d_lens_key(code)
        {
            return;
        }

        self.handler.on_key(KeyEvent {
            keycode,
            modifiers: Self::convert_modifiers(self.modifiers),
            kind,
            timestamp_ns: Self::timestamp_ns(),
        });

        // O palette de "Add Node" (tela cheia, Motion) é MODAL — enquanto aberto ele COME toda tecla:
        // caracteres imprimíveis vão pro campo de busca, Enter escolhe o topo do filtro, Backspace apaga,
        // Escape fecha. Vem PRIMEIRO (antes dos atalhos de painel/ferramenta) para uma letra digitada nunca
        // vazar num atalho de grafo embaixo. O `A` que ABRIU o palette foi capturado no quadro ANTERIOR
        // pelo painel (o palette só abre no quadro seguinte, na ponte), então a tecla de abertura flui normal.
        if self.command_palette_open() {
            if state == ElementState::Pressed {
                if let PhysicalKey::Code(code) = physical_key {
                    match code {
                        KeyCode::Escape => {
                            self.command_palette_close();
                            return;
                        }
                        KeyCode::Enter | KeyCode::NumpadEnter => {
                            self.command_palette_confirm();
                            return;
                        }
                        KeyCode::Backspace => {
                            self.command_palette_backspace();
                            return;
                        }
                        _ => {}
                    }
                }
                if let Some(s) = text.as_ref() {
                    for ch in s.chars() {
                        self.command_palette_type(ch);
                    }
                }
            }
            // Modal: engole TODA a tecla (press e release), aberto ou não haja o que digitar.
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
        // numa cena sem player. A política é pura (`crate::player_input`),
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
                self.player_keys.key(code, pressed);
            }
        }

        if let PhysicalKey::Code(code) = physical_key {
            let (next, consumed) = crate::flip_peek::key_transition(
                self.flip_peek,
                code,
                state == ElementState::Pressed,
                self.flip_active,
            );
            self.flip_peek = next;
            if consumed {
                return;
            }
        }

        // Texto vetorial: enquanto uma sessão de digitação está ativa (modo Text +
        // clicou no canvas), as teclas vão pro TEXTO — antes dos atalhos de forma e
        // do forward pros widgets. Ctrl/Super passam (Ctrl+Z etc. seguem globais).
        if self.vector_keys_live()
            && self.vec_text_editing()
            && state == ElementState::Pressed
            && !self.modifiers.control_key()
            && !self.modifiers.super_key()
        {
            if let PhysicalKey::Code(code) = physical_key {
                match code {
                    KeyCode::Backspace => {
                        self.vec_text_backspace();
                        return;
                    }
                    KeyCode::Escape => {
                        self.vec_text_finish();
                        return;
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        self.vec_text_newline();
                        return;
                    }
                    _ => {}
                }
            }
            if let Some(s) = text.as_ref() {
                let mut typed = false;
                for ch in s.chars() {
                    if !ch.is_control() {
                        self.vec_text_append(ch);
                        typed = true;
                    }
                }
                if typed {
                    return;
                }
            }
        }

        // ADR-0114 W6 — Edit Mode do Flip: Delete/Backspace apaga os TRAÇOS selecionados.
        //
        // **E CONSOME a tecla** (o `return`), que é o ponto: o objeto Flip continua
        // selecionado como ENTIDADE, e o caminho genérico de Delete apaga a entidade
        // selecionada. Sem o consumo, apagar um traço apagaria o desenho inteiro junto —
        // uma tecla, dois efeitos, e o segundo é catastrófico. (Mesmo padrão do bloco
        // vetorial logo abaixo, que consome pelo mesmo motivo.)
        if self.flip_wants_edit()
            && state == ElementState::Pressed
            && !repeat
            && self.modifiers.is_empty()
            // Um campo de texto FOCADO (o rename de camada, §4.C) fica com Backspace/Delete
            // para editar o texto — senão apagar uma letra do nome apagaria os traços
            // selecionados. Mesma guarda que os atalhos de tecla-única já usam.
            && !self.text_entry_focused()
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.flip_delete_selected()
        {
            return;
        }

        // ADR-0108 Fase 1: modo vetorial (flag PH2D_VEC_PEN) — U/I/D/X fazem a
        // booleana (Union/Intersect/Difference/Exclude) das 2 últimas regiões
        // fechadas; Delete/Backspace apaga o path
        // selecionado. Modo de teste dedicado (a pill/menu real entra no cutover,
        // Fase R). Só sem modificadores, pra não colidir com atalhos.
        if self.vector_keys_live()
            && state == ElementState::Pressed
            && !repeat
            && self.modifiers.is_empty()
            && let PhysicalKey::Code(code) = physical_key
        {
            let op = match code {
                KeyCode::KeyU => Some(ph2d_vec_boolean::PathfinderOp::Union),
                KeyCode::KeyI => Some(ph2d_vec_boolean::PathfinderOp::Intersect),
                KeyCode::KeyD => Some(ph2d_vec_boolean::PathfinderOp::Subtract),
                KeyCode::KeyX => Some(ph2d_vec_boolean::PathfinderOp::Exclude),
                _ => None,
            };
            if let Some(op) = op {
                self.vec_boolean(op);
                return;
            }
            // Mesma regra de área do bloco de clipboard acima: com o mouse SOBRE a
            // timeline, Delete apaga o KEYFRAME (o bloco da timeline pega no fall-through),
            // não a forma. Sobre o canvas, apaga a forma/vértice.
            if matches!(code, KeyCode::Delete | KeyCode::Backspace)
                && !self.cursor_over_timeline()
                && self.vec_delete_selected_vertex_or_path()
            {
                return;
            }
            // Texto vetorial: `T` entra/sai do modo Text (atalho-padrão de ferramenta
            // de texto). Enquanto uma sessão de texto está ATIVA, o `T` é capturado
            // antes daqui (vira a letra digitada) — este ramo só troca o modo.
            if code == KeyCode::KeyT {
                self.vec_text_toggle_mode();
                return;
            }
        }

        // **A ESCALA DA SELEÇÃO DE NÓS** (plano 25 §6, W3b) — `Tab`/`Shift+Tab` percorre, `Ctrl+A`
        // apanha todos. Sem estes dois, trabalhar uma forma de 40 nós é clique-a-clique, que era
        // literalmente a queixa do plano.
        //
        // ⚠️ Só no modo **Node**: noutro modo não há nó selecionado a que estas teclas se refiram,
        // e o `Tab` do app tem outros donos.
        if self.vector_keys_live()
            && self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Node
            && state == ElementState::Pressed
            && let PhysicalKey::Code(code) = physical_key
        {
            // `gfx` e `vec_pen` são campos DISJUNTOS de `self` — o empréstimo se divide, e a
            // cena não precisa de ser clonada por tecla premida.
            let back = self.modifiers.shift_key();
            let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
            if let Some(gfx) = self.gfx.as_ref() {
                match code {
                    // `Tab` anda para a frente, `Shift+Tab` para trás — o percurso do Inkscape.
                    KeyCode::Tab if !ctrl => {
                        self.vec_pen.step_vert_selection(&gfx.vec_scene, !back);
                        return;
                    }
                    // `Ctrl+A` (ou `Cmd+A`) apanha TODOS os nós do caminho selecionado.
                    KeyCode::KeyA if ctrl && self.vec_pen.select_all_verts(&gfx.vec_scene) => {
                        return;
                    }
                    _ => {}
                }
            }
        }

        // Arrow keys nudge the selection (nodes if any, else the whole path).
        // Allows Shift (coarse 10px, unlike the boolean block above); blocked by
        // Ctrl/Alt/Super and while drawing. Auto-repeat keeps moving but a held
        // arrow coalesces into ONE undo step (records only on the first press).
        if self.vector_keys_live()
            && state == ElementState::Pressed
            && !self.vec_pen.is_drawing()
            && !self.modifiers.control_key()
            && !self.modifiers.alt_key()
            && !self.modifiers.super_key()
            && let PhysicalKey::Code(code) = physical_key
        {
            let step = if self.modifiers.shift_key() {
                10.0
            } else {
                1.0
            };
            let delta = match code {
                KeyCode::ArrowLeft => Some((-step, 0.0)),
                KeyCode::ArrowRight => Some((step, 0.0)),
                KeyCode::ArrowUp => Some((0.0, -step)),
                KeyCode::ArrowDown => Some((0.0, step)),
                _ => None,
            };
            if let Some((dx, dy)) = delta
                && self.vec_nudge_selected(dx, dy, !repeat)
            {
                return;
            }
        }

        // **Os acordes de ARQUIVO** — salvar, abrir, importar malha, exportar
        // malha. Extraídos para o irmão `keyboard_files.rs` quando este arquivo
        // cruzou o cap de 600 LOC do HR-18; a CHAMADA fica exatamente onde o
        // bloco estava, porque a posição dele na cadeia é load-bearing (ele
        // precede o clipboard, que também usa Ctrl).
        if self.file_chords(physical_key, state, repeat) {
            return;
        }

        // ADR-0108 Fase 2: undo/redo + clipboard com Ctrl/Cmd. Ctrl+Z desfaz (GLOBAL,
        // em `handle_editor_key`), Ctrl+C/X/V copia/recorta/cola a SELEÇÃO (Shift+V
        // cola no lugar), Ctrl+D duplica, Ctrl+G agrupa (Shift+G desagrupa). C/X/V
        // cedem o atalho a um campo de texto focado (clipboard de texto do widget).
        //
        // **A ÁREA SOB O MOUSE é dona do atalho (regra do Blender).** Com o mouse SOBRE a
        // timeline, este bloco CEDE (`!cursor_over_timeline()`): copiar/colar ali é sobre
        // KEYFRAMES, não formas — o bloco geral da timeline (mais abaixo) pega a tecla no
        // fall-through. Sobre o canvas, este bloco vale e copia/cola as FORMAS. Sem isto o
        // atalho seguia a FERRAMENTA (vetor ativo ⇒ sempre formas), e copiar keyframes com
        // o mouse na timeline copiava o desenho (Enio, 2026-07-19). Mesma regra que o
        // `cursor_over_timeline` já aplica ao pan/zoom do meio.
        if self.vector_keys_live()
            && !self.cursor_over_timeline()
            && state == ElementState::Pressed
            && !repeat
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && let PhysicalKey::Code(code) = physical_key
        {
            // ADR-0110+: undo/redo saíram DAQUI para a fila GLOBAL (`handle_editor_key`
            // → `undo_request`), que cobre geometria E transform numa fila só. O bloco
            // vetorial mantém só os atalhos que são dele (save/copy/paste/dup/group).
            //
            // ⚠️ **O `if !text_focused` de C/X/V MORREU aqui, e a remoção é o ponto**
            // (BUGS #25): a guarda subiu para o `vector_keys_live()` do topo do bloco,
            // então ela cobre os CINCO atalhos em vez de três — Ctrl+D duplicava a forma
            // e Ctrl+G a agrupava com o rename da Hierarquia aberto, porque a composição
            // à mão foi escrita quando só C/X/V pareciam disputar com um campo de texto.
            let handled = match code {
                KeyCode::KeyC => {
                    self.vec_copy();
                    true
                }
                KeyCode::KeyX => {
                    self.vec_cut();
                    true
                }
                // Ctrl+Shift+V cola NO LUGAR (sem o deslocamento diagonal).
                KeyCode::KeyV => {
                    self.vec_paste(self.modifiers.shift_key());
                    true
                }
                KeyCode::KeyD => {
                    self.vec_duplicate_shortcut();
                    true
                }
                // Ctrl+G agrupa a seleção; Ctrl+Shift+G desagrupa.
                KeyCode::KeyG => {
                    self.vec_group(!self.modifiers.shift_key());
                    true
                }
                _ => false,
            };
            if handled {
                return;
            }
        }

        // Motion Nodes M1 Phase 1b-3: graph undo/redo with Ctrl/Cmd, while the
        // Motion tool is active. Returns early when handled so the same KeyZ does
        // NOT fall through to the painter / image-edit undo in `handle_editor_key`
        // (mirror of the Vector block above).
        if self.motion_keys_live()
            && state == ElementState::Pressed
            && !repeat
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && let PhysicalKey::Code(code) = physical_key
        {
            let handled = match code {
                KeyCode::KeyZ if self.modifiers.shift_key() => {
                    self.motion_redo();
                    true
                }
                KeyCode::KeyZ => {
                    self.motion_undo();
                    true
                }
                KeyCode::KeyY => {
                    self.motion_redo();
                    true
                }
                _ => false,
            };
            if handled {
                return;
            }
        }

        // Os atalhos que a TIMELINE reivindica (undo/redo · Delete das keys · `M` do marker ·
        // o acorde C/X/V/D/R/E do dope-sheet) moram no irmão `keyboard_timeline.rs`. A ORDEM
        // é a mesma: depois de Vector/Motion (ferramenta ativa fica com o acorde), antes dos
        // Escapes. `true` = consumiu.
        if self.timeline_key(physical_key, state, repeat) {
            return;
        }

        // As teclas que ENCERRAM um gesto em curso (Esc cancela, Enter confirma) moram no
        // irmão `keyboard_escapes.rs`. ⚠️ **A ORDEM entre elas É a lei** — quem consome
        // antes de quem —, e é por isso que elas viajam juntas em vez de por dono.
        // `true` = consumiu.
        if self.escape_key(physical_key, state, repeat) {
            return;
        }

        // Painter brush size: `[` shrinks, `]` grows the active brush
        // (Blender/Photoshop convention). Consumed only when the Painter tool is
        // active (the nudge downcast gates on it), so the brackets fall through
        // otherwise. `Pressed` covers held-key repeat so the size keeps changing.
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code @ (KeyCode::BracketLeft | KeyCode::BracketRight)) =
                physical_key
        {
            let dir = if code == KeyCode::BracketRight { 1 } else { -1 };
            if self.painter_nudge_brush_size(dir) {
                return;
            }
        }

        // Painter eraser toggle: `E` flips erase mode (Blender/PS convention).
        // Consumed only when the Painter tool is active (the toggle gates on it),
        // so `E` falls through otherwise. No modifiers, no repeat.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::KeyE))
            && !(self.modifiers.super_key() || self.modifiers.control_key())
            && self.painter_toggle_eraser()
        {
            return;
        }

        // **A cadeia do DELETE no Painter** — âncora → figura → falloff, e a ORDEM é a feature
        // (`keyboard_painter`). Corta ANTES do hero, cujo caminho genérico apagaria a ENTIDADE.
        if self.painter_delete_chain(state, physical_key) {
            return;
        }

        // **O clipboard da SELEÇÃO do Painter** (Ctrl+X/C/V/A/D, Ctrl+Shift+I) — modo-exclusivo, então
        // não disputa o Ctrl+A do vetor nem o Ctrl+C do grafo (`keyboard_painter`).
        if self.painter_selection_clipboard_chain(state, physical_key) {
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

        // M12 demo controls (only on key Down, no repeat).
        if matches!((state, repeat), (ElementState::Pressed, false))
            && let PhysicalKey::Code(code) = physical_key
        {
            self.handle_editor_key(code);
        }
    }
}
