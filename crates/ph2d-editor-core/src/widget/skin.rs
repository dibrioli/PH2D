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

/// **A moldura é uma CAIXA: a pele PREENCHE o que o artista desenhou** (BUGS_vector #26).
///
/// Dez dos doze tipos já faziam isso. Os outros dois carregavam um TETO — `CHECKBOX_BOX_PX.min(h)`
/// e `(h·0,25).clamp(2, 8)` — e o teto é a **política de LINHA de um painel**, não uma propriedade
/// do widget: ali toda linha tem a mesma altura, todo checkbox tem o mesmo tamanho, e é isso que
/// faz um formulário ler como formulário. O canvas não herda essa política.
///
/// ⚠️ **E o preço do teto foi MEDIDO, não deduzido:** a pele é pintada em px de **TELA** (o
/// `frame_of` da shell projeta a forma pela câmera), então com o teto o checkbox media **18 px em
/// TODA moldura de 28 a 192** — *dar zoom crescia o retângulo e não crescia o widget*. Era esse o
/// *"o checkbox sempre fica pequeno"* do report, e a mesma frase explica o *"o Slider tem sempre
/// altura fixa"* (a trilha pinava em 8 px a partir de 32 de moldura).
///
/// ⚠️ **A razão é formada como `h / ROW_H_PX`, nunca como `CHECKBOX_BOX_PX / ROW_H_PX`, e a ordem
/// é load-bearing:** `h / h` é `1.0` EXATO em IEEE-754, então na altura natural de uma linha a
/// pele é byte-idêntica ao painel **por construção**. Com os valores de hoje (18 e 28) a forma
/// ingénua também acerta — por acidente aritmético, medido —, e é a construção que sobrevive ao
/// próximo valor de token.
///
/// ⛔ **O que isto NÃO é:** não mexe em `CHECKBOX_BOX_PX` nem no teto do painel. Eles governam
/// TODOS os painéis do app, e movê-los para agradar ao canvas re-dimensionaria a interface inteira
/// — mover o número do consumidor errado. `None` continua a ser a lei de todo painel, ao bit.
fn skin_checkbox_box_px(rect: Rect) -> f32 {
    ph2d_tokens::CHECKBOX_BOX_PX * (rect.h / ph2d_tokens::ROW_H_PX)
}

/// A espessura da trilha do slider numa moldura de canvas: os MESMOS 25% do painel, **sem o teto
/// de linha**. Irmã de [`skin_checkbox_box_px`], e igualmente byte-idêntica ao painel em qualquer
/// moldura que o teto não morde (até 32 px) — o piso de legibilidade continua sendo do pintor.
fn skin_slider_track_px(rect: Rect) -> f32 {
    rect.h * 0.25 // LITERAL-PX-OK: the panel's own 25% geometry ratio, minus the row ceiling
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
    match kind {
        WidgetKind::Button => paint_button(
            &Button::new(PREVIEW_ID, label),
            rect,
            scene,
            text_system,
            theme,
        ),
        WidgetKind::Toggle => paint_toggle(&Toggle::new(PREVIEW_ID, label), rect, scene, theme),
        WidgetKind::Checkbox => {
            let mut c = Checkbox::new(PREVIEW_ID, label);
            c.box_px = Some(skin_checkbox_box_px(rect));
            paint_checkbox(&c, rect, scene, text_system, theme);
        }
        WidgetKind::Slider => {
            let mut s = Slider::new(PREVIEW_ID, label);
            s.value = PREVIEW_VALUE;
            s.track_px = Some(skin_slider_track_px(rect));
            paint_slider(&s, rect, scene, theme);
        }
        WidgetKind::ProgressBar => {
            let mut b = ProgressBar::new(PREVIEW_ID, label);
            // ⚠️ `Indeterminate` (o default) pinta uma lasca fixa cuja POSIÇÃO a shell anima — na
            // prévia estática ela leria como uma barra quebrada. A prévia é determinada.
            b.mode = crate::widget::ProgressMode::Determinate(PREVIEW_VALUE);
            paint_progress_bar(&b, rect, scene, text_system, theme);
        }
        WidgetKind::Tag => paint_tag(
            &Tag::new(PREVIEW_ID, label),
            rect,
            scene,
            text_system,
            theme,
        ),
        WidgetKind::TextInput => {
            let mut i = TextInput::new(PREVIEW_ID, label);
            i.value = label.to_string();
            paint_text_input(&i, rect, scene, text_system, theme);
        }
        WidgetKind::Card => {
            let mut c = Card::new(PREVIEW_ID);
            c.title = Some(label.to_string());
            paint_card(&c, rect, scene, text_system, theme);
        }
        WidgetKind::SectionHeader => paint_section_header(
            &SectionHeader::new(PREVIEW_ID, label),
            rect,
            scene,
            text_system,
            theme,
        ),
        WidgetKind::ListItem => paint_list_item(
            &ListItem::new(PREVIEW_ID, label),
            rect,
            scene,
            text_system,
            theme,
        ),
        WidgetKind::Spinner => paint_spinner(&Spinner::new(PREVIEW_ID, label), rect, scene, theme),
        WidgetKind::Divider => paint_divider(&Divider::new(PREVIEW_ID), rect, scene, theme),
    }
}

#[cfg(test)]
// ⚠️ Em `skin/tests.rs`, e NÃO num `skin_tests.rs` solto: o `ph2d-widget-sync` varre os `*.rs`
// do topo desta pasta para gerar o bloco de `mod`, então um irmão solto vira um "WIDGET" — sem
// showcase, sem a11y e sem opt-out, com o gate de staleness a sangrar. O dir e o arquivo dedupam
// para um `mod` só; é o precedente do `tool_rail/tests.rs` e a cicatriz do `command_palette`.
#[path = "skin/tests.rs"]
mod tests;
