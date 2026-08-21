//! Registro dos widgets — derivado do **retrato**, e não de uma tabela escrita à mão.
//!
//! ⚠️ **O `populate` corre uma vez, na construção, e o documento ainda não existe.** As linhas deste
//! painel são os nós do modelo, e quantos há é o que o artista modelou — então registram-se os ids
//! de uma **família** de tamanho fixo, e a linha `n` usa o id `n`.
//!
//! É o mesmo compromisso do painel de tokens, e o limite é honesto: [`MAX_ROWS`].

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

/// Quantas linhas o painel consegue mostrar.
///
/// ⚠️ **É um limite de REGISTRO, e ele nomeia o seu recurso:** `populate` corre antes de o documento
/// existir, então os ids têm de ser cunhados às cegas. Cada linha custa dois `NodeId` registados no
/// store — memória, nada mais.
///
/// ⛔ **O que ele NÃO pode fazer é cortar em silêncio.** Um nó além desta linha ficaria sem controle
/// e ninguém saberia porquê; por isso o `paint` **conta** e o rodapé diz quantos não coube. O gate
/// `rows_beyond_the_family_are_reported_not_dropped` prende isso.
///
/// 64 é o número escrito na primeira vez, e a única coisa que o justifica é ser muito maior do que
/// qualquer peça hoje autorável à mão (as cenas de smoke têm 1 a 4 nós). *Quando um documento real
/// passar disto, o rodapé diz — e aí o número muda com uma medição atrás.*
pub const MAX_ROWS: usize = 64;

/// Quantos verbos (e quantos referenciais) um seletor consegue mostrar.
///
/// ⚠️ Mesma natureza do [`MAX_ROWS`]: é um limite de **registro**, porque o `populate` corre antes
/// de o gizmo existir. Hoje `Mode::ALL` tem três; oito é folga sem custo (cada slot é um `NodeId`
/// no store), e a contagem de verdade continua a ser a do shell.
pub const MAX_MODES: u32 = 8;

pub fn populate(store: &mut WidgetStore) {
    for slot in 0..MAX_MODES {
        store.register(
            ids::model3d_mode_button(slot),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.register(
            ids::model3d_frame_button(slot),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.register(
            ids::model3d_add_button(slot),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.register(
            ids::model3d_op_button(slot),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.register(
            ids::model3d_mod_button(slot),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.register(
            ids::model3d_export_button(slot),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.register(
            ids::model3d_act_button(slot),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for node in 0..MAX_ROWS as u32 {
        let slider = ids::model3d_radius_slider(node);
        let chip = ids::model3d_radius_chip(node);
        store.register(
            slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        // ⚠️ O par slider↔campo é ligado **em 0..1**, e a faixa real é aplicada por linha no
        // `paint`: o teto de um raio é do NÓ (a caixa aceita menos que o cilindro), e uma escala
        // fixa aqui seria a mesma faixa para todos.
        store.link_slider_number_mapped(slider, chip, 1.0, 0.0);
    }
    store.register(
        ids::MODEL3D_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // A moldura móvel/redimensionável, como todo painel encaixado deste shell.
    store.register(
        ids::INSP_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::MODEL3D_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
}
