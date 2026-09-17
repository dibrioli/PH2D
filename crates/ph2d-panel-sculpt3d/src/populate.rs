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
        // A curva da pista viaja com a ligação: o store e o painel projectam pela MESMA lei.
        store.link_slider_number_curved(
            row.slider,
            row.chip,
            row.scale(),
            row.offset(),
            row.curva(),
        );
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
        // ⚠️ **E a fileira do ESFREGÃO, pela mesma lição:** ela é pintada e
        // hit-indexada com o verbo na mão, e sem esta linha o clique é
        // **descartado em silêncio**. *Um controlo nunca pintado e um morto sob
        // o dedo dão o MESMO report* — quem os separa é o gesto REAL do
        // `every_smear_control_is_clickable_where_it_is_drawn`.
        &crate::ids::SCULPT3D_SMEAR_MODE[..],
        // ⛔⛔ **E a fileira do BOX TRIM, pela MESMA lição — a OITAVA ocorrência
        // desta família.** Sete vezes um chip nasceu pintado, hit-indexado e
        // morto sob o ponteiro nesta crate, e a última custou o report *«os
        // outros 2 botões ainda não funcionam»*. ⭐ Hoje há censo DERIVADO
        // (`populate_censo_tests`) que compara o despacho com o registo, logo
        // esquecer esta linha reprova antes de chegar às mãos do dono — mas a
        // linha continua a ter de ser escrita.
        &crate::ids::SCULPT3D_TRIM_FORMA[..],
        // ⛔ **E a fileira do PINCEL DE PLANO, a NONA ocorrência desta família.**
        // O censo derivado apanha o esquecimento antes do dono, e a linha
        // continua a ter de ser escrita.
        &crate::ids::SCULPT3D_PLANO_INVERSAO[..],
        &crate::ids::SCULPT3D_PROJECT_MODE[..],
        &crate::ids::SCULPT3D_FOLGA_MODO[..],
        // ⛔⛔⛔ **E A SÉTIMA OCORRÊNCIA — report do dono (2026-09-15): *«os
        // outros 2 botões ainda não funcionam»*.** As três fileiras abaixo
        // (`3 + 6 + 4 = 13` chips) nasceram pintadas, hit-indexadas, **com braço
        // no `event.rs`** e **mortas sob o ponteiro**. Da mão do artista o
        // sintoma é o pior possível: clicar em `Scale / Translate` não muda
        // nada, logo o pincel FICA no modo de omissão e as outras duas
        // deformações lêem-se como **leis partidas** em vez de um clique
        // descartado.
        //
        // ⚠️ *Um controlo nunca pintado e um morto sob o dedo dão o MESMO
        // report* — e nenhum gate o via, pela razão que o bloco do tecido acima
        // já escreve: as fixturas de costura armam **outro** pincel, e com ele
        // na mão estas fileiras nem chegam a ser desenhadas.
        //
        // ⇒ a cura estrutural é o gate irmão
        // [`crate::populate_censo_tests`], que **deriva** esta lista do despacho
        // do `event.rs` em vez de a confiar a quem se lembrar. *Sete vezes é
        // onde uma lista escrita à mão deixa de ser um descuido e passa a ser um
        // defeito de desenho.*
        &crate::ids::SCULPT3D_POSE_MODE[..],
        &crate::ids::SCULPT3D_POSE_ARRASTO[..],
        &crate::ids::SCULPT3D_BOUNDARY_MODE[..],
        &crate::ids::SCULPT3D_BOUNDARY_FALLOFF[..],
        &crate::ids::SCULPT3D_ALPHA[..],
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
    // ⛔⛔⛔ **OS INTERRUPTORES SAEM DA TABELA QUE OS DESPACHA**, e não de uma
    // segunda lista escrita aqui — a MESMA lei que o bloco dos comandos logo
    // acima já aplica, e que esta lista violava havia dezassete entradas.
    //
    // ⚠️ **O `Pin far end` da POSE estava de fora** (report do dono, 2026-09-15,
    // apanhado pelo `every_pose_control_is_clickable_where_it_is_drawn`): ele é
    // o que separa uma rotação em torno do pivô de um arrasto rígido, logo
    // metade da espec §5.1 era inexprimível pelo artista — e o `Scale without
    // rotating` do lado dele idem. *Uma lista à mão ao lado de uma tabela é a
    // segunda resposta à mesma pergunta, e a que o artista toca é a que
    // envelhece.*
    //
    // ⚠️ **A lei de VISIBILIDADE de cada um fica na tabela e não aqui:** o
    // registo é incondicional de propósito — registar só os pintados faria o
    // registo depender de um estado que muda com um clique, que é a armadilha
    // que os blocos de chips acima nomeiam três vezes.
    for (id, _, _) in crate::event::toggles::TOGGLES {
        button(store, id);
    }

    // Os que NÃO são interruptores de tabela: um comando de fecho e os quatro
    // rádios que não viram um booleano.
    for id in [
        crate::ids::SCULPT3D_REF_MODE_ALL,
        crate::ids::SCULPT3D_SYM_X,
        crate::ids::SCULPT3D_SYM_Y,
        crate::ids::SCULPT3D_SYM_Z,
        crate::ids::SCULPT3D_CLOSE,
    ] {
        button(store, id);
    }
}

/// ⛔⛔⛔ **O CENSO que faz a oitava ocorrência ser impossível** — ver o doc do
/// módulo.
#[cfg(test)]
#[path = "populate_censo_tests.rs"]
mod censo_tests;
