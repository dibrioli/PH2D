//! **O registo dos widgets da secção CAMERA** (TOP-20 #7, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe a altura, a cerca e o alvo é o snapshot.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio.
//!
//! # ⚠️ As faixas são as do MOTOR, e o número é literal de propósito
//!
//! Este painel não depende do `ph2d-ecs` (ADR-0029), então as constantes do motor não são
//! importáveis. ⚠️ Sem as faixas, o arrasto teria passo livre sobre um valor que o commit satura, e
//! o artista veria o número a andar com a câmera parada. Há gate na shell a prender os dois lados.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, DropdownState, TextInputState};

/// `(id, valor de partida, mínimo, máximo, passo)` — os campos numéricos da secção.
///
/// ⚠️ **Os valores de partida são os do `GameCamera::default()`/`CameraFollow::default()`**, e não
/// zeros: um amortecimento que nasce a `0` lê-se como um campo partido, e é o mesmo argumento que
/// pôs a duração do timer a um segundo em vez de zero.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 16] = [
    // A câmera. ⚠️ A faixa da altura é a do motor (`CAMERA_MIN/MAX_HEIGHT_WORLD`).
    (ids::INSP_CAMERA_HEIGHT, 10.0, 0.5, 100.0, 0.5), // LITERAL-PX-OK: metros
    (ids::INSP_CAMERA_OFFSET_X, 0.0, -1000.0, 1000.0, 0.1), // LITERAL-PX-OK: metros
    (ids::INSP_CAMERA_OFFSET_Y, 0.0, -1000.0, 1000.0, 0.1), // LITERAL-PX-OK: metros
    (ids::INSP_CAMERA_PRIORITY, 0.0, -1000.0, 1000.0, 1.0), // LITERAL-PX-OK: contagem
    // ⭐⭐⭐ **O DOLLY** (plano 24, W5), e a FAIXA nomeia o recurso de cada ponta:
    //
    // - **`0,9` em cima é o DOMÍNIO DA LEI.** Ela é `(1−δ)/(1−kδ)`, logo em `δ = 1` o plano do
    //   MUNDO tem tamanho aparente ZERO e toda camada colapsa — uma pista não sabe exprimir
    //   «estritamente abaixo de 1», e a `0,9` o céu desta cena já está a `11 %` (medido:
    //   `k = 0,12` → `0,112` · `0,35` → `0,146` · `0,65` → `0,241`).
    // - **`−1` em baixo é a SATURAÇÃO, medida:** o céu lê `1,79×` a `−1`, `2,42×` a `−2` e `2,94×`
    //   a `−3` — cada passo compra menos, e o que a pista ganharia em curso perderia em resolução
    //   onde o artista de facto trabalha.
    // ⚠️ A faixa é a PORTA partilhada com o applier (auditoria 26, §2.6).
    (
        ids::INSP_CAMERA_DOLLY,
        0.0,
        ph2d_editor_core::screens::hero::DOLLY_MIN as f64,
        ph2d_editor_core::screens::hero::DOLLY_MAX as f64,
        0.05, // LITERAL-PX-OK: fracção adimensional
    ),
    // Quem ela segue.
    (ids::INSP_CAMERA_DAMP_X, 5.0, 0.0, 60.0, 0.5), // LITERAL-PX-OK: 1/s
    (ids::INSP_CAMERA_DAMP_Y, 5.0, 0.0, 60.0, 0.5), // LITERAL-PX-OK: 1/s
    (ids::INSP_CAMERA_DEAD_X, 0.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fracção
    (ids::INSP_CAMERA_DEAD_Y, 0.0, 0.0, 1.0, 0.05), // LITERAL-PX-OK: fracção
    (ids::INSP_CAMERA_LOOK_X, 0.0, 0.0, 5.0, 0.05), // LITERAL-PX-OK: segundos
    (ids::INSP_CAMERA_LOOK_Y, 0.0, 0.0, 5.0, 0.05), // LITERAL-PX-OK: segundos
    (ids::INSP_CAMERA_FOLLOW_OFF_X, 0.0, -1000.0, 1000.0, 0.1), // LITERAL-PX-OK: metros
    (ids::INSP_CAMERA_FOLLOW_OFF_Y, 0.0, -1000.0, 1000.0, 0.1), // LITERAL-PX-OK: metros
    // A cerca.
    (ids::INSP_CAMERA_MIN_X, -10.0, -100_000.0, 100_000.0, 0.5), // LITERAL-PX-OK: metros
    (ids::INSP_CAMERA_MIN_Y, -10.0, -100_000.0, 100_000.0, 0.5), // LITERAL-PX-OK: metros
    (ids::INSP_CAMERA_MAX_X, 10.0, -100_000.0, 100_000.0, 0.5),  // LITERAL-PX-OK: metros
];

/// ⚠️ **O `MAX_Y` sai da tabela acima por o array ser de 15** — ele entra à parte para a tabela não
/// precisar de mudar de tamanho quando um campo novo chegar. *Um número de tamanho de array é a
/// coisa que toda feature nova edita no teste de outra pessoa.*
const MAX_Y: (ph2d_a11y::NodeId, f64, f64, f64, f64) =
    (ids::INSP_CAMERA_MAX_Y, 10.0, -100_000.0, 100_000.0, 0.5); // LITERAL-PX-OK: metros

/// O tamanho do formato `Custom` (plano GEF 17), em píxeis: a faixa é a do `GameFormat`.
const CUSTOM_W: (ph2d_a11y::NodeId, f64, f64, f64, f64) = (
    ids::INSP_CAMERA_CUSTOM_W,
    1280.0, // LITERAL-PX-OK: largura de partida do `GameFormat::default()`
    ph2d_editor_core::game_format::CUSTOM_MIN_PX as f64,
    ph2d_editor_core::game_format::CUSTOM_MAX_PX as f64,
    1.0, // LITERAL-PX-OK: passo de um píxel
);
/// …a altura.
const CUSTOM_H: (ph2d_a11y::NodeId, f64, f64, f64, f64) = (
    ids::INSP_CAMERA_CUSTOM_H,
    720.0, // LITERAL-PX-OK: altura de partida do `GameFormat::default()`
    ph2d_editor_core::game_format::CUSTOM_MIN_PX as f64,
    ph2d_editor_core::game_format::CUSTOM_MAX_PX as f64,
    1.0, // LITERAL-PX-OK: passo de um píxel
);

pub(crate) fn populate_camera(store: &mut WidgetStore) {
    // O formato do jogo (plano GEF 17): as entradas do seletor e a moldura são BOTÕES; só o chip é
    // `Dropdown`, e ele guarda o `open`, nunca a escolha (essa vem do snapshot).
    crate::populate::register_button_ids(store, &ids::INSP_CAMERA_FORMAT_OPT);
    crate::populate::register_button_ids(store, &ids::INSP_CAMERA_FRAME);
    store.register(
        ids::INSP_CAMERA_FORMAT_PICK,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    store.register(
        ids::INSP_CAMERA_TARGET,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for id in [
        ids::INSP_CAMERA_ACTIVE,
        ids::INSP_CAMERA_PREVIEW,
        // ⚠️ **Os 32 bits da máscara são caixas** — sem isto a grade é pintada e morta sob o dedo.
    ]
    .into_iter()
    .chain(ids::INSP_CAMERA_CULL_BIT)
    {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    for (id, value, lo, hi, step) in NUMEROS
        .into_iter()
        .chain([MAX_Y, CUSTOM_W, CUSTOM_H])
    {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, lo, hi, step);
    }
    // ⚠️ **A máscara nasce RECOLHIDA**, como a irmã da visibilidade: são 32 caixas numa pergunta
    // avançada, e abri-la por omissão empurraria os campos que interessam para fora do ecrã.
    // ⛔⛔ **`_if_unchosen`, e não o `set_collapsed` cru.** Hoje este `populate` corre UMA vez (o
    // `HeroScreen::new` do arranque), logo a escrita crua não mordia; ⚠️ mas com ela a gaveta é
    // **INABRÍVEL** na primeira vez que alguém re-popular este painel, como cinco painéis da shell
    // já fazem por interacção. Reproduzido pela porta do produto em 2026-09-21:
    // `true` → clique → `false` → `populate` → **`true`**.
    store.set_collapsed_if_unchosen(ids::INSP_CAMERA_CULL_HEADER, true);
    // ⛔ Sem a marca o clique no cabeçalho MORRE (`apply_click` só dobra o que está marcado) — a
    // máscara viveu assim até 24/10, o gate dela abria-a à mão. Os grupos (doc CodeEditor 12)
    // nascem abertos.
    for id in [
        ids::INSP_CAMERA_CULL_HEADER,
        ids::INSP_CAMERA_FOLLOW_GROUP,
        ids::INSP_CAMERA_LIMITS_GROUP,
    ] {
        store.mark_collapsible_section(id);
    }
}
