//! **A cadeia do teclado** — ramos do `key_input` ([`super`]): as teclas 3D antes do store, o texto
//! vetorial, o Delete do Flip e os atalhos do vetor, a selecção de nós e o nudge, os acordes de ficheiro e de
//! clipboard, o undo do grafo, e a cauda das cadeias — timeline, escapes, Painter e Hierarquia. Os corpos
//! MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ O nome começa por `keyboard` DE PROPÓSITO: o censo `the_key_blocks_ask_whether_the_keys_are_live` varre
//! a família `keyboard*.rs`, e um bloco vetorial fora dela ficaria fora da pergunta «as teclas estão vivas?».
//! Os blocos eram de nível de topo do `key_input` e continuam à mesma indentação — a agulha exacta do
//! `the_hierarchy_has_a_delete_key` casa no texto emendado.

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
}
