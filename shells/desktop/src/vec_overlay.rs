//! O que o frame vetorial desenha POR CIMA da cena, por modo de desenho (ADR-0112).
//!
//! Há duas famílias de overlay, com regras de visibilidade DIFERENTES — confundir as
//! duas foi o bug do P1 (as guias de snap sumiam no modo Select):
//!
//! - **Overlays de edição** (âncoras, handles bézier, alças de gradiente, marquee de
//!   box-select): só FORA do Select. No Select quem transforma a forma é o gizmo de
//!   sprite; as alças de nó comeriam o clique dele (ADR-0112).
//! - **Guias de snap** (as linhas que explicam o encaixe vivo): valem em QUALQUER
//!   modo. O gizmo-move do Select encaixa a forma igual ao pen/shape-tool/node-edit,
//!   então a guia precisa aparecer lá também — era o que faltava.
//!
//! A decisão vive aqui, fora do `render_loop`, para ser testável sem gfx/câmera: é o
//! ponto que trava o P1 (uma regressão que re-agrupasse a guia sob o guard de modo
//! quebra o teste abaixo, não só o smoke).

use ph2d_tool_vector::DrawMode;

/// Quais overlays vetoriais este frame desenha, dado se a tool está ativa e o modo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct VecOverlayPlan {
    /// Âncoras, handles, alças de gradiente e marquee — edição/seleção-de-forma,
    /// que não aparecem no Select (lá manda o gizmo, ADR-0112).
    pub edit: bool,
    /// Guias de alinhamento do snap — desenhadas em TODOS os modos, inclusive o
    /// Select (gizmo-move).
    pub snap_guides: bool,
    /// A **gaiola do Envelope** (ADR-0129, Fatia 1) — só no **Node**: no Select manda o gizmo, no
    /// Pen/Shape o clique tem outro dono. Este flag só diz que a gaiola *pode* aparecer neste modo;
    /// se a forma selecionada é de fato um envelope é o [`crate::envelope_gesture::view`] que decide
    /// (devolve `None` quando não é).
    pub envelope_cage: bool,
    /// A **alça do texto em caminho** (plano 22, W5) — só no **Select**. ⚠️ Ao contrário da
    /// gaiola do envelope (que é do Node), esta vive no Select: no Node ela se perdia no meio das
    /// âncoras dos outros paths (Enio, smoke), e no Select o gizmo é inócuo sobre um texto
    /// vinculado (identidade) — então o Select não tem âncora nenhuma a poluir a tela. Este flag
    /// só diz que a alça *pode* aparecer; se há um texto vinculado na seleção é o
    /// [`crate::vec_text_ride::handle::world`] que decide (devolve `None` quando não há).
    pub textpath_handle: bool,
    /// As **duas alças do Pattern on Path** (plano 23, W4) — Start e End do trecho, só no
    /// **Select**, pela MESMA razão da alça do texto (no Node se perderiam nas âncoras). Este flag
    /// só diz que elas *podem* aparecer; se há um pattern vinculado no primário é o
    /// [`crate::pattern_live::handle::world`] que decide (devolve `None` quando não há).
    pub patternpath_handles: bool,
    /// **As alças de LARGURA** (plano 25 §5) — uma por parada do perfil, na forma selecionada.
    pub width_handles: bool,
    // NOTA: o antigo `corner_handles` (a alça de raio na bissetriz, só no Node) foi REMOVIDO — o
    // arredondar/chanfrar quina virou o par de ferramentas Fillet / Chamfer (o gesto de
    // clicar-e-arrastar sobre a quina). A consolidação que o Enio pediu.
}

/// A política de visibilidade dos overlays vetoriais deste frame.
#[must_use]
pub(crate) fn vec_overlay_plan(vector_active: bool, mode: DrawMode) -> VecOverlayPlan {
    VecOverlayPlan {
        // O **Build** também fica de fora, e pela mesma razão do Select: o que ele
        // manipula é a REGIÃO, não a âncora. Âncoras e handles por cima de um emaranhado
        // de formas sobrepostas só atrapalham a leitura das faces — que é a única coisa
        // que o artista precisa enxergar ali. O **Pick Shapes** (Blend) idem: o que se
        // manipula é a LISTA de formas, não os nós de uma delas.
        edit: vector_active
            && !matches!(
                mode,
                DrawMode::Select | DrawMode::Build | DrawMode::PickBlend
            ),
        snap_guides: vector_active,
        // Node-only (mesma razão de modo do `edit`, mais estreita). Se a seleção é um envelope é
        // `envelope_gesture::view` que resolve — aqui é só a política de MODO.
        envelope_cage: vector_active && mode == DrawMode::Node,
        // ⚠️ **Select**, não Node — a exceção deliberada entre os overlays. No Node a bolinha se
        // confundia com as âncoras dos outros paths; no Select não há âncoras, e o gizmo é inócuo
        // sobre o texto vinculado. A alça é grande e colorida justamente para saltar do gizmo.
        textpath_handle: vector_active && mode == DrawMode::Select,
        // As alças do Pattern seguem a MESMA regra da alça do texto (Select-only, mesma razão).
        patternpath_handles: vector_active && mode == DrawMode::Select,
        // ⚠️ **Modo Width e SÓ ele.** Estas alças ficam FORA da curva, à distância da fita, e num
        // multiplicador pequeno pousam a milímetros das âncoras — no Node elas competiriam com o
        // que aquele modo existe para editar. É a mesma razão que fez a alça de quina virar as
        // ferramentas Fillet/Chamfer.
        width_handles: vector_active && mode == DrawMode::Width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O P1: no modo Select as guias de snap TÊM que aparecer (o gizmo-move encaixa a
    /// forma), mas os overlays de edição de nó NÃO — lá quem fala é o gizmo.
    #[test]
    fn select_mode_shows_snap_guides_but_not_edit_overlays() {
        let plan = vec_overlay_plan(true, DrawMode::Select);
        assert!(
            plan.snap_guides,
            "guia de snap no Select (gizmo-move encaixa)"
        );
        assert!(!plan.edit, "âncoras/handles NÃO no Select (ADR-0112)");
    }

    /// **O Shape Builder não desenha âncoras.** O que ele manipula é a região; um mapa de
    /// nós por cima de formas sobrepostas esconde exatamente o que o artista precisa ver.
    #[test]
    fn build_mode_shows_no_node_overlays_because_it_manipulates_regions() {
        let plan = vec_overlay_plan(true, DrawMode::Build);
        assert!(!plan.edit, "sem âncoras/handles no Build");
        assert!(plan.snap_guides, "as guias de snap valem em todo modo");
    }

    /// **As alças de largura pertencem ao modo Width e a mais nenhum.** Elas vivem FORA da curva,
    /// e num multiplicador pequeno pousam em cima das âncoras — no Node roubariam o clique do nó,
    /// que é o defeito que tirou a alça de quina de lá.
    #[test]
    fn the_width_handles_belong_to_width_mode_alone() {
        assert!(vec_overlay_plan(true, DrawMode::Width).width_handles);
        for m in [
            DrawMode::Select,
            DrawMode::Node,
            DrawMode::Pen,
            DrawMode::Pencil,
            DrawMode::Fillet,
            DrawMode::Chamfer,
        ] {
            assert!(
                !vec_overlay_plan(true, m).width_handles,
                "as alças de largura apareceram no modo {m:?}"
            );
        }
        assert!(
            !vec_overlay_plan(false, DrawMode::Width).width_handles,
            "com a tool inativa não há overlay nenhum"
        );
    }

    /// Nos modos de desenho/edição de nó E nas ferramentas de QUINA, os overlays de edição
    /// aparecem. Fillet/Chamfer PRECISAM das âncoras desenhadas: são elas as quinas que se vai
    /// clicar — sem o overlay, o alvo do gesto fica invisível.
    #[test]
    fn draw_node_and_corner_tool_modes_show_edit_overlays() {
        for mode in [
            DrawMode::Pen,
            DrawMode::Node,
            DrawMode::Shape,
            DrawMode::Fillet,
            DrawMode::Chamfer,
        ] {
            let plan = vec_overlay_plan(true, mode);
            assert!(plan.edit, "{mode:?} desenha âncoras/handles");
            assert!(plan.snap_guides, "{mode:?} desenha guias de snap");
        }
    }

    /// **Pick Shapes (Blend) não desenha âncoras** — como o Build, o que ele manipula é a lista de
    /// formas, não os nós de uma. O realce das escolhidas + a prévia do spine vêm de outro passe.
    #[test]
    fn pickblend_mode_shows_no_node_overlays() {
        let plan = vec_overlay_plan(true, DrawMode::PickBlend);
        assert!(!plan.edit, "sem âncoras/handles no Pick Shapes");
        assert!(plan.snap_guides, "guias de snap valem em todo modo");
    }

    /// Tool inativa: nenhum overlay vetorial (o canvas é de outra ferramenta).
    #[test]
    fn inactive_tool_draws_no_vector_overlays() {
        let plan = vec_overlay_plan(false, DrawMode::Node);
        assert!(!plan.edit);
        assert!(!plan.snap_guides);
        assert!(!plan.envelope_cage);
        assert!(!plan.textpath_handle);
        assert!(!plan.patternpath_handles);
    }

    /// **As alças do Pattern são do SELECT, e só dele** (plano 23, W4) — a mesma razão da alça do
    /// texto: no Node se perderiam nas âncoras dos outros paths.
    #[test]
    fn the_patternpath_handles_belong_to_select_mode_alone() {
        assert!(vec_overlay_plan(true, DrawMode::Select).patternpath_handles);
        for mode in [
            DrawMode::Node,
            DrawMode::Pen,
            DrawMode::Shape,
            DrawMode::Build,
            DrawMode::PickBlend,
        ] {
            assert!(
                !vec_overlay_plan(true, mode).patternpath_handles,
                "{mode:?} NÃO desenha as alças do pattern"
            );
        }
        assert!(!vec_overlay_plan(false, DrawMode::Select).patternpath_handles);
    }

    // NOTA: o teste `the_corner_radius_handle_belongs_to_node_mode_alone` saiu com o campo
    // `corner_handles` — a alça de raio virou o par de ferramentas Fillet / Chamfer. A exclusão de
    // forma VIVA (o que aquele teste protegia) segue viva no `has_derived_verts`, que as
    // ferramentas de quina consultam (gate `every_host_that_rewrites_verts_faces_the_radius_handle`).

    /// **A gaiola do Envelope é do NODE, e só dele** (ADR-0129 Fatia 1). Fora do Node
    /// (Select/Pen/Shape/Build/PickBlend) o clique tem outro dono e a gaiola não pode existir; tool
    /// inativa idem.
    #[test]
    fn the_envelope_cage_belongs_to_node_mode_alone() {
        assert!(vec_overlay_plan(true, DrawMode::Node).envelope_cage);
        for mode in [
            DrawMode::Select,
            DrawMode::Pen,
            DrawMode::Shape,
            DrawMode::Build,
            DrawMode::PickBlend,
        ] {
            assert!(
                !vec_overlay_plan(true, mode).envelope_cage,
                "{mode:?} NÃO desenha a gaiola do envelope"
            );
        }
        assert!(
            !vec_overlay_plan(false, DrawMode::Node).envelope_cage,
            "tool inativa não desenha a gaiola"
        );
    }

    /// **A alça do texto em caminho é do SELECT, e só dele** (Enio, smoke: no Node ela se confunde
    /// com as âncoras). ⚠️ É a exceção entre os overlays — a gaiola do envelope e as âncoras são do
    /// Node; esta é do Select justamente porque lá não há âncoras e o gizmo é inócuo sobre o texto
    /// vinculado.
    #[test]
    fn the_textpath_handle_belongs_to_select_mode_alone() {
        assert!(vec_overlay_plan(true, DrawMode::Select).textpath_handle);
        for mode in [
            DrawMode::Node,
            DrawMode::Pen,
            DrawMode::Shape,
            DrawMode::Build,
            DrawMode::PickBlend,
        ] {
            assert!(
                !vec_overlay_plan(true, mode).textpath_handle,
                "{mode:?} NÃO desenha a alça do texto"
            );
        }
        assert!(
            !vec_overlay_plan(false, DrawMode::Select).textpath_handle,
            "tool inativa não desenha a alça"
        );
    }
}
