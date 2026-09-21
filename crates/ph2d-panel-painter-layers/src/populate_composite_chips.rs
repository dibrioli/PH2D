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
//! lado nenhum:** o chip do tamanho mostra o **multiplicador** (`0`..[`MAX_COMPOSITE_LAYER_SIZE`])
//! e a barra guarda a fracção. Escrita também no pintor, as duas divergiriam no dia em que o tecto
//! mudasse — com o polegar a pousar num número e o chip a escrever outro, que é exactamente o
//! defeito que o irmão `register_spray_count_chip` já tem escrito no cabeçalho dele.

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
    let tecto = f64::from(ph2d_tool_painter::MAX_COMPOSITE_LAYER_SIZE);
    for (sid, chip) in ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE
        .into_iter()
        .zip(ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP)
    {
        registar(store, chip, 1.0);
        store.link_slider_number_mapped(
            sid,
            chip,
            ph2d_tool_painter::MAX_COMPOSITE_LAYER_SIZE,
            0.0,
        );
        store.set_number_range(chip, 0.0, tecto, STEP);
    }
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
