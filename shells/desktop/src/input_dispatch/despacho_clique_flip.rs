//! **O clique, a PREVIEW, o FLIP e os gizmos de nó** — ramos do `on_mouse_input` ([`super`]): a preview da UI
//! desenhada (dona do clique), os Up que fecham traços e arrastos em curso, a costura da coluna e as guias, e os
//! press do Flip (Draw, Pairs, Fill, Colorize, Trace, Erase, Reshape, Edit) e dos gizmos de pose, de selecção,
//! de warp e de field. Os corpos MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da
//! chamada, pela mesma ordem: todos os Up antes de todos os press.
//!
//! ⚠️ Cada ramo devolve `true` onde o handler fazia `return;`, e o índice devolve ao receber `true`.

use super::*;

impl crate::App {
    /// Os press do Flip no canvas (Draw, Pairs, Fill, Colorize, Trace, Erase, Reshape, Edit) e dos gizmos de pose,
    /// de selecção, de warp (só no canvas) e de field.
    pub(super) fn ramo_flip_premidos(
        &mut self,
        kind: PointerKind,
        mapped_button: ph2d_host::PointerButton,
        menu_open_before: bool,
        on_canvas: bool,
    ) -> bool {
        if self.flip_wants_canvas()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Tween v2 — correção de pares: enquanto a sessão Pairs está aberta, o clique do
        // canvas RE-PAREIA (sobrepõe o modo atual — Draw/Erase/etc — porque é um sub-modo do
        // fluxo de tween, não um modo de desenho). Uma chamada faz tudo (não é arrasto).
        if self.flip_wants_tween_pairs()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_tween_pairs_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip bucket (W4): um CLIQUE no modo Fill preenche a região sob o cursor.
        // Não é um gesto de arrasto — uma chamada faz tudo.
        if self.flip_wants_fill()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_fill_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // ADR-0114 C2: a pen-DOWN no modo Colorize começa um RABISCO (arrasto), como o Draw.
        if self.flip_wants_colorize()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_colorize_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Shift & Trace: a pen-DOWN no modo Trace pega o fantasma sob o cursor (Ctrl =
        // girar) e CONSOME mesmo errando — o Trace é dono do canvas (cair no gizmo
        // moveria o objeto no meio do posicionamento da referência).
        if self.flip_wants_trace()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_trace_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip eraser (T2.9): a pen-DOWN in Erase mode on the canvas begins an
        // erase gesture (Select falls through to gizmo/pick, like Draw).
        if self.flip_wants_erase()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_erase_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip sculpt (W5): a pen-DOWN no modo Reshape começa um gesto de escultura
        // (Select cai no gizmo/pick, como o Draw e a borracha).
        if self.flip_wants_reshape()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_reshape_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip W7.5: um handle do gizmo de POSE sob o cursor abre o arrasto de pose
        // (rotate/scale da instância). Vem ANTES do arm de canvas do Edit — um handle
        // registrado no hit-index torna `on_canvas` falso, então sem este arm o clique
        // cairia no caminho genérico de gizmo (que escreve o `Transform` do OBJETO).
        // O método só consome quando o hit é `GizmoTarget::FlipPose`.
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.flip_pose_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip §4.A: um handle do gizmo de SELEÇÃO sob o cursor abre o arrasto de
        // seleção (rotate/scale assado nos pontos). Como o da pose, vem ANTES do arm de
        // canvas do Edit — um handle no hit-index torna `on_canvas` falso. O método só
        // consome quando o hit é `GizmoTarget::FlipSelection` (mutuamente exclusivo com
        // o da pose, então nunca disputam o mesmo clique).
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.flip_selection_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Motion Nodes: um handle do gizmo de FIELD sob o cursor abre o arrasto (rotate/
        // scale/translate escrito nos params do NÓ). Como os do Flip, vem ANTES do caminho
        // genérico de gizmo — um handle no hit-index o alcançaria e escreveria um
        // `Transform`. O método só consome quando o hit é `GizmoTarget::MotionField`, que
        // só existe com a tool Motion ativa + um field espacial selecionado no grafo.
        // Motion Nodes: uma ALÇA do gizmo dos deformadores de quadrilátero (Corner Pin +
        // Bezier Warp) sob o cursor abre o arrasto, que escreve os params do NÓ. Vem antes
        // do gizmo de field e do genérico pela mesma razão que aquele: uma alça alcançada
        // pelo caminho genérico escreveria um `Transform` de entidade. O método só consome
        // quando há retrato publicado — ou seja com a tool Motion activa e um dos dois nós
        // seleccionado.
        //
        // ⚠️ **E ele exige `on_canvas`, ao contrário dos irmãos** — Enio, 2026-08-23:
        // *"se colocar transform antes, não é possível conectar transform em Bezier
        // Warp"*. Os gizmos acima consomem pelo HIT-INDEX (`GizmoTarget::…`), que já
        // sabe das regiões; este faz o seu próprio hit-test em coordenadas de MUNDO, e
        // sem guarda ele convertia um clique **no painel do grafo** para o mundo, calhava
        // de cair sobre uma alça, e ENGOLIA o gesto de ligar um fio. *Um consumidor que
        // decide sozinho tem de saber sozinho onde ele vale.*
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && on_canvas
            && self.warp_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.field_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip Edit Mode (W6): um CLIQUE no modo Edit seleciona o TRAÇO sob o cursor
        // (Shift alterna; no vazio, desmarca). Consome mesmo errando o traço — no Edit o
        // gizmo de objeto não manda, senão o arrasto seguinte moveria o objeto inteiro.
        if self.flip_wants_edit()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_edit_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        false
    }
}
