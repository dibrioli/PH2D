//! ⭐⭐⭐ **Os chips numéricos das duas barras de cada camada do Composite Brush.**
//!
//! ⛔⛔ **Eles existem por ordem do dono** (2026-09-20, com foto: *«sliders fora do padrão do app.
//! corrija. sliders no padrão»*). Até esse report o cartão pintava uma barra **NUA** com um
//! mostrador de texto ao lado, e o doc dela defendia a escolha com *«o padrão da opacidade do
//! painel de Layers, que nunca empilha num painel estreito»* — ⚠️ **verdade sobre aquela linha e
//! falsa sobre ESTE painel**, onde toda fileira de valor é a CAIXA ÚNICA de 2026-09-02 (rótulo
//! dentro à esquerda, valor dentro à direita, preenchimento a dizer a fracção). *Um padrão citado
//! de outro painel é uma segunda resposta com proveniência — e proveniência lê-se como
//! justificação.*
//!
//! # As três coisas que um chip precisa, e nenhuma é opcional
//!
//! 1. **ESTADO** no `WidgetStore` (`NumberInput`) — sem ele o pintor cai no ramo *«sem buffer»* e
//!    desenha um número que não aceita o cursor: pintado, e morto sob o dedo.
//! 2. **LIGAÇÃO** à barra (`link_slider_number*`) — é ela que faz uma edição no chip voltar como o
//!    `ValueChanged` **DA BARRA**, que é o canal que o `is_forwardable_brush_slider` já encaminha.
//!    ⭐ É por isso que esta wave não abre **uma única rota nova** no dreno.
//! 3. **FAIXA** (`set_number_range`) — é o que torna o arrasto do chip proporcional ao intervalo
//!    dele e o clampa; sem ela o passo é o de omissão e o número corre à frente do dedo.
//!
//! ⚠️⚠️ **A Strength liga DIRECTO e o tamanho liga PROJECTADO, e a projecção vive AQUI e em mais
//! lado nenhum:** o chip do tamanho mostra **px de diâmetro** (dono 08/11: *«Dentro de composite os
//! sizes devem ser em px»*) e a barra guarda o multiplicador EFECTIVO sobre o tecto
//! [`MAX_COMPOSITE_LAYER_SIZE`]; o motor continua a guardar o multiplicador. Escrita também no pintor,
//! as duas divergiriam — com o polegar a pousar num número e o chip a escrever outro, que é
//! exactamente o defeito que o irmão `register_spray_count_chip` já tem escrito no cabeçalho dele.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::TextInputState;

/// O passo do chip sobre a pista `0..1` de uma Strength — o mesmo dos chips do pincel.
const STEP: f64 = 0.01; // LITERAL-PX-OK: chip 0..1 track step (non-design behaviour value)

/// Registar + ligar + dimensionar os `2 × N_CAMADAS` chips do cartão.
///
/// ⚠️ **Por ARRAY e nunca por índice** — a extensão de três para cinco camadas passou pelo
/// `populate` sem uma linha exactamente porque os laços iteram os arrays dos ids; uma lista à mão
/// teria deixado quatro controlos mortos sob o dedo.
pub(crate) fn register_composite_chips(store: &mut WidgetStore) {
    for (sid, chip) in ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH
        .into_iter()
        .zip(ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP)
    {
        registar(store, chip, 0.0);
        store.link_slider_number(sid, chip);
        store.set_number_range(chip, 0.0, 1.0, STEP);
    }
    for (sid, chip) in ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS
        .into_iter()
        .zip(ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CHIP)
    {
        registar(store, chip, 1.0);
        store.link_slider_number(sid, chip);
        store.set_number_range(chip, 0.0, 1.0, STEP);
    }
    let b = &crate::paint_brush::FALLBACK_BRUSH;
    for pos in 0..ph2d_tool_painter::N_COMPOSITE_LAYERS {
        registar(
            store,
            ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[pos],
            f64::from(tamanho_px(b, pos)),
        );
        liga_tamanho(store, b, pos);
    }
}

/// Diâmetro = 2 × raio: a mesma unidade do «Size (px)» do pincel (`paint_wetpaint_presets`).
const DIAMETRO_POR_RAIO: f32 = 2.0; // LITERAL-PX-OK: diameter = 2 × radius, a law, not a size
/// O passo do chip do tamanho: 1 px de diâmetro, como o «Size (px)» do pincel.
const PASSO_PX: f64 = 1.0; // LITERAL-PX-OK: Size px step (1 px of diameter)

/// Quantos px de diâmetro vale a pista inteira (`0..1` = `0..MAX` multiplicadores) com o pincel de agora.
fn px_por_pista(b: &ph2d_tool_painter::BrushSettings) -> f32 {
    let raio = b.size_px.max(ph2d_tool_painter::BRUSH_SIZE_MIN_PX);
    DIAMETRO_POR_RAIO * raio * ph2d_tool_painter::MAX_COMPOSITE_LAYER_SIZE
}

/// A pista da barra do tamanho: o multiplicador EFECTIVO (com o piso do Spacing) sobre o tecto — a barra e o chip
/// dizem o que é pintado.
pub(crate) fn pista_do_tamanho(b: &ph2d_tool_painter::BrushSettings, pos: usize) -> f32 {
    (b.composite_layer_size_effective(pos) / ph2d_tool_painter::MAX_COMPOSITE_LAYER_SIZE)
        .clamp(0.0, 1.0)
}

/// O «Size (px)» da camada `pos`: o diâmetro que ela pinta, inteiro.
pub(crate) fn tamanho_px(b: &ph2d_tool_painter::BrushSettings, pos: usize) -> f32 {
    (pista_do_tamanho(b, pos) * px_por_pista(b)).round()
}

/// A pista do px INTEIRO — a barra e o chip pousam no mesmo número que o chip escreve.
pub(crate) fn pista_do_px(b: &ph2d_tool_painter::BrushSettings, pos: usize) -> f32 {
    tamanho_px(b, pos) / px_por_pista(b)
}

/// O chip do tamanho mostra px inteiros, como o «Size (px)» do pincel.
pub(crate) const CASAS_DO_PX: usize = 0;

/// Liga o chip do tamanho em px à barra (`px = pista × px_por_pista`) e a faixa `[piso, tecto]` em px. ⚠️ Corre a
/// CADA quadro (os dois painéis): a escala segue o Size e o Spacing do pincel; registada uma vez, mentiria ao
/// primeiro gesto no Size. Um chip escrito volta à barra como `px / px_por_pista`, logo ao motor como
/// `px / (2 · raio)` multiplicadores.
pub(crate) fn liga_tamanho(
    store: &mut WidgetStore,
    b: &ph2d_tool_painter::BrushSettings,
    pos: usize,
) {
    let (sid, chip) = (
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE[pos],
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[pos],
    );
    let escala = px_por_pista(b);
    store.link_slider_number_mapped_integer(sid, chip, escala, 0.0);
    let piso = b.composite_layer_size_floor() / ph2d_tool_painter::MAX_COMPOSITE_LAYER_SIZE;
    store.set_number_range(chip, f64::from(piso * escala), f64::from(escala), PASSO_PX);
}

/// O estado inicial de um chip — o valor vivo é espelhado a cada quadro pelo pintor.
fn registar(store: &mut WidgetStore, chip: ph2d_a11y::NodeId, inicial: f64) {
    store.register(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: inicial,
            buffer: String::new(),
            caret: 0,
            last_committed: inicial,
            selection_anchor: None,
        },
    );
}
