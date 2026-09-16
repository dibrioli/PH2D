//! ⭐⭐⭐ **AS LINHAS DA PILHA DE AJUSTES — a barra é CAIXA ÚNICA, o interruptor é linha de propriedade.**
//!
//! Filho do [`super`] por responsabilidade: aquele diz *que editor cada ajuste tem*; este diz *como
//! uma linha de ajuste se desenha*.
//!
//! # ⛔⛔ Porque saiu
//!
//! Até 2026-09-16 a barra era `nome | trilho nu`, com o nome no literal `ADJ_LABEL_W = 44,0`
//! pintado em `Base` — **17 dos 44** nomes saíam cortados em toda largura (o `Contrast` do próprio
//! comentário incluído), e **nenhuma barra mostrava número**: pôr o contraste em `+25` era arrastar
//! até parecer certo. A spec §2 manda um valor com fracção ser **caixa única** (nome dentro, número
//! editável) e a ordem do dono de 2026-06-26 já a tinha dado ao resto do pincel.
//!
//! ⚠️ **O número é `pista × escala + deslocamento` na unidade do ARTISTA** (`%`, `°`, `px`, níveis),
//! e o mapeamento vem da crate de efeitos, ao lado do setter
//! (`ph2d_tool_painter::adjustment_slider_numbers` e irmãs). O chip é ligado ao slider pelo MESMO
//! mapeamento, logo uma edição volta à ferramenta como o `ValueChanged` do slider — o encaminhamento
//! de sempre. ⛔ O *Gamma* do *Levels* não é afim na pista: ali o número mostra-se e não se escreve.
//!
//! # ⚠️ O interruptor continua linha de propriedade
//!
//! A chave vai para o início da coluna do controlo (onde a marca de uma caixa de marcar vive), e a
//! coluna é a da SECÇÃO — medida sobre os nomes dos interruptores da pilha.

use super::*;
use ph2d_editor_core::widget::{
    DEFAULT_CHIP_W, DEFAULT_LABEL_W, PropertyRow, Seccao, TextInputState, colunas_da_linha,
    paint_property_label, paint_slider_with_chip_layout_adaptive,
};
use ph2d_text::TextSystem;
use ph2d_tool_painter::SliderNumber;

/// ⭐ **A secção dos interruptores de uma pilha, medida sobre os nomes que ELA pinta.**
///
/// ⚠️ Uma pilha sem interruptores não tem coluna a medir; cai na metade, que é o default.
pub(super) fn seccao_de(text_system: &mut TextSystem, nomes: &[&str]) -> Seccao {
    if nomes.is_empty() {
        return Seccao::apenas_campos(1);
    }
    Seccao::medida(text_system, 1, nomes)
}

/// As colunas de uma linha de interruptor.
fn colunas(row: Rect, sec: Seccao) -> PropertyRow {
    colunas_da_linha(row.x, row.w, row.y, ROW_H_PX, sec)
}

/// O nome do interruptor, na coluna — alinhado à direita, elidido, no peso em que foi medido.
fn nome(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, label: &str, linha: &PropertyRow) {
    let font = TypeToken::Sm.px();
    paint_property_label(
        ctx.text_system,
        ctx.scene,
        label,
        linha.label.x,
        linha.label.y + (linha.label.h - font) * 0.5,
        font,
        linha.label.w,
        resolve(ColorToken::Text2, theme),
    );
}

/// ⭐⭐ **Regista o número de uma barra e LIGA-O ao slider pela unidade do slot.**
///
/// Devolve o id a pintar como chip — o `NodeId(0)` do widget («sem chip») quando o número só se
/// mostra.
///
/// ⚠️ **Liga a cada quadro, e é de propósito:** o id é por camada e por slot, a espécie de uma camada
/// não muda, e a ligação é uma inserção idempotente. Ligar uma vez no `populate` é impossível — as
/// camadas nascem depois dele.
fn ligar_numero(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    numero: SliderNumber,
) -> ph2d_a11y::NodeId {
    let SliderNumber::Affine {
        scale,
        offset,
        integer,
    } = numero
    else {
        return ph2d_a11y::NodeId(0);
    };
    store.register_if_absent(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: f64::from(offset),
            buffer: String::new(),
            caret: 0,
            last_committed: f64::from(offset),
            selection_anchor: None,
        },
    );
    let (lo, hi) = (f64::from(offset), f64::from(offset + scale));
    if integer {
        store.link_slider_number_mapped_integer(slider, chip, scale, offset);
        store.set_number_range(chip, lo, hi, 1.0);
    } else {
        store.link_slider_number_mapped(slider, chip, scale, offset);
        // O passo é 1 % da pista, no espaço do número — o mesmo das barras irmãs do pincel.
        const PASSO: f64 = 0.01; // LITERAL-PX-OK: passo de 1 % da pista (comportamento, não layout)
        store.set_number_range(chip, lo, hi, PASSO * f64::from(scale));
    }
    chip
}

/// ⭐⭐⭐ **Uma barra da pilha — a CAIXA ÚNICA, com o nome dentro e o número na unidade do slot.**
///
/// A pista é derivada dos params vivos a cada quadro, logo o chamador passa-a. Devolve o próximo `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_barra(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    (slider, chip): (ph2d_a11y::NodeId, ph2d_a11y::NodeId),
    label: &str,
    pista: f32,
    numero: SliderNumber,
    row: Rect,
) -> f32 {
    let pista = pista.clamp(0.0, 1.0);
    register_slider(
        ctx.host.store_mut(),
        slider,
        pista,
        SliderOrientation::Horizontal,
    );
    let chip_id = ligar_numero(ctx.host.store_mut(), slider, chip, numero);
    let valor = numero.at(pista);
    // ⚠️ Uma CONTAGEM mostra-se sem casas; o número que só se mostra leva duas; o resto deixa o chip
    //    formatar como toda caixa do app.
    let texto = match numero {
        SliderNumber::Affine { integer: true, .. } => Some(format!("{}", valor.round())),
        SliderNumber::Shown(v) => Some(format!("{v:.2}")),
        SliderNumber::Affine { .. } => None,
    };
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let usado = paint_slider_with_chip_layout_adaptive(
        Rect::new(row.x, row.y, row.w, ROW_H_PX),
        label,
        pista,
        f64::from(valor),
        texto.as_deref(),
        slider,
        chip_id,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    row.y + usado + ph2d_tokens::control_gap_px()
}

/// Render one toggle row (name column + a switch at the START of the control column, where a
/// checkbox mark also lives) painted with the live param value: the shared body of the generic
/// toggle rack AND the Channel-Mixer / Black & White switches. Registers the switch as a button
/// (click → the tool flips the param); the caller advances `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_toggle_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
    row: Rect,
    sec: Seccao,
) {
    let linha = colunas(row, sec);
    nome(ctx, theme, label, &linha);
    let toggle_rect = Rect::new(
        linha.control.x,
        row.y + (ROW_H_PX - ADJ_TOGGLE_H) * 0.5,
        ADJ_TOGGLE_W.min(linha.control.w),
        ADJ_TOGGLE_H,
    );
    let toggle = Toggle::new(id, "")
        .visual(ctx.host.store().toggle_visual(id))
        .on(on);
    paint_toggle(&toggle, toggle_rect, ctx.scene, theme);
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, toggle_rect);
}
