//! **A linha de painel LÊ o estado da trilha no store — e é ela que multiplica a wave.**
//!
//! O `paint_slider_with_chip` serve ~67 chamadas espalhadas por 32 ficheiros: praticamente toda
//! linha numérica do app passa por aqui. Enquanto ele descartava o estado, nenhuma dessas linhas
//! podia acender, e migrá-las uma a uma seria 67 edições para uma lei só.
//!
//! ⚠️ **O doc-header daquele módulo já PROMETIA isto** (*"reads the slider's state … straight from
//! the WidgetStore"*) e não cumpria — ele lia o store para o CHIP e passava o neutro para a
//! trilha. Uma promessa escrita e não cumprida é pior que ausente: ela faz a próxima pessoa
//! procurar o defeito noutro lugar.
//!
//! ⚠️ **O gate afirma a PROPRIEDADE, nunca o endereço.** Ele não procura a chamada
//! `store.slider_visual(slider_id)` no fonte: pinta a linha duas vezes, com o mesmo valor e o
//! mesmo `id`, mudando só o que está no STORE, e exige que a tinta difira. Um scanner de fonte
//! ficaria verde no dia em que alguém trocasse a porta por duas perguntas separadas — e é
//! precisamente esse o passo em falso desta família.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderOrientation, SliderState, paint_slider_with_chip};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const SLIDER: NodeId = NodeId(1);
const CHIP: NodeId = NodeId(2);

/// Pinta uma linha de painel com a trilha no estado dado e devolve a tinta.
fn row_ink(state: SliderState) -> (Vec<u32>, Vec<u32>) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        SLIDER,
        InteractiveState::Slider {
            state,
            value: 0.5,
            orientation: SliderOrientation::Horizontal,
        },
    );
    let mut hit = HitIndex::default();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_slider_with_chip(
        Rect::new(0.0, 0.0, 240.0, 24.0),
        "Opacity",
        0.5,
        SLIDER,
        CHIP,
        &store,
        &mut hit,
        &mut scene,
        &mut text,
        Theme::Forge,
    );
    let e = scene.inner().encoding();
    (e.path_data.clone(), e.draw_data.clone())
}

/// **A linha inteira acende com a trilha.**
///
/// **Mutação que deve sangrar:** o `paint_slider_with_chip` voltar a passar o par NEUTRO
/// (`(SliderState::Normal, motion::SETTLED)`) à `paint_slider_track` — as três tintas colapsam
/// numa e as ~67 linhas do app ficam mudas outra vez.
#[test]
fn the_panel_row_reads_the_sliders_state() {
    let normal = row_ink(SliderState::Normal);
    assert_ne!(
        row_ink(SliderState::Hovered),
        normal,
        "a linha de painel nao le o estado da trilha: ~67 linhas do app ficam mudas sob o ponteiro"
    );
    assert_ne!(
        row_ink(SliderState::Dragging),
        normal,
        "a linha nao mostra que a trilha esta AGARRADA"
    );
}

/// **O CONTROLO: um `id` que o store nunca viu pinta o mundo de antes.**
///
/// ⚠️ Sem esta metade o gate acima seria satisfeito por qualquer coisa que reagisse — inclusive
/// por uma lei que tingisse toda linha do app no arranque. Aqui a trilha não está registada, a
/// fallback do `slider_visual` cai em `Normal` (nada é `hot` nem `active`), e o resultado tem de
/// ser byte-idêntico à linha em repouso.
#[test]
fn an_unregistered_row_paints_the_world_as_it_was() {
    let store = WidgetStore::with_capacity(4);
    let mut hit = HitIndex::default();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_slider_with_chip(
        Rect::new(0.0, 0.0, 240.0, 24.0),
        "Opacity",
        0.5,
        SLIDER,
        CHIP,
        &store,
        &mut hit,
        &mut scene,
        &mut text,
        Theme::Forge,
    );
    let e = scene.inner().encoding();
    assert_eq!(
        (e.path_data.clone(), e.draw_data.clone()),
        row_ink(SliderState::Normal),
        "uma linha que o store nunca viu deixou de pintar o mundo de antes"
    );
}
