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
    /// As alças de **raio de quina** (Live Corners) — só no **Node**, e é mais estreito
    /// que o `edit`.
    ///
    /// No **Pen** o clique cria e edita ponto: uma alça de raio ali disputaria o pixel com
    /// o gesto de desenhar, e arredondar quina não é o que a caneta faz. No **Shape** a
    /// forma é PARAMÉTRICA — o raio dela é um campo do `VecShape` (o painel), não o
    /// `corner_radius` do vértice; a geometria é re-cozida do zero a cada mudança de
    /// parâmetro e o raio por-vértice seria varrido junto. No **Select** manda o gizmo.
    ///
    /// Sobra o Node — que é exatamente onde o Illustrator e o Affinity a põem.
    pub corner_handles: bool,
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
        corner_handles: vector_active && mode == DrawMode::Node,
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
        assert!(!plan.corner_handles, "e sem alça de raio de quina");
        assert!(plan.snap_guides, "as guias de snap valem em todo modo");
    }

    /// Nos modos de desenho/edição de nó, os dois overlays aparecem.
    #[test]
    fn draw_and_node_modes_show_both_overlays() {
        for mode in [DrawMode::Pen, DrawMode::Node, DrawMode::Shape] {
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
        assert!(!plan.corner_handles, "e sem alça de raio de quina");
        assert!(plan.snap_guides, "guias de snap valem em todo modo");
    }

    /// Tool inativa: nenhum overlay vetorial (o canvas é de outra ferramenta).
    #[test]
    fn inactive_tool_draws_no_vector_overlays() {
        let plan = vec_overlay_plan(false, DrawMode::Node);
        assert!(!plan.edit);
        assert!(!plan.snap_guides);
        assert!(!plan.corner_handles);
    }

    /// **A alça de raio de quina é do NODE, e só dele** — mais estreita que o `edit`.
    ///
    /// O `Shape` é o que este teste realmente protege: uma forma paramétrica re-cozinha a
    /// geometria INTEIRA a cada mudança de parâmetro (`recook_into` sobrescreve `verts`),
    /// então um `corner_radius` gravado num vértice dela seria varrido no próximo arrasto
    /// de slider — a alça funcionaria por um frame e depois "esqueceria" sozinha. O raio de
    /// uma Live Shape é um campo DELA (o painel); o raio por-vértice é para path desenhado.
    #[test]
    fn the_corner_radius_handle_belongs_to_node_mode_alone() {
        assert!(vec_overlay_plan(true, DrawMode::Node).corner_handles);
        for mode in [DrawMode::Select, DrawMode::Pen, DrawMode::Shape] {
            let plan = vec_overlay_plan(true, mode);
            assert!(
                !plan.corner_handles,
                "{mode:?} NÃO desenha alça de raio de quina"
            );
        }
    }
}
