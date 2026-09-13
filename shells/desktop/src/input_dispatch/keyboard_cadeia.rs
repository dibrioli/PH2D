//! **A cadeia do teclado** — ramos do `key_input` ([`super`]), corpos verbatim pela mesma ordem: teclas 3D, texto
//! vetorial, Delete do Flip, atalhos do vetor, nós e nudge, acordes, undo do grafo, e a cauda (timeline, escapes,
//! Painter, Hierarquia). ⚠️ O nome começa por `keyboard` DE PROPÓSITO (o `the_key_blocks_ask_whether_the_keys_are_live`
//! varre `keyboard*.rs`), e os blocos ficam à mesma indentação (a agulha do `the_hierarchy_has_a_delete_key`).

use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

impl crate::App {
    /// A cauda das cadeias: a timeline, as teclas que encerram um gesto, o tamanho e a borracha do pincel, o Delete
    /// e o clipboard da selecção do Painter, e o Delete/duplicar da Hierarquia.
    pub(super) fn ramo_teclas_timeline_painter_hierarquia(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
    ) -> bool {
        // Os atalhos que a TIMELINE reivindica (undo/redo · Delete das keys · `M` do marker ·
        // o acorde C/X/V/D/R/E do dope-sheet) moram no irmão `keyboard_timeline.rs`. A ORDEM
        // é a mesma: depois de Vector/Motion (ferramenta ativa fica com o acorde), antes dos
        // Escapes. `true` = consumiu.
        if self.timeline_key(physical_key, state, repeat) {
            return true;
        }

        // As teclas que ENCERRAM um gesto em curso (Esc cancela, Enter confirma) moram no
        // irmão `keyboard_escapes.rs`. ⚠️ **A ORDEM entre elas É a lei** — quem consome
        // antes de quem —, e é por isso que elas viajam juntas em vez de por dono.
        // `true` = consumiu.
        if self.escape_key(physical_key, state, repeat) {
            return true;
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
                return true;
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
            return true;
        }

        // **A cadeia do DELETE no Painter** — âncora → figura → falloff, e a ORDEM é a feature
        // (`keyboard_painter`). Corta ANTES da Hierarquia, cujo `Delete` apagaria a ENTIDADE.
        if self.painter_delete_chain(state, physical_key) {
            return true;
        }

        // **O clipboard da SELEÇÃO do Painter** (Ctrl+X/C/V/A/D, Ctrl+Shift+I) — modo-exclusivo, então
        // não disputa o Ctrl+A do vetor nem o Ctrl+C do grafo (`keyboard_painter`).
        if self.painter_selection_clipboard_chain(state, physical_key) {
            return true;
        }

        // ⭐⭐⭐ **A HIERARQUIA: `Delete` apaga a selecção, `Ctrl/Cmd+D` duplica-a** (report do Enio,
        // 2026-08-30 — `keyboard_hierarchy`).
        //
        // ⛔ **O «caminho genérico do hero» que os comentários acima invocam NÃO EXISTE** — o
        // `KEY_DELETE` do dispatcher vira `GraphKey::Delete`, e o único consumidor dele na árvore é
        // o painel do grafo de motion. Apagar um objeto só era possível pelo menu de contexto.
        //
        // ⚠️ Ela entra AQUI — depois de toda cadeia específica e antes do encaminhamento ao widget
        // focado — e é gateada ao ponteiro estar sobre o painel: sem isso roubaria o `Delete` do
        // traço do Flip, do nó de curva, da figura do Painter, da key da timeline e do nó do grafo.
        if self.hierarchy_key_chain(state, repeat, physical_key) {
            return true;
        }
        false
    }

    /// A selecção de nós (`Tab`, `Ctrl+A`), o nudge pelas setas, os acordes de ficheiro, o clipboard do vetor (a área
    /// sob o rato é dona do atalho) e o undo/redo do grafo do Motion.
    pub(super) fn ramo_teclas_nos_ficheiros_e_acordes(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
    ) -> bool {
        // **A ESCALA DA SELEÇÃO DE NÓS** (plano 25 §6, W3b) — `Tab`/`Shift+Tab` percorre, `Ctrl+A`
        // apanha todos. Sem estes dois, trabalhar uma forma de 40 nós é clique-a-clique, que era
        // literalmente a queixa do plano.
        //
        // ⚠️ Só no modo **Node**: noutro modo não há nó selecionado a que estas teclas se refiram,
        // e o `Tab` do app tem outros donos.
        if self.vector_keys_live()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node
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
                        self.vec.pen.step_vert_selection(&gfx.vec_scene, !back);
                        return true;
                    }
                    // `Ctrl+A` (ou `Cmd+A`) apanha TODOS os nós do caminho selecionado.
                    KeyCode::KeyA if ctrl && self.vec.pen.select_all_verts(&gfx.vec_scene) => {
                        return true;
                    }
                    _ => {}
                }
            }
        }

        // Arrow keys nudge the selection (nodes if any, else the whole path).
        // Allows Shift (coarse 10px, unlike the boolean block above); blocked by
        // Ctrl/Alt/Super and while drawing. Auto-repeat keeps moving. (Until 2026-09-12 a
        // held arrow recorded ONE step in the vector `History`, on the first press; that
        // stack died unread — the Ctrl+Z is the global queue's.)
        if self.vector_keys_live()
            && state == ElementState::Pressed
            && !self.vec.pen.is_drawing()
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
                && self.vec_nudge_selected(dx, dy)
            {
                return true;
            }
        }

        // **Os acordes de ARQUIVO** — salvar, abrir, importar malha, exportar
        // malha. Extraídos para o irmão `keyboard_files.rs` quando este arquivo
        // cruzou o cap de 600 LOC do HR-18; a CHAMADA fica exatamente onde o
        // bloco estava, porque a posição dele na cadeia é load-bearing (ele
        // precede o clipboard, que também usa Ctrl).
        if self.file_chords(physical_key, state, repeat) {
            return true;
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
                return true;
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
                return true;
            }
        }
        false
    }

    /// A digitação do texto vetorial, o Delete dos traços do Flip no Edit, e as teclas nuas do vetor (booleanas,
    /// Delete da forma, `T` do modo Text).
    pub(super) fn ramo_teclas_texto_flip_e_vetor(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
        text: &Option<winit::keyboard::SmolStr>,
    ) -> bool {
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
                        return true;
                    }
                    KeyCode::Escape => {
                        self.vec_text_finish();
                        return true;
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        self.vec_text_newline();
                        return true;
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
                    return true;
                }
            }
        }

        // ADR-0114 W6 — Edit Mode do Flip: Delete/Backspace apaga os TRAÇOS selecionados.
        //
        // **E CONSOME a tecla** (o `return`), que é o ponto: o objeto Flip continua
        // selecionado como ENTIDADE, e a cadeia da Hierarquia apaga a entidade
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
            return true;
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
                return true;
            }
            // Mesma regra de área do bloco de clipboard acima: com o mouse SOBRE a
            // timeline, Delete apaga o KEYFRAME (o bloco da timeline pega no fall-through),
            // não a forma. Sobre o canvas, apaga a forma/vértice.
            if matches!(code, KeyCode::Delete | KeyCode::Backspace)
                && !self.cursor_over_timeline()
                && self.vec_delete_selected_vertex_or_path()
            {
                return true;
            }
            // Texto vetorial: `T` entra/sai do modo Text (atalho-padrão de ferramenta
            // de texto). Enquanto uma sessão de texto está ATIVA, o `T` é capturado
            // antes daqui (vira a letra digitada) — este ramo só troca o modo.
            if code == KeyCode::KeyT {
                self.vec_text_toggle_mode();
                return true;
            }
        }
        false
    }

    /// A cena de escultura (a divisão `Ctrl+Alt+Q` antes do resto) e o modelador 3D tomam as teclas deles antes do
    /// store.
    pub(super) fn ramo_teclas_3d(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
    ) -> bool {
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
            return true;
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
            return true;
        }
        // ⭐ **AS TECLAS DO MODELADOR 3D, numa porta só** — ver
        // [`keyboard_field3d`](super::keyboard_field3d).
        if self.field3d_keys(physical_key, state) {
            return true;
        }
        false
    }
}
