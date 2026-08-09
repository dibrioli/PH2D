//! O registro dos widgets — percorre a MESMA lista que o `paint`.
//!
//! ⚠️ Um widget que o `paint` põe no índice de hit e que ninguém regista tem `is_focusable() ==
//! false`, e o clique dele é descartado **em silêncio**: sem erro de compilação, sem warning, só
//! um controle que não faz nada. É a classe que o `architecture_panel_wiring_parity` existe para
//! pegar — e derivar esta lista da tabela que o `paint` percorre é o que a torna algo que ninguém
//! pode esquecer, nem o gerador.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, DropdownState, ListItemState, SliderOrientation,
    SliderState, TagState, TextInputState, ToggleState, WidgetKind, format_number,
};

use crate::rows::{Row, with_rows};

/// O estado inicial de uma row que RESPONDE, ou `None` para as que só desenham.
///
/// ⚠️ **O valor inicial é o neutro do tipo, não um valor "bonito".** A prévia do canvas mostra um
/// slider a meio porque um slider a zero é indistinguível de uma trilha quebrada — mas ali não há
/// gesto: é uma foto. Aqui há, e um painel que nasce com meia opacidade afirmaria um valor que o
/// artista não escolheu. O zero é honesto: nada foi mexido ainda.
fn initial(kind: WidgetKind) -> Option<InteractiveState> {
    Some(match kind {
        // ⚠️ **Os dois botões partilham o estado, e isso não é atalho:** um botão de ícone é um
        // botão cuja face é um glifo — o que muda é o que ele DESENHA, nunca o que ele SENTE. Um
        // `InteractiveState::IconButton` seria um segundo nome para *pressionado*, e o pintor
        // teria de aceitar os dois.
        WidgetKind::Button | WidgetKind::IconButton => InteractiveState::Button {
            state: ButtonState::Normal,
        },
        // ⚠️ **A família de LISTA nasce com a PRIMEIRA marcada, não com nenhuma:** um controle de
        // opções sem seleção desenha N itens e nenhum aceso, que o artista lê como *quebrado* e
        // não como *vazio*. `Tabs` e `SegmentedAdaptive` partilham o estado pelo mesmo motivo que
        // os dois botões o partilham — o que muda entre eles é o DESENHO, não o que eles sentem.
        WidgetKind::Tabs | WidgetKind::SegmentedAdaptive => InteractiveState::Tabs { selected: 0 },
        WidgetKind::RadioGroup => InteractiveState::Radio {
            state: ButtonState::Normal,
            selected_index: 0,
        },
        // ⚠️ **`open: false`, e o despacho GENÉRICO é quem o alterna** — um clique no chip de um
        // `InteractiveState::Dropdown` registado já vira `open` (`dropdown_click_toggles_open`).
        // Nada neste painel abre a lista: ele só a PINTA quando o store diz que ela está aberta.
        WidgetKind::Dropdown => InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
        WidgetKind::Toggle => InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: false,
        },
        WidgetKind::Checkbox => InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
        WidgetKind::Slider => InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
        WidgetKind::TextInput => InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
        WidgetKind::Tag => InteractiveState::Tag {
            state: TagState::Normal,
        },
        WidgetKind::ListItem => InteractiveState::ListItem {
            state: ListItemState::Normal,
            selected: false,
        },
        WidgetKind::NumberInput => InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            // ⚠️ O buffer ESPELHA o valor, e a formatação sai da porta do próprio widget: uma
            // segunda regra aqui faria o campo nascer mostrando um texto que o pintor não
            // escreveria, e a divergência só apareceria no primeiro foco.
            buffer: format_number(0.0),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
        // Os que só desenham. ⚠️ `None` é a resposta, e o `is_control` é quem a decide — ver o
        // doc dele: três cópias desta pergunta divergiriam com modos de falha diferentes.
        //
        // ⚠️ O `LevelMeter` está aqui por NATUREZA, não por omissão: ele é um READOUT (nível de
        // áudio), e não existe `InteractiveState::LevelMeter` para registar. Um medidor que
        // acendesse ao clique seria o item-de-menu-morto pintado com outro nome.
        WidgetKind::ProgressBar
        | WidgetKind::SectionHeader
        | WidgetKind::Card
        | WidgetKind::Spinner
        | WidgetKind::Divider
        | WidgetKind::LevelMeter
        | WidgetKind::ColorSwatch => return None,
    })
}

/// ⚠️ **`register_if_absent`, pela razão do [`adopt`]:** as opções são re-adotadas a cada quadro,
/// e um `register` reporia `Normal` por cima do `Pressed` no meio do clique — a opção acenderia e
/// apagaria debaixo do dedo.
fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register_if_absent(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Os punhos de chrome — mover e redimensionar.
///
/// ⚠️ Eles moram AQUI, e não no `pre_populate` do hero: aquele arquivo os regista para o Inspector
/// e a Hierarquia com um comentário dizendo que *"movem para o `populate` do painel quando a crate
/// possuir a alocação de NodeId"* — esta possui, então esta o faz.
fn chrome(store: &mut WidgetStore) {
    for (id, kind) in [
        (ids::AUTHORED_DRAG_HANDLE, BlenderHitKind::DragHandle),
        (ids::AUTHORED_RESIZE_HANDLE, BlenderHitKind::ResizeHandle),
        (
            ids::AUTHORED_RESIZE_HANDLE_BL,
            BlenderHitKind::ResizeHandleBl,
        ),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::AUTHORED_PANEL,
                kind,
            },
        );
    }
}

pub fn populate(store: &mut WidgetStore) {
    button(store, ids::AUTHORED_CLOSE);
    chrome(store);
    with_rows(|table| {
        for row in table {
            adopt(store, row);
        }
    });
}

/// **UMA row entra no store** — a porta única, e ela tem DOIS chamadores por uma razão de TEMPO.
///
/// ⚠️ **O `populate` corre uma vez; o documento muda por quadro.** A tabela viva é publicada a
/// cada frame pelo `render_loop`, mas o registo acontecia só no arranque, sobre a tabela
/// COMPILADA — então toda row que o artista autora e o `generated/panel.rs` ainda não conhece
/// nascia com um id que ninguém registou: `is_focusable` diz `false`, e o dispatcher **larga o
/// clique em silêncio**. Pintada, com retângulo de hit, e morta.
///
/// ⚠️ **Era invisível porque as chaves COINCIDIAM:** o golden foi gerado da mesma cena do smoke,
/// então as rows vivas tinham exatamente os ids que o `populate` já registara. Bastou o artista
/// desenhar uma seção a mais para o vão aparecer (report do Enio, 2026-08-09: *"não consigo
/// contraí-la"*) — e o modo de falha é o mesmo para um controle novo, não só para um cabeçalho.
///
/// ⚠️ **Nem `register` nem `register_if_absent`: a pergunta é sobre a VARIANTE.** Quem pinta chama
/// isto a cada quadro, então um `register` devolveria o slider ao zero e o botão a `Normal` com o
/// dedo em cima — o valor do artista apagado sessenta vezes por segundo. Mas um
/// `register_if_absent` sozinho guarda o estado do tipo **ANTIGO**, e isso não é um caso de canto:
/// **é o caminho normal de toda row que o artista cria.** O `WidgetEdit::Wear` insere
/// `WidgetKind::Button` e só depois ele escolhe o tipo que queria, então todo controle autorado
/// passa por Button e o id entra no store como Button antes de ser o que é.
///
/// ⚠️ **O report que trouxe isto tinha duas metades e uma causa** (Enio, 2026-08-09): *"o checkbox
/// que criei não fica checado mas EMITE SINAL; o que você criou não emite sinal mas pode ser
/// checado"*. Com o estado obsoleto, o `paint` lê um `Button` onde devia ler um `Checkbox` — não
/// há o que marcar — e a cadeia do `event` cai no `else` final, que é o `Fired` dos BOTÕES, o
/// único intent que vira **sinal**. A metade *"o seu funciona"* é o CONTROLE que nomeia a causa: a
/// row da tabela compilada foi registada com o tipo certo no arranque e nunca trocou.
///
/// A lei, então: **o estado obsoleto é substituído; o estado do tipo CERTO é preservado.** A
/// pergunta é `discriminant`, e não uma tabela de pares — `Button`/`IconButton` partilham
/// `InteractiveState::Button` de propósito, e `Tabs`/`SegmentedAdaptive` partilham o `Tabs`: a
/// variante já é a resposta, e uma lista à mão nasceria sem o próximo par.
pub(crate) fn adopt(store: &mut WidgetStore, row: &Row) {
    // A pergunta é feita ao `is_control`, e o `initial` a espelha. As duas concordam por
    // construção — há gate a exigi-lo, porque uma delas mudar sozinha é o caminho para um
    // controle registado que o `paint` não desenha (ou o contrário).
    if let Some(st) = initial(row.kind) {
        debug_assert!(Row::is_control(row), "registado sem ser controle");
        let stale = store
            .get(row.id)
            .is_none_or(|cur| std::mem::discriminant(cur) != std::mem::discriminant(&st));
        if stale {
            store.register(row.id, st);
        }
    }
    // ⚠️ **Um cabeçalho não é registado como widget** — ele não tem `InteractiveState`, e o
    // despacho o trata ANTES do `switch` justamente por isso. O que ele precisa é de ser
    // MARCADO: sem esta linha o chevron desenha, o retângulo de hit existe, o clique chega —
    // e não dobra nada, porque o despacho não sabe que aquele id é uma seção.
    //
    // ⚠️ **E o `else` é a outra metade do mesmo defeito**, pelo lado inverso: o despacho consulta
    // a marca ANTES do switch, então uma que sobreviva à troca de tipo faz o clique DOBRAR uma
    // seção que já não existe em vez de marcar a caixa (medido). A pergunta *"esta row é um
    // cabeçalho?"* é feita uma vez e a resposta é ESTABELECIDA, nas duas direções — em vez de
    // acrescentada e nunca retirada.
    if row.folds_a_section() {
        store.mark_collapsible_section(row.id);
    } else {
        store.unmark_collapsible_section(row.id);
    }
    // ⚠️ **A swatch é ALVO DE PICKER, e a cor dela é PUBLICADA a cada quadro.** O `pointer_down`
    // do editor abre o picker OKLCH partilhado para qualquer id marcado assim, semeando-o com o
    // `widget_color` — então publicar a cor é o que faz o picker abrir na cor da forma em vez de
    // num cinzento inventado. É o padrão do painel de vetor (Stroke/Fill), aqui derivado da row.
    if row.opens_a_picker() {
        store.register_picker_swatch(row.id);
        if let Some(rgba) = row.rgba {
            store.set_widget_color(row.id, rgba);
        }
    }
    options(store, row);
}

/// **As opções de um controle de LISTA** — as quatro variantes, e a escondida mesmo fechada.
///
/// ⚠️ **Foi o seam quem achou isto, na primeira corrida, e é a falha que ele existe para pegar:**
/// as opções eram pintadas e ganhavam retângulo de hit no passe diferido, e o clique nelas era
/// **descartado em silêncio** — sem `InteractiveState` o `is_focusable` diz `false` e o despacho
/// larga o evento antes de chegar ao painel. Lista aberta, opção acesa, clique morto.
///
/// ⚠️ **`Button`, e é o mesmo raciocínio dos dois botões acima:** a linha de uma opção é clicada e
/// mais nada — o que ela DESENHA (o realce, o tique da marcada) é do pintor da lista, que lê o
/// estado do DROPDOWN e não o da opção. Um `InteractiveState` próprio seria um segundo nome para
/// *pressionado*, guardado onde ninguém o lê.
///
/// ⚠️ **E o registo é incondicional, não gateado no `open`.** O `populate` corre uma vez, e o
/// `open` é estado que muda com o clique: registar só quando aberto daria um id que nasce
/// focusável **depois** de o passe de pintura já ter registado o retângulo dele — a mesma corrida,
/// um frame adiante.
fn options(store: &mut WidgetStore, row: &Row) {
    if !row.kind.takes_options() {
        return;
    }
    for i in 0..row.options.len() {
        button(store, ids::authored_option_id(&row.key, i));
    }
}

#[cfg(test)]
#[path = "populate_tests.rs"]
mod tests;
