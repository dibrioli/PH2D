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
        &crate::ids::SCULPT3D_VERB[..],
        // ⚠️ **Os TRÊS entram, e o painel pinta só os oferecidos** — registrar
        // só os pintados faria o registro depender do verbo corrente, e um chip
        // que nasce vivo num verbo e morto noutro é a forma mais cara deste bug.
        &crate::ids::SCULPT3D_REF_MODE[..],
        // ⚠️ **Os SETE entram, e o painel pinta a fileira só com o filtro
        // ARMADO** — a mesma lei dos vizinhos: registrar só os pintados faria
        // o registro depender de um estado que muda com um clique, e um chip
        // que nasce vivo armado e morto desarmado é o bug caro desta família.
        &crate::ids::SCULPT3D_FILTER_KIND[..],
        // ⛔⛔ **AS DUAS FILEIRAS DO FILTRO DE TECIDO, e elas caíram no MESMO
        // buraco na hora em que nasceram** (07/09): pintadas, hit-indexadas e
        // **mortas sob o ponteiro**. Quem as apanhou foi o
        // `every_filter_law_is_pickable_and_writes_its_own`, que clica de
        // verdade — *um widget não está pronto quando pinta; está pronto quando
        // um teste CLICA nele.*
        //
        // ⚠️ **Os cinco tipos e as duas orientações entram SEMPRE**, mesmo que o
        // painel só pinte a fileira do referencial com uma lei de tecido
        // escolhida: registar só os pintados faria o registo depender de um
        // estado que muda com um clique.
        &crate::ids::SCULPT3D_CLOTH_FILTER_KIND[..],
        &crate::ids::SCULPT3D_CLOTH_FILTER_ORIENT[..],
        // ⚠️ Mesma lei do vizinho de cima: os três entram, e o painel pinta
        // a fileira só onde o verbo declara campo E o nível é Pro.
        &crate::ids::SCULPT3D_ELASTIC_SCALES[..],
        &crate::ids::SCULPT3D_UI_LEVEL[..],
        // ⚠️ Os DOIS entram sempre: registrar só o motor corrente faria o chip do
        // outro nascer morto — e um chip pintado que ninguém registrou é uma
        // affordance que não faz nada.
        &crate::ids::SCULPT3D_RETOPO_MODE[..],
        &crate::ids::SCULPT3D_FALLOFF[..],
        // ⛔⛔ **AS TRÊS FILEIRAS DO TECIDO FALTAVAM AQUI desde 06/09** — os oito
        // modos e as três áreas nasceram pintados, hit-indexados e **mortos sob
        // o ponteiro**, que é literalmente o defeito que o cabeçalho deste
        // ficheiro descreve. E nenhum gate o viu: o
        // `every_painted_control_is_clickable_where_it_is_drawn` arma o **Crease**,
        // e com outro pincel na mão a fileira do tecido nem é desenhada. *A
        // fixtura tem de conter o fenómeno* — a SEXTA vez que este módulo o
        // escreve, e a primeira em que a frase custou uma wave inteira de
        // controlos.
        &crate::ids::SCULPT3D_CLOTH_MODE[..],
        &crate::ids::SCULPT3D_CLOTH_AREA[..],
        &crate::ids::SCULPT3D_CLOTH_FORCE_FALLOFF[..],
        &crate::ids::SCULPT3D_ALPHA[..],
        &crate::ids::SCULPT3D_DETAIL[..],
        &crate::ids::SCULPT3D_ADD[..],
        &crate::ids::SCULPT3D_MASK_OP[..],
        &crate::ids::SCULPT3D_TRANSFORM[..],
        &crate::ids::SCULPT3D_MATCAP[..],
    ] {
        for &id in group {
            button(store, id);
        }
    }

    // Os cabeçalhos são interativos (o chevron os dobra), então são registrados
    // como qualquer outro controle — um chevron pintado que ninguém registrou é
    // uma affordance que não faz nada.
    //
    // ⚠️ **A lista é a de [`rows::section_headers`], e é a MESMA que o
    // `event::is_section_header` percorre.** Enquanto eram duas listas à mão,
    // esta registrava quatro cabeçalhos de botão e aquela comparava três: o
    // `SCULPT3D_SEC_BAKE` nascia registrado, focável e **sem braço** — a dobra
    // pintada que não acontece.
    for id in rows::section_headers() {
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
        crate::ids::SCULPT3D_REF_MODE_ALL,
        crate::ids::SCULPT3D_SYM_X,
        crate::ids::SCULPT3D_SYM_Y,
        crate::ids::SCULPT3D_SYM_Z,
        crate::ids::SCULPT3D_ALPHA_PREVIEW,
        crate::ids::SCULPT3D_WIREFRAME,
        crate::ids::SCULPT3D_ACCUMULATE,
        crate::ids::SCULPT3D_FRONT_FACES,
        crate::ids::SCULPT3D_SURFACE_ONLY,
        crate::ids::SCULPT3D_SCRAPE_DYNAMIC,
        crate::ids::SCULPT3D_CLOTH_PIN,
        crate::ids::SCULPT3D_CLOTH_PERSISTENT,
        crate::ids::SCULPT3D_CLOTH_COLLISIONS,
        // ⭐⭐ **OS CONTROLOS DO FILTRO DE TECIDO** (2026-09-08). ⚠️ **Um `const`
        // pintado e não registado é um controlo MORTO sob o dedo** — o gate
        // `every_painted_control_is_clickable_where_it_is_drawn` apanhou os
        // quatro na primeira corrida, e a mensagem dele nomeia o mecanismo: *um
        // id ausente do store não é focável*.
        crate::ids::SCULPT3D_CFILTER_COLLISIONS,
        crate::ids::SCULPT3D_CFILTER_AXIS[0],
        crate::ids::SCULPT3D_CFILTER_AXIS[1],
        crate::ids::SCULPT3D_CFILTER_AXIS[2],
        crate::ids::SCULPT3D_CLOSE,
    ] {
        button(store, id);
    }
}
