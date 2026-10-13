//! Os módulos WGSL da pilha de efeitos — a porta ÚNICA que o [`crate::fx_stack::FxStackPass`] e o
//! gate de naga leem (saiu do `fx_stack.rs` pelo tecto de LOC, doc 132).

use crate::fx_stack_field::FIELD_WGSL;
use crate::fx_stack_noise::{NOISE_WGSL, WARP_WGSL};
use crate::fx_stack_piramide::PIRAMIDE_WGSL;
use crate::fx_stack_shader::{
    FX_STACK_MID_WGSL, FX_STACK_OUT_WGSL, FX_STACK_WGSL, kind_consts_wgsl,
};

/// **Os dois módulos WGSL que a pilha compila** — porta ÚNICA, com dois consumidores: o
/// [`FxStackPass::new`] (que os manda ao dispositivo) e o gate de naga (que os valida sem GPU).
///
/// ⚠️ **É porta única por um motivo que já mordeu este repositório:** um gate que montasse a sua
/// própria concatenação ficaria VERDE sobre um produto que deixou de prefixar o bloco
/// compartilhado — validar-se-ia a si mesmo. Aqui o gate valida exactamente a string que o
/// dispositivo recebe.
///
/// ⚠️ **As leis de mistura vêm do MESMO arquivo que o compositor de camadas compila** — o
/// `blend_modes.wgsl` foi extraído dele quando ganhou este segundo consumidor. *Como duas cores se
/// combinam* é pergunta já respondida e pinada bit a bit contra o Rust; uma cópia aqui divergiria
/// no único lugar onde ninguém lê um número: uma captura de tela.
pub(crate) fn module_sources() -> [(&'static str, String); 2] {
    let kinds = kind_consts_wgsl();
    let blend = crate::layer_compositor::BLEND_MODES_WGSL;
    // ⚠️ **O ajuste de cor vem do arquivo COMPARTILHADO**, o mesmo movimento das leis de
    // mistura: a lei do Color Adjust É a do `AdjustmentKind::HueSaturationBrightness` do
    // Painter, e uma cópia local seria a segunda resposta que este prefixo existe para não
    // ter (há gate).
    let adjust = crate::layer_compositor::COLOUR_ADJUST_WGSL;
    let grupo = crate::fx_stack_borrao_grupo::borrao_grupo_wgsl();
    [
        (
            "mid",
            // ⚠️ O RUÍDO entra ANTES do fold (o `cs_op_warp` do `WARP_WGSL` chama o `fbm`) e o
            // WARP depois dele (ele chama o `tap_img`). A ordem é a das dependências, não gosto.
            format!(
                "{blend}\n{adjust}\n{kinds}{NOISE_WGSL}{FX_STACK_WGSL}{FX_STACK_MID_WGSL}{FIELD_WGSL}{WARP_WGSL}{PIRAMIDE_WGSL}{grupo}"
            ),
        ),
        (
            "out",
            // O módulo de SAÍDA escreve noutro formato de storage e não tem o `dst` que o warp
            // usa — ele não recebe o passe, só o campo de que o resto do fold não depende.
            format!("{blend}\n{adjust}\n{kinds}{FX_STACK_WGSL}{FX_STACK_OUT_WGSL}"),
        ),
    ]
}
