//! **O clique, os PICKS e as ALÇAS antes da ferramenta** — ramos do `on_mouse_input` ([`super`]): o que
//! vale no modo Select (Set Center, duplo-clique no texto, alças do conector, do texto e das fichas), os picks
//! MODAIS (caminho-guia, alvo do osso inteligente, corpo da junta, montagem e corda da roldana), a explosão e
//! a atracção, o desenho de junta, o motion path, e os Up que fecham essas alças. Os corpos MUDARAM-SE
//! verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Cada ramo devolve `true` onde o handler fazia `return;`, e o índice devolve ao receber `true`.

use super::*;

impl crate::App {
    /// Os Up que fecham as alças independentes de modo: conector, texto em caminho, osso, fichas e âncora do
    /// motion path (esta fecha também o passo de undo que o press abriu).
    pub(super) fn ramo_alcas_soltas(
        &mut self,
        kind: PointerKind,
        mapped_button: ph2d_host::PointerButton,
        evt: PointerEvent,
    ) -> bool {
        // O Up que FECHA o arrasto de alça (ele nasceu no Select, e é lá que morre).
        if self.vec.conn_handle.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            if let Some(w) = self.vec_world_at((evt.x, evt.y)) {
                self.conn_handle_up(w);
            } else {
                self.conn_handle_cancel();
            }
            return true;
        }
        // O Up que fecha o arrasto da alça do texto — nasceu no Select, morre no Select.
        if self.vec.textpath_handle_drag
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            self.vec.textpath_handle_drag = false;
            return true;
        }
        // ⭐⭐⭐ **O Up que fecha o arrasto de uma ALÇA DE OSSO.**
        //
        // ⚠️ Ela CONSOME o gesto: sem isto, soltar depois de girar um osso cai na cadeia de baixo e
        // a forma sob o cursor é seleccionada.
        //
        // ⛔⛔ **Ele vivia DENTRO do bloco `vector_tool_active() && modo != Select`** e a alça passou
        // a poder ser agarrada em todo modo (auditoria de 2026-09-08) ⇒ no modo **Select** o slot
        // era armado e **nunca** libertado: o osso seguia o rato para sempre, sem botão nenhum
        // apertado. *Um slot de arrasto tem de ser largado onde quer que possa ser agarrado* — e é
        // por isso que ele passou para esta família, que é a dos irmãos independentes de modo.
        if self.skeleton.bone_pose.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            self.skeleton.bone_pose = None;
            return true;
        }
        // O Up que fecha o arrasto de uma ficha do PATTERN (W4) — mesma vida da do texto.
        if self.vec.patternpath_handle.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            self.vec.patternpath_handle = None;
            return true;
        }
        // O Up que fecha o arrasto de uma ÂNCORA do motion path — e que FECHA o passo de
        // undo que o press abriu. Sem este `commit_if_changed` o `begin` fica pendurado e
        // o próximo gesto o herda: um Ctrl+Z desfaria os dois de uma vez.
        if self.motion_shell.path_drag.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            self.motion_shell.path_drag = None;
            self.timeline.history.commit_if_changed(&self.timeline.doc);
            return true;
        }
        false
    }
}
