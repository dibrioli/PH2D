//! ⭐⭐ **IMAGE ▸ PAINT** — o modo que esta família declara e como ele abre e larga o Painter sobre a
//! imagem activa (spec/06 F2 + a F3 da Imagem; D6: *imagem → Object · Paint*).
//!
//! O Painter já é «o desta entidade» (`PainterTool::bound_doc` = a selecção): entrar no modo é pôr
//! o Painter em mãos com a selecção colapsada ao activo, e o cadeado da selecção
//! (`ph2d_editor_core::object_mode::decide`) guarda-a lá enquanto o modo durar.
//!
//! ⚠️ **A porta do IMG não se aplica aqui:** o modo É a porta do Painter. O botão dele saiu da
//! barra IMG na mesma fase (dois caminhos para o mesmo módulo divergem — spec/06 §5).

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_painter::PainterTool;

/// ⭐ **Os modos que esta família declara**, por tipo.
pub const MODES: &[(ObjectKind, ObjectMode)] = &[(ObjectKind::Image, ObjectMode::Paint)];

/// O id do Painter no registo de ferramentas.
const PAINTER: &str = "painter";

/// O Painter está em mãos **sobre uma imagem** — e não sobre a tela da peça 3D, que é a pintura
/// da escultura (`ph2d_app_sculpt3d::painter_na_malha`, o Paint do Sculpt na F3).
#[must_use]
pub fn holds_an_image(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        .is_some_and(|p| !p.on_screen_canvas())
}

/// ⭐ **Entra**: colapsa a selecção ao activo (o último acrescentado) e põe o Painter em mãos.
/// Devolve `false` se o registo recusou.
///
/// ⚠️ Colapsar ANTES de activar: o Painter lê a selecção ao activar-se para saber que documento
/// abrir — ver [`crate::painter_lock::collapse_to_last`].
pub fn enter(tools: &mut ToolRegistry, hero: &mut HeroScreen) -> bool {
    crate::painter_lock::collapse_to_last(hero);
    holds_an_image(tools) || tools.set_active(&ToolId::new(PAINTER))
}

/// **Larga**: devolve o canvas à ferramenta de omissão, se o Painter ainda o tem.
pub fn leave(tools: &mut ToolRegistry) {
    if tools.active().is_some_and(|t| t.id() == ToolId::new(PAINTER)) {
        tools.activate_default();
    }
}
