//! Sheet Packer — a declaração do [`ToolManifest`].
//!
//! Ação one-shot: com sprites selecionados, clicar `[SHEET]` na fila de Image Tools cria **uma
//! folha** com eles dentro (plano `docs/Sprite_projeto/17` §7).
//!
//! ### Handler em modo sombra
//!
//! Como os irmãos da fila (`make_square`, `real_size`, `rasterize`), o trabalho real corre na
//! shell — ele muda o `SimWorld` **e** o documento vetorial —, então o slot `handler` aponta para
//! um no-op até o despachante genérico existir. O manifesto regista na mesma, e é isso que faz o
//! pill nascer derivado do registry em vez de escrito à mão.

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, ToolHandler, ToolManifest, Zone};

use crate::icon::sheet_packer_bezpath;

/// Handler em sombra. O trabalho real está no dreno de
/// `EditorAction::OneShotImageOp { tool_id: "sheet_packer", entity_bits }` da shell.
fn shadow_handler() {}

/// O manifesto canônico do Sheet Packer.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "sheet_packer",
    label_key: "tool.sheet_packer.label",
    icon_fn: sheet_packer_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    // A fila, por ordem: trim 40 → make_square 50 → bgremoval 60 → real_size 70 → padding 80 →
    // color_equalization 90 → equalize_sizes 100 → rasterize 110 → upscale 120 → painter 130 →
    // **sheet_packer 140**.
    //
    // ⚠️ **O número CONTA-SE contra TODAS as fontes, e eu errei-o à primeira.** Contei só os
    // `manifest.rs` e escrevi 120 — mas o `upscale` declara o manifesto dentro do `lib.rs`, e
    // 120 já era dele. Dois tools com a mesma ordem ficam à mercê do desempate de iteração: o
    // pill trocaria de lugar entre execuções, e nada apontaria para a causa. Quem apanhou foi o
    // gate `image_tools_cluster_in_canonical_order`, comparando a fila VIVA com a que o
    // `ph2d-tool-sync` deriva — *é para isto que ele existe.*
    order: 140,
    a11y_role: Role::Button,
    handler: ToolHandler::OneShot {
        on_click: shadow_handler as HandlerFn,
    },
    // ⚠️ **Zero, e é honesto:** este tool não aloca buffer de pixels. Ele arranja RETÂNGULOS (a
    // geometria do `layout`) e escreve poses; os pixels das peças ficam onde estavam, na GPU.
    // Quem vai alocar uma folha é o *bake*, que é wave própria e declarará o seu próprio budget.
    memory_budget: MemoryBudget::new(0, 0, 0),
    // Cria uma entidade e reparenta as selecionadas — mas sob o modo sombra o stub não toca em
    // estado nenhum. Vira `true` quando o despachante genérico levar o handler real.
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_id_matches_label_key_slug() {
        // Forma que o gate de i18n (HR-15) cobra: `tool.<id>.label`.
        let stripped = MANIFEST
            .label_key
            .strip_prefix("tool.")
            .and_then(|s| s.strip_suffix(".label"))
            .expect("label_key shape");
        assert_eq!(stripped, MANIFEST.id);
    }

    #[test]
    fn manifest_a11y_role_is_button() {
        assert_eq!(MANIFEST.a11y_role, Role::Button);
    }

    #[test]
    fn manifest_lives_in_image_tools_cluster() {
        assert_eq!(MANIFEST.cluster, "image_tools");
        assert_eq!(MANIFEST.zone, Zone::TopRight);
    }

    /// ⚠️ A ordem é CONTADA contra a fila que existe: dois tools com o mesmo número ficam à
    /// mercê do desempate de iteração, e o pill trocaria de lugar entre execuções.
    #[test]
    fn manifest_order_is_right_of_every_existing_image_tool() {
        // O `painter` (130) é o último da fila hoje; o `upscale` (120) foi o que a 1ª tentativa
        // atropelou. A barra é o MAIOR que existe, não o do vizinho que se lembrou.
        const { assert!(MANIFEST.order > 130) };
    }
}
