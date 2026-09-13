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

    /// O Set Center armado, o duplo-clique no texto e as alças do conector no Select, o pick do caminho-guia, o
    /// pick do alvo do osso inteligente e as alças do osso fora do modo Osso.
    pub(super) fn ramo_select_modais(
        &mut self,
        kind: PointerKind,
        mapped_button: ph2d_host::PointerButton,
        evt: PointerEvent,
        menu_open_before: bool,
        on_canvas: bool,
    ) -> bool {
        // "Set Center" armado (ADR-0112): a pressão põe a ORIGEM da forma selecionada
        // sob o cursor e desarma. Vale em QUALQUER modo — inclusive Select, onde o
        // pivô do gizmo é o que se está ajustando.
        if self.vec.pivot_edit
            && self.vector_tool_active()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && on_canvas
            && !menu_open_before
        {
            self.vec.pivot_edit = false;
            if self.vec_set_origin_to_cursor(evt.x, evt.y) {
                return true;
            }
        }
        // Duplo-clique num TEXTO no modo Select ⇒ entra na edição dele (o gesto padrão
        // de todo editor vetorial). Antes do bloco abaixo porque no Select a tool NÃO
        // captura o canvas — sem isto a pressão iria para o gizmo e arrastaria a forma.
        //
        // **NÃO use `on_canvas` aqui**: ele exige o `hit_index` VAZIO sob o cursor, e o
        // gizmo REGISTRA os hits dele no `hit_index` (as alças + o interior "Translate").
        // Como o 1º clique do par SELECIONA o objeto, o gizmo passa a cobrir a forma —
        // então no 2º clique `on_canvas` é falso e o duplo-clique nunca dispararia (o bug
        // do 1º smoke). O que vale aqui é: fora de painel, e o único widget sob o cursor
        // pode ser o gizmo — que é exatamente o que está por cima do texto.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.vec_text_double_click(evt.x, evt.y)
        {
            return true;
        }
        // **As alças de ponta do conector** (os dois círculos), no modo Select. Mesmo lugar e
        // mesma razão do duplo-clique de texto acima: no Select a tool não captura o canvas, e
        // sem este arm a pressão iria para o picking/gizmo — que selecionaria a forma ATRÁS da
        // alça em vez de arrastá-la.
        //
        // O `conn_handle_down` só devolve `true` quando o cursor está mesmo sobre uma alça de
        // um conector SELECIONADO; em qualquer outro caso o clique segue o caminho de sempre.
        // É esse contrato que mantém o resto do editor intacto.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(w) = self.vec_world_at((evt.x, evt.y))
            && self.conn_handle_down(w)
        {
            return true;
        }
        // **O Picker de caminho-guia ARMADO** (Enio 2026-07-23): enquanto se escolhe um guia, o
        // clique no canvas PRENDE (ou desiste no vazio) — nunca seleciona nem arrasta ficha. Por
        // isso precede as alças e o picking/gizmo: um pick em curso é modal, e o clique é dele.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.vec.path_pick.is_some()
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(w) = self.vec_world_at((evt.x, evt.y))
        {
            self.vec_path_pick_click(w);
            return true;
        }
        // ⭐⭐⭐ **O PICK DO ALVO de um osso inteligente** — a mesma classe modal, e **independente
        // de ferramenta** de propósito.
        //
        // ⛔⛔ **Report do dono (2026-09-08): *«Pick object deve inibir a criação de bones. Ao tentar
        // fazer o pick no canvas criou um osso indesejado»*.** Ele arma o pick a partir da secção
        // Skeleton, logo está na ferramenta **Bone** — onde um `Down` no canvas **cria um osso**. A
        // 1.ª versão deste pick não consumia o press: ela esperava que a SELECÇÃO mudasse, e no modo
        // *Criar* o clique não selecciona coisa nenhuma, **desenha**.
        //
        // ⇒ *um pick modal que não consome o press herda o gesto da ferramenta em que foi armado*, e
        // a ferramenta em que este é armado é a única que CRIA no clique. Precede as alças e o
        // picking/gizmo, como os irmãos.
        if self.skeleton.smart_pick.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
        {
            self.smart_pick_click(evt.x, evt.y);
            return true;
        }
        // ⭐⭐⭐ **AS ALÇAS DO OSSO PEGAM EM TODO MODO DE VECTOR, e não só no modo Osso.**
        //
        // ⛔⛔ **Achado da auditoria de 2026-09-08:** o arco de limite — e a alça da força, e a
        // ponta da corrente — é **pintado e ACENDE sob o rato nos 14 modos** (o
        // `refresh_bone_hover` não se gateia pelo modo, de propósito, porque o `vec_overlay::bones`
        // também não) e o `Down` só era lido dentro do `DrawMode::Bone`. *Um controlo que acende
        // debaixo do dedo e não responde é a espécie de morto que este repo já pagou três vezes.*
        //
        // ⚠️ **QUAIS alças é a porta [`crate::bone_pick::grabbable_outside_bone_mode`]**, e a
        // linha é o VERBO: entram as quatro que nenhuma outra ferramenta sabe exprimir; girar e
        // deslocar ficam com o gizmo de sprite, que já os faz.
        //
        // ⛔ **Dentro do modo Osso este arm NÃO corre** — lá a `bone_gesture::press` decide, e ela
        // distingue *Criar* de *Transformar*: em *Criar*, pousar sobre uma alça só ACENDE o osso,
        // que é o desenho e não um esquecimento.
        //
        // ⛔ **Consome o press**, como os picks modais acima e pela mesma razão: sem o `return;` o
        // gesto cai na cadeia de baixo e o modo Select começa um marquee por cima do arrasto.
        if mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.vector_tool_active()
            && self.vec.draw_config.mode != ph2d_tool_vector::DrawMode::Bone
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(h) = self.bone_handle_at((evt.x, evt.y))
        {
            self.skeleton.bone_pose = Some((h.bone, h.part));
            return true;
        }
        false
    }
}
