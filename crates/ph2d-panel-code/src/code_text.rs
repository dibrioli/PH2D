//! ⭐ **A nitidez da letra do CÓDIGO** (dono 10/11: «opções de renderização exclusivas do script»):
//! a escolha do menu View ▸ Text rendering da aba, separada da do app (Edit ▸ Preferences…). Vale
//! para tudo o que pinta pela mono do código (o texto, os números da calha e os rótulos de
//! [`crate::mono_label`], que são a mesma letra). Os mesmos quatro modos do app, lidos só nos
//! ingredientes de BORDA: a dica, o encaixe horizontal e o escurecimento. ⛔ Nem o reforço de peso
//! (o código é o `400` da maquete) nem o espaçamento (a grelha da mono tem colunas iguais).
//!
//! O de fábrica é `Default` (dica ligada, sem encaixe, sem escurecimento): o código de sempre.

use ph2d_a11y::NodeId;
use ph2d_tokens::{TextRendering, TextRenderingParams};
use ph2d_vector::FontEmbolden;

thread_local! {
    static MODE: std::cell::Cell<TextRendering> = const { std::cell::Cell::new(TextRendering::Default) };
}

/// Escolhe o modo da letra do código.
pub fn set_mode(mode: TextRendering) {
    MODE.with(|c| c.set(mode));
}

/// O modo da letra do código.
#[must_use]
pub fn mode() -> TextRendering {
    MODE.with(std::cell::Cell::get)
}

fn params() -> TextRenderingParams {
    mode().params()
}

/// A dica (autohinter) dos glifos do código.
#[must_use]
pub fn hint() -> bool {
    params().hint
}

/// O x de um glifo do código encaixado como o modo pede (a pintura, o cursor e o clique leem-no).
#[must_use]
pub fn snap(x: f32) -> f32 {
    ph2d_editor_core::paint::snap_x_apply(x, params().snap_x)
}

/// O escurecimento de um glifo do código de `size` px na tinta `ink` sobre o fundo `bg`: só a tinta
/// mais clara que o fundo (claro sobre escuro) o recebe.
#[must_use]
pub fn embolden(size: f32, ink: ph2d_vector::Color, bg: ph2d_vector::Color) -> FontEmbolden {
    let a = ph2d_tokens::code_stem_darkening_px(params(), size);
    ph2d_editor_core::paint::font_embolden_on(a, ink, bg)
}

/// As linhas do submenu, pela ordem de [`TextRendering::ALL`]: (id, rótulo, rótulo com a marca).
pub const ROWS: [(NodeId, &str, &str); 4] = [
    (
        ph2d_tool_registry::hash_node_id("code_cmd_text_default"),
        "code.cmd.text_default",
        "code.cmd.text_default_on",
    ),
    (
        ph2d_tool_registry::hash_node_id("code_cmd_text_crisp_heavy"),
        "code.cmd.text_crisp_heavy",
        "code.cmd.text_crisp_heavy_on",
    ),
    (
        ph2d_tool_registry::hash_node_id("code_cmd_text_crisp_heavy_plus"),
        "code.cmd.text_crisp_heavy_plus",
        "code.cmd.text_crisp_heavy_plus_on",
    ),
    (
        ph2d_tool_registry::hash_node_id("code_cmd_text_crisp_darkened"),
        "code.cmd.text_crisp_darkened",
        "code.cmd.text_crisp_darkened_on",
    ),
];

fn index(id: NodeId) -> Option<usize> {
    ROWS.iter().position(|r| r.0 == id)
}

/// O rótulo da linha `id` (com a marca na do modo em vigor); `None` = não é uma linha daqui.
#[must_use]
pub fn label(id: NodeId) -> Option<&'static str> {
    let i = index(id)?;
    Some(if TextRendering::ALL[i] == mode() {
        ROWS[i].2
    } else {
        ROWS[i].1
    })
}

/// Corre a linha `id` (escolhe o modo dela). `true` = era uma linha daqui.
pub fn choose(id: NodeId) -> bool {
    index(id).map(|i| set_mode(TextRendering::ALL[i])).is_some()
}
