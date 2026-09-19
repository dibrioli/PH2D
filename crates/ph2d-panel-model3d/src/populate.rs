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
/// ⭐⭐⭐ **E ELE É DERIVADO DA MAIOR FORMA, desde 2026-09-13** — hoje `2 × 27 + 31`, mais a folga da
/// CENA:
///
/// | parcela | de onde vem |
/// |---|---|
/// | `2 × MAX_POLYGON_VERTICES` | um polígono tem **duas** linhas por vértice, e `27` é o teto dele |
/// | `+ EXTRAS_DE_UM_NO` | o que **todo** nó tem além dos vértices, no PIOR estado: `3` de posição, `3` de rotação, o filete, o raio de junção, a resolução do contorno… e as do material com o brilho, o verniz e a subsuperfície acesos |
/// | `+ MAX_SCENE_ROWS` | ⭐ o que a CENA apende **depois** das do nó — e era este que faltava |
///
/// ⚠️ **Ele era `64`, escrito «na primeira vez» e sem medição** — e a nota dele dizia, por escrito,
/// *«quando um documento real passar disto, o número muda com uma medição atrás»*. O material por
/// objecto (`docs/Render3d/05`) acrescentou **5** linhas a toda folha, e foi esse o dia.
///
/// ⚠️ **E os dois gates da `polygon_rows_tests` prendem-no pelos DOIS lados**: o polígono no teto
/// tem de caber, e o vértice seguinte **não** pode caber. *Uma tolerância que só se defende de um
/// lado não descreve nada.*
///
/// ⭐⭐ **No BRILHO PRÓPRIO ele NÃO subiu, e a razão foi uma régua corrigida** (`docs/Render3d/05`
/// §20): a régua contava **params** e o painel pinta **linhas** — as duas amostras de cor dobram `6`
/// params em `2`. A família estava sobre-provisionada em exactamente `2`, e o brilho consumiu essa
/// folga.
///
/// ⭐⭐⭐ **E no VERNIZ ele subiu, `69 → 74`** (§21), e nas ÚLTIMAS CINCO ENTRADAS do OpenPBR
/// **`74 → 79`** (§22) — as duas com a medição ao lado. O pior estado é o brilho *e* o verniz
/// acesos, com o metal abaixo de `1` (que é quando as duas linhas só-dieléctricas aparecem):
///
/// | vértices | tudo apagado | tudo aceso |
/// |---|---|---|
/// | `3` | `26` | `31` |
/// | `16` | `52` | `57` |
/// | **`27`** | `74` | **`79`** |
///
/// ⭐⭐⭐ **E na SUBSUPERFÍCIE subiu `79 → 85`** (17/09, `docs/Render3d/10`) — seis linhas: o peso, a
/// cor, o raio, a escala do raio (uma amostra sobre três canais), a fase e a parede fina. O pior
/// estado passa a ter também a subsuperfície acesa:
///
/// | vértices | tudo apagado | tudo aceso |
/// |---|---|---|
/// | `3` | `32` | `37` |
/// | **`27`** | `80` | **`85`** |
///
/// ⭐ **O preço MEDIDO de cada subida:** cada linha regista `6` widgets, logo cinco linhas custam
/// **`30` widgets e 5 `String`** no store, uma vez, no arranque — o mesmo nas quatro.
///
/// ⛔⛔ **E a frase que aqui estava — *«o material FECHOU: são as `15` entradas do OpenPBR, e não há
/// mais nenhuma para apender; este teto deixa de crescer por material»* — MORREU em 17/09.** Ela
/// era verdade sobre a FATIA de 14/09 e falsa sobre o modelo: o `open_pbr_surface` tem **41**
/// entradas, e a própria `ph2d-material` declarava por escrito, na mesma semana, que
/// `transmission`, `subsurface`, `fuzz`, `thin_film` e `geometry_opacity` ficavam *«⛔ nesta
/// fatia»*. ⚠️ **A que fica de pé é a do parágrafo seguinte**, e é ela que decidiu esta subida
/// também: *uma lista fecha-se contra o que se construiu, nunca contra o que existe.*
///
/// ⛔⛔ **A alternativa era baixar o `MAX_POLYGON_VERTICES` de `27` para `24`**: tirar três vértices
/// ao artista porque uma peça passou a poder ser envernizada. *Um teto de registo cujo recurso é
/// memória a mandar num teto de FORMA é o caminho lento a definir o rápido* (`CLAUDE.md` §0.0).
///
/// ⭐⭐⭐ **E DESDE 2026-09-19 ELE É DERIVADO, e conta a CENA** — ver [`MAX_ROWS_DE_UM_NO`] e
/// [`MAX_SCENE_ROWS`].
pub const MAX_ROWS: usize = MAX_ROWS_DE_UM_NO + MAX_SCENE_ROWS;

/// ⭐⭐⭐ **Quantas linhas o NÓ ESCOLHIDO pode pedir, no pior estado.**
///
/// ⚠️ **Era isto que o `MAX_ROWS` media, e por isso ele estava errado desde a camada de estilo:** o
/// retrato que o painel recebe **não** é só do nó — quem o publica apende, no fim, as fileiras da
/// CENA. Ver [`MAX_SCENE_ROWS`].
pub const MAX_ROWS_DE_UM_NO: usize =
    2 * ph2d_field::MAX_POLYGON_VERTICES as usize + EXTRAS_DE_UM_NO;

/// ⭐ **O que TODO nó tem além das linhas dos vértices** — `3` de posição, `3` de rotação, o filete,
/// o raio de junção, a resolução do contorno… e as do material no pior estado (brilho e verniz
/// acesos, metal abaixo de `1`, subsuperfície acesa).
///
/// ⚠️ **Ele é AFIRMADO pelo gate `one_more_vertex_would_not_fit`** (`ph2d-app-field3d`), que o mede
/// no produtor das linhas: *um número que não se mexe enquanto a grandeza muda é a forma mais
/// silenciosa de um teto deixar de descrever o que mede.*
pub const EXTRAS_DE_UM_NO: usize = 31;

/// ⭐⭐⭐ **A FOLGA DA CENA — quantas fileiras o retrato pode ganhar DEPOIS das do nó.**
///
/// # ⛔⛔⛔ O painel podia ENGOLIR uma secção inteira, em silêncio
///
/// Auditoria de 2026-09-19 (`docs/Render3d/11` §10.9). O `publish_snapshot` apende as fileiras da
/// camada de ESTILO **no fim** da lista, e o [`crate::paint`] corta em `MAX_ROWS`. Medido, com o
/// teto a valer exactamente o pior nó (`85`):
///
/// | vértices do polígono | linhas do nó | + estilo | fileiras de ESTILO visíveis |
/// |---:|---:|---:|---|
/// | `22` | `75` | `85` | 10 de 10 |
/// | `23` | `77` | `87` | **8 de 10** |
/// | **`27`** | `85` | `95` | **0 de 10 — a secção INTEIRA desaparece** |
///
/// ⚠️ O rodapé dizia `(+10)`, que é honesto e **inútil**: o artista não sabe que existe uma secção
/// chamada *Style*, logo `+10` lê-se como *«dez números do meu nó ficaram de fora»*.
///
/// # ⚠️ De que recurso ele é, MEDIDO
///
/// Do **registo de widgets**, e de mais nada: cada linha custa `2 + MAX_CHOICES` = **6** entradas no
/// store, cunhadas uma vez no arranque. Medido em 2026-09-19 pelo gate
/// `o_preco_do_teto_de_linhas_e_o_registo` (`populate_preco_tests.rs`), que re-mede a tabela:
/// o `populate` inteiro cunha **912** entradas, das quais **192** são esta folga, e corre em
/// **`172,9 µs` em `--release`** (`650,0 µs` em debug) — **uma vez, no arranque**. ⛔ E **nada por
/// quadro**: o `paint` percorre `snapshot.rows`, que tem o tamanho do RETRATO e não o do registo.
///
/// ⛔ **E ele NÃO é `10`.** A camada de estilo cresce dentro da mesma jornada em que este número foi
/// escrito (a suavidade da curvatura e a nitidez separada de aresta e de cova), e *um teto que
/// descreve a população de hoje é um teto que reprova amanhã sobre produto correcto*. O que o prende
/// é o gate `as_fileiras_da_cena_cabem_na_folga_delas`, que conta as fileiras **no produtor**.
pub const MAX_SCENE_ROWS: usize = 32;

/// ⛔ **O piso da folga é ERRO DE COMPILAÇÃO.** Com `0` o teto volta a ser o de 2026-09-19 —
/// exactamente o estado em que a secção de estilo desaparecia inteira —, e a camada dela já pede
/// **12** fileiras (`10` na manhã do mesmo dia: *ela cresce dentro da própria jornada*).
///
/// ⚠️ **Um `assert!` num teste sobre duas constantes é DOBRADO pelo compilador antes de correr**, e
/// o clippy recusa-o em voz alta (`this assertion has a constant value`). *A lei mora ao lado do
/// número, e quem a viola não chega a compilar.*
const _: () = assert!(MAX_SCENE_ROWS >= 16);

/// Quantos botões uma linha de **escolha** pode oferecer.
///
/// ⚠️ **É um limite de REGISTO, e a mesma natureza do [`MAX_ROWS`]**: o `populate` corre antes de a
/// peça existir e cunha a família às cegas. Hoje o único consumidor é o eixo, que tem **três**
/// ([`ph2d_field::Axis::ALL`]); a folga de um é para a escolha seguinte não obrigar a mexer aqui, e
/// o gate `every_choice_row_fits_the_registered_family` afirma que ninguém a estourou.
pub const MAX_CHOICES: u32 = 4;

/// Quantos verbos (e quantos referenciais) um seletor consegue mostrar.
///
/// ⚠️ Mesma natureza do [`MAX_ROWS`]: é um limite de **registro**, porque o `populate` corre antes
/// de o gizmo existir. Cada slot é um `NodeId` no store, e o custo é `famílias × MAX_MODES`.
///
/// ⛔⛔ **ESTE NÚMERO ERA UMA MINA, e ela quase disparou** (2026-08-30): ele estava em `8`, e o
/// `ph2d_field::UnaryKind::ALL` tinha **exactamente 8**. O modificador seguinte nasceria com o chip
/// **não pintado** (`paint.rs` faz `.take(MAX_MODES)`) e **sem id registado** — e `apply_click` faz
/// `match store.get_mut(id)`, que devolve `None`: *o evento nunca nasce*. ⚠️ Nenhum portão o via: o
/// `every_painted_button_answers_a_real_click` varre **o que o painel pintou**, e um chip que nunca
/// é pintado não entra na varredura.
///
/// ⭐ **A fileira WRAPPA** (`segmented_row_counts`), então o teto nunca foi visual — é só registo.
/// `16` é o dobro da maior família de hoje, e quem prova que ele chega é o
/// `the_panel_registers_a_slot_for_every_modifier`, que compara os dois lados.
pub const MAX_MODES: u32 = 16;

/// ⭐⭐ **AS FAMÍLIAS DE BOTÕES DO PAINEL, NUMA LISTA SÓ** (W48).
///
/// # Por que ela existe
///
/// A W47 acrescentou duas fileiras e tocou **quatro** dos cinco sítios que um controle precisa (o
/// campo no retrato, a linha no `paint`, o braço no `event.rs`, a família de ids) — e esqueceu este.
/// Os chips pintavam, o índice de acerto tinha-os, e `apply_click` faz `match store.get_mut(id)`:
/// um id não registado devolve `None` e **o evento nunca nasce**. Enio, 2026-08-23: *"nenhum botão
/// funcionou"*.
///
/// ⚠️ Enquanto isto eram sete `store.register` copiados, esquecer o oitavo era a coisa **mais
/// natural do mundo** — não há erro de compilação, não há teste que note, e o sintoma é um botão
/// bonito e morto. Com uma lista, acrescentar a família **é** registá-la.
///
/// ⛔ Isto não substitui o gate: a lei
/// `every_painted_button_answers_a_real_click` (`tests/seam.rs`) varre **o que o painel registou ao
/// pintar** e exige que cada um responda a um clique de verdade — ela apanha esta falha e as outras
/// duas que o cabeçalho daquele arquivo já nomeava.
const CHIP_FAMILIES: &[fn(u32) -> ph2d_a11y::NodeId] = &[
    crate::ids::model3d_select_button,
    crate::ids::model3d_add_button,
    crate::ids::model3d_op_button,
    crate::ids::model3d_verb_button,
    crate::ids::model3d_character_button,
    crate::ids::model3d_mod_button,
    crate::ids::model3d_export_button,
    crate::ids::model3d_act_button,
    crate::ids::model3d_view_button,
    crate::ids::model3d_camera_button,
    crate::ids::model3d_shading_button,
    crate::ids::model3d_look_button,
    crate::ids::model3d_exposure_button,
];

/// ⭐⭐ **Quantas famílias de chip o painel regista** — derivado da lista, nunca escrito à mão.
///
/// ⚠️ Ele existe para o gate `every_chip_family_dispatches_its_own_intent` (`tests/seam.rs`), que é
/// de outra crate e por isso não vê a lista. *Sem ele, a tabela daquele gate seria uma segunda
/// contagem a envelhecer* — e a lição está a três linhas daqui, no defeito que o `CHIP_FAMILIES`
/// curou.
pub const CHIP_FAMILY_COUNT: usize = CHIP_FAMILIES.len();

pub fn populate(store: &mut WidgetStore) {
    for slot in 0..MAX_MODES {
        for family in CHIP_FAMILIES {
            store.register(
                family(slot),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    for node in 0..MAX_ROWS as u32 {
        let slider = crate::ids::model3d_radius_slider(node);
        let chip = crate::ids::model3d_radius_chip(node);
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
        // ⭐ **A fileira de escolha da mesma linha** — ver [`ParamRow::choices`]. Só uma das duas
        // formas é pintada por quadro, mas as duas são registadas às cegas aqui pela razão que o
        // topo deste arquivo dá: o `populate` corre **antes** de a peça existir.
        for cell in 0..MAX_CHOICES {
            store.register(
                crate::ids::model3d_choice_button(node, cell),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    store.register(
        crate::ids::MODEL3D_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // A moldura móvel/redimensionável, como todo painel encaixado deste shell.
    store.register(
        crate::ids::INSP_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::MODEL3D_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
}

/// ⚠️ **O preço do teto, medido** — ver [`MAX_SCENE_ROWS`].
#[cfg(test)]
#[path = "populate_preco_tests.rs"]
mod preco_tests;
