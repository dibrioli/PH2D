//! Registro dos widgets — percorrido a partir de [`crate::rows::rows`], então uma
//! row pintada não pode ser uma row não-registrada.
//!
//! Um widget que o `paint` hit-indexa e ninguém registra tem
//! `is_focusable() == false`, e o clique dele é descartado **em silêncio** — sem
//! erro de compilação, sem warning, só um controle que não faz nada (a classe de
//! bug que o `architecture_panel_wiring_parity` existe para pegar). Derivar esta
//! lista da MESMA tabela que o `paint` percorre é o que tira isso da lista de
//! coisas que alguém pode esquecer.
//!
//! ⚠️ **Os rádios e botões NÃO saem da tabela de rows, e por isso são um laço de
//! LISTAS.** O `paint_segmented_adaptive` registra retângulo de hit mas **não**
//! entrada de store, então uma opção não registrada fica pintada, hit-indexada e
//! morta sob o mouse — a falha exata que as 36 células da matriz de física
//! ensinaram. Uma lista de listas e não um laço por array: o quinto grupo nasce
//! fora da regra se cada um tiver o seu.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

use crate::rows;

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub fn populate(store: &mut WidgetStore) {
    for row in rows::rows() {
        store.register(
            row.slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            row.chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped(row.slider, row.chip, row.scale(), row.offset());
        // ⚠️ A faixa é registrada AQUI, e não é opcional: sem ela o chip deriva o
        // passo do arrasto do texto do buffer e percorre ~50 unidades por pixel,
        // então um pixel de arrasto bate no teto e o chip vira um interruptor
        // min↔max. Digitar continua funcionando, que é por que esta classe de bug
        // sobrevive à revisão.
        store.set_number_range(row.chip, f64::from(row.min), f64::from(row.max), row.step);
    }

    for group in [
        &ids::SCULPT3D_VERB[..],
        &ids::SCULPT3D_FALLOFF[..],
        &ids::SCULPT3D_ALPHA[..],
        &ids::SCULPT3D_DETAIL[..],
        &ids::SCULPT3D_ADD[..],
        &ids::SCULPT3D_MASK_OP[..],
        &ids::SCULPT3D_MATCAP[..],
    ] {
        for &id in group {
            button(store, id);
        }
    }

    // Os cabeçalhos são interativos (o chevron os dobra), então são registrados
    // como qualquer outro controle — um chevron pintado que ninguém registrou é
    // uma affordance que não faz nada.
    for section in rows::SECTIONS {
        button(store, section.id);
    }
    for id in [
        ids::SCULPT3D_SEC_TOOL,
        ids::SCULPT3D_SEC_SYMMETRY,
        ids::SCULPT3D_SEC_TOPOLOGY,
        ids::SCULPT3D_SEC_SCENE,
    ] {
        button(store, id);
    }

    // Comandos e toggles. Registrados como `Button` — inclusive os três eixos do
    // espelho e o dyntopo, que PARECEM checkbox e não são: um `Checkbox` emite
    // `Toggled`, que o `event.rs` deste painel não encaminha, então ele nasceria
    // registrado e morto (o mesmo aviso que o painel de física carrega).
    // ⚠️ **Todo comando de um toque é registrado A PARTIR da tabela que os
    // despacha**, e não de uma segunda lista escrita à mão. A lista à mão que
    // morava aqui apodreceu na primeira adição — o botão de assar o AO nasceu
    // pintado, hit-indexado e **morto sob o mouse**, e quem o pegou foi o
    // `every_painted_control_is_clickable_where_it_is_drawn`. Derivando, um
    // comando novo nasce clicável pelo mesmo commit que o faz existir.
    for (id, _) in crate::event::COMMANDS {
        button(store, *id);
    }
    // Os que NÃO são comandos de um toque: as três opções de simetria (chips de
    // um grupo), os dois toggles que o `event` resolve por outra rota, e o fechar
    // do painel.
    for id in [
        ids::SCULPT3D_SYM_X,
        ids::SCULPT3D_SYM_Y,
        ids::SCULPT3D_SYM_Z,
        ids::SCULPT3D_WIREFRAME,
        ids::SCULPT3D_ACCUMULATE,
        ids::SCULPT3D_CLOSE,
    ] {
        button(store, id);
    }
}
