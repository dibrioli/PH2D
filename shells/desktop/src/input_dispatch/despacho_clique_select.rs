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

    /// Os picks modais da física (corpo da junta, montagem e corda da roldana), a explosão/atracção, o desenho de
    /// junta, as alças do texto e das fichas no Select, e o duplo-clique e a âncora do motion path.
    pub(super) fn ramo_picks_de_fisica_e_alcas(
        &mut self,
        kind: PointerKind,
        mapped_button: ph2d_host::PointerButton,
        evt: PointerEvent,
        menu_open_before: bool,
    ) -> bool {
        // **O eyedropper de corpo do joint** (§12) — mesma classe de pick modal do
        // acima, mas independente de ferramenta: armado, o próximo Down no canvas
        // escolhe o corpo sob o cursor e religa aquela ponta. Precede o
        // picking/gizmo; nenhum outro objeto precisa estar pré-selecionado.
        if self.joint_body_pick.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
        {
            self.joint_body_pick_click(evt.x, evt.y);
            return true;
        }
        // **O eyedropper de MONTAGEM da roldana** (§13, W-Pulley W3) — a mesma
        // classe de pick modal, uma família adiante: armado, o próximo Down
        // escolhe o CORPO em que o eixo daquela roldana se monta.
        if self.wheel_body_pick.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
        {
            self.wheel_mount_pick_click(evt.x, evt.y);
            return true;
        }
        // **O eyedropper de CORDA da roldana** (§13, W-Pulley W1) — a mesma classe
        // de pick modal, e o único cujo alvo NÃO é um sprite: uma corda é uma
        // linha, e ela é apontada pela ROTA que o overlay desenha.
        if self.wheel_rope_pick.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
        {
            self.wheel_rope_pick_click(evt.x, evt.y);
            return true;
        }
        // **A EXPLOSÃO e a ATRAÇÃO** (W-Hand) — modal como os picks acima e pela
        // MESMA razão estrutural: elas precisam só de um PONTO, então não podem
        // pendurar no pick de canvas (que só dispara quando há algo sob o cursor).
        // A MÃO fica onde estava, dentro do pick, para a seleção seguir acontecendo;
        // quem decide de qual família a ferramenta é é `needs_a_body`, uma porta só.
        if mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && !self.physics.interaction.tool.needs_a_body()
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.poke_press(evt.x, evt.y)
        {
            return true;
        }
        // **O gesto de DESENHAR um joint** (W-J4) — a 2ª rota de criação, e a única
        // em que as âncoras nascem NOS pontos que a mão indicou. Modal como o
        // eyedropper acima (precede picking/gizmo, independe de ferramenta): o
        // press começa a banda elástica, o Move a estica, o release cria.
        if self.physics.joint_draw_armed
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.joint_draw_press(evt.x, evt.y)
        {
            return true;
        }
        if self.physics.joint_draw.is_some() {
            match kind {
                PointerKind::Move => {
                    self.joint_draw_move(evt.x, evt.y);
                    return true;
                }
                PointerKind::Up => {
                    self.joint_draw_release(evt.x, evt.y);
                    return true;
                }
                _ => {}
            }
        }
        // **A alça do TEXTO EM CAMINHO** (W5), no modo Select — irmã da do conector, mesmo lugar
        // e mesma razão: no Select a tool não captura o canvas, e sem este arm a pressão iria
        // para o picking/gizmo. O gizmo é inócuo sobre um texto vinculado (vive na identidade),
        // então o Select é a casa natural da alça — e sem as âncoras do Node ela não se confunde
        // com ponto de objeto nenhum (Enio, smoke). Só devolve `true` sobre a alça de um texto
        // vinculado SELECIONADO; qualquer outro caso segue o caminho de sempre.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(w) = self.vec_world_at((evt.x, evt.y))
            && self.vec_textpath_handle_down(w)
        {
            return true;
        }
        // O MESMO guard para as alças do PATTERN (W4): no Select a tool não captura o canvas e o
        // gizmo é inócuo sobre um motivo vinculado, então a ficha precisa deste arm antes do
        // picking/gizmo. Irmão do `vec_textpath_handle_down` logo acima.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(w) = self.vec_world_at((evt.x, evt.y))
            && self.vec_patternpath_handle_down(w)
        {
            return true;
        }
        // **DUPLO-clique no CAMINHO insere um ponto** (ADR-0141), ANTES do arrasto de âncora:
        // um duplo-clique sobre a curva é "adicionar ponto", não "arrastar". O 1º clique do
        // par devolve `false` (não é duplo) e cai adiante como um clique normal; só o 2º sobre
        // a curva consome.
        if mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.motion_path_curve_double_click(evt.x, evt.y)
        {
            return true;
        }
        // **A ÂNCORA do MOTION PATH** (ADR-0141), antes do picking/gizmo pela mesma razão
        // que as alças acima: a âncora do primeiro key cai em cima do sprite, e sem este
        // arm a pressão iria para o gizmo — que moveria o OBJETO onde o dedo pediu a
        // CURVA. Sem gate de ferramenta de propósito (ver `motion_path_anchor_down`).
        if mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.motion_path_anchor_down(evt.x, evt.y)
        {
            return true;
        }
        false
    }
}
