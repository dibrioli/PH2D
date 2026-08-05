//! **A PELE POR-WIDGET** — o degrau 2 do §2 do plano UI/UX: uma forma autorada no canvas é
//! pintada pelo **pintor REAL** do catálogo, no lugar dela.
//!
//! # A pergunta desta wave, e a única resposta que não bifurca
//!
//! O §2 do plano recusou as duas respostas fáceis (*o desenho VIRA o widget* — reimplementar
//! drag/foco/teclado no canvas; *o widget VIRA desenho* — trocar 44 widgets testados por um
//! interpretador) e escolheu a terceira: **o desenho é a PELE, o widget é o COMPORTAMENTO, o
//! token é a PONTE**. Este módulo é a ponte.
//!
//! ⚠️ **O canvas chama o PINTOR REAL, nunca uma cópia.** Uma prévia que redesenhasse o botão à mão
//! seria uma segunda resposta a *"que aparência tem este widget?"*, e a divergência entre as duas
//! só apareceria numa screenshot — o modo de falha mais caro que este repo conhece, porque
//! ninguém lê número em screenshot. Daí a forma deste arquivo: uma função, um `match`, e cada
//! braço termina no `paint_*` que o painel nativo já chama.
//!
//! # A APARÊNCIA vem dos TOKENS, não do desenho — e isso é decisão, não omissão
//!
//! O plano falava em *"cantos, sombra, gradiente, cores por estado"* saindo do desenho. Medido: os
//! painéis **não aceitam** nada disso — a assinatura deles é `(dados, rect, scene, text, theme)` e
//! toda cor/raio sai de `ph2d_tokens`. Dar-lhes um canal de estilo por-widget seria 44 assinaturas
//! novas **e** uma segunda porta para a aparência, no dia seguinte ao da W6.1 ter feito a tabela de
//! cor autorável.
//!
//! ⇒ **O desenho responde ONDE e O QUÊ** (o retângulo e o tipo); **os tokens respondem COMO**. Um
//! mapeamento por-tipo (*"o preenchimento da forma vira a cor da swatch"*) é uma tabela de 44 casos
//! especiais e está **deliberadamente NÃO construído**: ele nasce no dia em que um tipo precisar de
//! um parâmetro que o token não exprime.
//!
//! # O RÓTULO é o `Name` da entidade
//!
//! Não há campo de rótulo no componente. O nome que o artista já digita na Hierarquia **é** o
//! rótulo — a mesma lei que a W5c usou para os eixos de variant, e pelo mesmo motivo: um segundo
//! lugar para digitar o nome de uma coisa é um segundo lugar que discorda do primeiro.
//!
//! # O ESTADO é `Normal`, e a fronteira tem dono
//!
//! *idle / hover / press / disabled* é a **W7** (a máquina de estados + Smart Animate), que o plano
//! escalona depois desta. Um widget que respondesse ao mouse aqui seria comportamento no canvas —
//! exactamente o que o §2 recusou.

use crate::interaction::InteractiveState;
use crate::widget::{
    Button, Card, Checkbox, Divider, ListItem, ProgressBar, SectionHeader, Slider, Spinner, Tag,
    TextInput, Toggle, paint_button, paint_card, paint_checkbox, paint_divider, paint_list_item,
    paint_progress_bar, paint_section_header, paint_slider, paint_spinner, paint_tag,
    paint_text_input, paint_toggle,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// O valor que um controle contínuo mostra na prévia.
///
/// ⚠️ Um slider em `0` ou em `1` é indistinguível de uma trilha quebrada — a barra preenchida
/// desaparece ou come a trilha inteira, e o artista não consegue julgar nem uma nem outra. O meio
/// é o único ponto em que as **duas** metades do controle são visíveis ao mesmo tempo. Ele não é
/// um limite (não há recurso do qual seja): é o que a prévia mostra até a W7 ligar o valor a uma
/// fonte.
const PREVIEW_VALUE: f32 = 0.5;

/// O id que os pintores recebem na prévia.
///
/// ⚠️ **Nenhum pintor do catálogo lê o próprio `id` ao pintar** (conferido: eles recebem o estado
/// por argumento) — o id existe para a a11y e para o `WidgetStore`, e uma pele de canvas não tem
/// nem uma nem outro. Um id REAL aqui seria pior que inútil: ele colidiria com o widget homônimo
/// do painel nativo no store que a shell possui.
const PREVIEW_ID: NodeId = NodeId(0);

/// **A PELE PREENCHE A MOLDURA — uma frase, doze tipos** (BUGS_vector #26).
///
/// Dez dos doze já faziam isso. Os outros dois desenhavam o widget **centrado dentro** dela — a
/// caixa do checkbox a 64,3% da altura, a trilha do slider a 25% —, e o resto era o *padding* de
/// uma LINHA de painel. Numa linha isso está certo: ali toda linha tem a mesma altura, todo
/// checkbox tem o mesmo tamanho, e é isso que faz um formulário ler como formulário.
///
/// ⚠️ **No canvas não há linha, e é isso que o torna um defeito visível:** o gizmo abraça o
/// RETÂNGULO, então uma pele que ocupa 25% dele deixa uma folga vertical que o artista vê a cada
/// gesto — e que faz o **snap encaixar no lugar errado**, porque o que encaixa é a moldura e o
/// que se vê é a tinta. Enio, 2026-08-05: *"por que o gizmo não se ajusta perfeitamente na
/// vertical … seria interessante se ajustar para fins de snap perfeito e também manter o padrão
/// dos outros widgets"*.
///
/// ⚠️ **A primeira correção parou a meio caminho** e a foto do report media exactamente a lei
/// dela: 64,3% e 25,0% previstos, `~67%` e `~24%` medidos na tela. Ela curou *o widget não cresce
/// com a moldura*; esta cura *o widget não é a moldura*. O padding de linha não é do canvas.
///
/// ⛔ **O que isto continua NÃO sendo:** não mexe em `CHECKBOX_BOX_PX` nem no teto do painel. Eles
/// governam TODOS os painéis do app, e movê-los para agradar ao canvas re-dimensionaria a
/// interface inteira — mover o número do consumidor errado. `None` continua a ser a lei de todo
/// painel, **ao bit**, e a mutação que a remove derruba três gates.
///
/// ⚠️ **O token não deixa de significar: ele passa a ser o TAMANHO NATURAL do objeto.** Uma
/// moldura da altura do token pinta exactamente a tinta que o app pinta numa linha — o que muda é
/// que a moldura passou a medir o *widget* em vez da *linha que o hospeda*.
fn skin_checkbox_box_px(rect: Rect) -> f32 {
    // A caixa é QUADRADA, então preencher a altura só é possível enquanto ela couber na largura.
    // ⚠️ O `min` da largura mora AQUI e não no pintor: o pintor limita pela altura (a lei que todo
    // painel usa e que uma linha estreita nunca exercita), e alargá-la mudaria a rota do painel
    // para agradar ao canvas — exactamente o que a nota acima recusa.
    rect.h.min(rect.w)
}

/// A espessura da trilha do slider numa moldura de canvas: a moldura inteira. Irmã de
/// [`skin_checkbox_box_px`] e a mesma frase — o que sobrava era o padding da linha.
fn skin_slider_track_px(rect: Rect) -> f32 {
    rect.h
}

/// **Que widget do catálogo esta forma veste.**
///
/// ⚠️ O código de cada variante é um **literal explícito**, nunca a ordem do enum: o número viaja
/// no documento, e reordenar o enum re-pintaria em silêncio toda arte já salva. Acrescentar um
/// tipo é acrescentar um código novo no fim — nunca reciclar um.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WidgetKind {
    Button,
    Toggle,
    Checkbox,
    Slider,
    ProgressBar,
    Tag,
    TextInput,
    Card,
    SectionHeader,
    ListItem,
    Spinner,
    Divider,
}

impl WidgetKind {
    /// Todos os tipos que uma forma pode vestir hoje, na ordem em que o painel os oferece.
    ///
    /// ⚠️ **A fronteira desta lista é ESTRUTURAL, não um orçamento.** Um widget cuja aparência é
    /// função de *(retângulo, rótulo, estado)* é vestível por uma forma **hoje**. Um widget cuja
    /// aparência é função de uma **LISTA** (Tabs, TreeView, RadioGroup, Dropdown, Combobox) precisa
    /// de filhos autorados — e filhos autorados são o degrau 3 (a árvore vira `Panel`, W8b). Pôr
    /// um deles aqui obrigaria a inventar a lista, e a prévia mostraria itens que o documento não
    /// tem.
    pub const ALL: [WidgetKind; 12] = [
        WidgetKind::Button,
        WidgetKind::Toggle,
        WidgetKind::Checkbox,
        WidgetKind::Slider,
        WidgetKind::ProgressBar,
        WidgetKind::Tag,
        WidgetKind::TextInput,
        WidgetKind::Card,
        WidgetKind::SectionHeader,
        WidgetKind::ListItem,
        WidgetKind::Spinner,
        WidgetKind::Divider,
    ];

    /// O código que viaja no documento. Estável para sempre.
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            WidgetKind::Button => 1,
            WidgetKind::Toggle => 2,
            WidgetKind::Checkbox => 3,
            WidgetKind::Slider => 4,
            WidgetKind::ProgressBar => 5,
            WidgetKind::Tag => 6,
            WidgetKind::TextInput => 7,
            WidgetKind::Card => 8,
            WidgetKind::SectionHeader => 9,
            WidgetKind::ListItem => 10,
            WidgetKind::Spinner => 11,
            WidgetKind::Divider => 12,
        }
    }

    /// A tradução de volta. **`None` é o caso que este canal existe para suportar**: um documento
    /// autorado por um build mais novo carrega um código que este não conhece, e a resposta certa
    /// é *desenhe a forma* — nunca um retângulo vazio, e nunca recusar o arquivo.
    #[must_use]
    pub fn from_code(code: u16) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.code() == code)
    }

    /// **O IDENTIFICADOR do variante, tal como o Rust o escreve** — o que o codegen do W8b põe no
    /// código gerado (`WidgetKind::Slider`).
    ///
    /// ⚠️ **Ela mora aqui porque o dono da lista é quem pode respondê-la.** O gerador
    /// (`ph2d-ui-codegen`) **não depende** desta crate de propósito: sem alcance ao catálogo ele
    /// não CONSEGUE ter opinião sobre o que um `Slider` é, e uma tabela `código → nome` do lado
    /// dele seria uma segunda resposta que driftaria no dia em que um tipo entrasse — em silêncio,
    /// porque um número desconhecido não falha, ele só não casa.
    ///
    /// ⚠️ **É o identificador, NÃO um rótulo:** ele atravessa para dentro de código-fonte, então
    /// traduzi-lo ou embelezá-lo produziria um arquivo que não compila. O nome que o artista lê é
    /// o [`Self::i18n_key`].
    #[must_use]
    pub const fn ident(self) -> &'static str {
        match self {
            WidgetKind::Button => "Button",
            WidgetKind::Toggle => "Toggle",
            WidgetKind::Checkbox => "Checkbox",
            WidgetKind::Slider => "Slider",
            WidgetKind::ProgressBar => "ProgressBar",
            WidgetKind::Tag => "Tag",
            WidgetKind::TextInput => "TextInput",
            WidgetKind::Card => "Card",
            WidgetKind::SectionHeader => "SectionHeader",
            WidgetKind::ListItem => "ListItem",
            WidgetKind::Spinner => "Spinner",
            WidgetKind::Divider => "Divider",
        }
    }

    /// A chave i18n do nome deste tipo, para o painel.
    #[must_use]
    pub const fn i18n_key(self) -> &'static str {
        match self {
            WidgetKind::Button => "panel.vector.widget.kind.button",
            WidgetKind::Toggle => "panel.vector.widget.kind.toggle",
            WidgetKind::Checkbox => "panel.vector.widget.kind.checkbox",
            WidgetKind::Slider => "panel.vector.widget.kind.slider",
            WidgetKind::ProgressBar => "panel.vector.widget.kind.progress",
            WidgetKind::Tag => "panel.vector.widget.kind.tag",
            WidgetKind::TextInput => "panel.vector.widget.kind.text_input",
            WidgetKind::Card => "panel.vector.widget.kind.card",
            WidgetKind::SectionHeader => "panel.vector.widget.kind.section",
            WidgetKind::ListItem => "panel.vector.widget.kind.list_item",
            WidgetKind::Spinner => "panel.vector.widget.kind.spinner",
            WidgetKind::Divider => "panel.vector.widget.kind.divider",
        }
    }
}

/// **A PORTA ÚNICA**: pinta a pele de `kind` em `rect`, pelo pintor real do catálogo.
///
/// `label` é o `Name` da entidade — o rótulo que o widget mostra.
///
/// ⚠️ Quem chama isto tem de ser quem chamaria o painel nativo: mesma `VectorScene`, mesmo
/// `TextSystem`, mesmo `Theme`. É essa igualdade de argumentos que torna o gate de bytes possível
/// — a prévia e o painel não podem divergir se percorrem a MESMA função com a MESMA entrada.
pub fn paint_widget_skin(
    kind: WidgetKind,
    label: &str,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_widget_skin_with(
        kind,
        label,
        PREVIEW_ID,
        None,
        rect,
        scene,
        text_system,
        theme,
    );
}

/// **A MESMA porta, com o que o widget É agora** — a pele viva (plano UI/UX W8b.2).
///
/// `live` é a fatia do [`WidgetStore`](crate::interaction::WidgetStore) daquele id: o valor de um
/// slider, o on/off de um toggle, o texto de um campo, e o *hot/active* que o ponteiro escreve.
///
/// ⚠️ **Um painel gerado e a prévia do canvas percorrem ESTA função, e é isso que os impede de
/// divergir.** Um segundo `match` sobre os doze tipos seria a segunda resposta a *"que aparência
/// tem um Slider?"*, e a divergência entre as duas só apareceria numa screenshot — o modo de falha
/// mais caro que este repo conhece. A prévia é literalmente **esta função sem estado**: `None` cai
/// nos valores de prévia, que é o que uma pele de canvas tem para mostrar.
///
/// ⚠️ **Estado que não casa com o tipo cai no valor de prévia, e nunca entra em pânico.** É
/// alcançável (um `populate` que declare o tipo errado), e o produto certo é desenhar o widget com
/// a aparência neutra — uma tela que explode por causa de um registro trocado troca um controle
/// errado por nenhum app.
#[allow(clippy::too_many_arguments)]
pub fn paint_widget_skin_with(
    kind: WidgetKind,
    label: &str,
    id: NodeId,
    live: Option<&InteractiveState>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let _ = id;
    match kind {
        WidgetKind::Button => {
            let mut b = Button::new(id, label);
            if let Some(InteractiveState::Button { state }) = live {
                b.state = *state;
            }
            paint_button(&b, rect, scene, text_system, theme);
        }
        WidgetKind::Toggle => {
            let mut t = Toggle::new(id, label);
            if let Some(InteractiveState::Toggle { state, on }) = live {
                t.state = *state;
                t.on = *on;
            }
            paint_toggle(&t, rect, scene, theme);
        }
        WidgetKind::Checkbox => {
            let mut c = Checkbox::new(id, label);
            c.box_px = Some(skin_checkbox_box_px(rect));
            if let Some(InteractiveState::Checkbox { state, value }) = live {
                c.state = *state;
                c.value = *value;
            }
            paint_checkbox(&c, rect, scene, text_system, theme);
        }
        WidgetKind::Slider => {
            let mut s = Slider::new(id, label);
            s.value = PREVIEW_VALUE;
            s.track_px = Some(skin_slider_track_px(rect));
            if let Some(InteractiveState::Slider { state, value, .. }) = live {
                s.state = *state;
                // ⚠️ Pela porta do widget, nunca pelo campo: `set_value` é quem limita a `0..=1`,
                // e escrever o campo cru pintaria uma barra fora da trilha.
                s.set_value(*value);
            }
            paint_slider(&s, rect, scene, theme);
        }
        WidgetKind::ProgressBar => {
            let mut b = ProgressBar::new(id, label);
            // ⚠️ `Indeterminate` (o default) pinta uma lasca fixa cuja POSIÇÃO a shell anima — na
            // prévia estática ela leria como uma barra quebrada. A prévia é determinada.
            b.mode = crate::widget::ProgressMode::Determinate(PREVIEW_VALUE);
            paint_progress_bar(&b, rect, scene, text_system, theme);
        }
        WidgetKind::Tag => {
            let mut t = Tag::new(id, label);
            if let Some(InteractiveState::Tag { state }) = live {
                t.state = *state;
            }
            paint_tag(&t, rect, scene, text_system, theme);
        }
        WidgetKind::TextInput => {
            let mut i = TextInput::new(id, label);
            i.value = label.to_string();
            if let Some(InteractiveState::TextInput {
                state, text, caret, ..
            }) = live
            {
                i.state = *state;
                i.value = text.clone();
                i.caret_byte = *caret;
            }
            paint_text_input(&i, rect, scene, text_system, theme);
        }
        WidgetKind::Card => {
            let mut c = Card::new(id);
            c.title = Some(label.to_string());
            paint_card(&c, rect, scene, text_system, theme);
        }
        WidgetKind::SectionHeader => paint_section_header(
            &SectionHeader::new(id, label),
            rect,
            scene,
            text_system,
            theme,
        ),
        WidgetKind::ListItem => {
            let mut l = ListItem::new(id, label);
            if let Some(InteractiveState::ListItem { state, selected }) = live {
                l.state = *state;
                l.selected = *selected;
            }
            paint_list_item(&l, rect, scene, text_system, theme);
        }
        WidgetKind::Spinner => paint_spinner(&Spinner::new(id, label), rect, scene, theme),
        WidgetKind::Divider => paint_divider(&Divider::new(id), rect, scene, theme),
    }
}

#[cfg(test)]
// ⚠️ Em `skin/tests.rs`, e NÃO num `skin_tests.rs` solto: o `ph2d-widget-sync` varre os `*.rs`
// do topo desta pasta para gerar o bloco de `mod`, então um irmão solto vira um "WIDGET" — sem
// showcase, sem a11y e sem opt-out, com o gate de staleness a sangrar. O dir e o arquivo dedupam
// para um `mod` só; é o precedente do `tool_rail/tests.rs` e a cicatriz do `command_palette`.
#[path = "skin/tests.rs"]
mod tests;
